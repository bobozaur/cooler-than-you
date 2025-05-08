mod interrupts;
mod monitor;
mod pins;

use arduino_hal::{
    pac::TC0,
    port::{
        Pin,
        mode::{Input, PullUp},
    },
};
use avr_device::interrupt;
use monitor::{
    BacklightMonitor, LedButtonMonitor, MonitorFocusKind, MonitorState, PowerButtonMonitor,
    SpeedDownButtonMonitor, SpeedUpButtonMonitor,
};
use pins::{
    BacklightMonitorPin, LedMonitorPin, PowerMonitorPin, SpeedDownMonitorPin, SpeedUpMonitorPin,
};
use shared::{DeviceCommand, DeviceState};

use crate::{
    interrupt_cell::InterruptCell,
    shared_state::{SHARED_STATE, SharedState},
};

/// Timer context that gets setup prior to enabling interrupts
/// and is used exclusively from the timer interrupt.
static MONITOR_CTX: InterruptCell<MonitorContext> = InterruptCell::uninit();

/// Configure timer0 to overflow every millisecond for time tracking
/// and constructs the [`InterruptCell`] used exclusively
/// in the timer overflow interrupt.
///
/// Formula: 16 MHz / (64 * (1 + 249)) = 1000 Hz
pub fn setup_timed_monitor(
    timer: &TC0,
    speed_up_mon_pin: Pin<Input<PullUp>, SpeedUpMonitorPin>,
    speed_down_mon_pin: Pin<Input<PullUp>, SpeedDownMonitorPin>,
    power_mon_pin: Pin<Input<PullUp>, PowerMonitorPin>,
    led_mon_pin: Pin<Input<PullUp>, LedMonitorPin>,
    backlight_mon_pin: Pin<Input<PullUp>, BacklightMonitorPin>,
) {
    // WGM
    timer.tccr0a.write(|w| w.wgm0().bits(0b10));
    timer.tccr0b.write(|w| w.wgm02().clear_bit());

    // Prescaler
    timer.tccr0b.write(|w| w.cs0().prescale_64());
    timer.ocr0a.write(|w| w.bits(249));

    // Enable the timer interrupt
    timer.timsk0.write(|w| w.ocie0a().set_bit());

    // Initialize the timer context.
    MONITOR_CTX.init(MonitorContext::new(
        SpeedUpButtonMonitor::new(speed_up_mon_pin),
        SpeedDownButtonMonitor::new(speed_down_mon_pin),
        PowerButtonMonitor::new(power_mon_pin),
        LedButtonMonitor::new(led_mon_pin),
        BacklightMonitor::new(backlight_mon_pin),
    ));
}

/// Contains components used exclusively in the timer interrupt.
struct MonitorContext {
    speed_up_monitor: SpeedUpButtonMonitor,
    speed_down_monitor: SpeedDownButtonMonitor,
    power_monitor: PowerButtonMonitor,
    led_monitor: LedButtonMonitor,
    backlight_monitor: BacklightMonitor,
    monitor_state: MonitorState,
    buttons_state: u64,
    buttons_history: u8,
}

impl MonitorContext {
    #[inline]
    fn new(
        speed_up_monitor: SpeedUpButtonMonitor,
        speed_down_monitor: SpeedDownButtonMonitor,
        power_monitor: PowerButtonMonitor,
        led_monitor: LedButtonMonitor,
        backlight_monitor: BacklightMonitor,
    ) -> Self {
        Self {
            monitor_state: MonitorState::Active,
            buttons_state: 0,
            buttons_history: 0,
            speed_up_monitor,
            speed_down_monitor,
            power_monitor,
            led_monitor,
            backlight_monitor,
        }
    }

    #[inline]
    fn monitor(&mut self) {
        interrupt::free(|cs| {
            let shared_state = &mut *SHARED_STATE.borrow(cs).borrow_mut();

            let speed_up_pressed = self.speed_up_monitor.is_pressed();
            let speed_down_pressed = self.speed_down_monitor.is_pressed();
            let power_pressed = self.power_monitor.is_pressed();
            let led_pressed = self.led_monitor.is_pressed();

            let backlight_active = self.backlight_monitor.is_active();

            let any_button_pressed =
                speed_up_pressed || speed_down_pressed || power_pressed || led_pressed;

            match &self.monitor_state {
                MonitorState::Active => {
                    self.buttons_state = (self.buttons_state << 1) ^ u64::from(any_button_pressed);

                    // A button short press requires the state to be low for 40ms. We therefore
                    // look for a sequence of a 0 bit followed by 40 `1` bits and handle that
                    // depending on the button priority and which ones are pressed.
                    if self.buttons_state << 23 == 0x7FFF_FFFF_FF80_0000 {
                        if speed_up_pressed {
                            self.monitor_state = MonitorState::Paused;

                            Self::speed_button_pressed(
                                shared_state,
                                backlight_active,
                                DeviceState::increase_fan_speed,
                                DeviceCommand::SpeedUp,
                            );
                        } else if speed_down_pressed {
                            self.monitor_state = MonitorState::Paused;

                            Self::speed_button_pressed(
                                shared_state,
                                backlight_active,
                                DeviceState::decrease_fan_speed,
                                DeviceCommand::SpeedDown,
                            );
                        } else if power_pressed {
                            self.monitor_state = MonitorState::Focused(MonitorFocusKind::Power);
                        } else if led_pressed {
                            self.monitor_state = MonitorState::Focused(MonitorFocusKind::Leds);
                        }
                    }
                }
                MonitorState::Paused => {
                    if !any_button_pressed {
                        self.monitor_state = MonitorState::Active;
                        self.buttons_history = 0;
                        self.buttons_state = 0;
                    }
                }
                MonitorState::Focused(kind) => {
                    let (button_pressed, short_press_fn_opt, long_press_fn_opt) = match kind {
                        MonitorFocusKind::Power => {
                            (power_pressed, Some(DeviceState::toggle_power), None)
                        }
                        MonitorFocusKind::Leds => {
                            (led_pressed, None, Some(DeviceState::toggle_leds))
                        }
                    };

                    self.buttons_state = (self.buttons_state << 1) ^ u64::from(button_pressed);

                    if self.buttons_history < 21 {
                        if self.buttons_state == u64::MAX {
                            self.buttons_state = 0;
                            self.buttons_history += 1;
                        } else if !button_pressed {
                            // Short press triggered
                            self.monitor_state = MonitorState::Paused;

                            if let Some(short_press_fn) = short_press_fn_opt {
                                shared_state.update_device_state(short_press_fn);
                            } else {
                                // LED button short press
                                //
                                // For visibility, we still want to the state to be sent.
                                shared_state.update_device_state(|_| ());
                            }
                        }
                    } else if self.buttons_state == 0x00FF_FFFF_FFFF_FFFF {
                        // Long press triggered
                        self.monitor_state = MonitorState::Paused;

                        if let Some(long_press_fn) = long_press_fn_opt {
                            shared_state.update_device_state(long_press_fn);
                        }
                    }
                }
            }
        });
    }

    #[inline]
    fn speed_button_pressed<F>(
        shared_state: &mut SharedState,
        backlight_active: bool,
        state_change_fn: F,
        repeat_command: DeviceCommand,
    ) where
        F: FnOnce(&mut DeviceState),
    {
        // Fan speed does not change when the device is powered off.
        if !shared_state.device_state().power_enabled() {
            return;
        }

        let repeat_command_fn = |ds: &mut DeviceState| ds.set_repeat_command(Some(repeat_command));

        if backlight_active {
            shared_state.update_device_state(state_change_fn);
        } else {
            shared_state.update_device_state(repeat_command_fn);
        }
    }
}
