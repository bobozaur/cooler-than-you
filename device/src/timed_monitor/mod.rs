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
    BacklightMonitor, LedButtonMonitor, MonitorState, PowerButtonMonitor, SpeedDownButtonMonitor,
    SpeedUpButtonMonitor,
};
use pins::{
    BacklightMonitorPin, LedMonitorPin, PowerMonitorPin, SpeedDownMonitorPin, SpeedUpMonitorPin,
};

use crate::{interrupt_cell::InterruptCell, shared_state::SHARED_STATE};

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
            speed_up_monitor,
            speed_down_monitor,
            power_monitor,
            led_monitor,
            backlight_monitor,
            monitor_state: MonitorState::Active,
            buttons_state: 0,
            buttons_history: 0,
        }
    }

    #[inline]
    fn monitor(&mut self) {
        interrupt::free(|cs| {
            let shared_state = &mut SHARED_STATE.borrow(cs).borrow_mut();

            let speed_up_pressed = self.speed_up_monitor.is_pressed();
            let speed_down_pressed = self.speed_down_monitor.is_pressed();
            let power_pressed = self.power_monitor.is_pressed();
            let led_pressed = self.led_monitor.is_pressed();

            let backlight_active = self.backlight_monitor.is_active();

            let any_button_pressed =
                speed_up_pressed || speed_down_pressed || power_pressed || led_pressed;

            match self.monitor_state {
                MonitorState::Active => {
                    self.buttons_state = (self.buttons_state << 1) ^ u64::from(any_button_pressed);

                    // A button short press requires the state to be low for 40ms. We therefore
                    // look for a sequence of a 0 bit followed by 40 `1` bits and handle that
                    // depending on the button priority and which ones are pressed.
                    if self.buttons_state << 23 == 0x7FFF_FFFF_FF80_0000 {
                        if speed_up_pressed {
                            self.monitor_state = MonitorState::Paused;

                            if backlight_active {
                                shared_state.send_state = true;
                                shared_state.device_state.fan_speed.increase();
                            }
                        } else if speed_down_pressed {
                            self.monitor_state = MonitorState::Paused;

                            if backlight_active {
                                shared_state.send_state = true;
                                shared_state.device_state.fan_speed.decrease();
                            }
                        } else if power_pressed {
                            self.monitor_state = MonitorState::PowerFocused;
                        } else if led_pressed {
                            self.monitor_state = MonitorState::LedFocused;
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
                MonitorState::PowerFocused => {
                    self.buttons_state = (self.buttons_state << 1) ^ u64::from(power_pressed);

                    if self.buttons_history < 21 {
                        if self.buttons_state == u64::MAX {
                            self.buttons_state = 0;
                            self.buttons_history += 1;
                        }
                        // Short press triggered
                        else if !power_pressed {
                            self.monitor_state = MonitorState::Paused;

                            shared_state.send_state = true;
                            shared_state.device_state.power_enabled =
                                !shared_state.device_state.power_enabled;
                        }
                    }
                    // Long press triggered
                    else if self.buttons_state == 0x00FF_FFFF_FFFF_FFFF {
                        self.monitor_state = MonitorState::Paused;
                    }
                }
                MonitorState::LedFocused => {
                    self.buttons_state = (self.buttons_state << 1) ^ u64::from(led_pressed);

                    if self.buttons_history < 21 {
                        if self.buttons_state == u64::MAX {
                            self.buttons_state = 0;
                            self.buttons_history += 1;
                        }
                        // Short press triggered
                        else if !led_pressed {
                            self.monitor_state = MonitorState::Paused;
                        }
                    }
                    // Long press triggered
                    else if self.buttons_state == 0x00FF_FFFF_FFFF_FFFF {
                        self.monitor_state = MonitorState::Paused;
                        shared_state.send_state = true;

                        shared_state.device_state.leds_enabled =
                            !shared_state.device_state.leds_enabled;
                    }
                }
            }
        });
    }
}
