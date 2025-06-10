mod interrupts;
mod pins;

use arduino_hal::{
    pac::TC0,
    port::{
        Pin, PinOps,
        mode::{Input, PullUp},
    },
};
use avr_device::interrupt;
use pins::{
    BacklightMonitorPin, LedMonitorPin, PowerMonitorPin, SpeedDownMonitorPin, SpeedUpMonitorPin,
};
use shared::{DeviceCommand, DeviceState};

use crate::{
    interrupt_cell::InterruptCell,
    shared_state::{SHARED_STATE, SharedState},
};

/// Monitor context that gets setup prior to enabling interrupts and is used exclusively from the
/// `TIMER0_COMPA` interrupt.
static MONITOR_CTX: InterruptCell<MonitorContext> = InterruptCell::uninit();

/// Sets up `TIMER0_COMPA` interrupt to trigger every millisecond for time tracking and constructs
/// the [`InterruptCell`] used exclusively within it.
///
/// Timer comparison value formula: 16 MHz / (64 * (1 + 249)) = 1000 Hz
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

type SpeedUpButtonMonitor = ButtonMonitor<SpeedUpMonitorPin>;
type SpeedDownButtonMonitor = ButtonMonitor<SpeedDownMonitorPin>;
type PowerButtonMonitor = ButtonMonitor<PowerMonitorPin>;
type LedButtonMonitor = ButtonMonitor<LedMonitorPin>;

/// Buttons monitor context. Contains components used exclusively in the `TIMER0_COMPA` interrupt.
///
/// The monitoring happens through the [`MonitorContext::monitor`] method, which is meant to be
/// called once every millisecond. This is being done in the `TIMER0_COMPA` interrupt.
///
/// The design used for monitoring is based off of reverse engineering how the buttons worked.
///
/// Notable mentions:
/// - Short presses are presses at least as long as 40ms.
/// - Long presses are presses at least as long as 1400ms.
/// - Buttons are not handled individually; their state is shared. This means that pressing a button
///   for 10ms and another one for 30ms will result in a button priority being enforced.
/// - Button priority is Speed Up > Speed Down > Power > LED
/// - Speed buttons do not work with the backlight off; they just trigger a wake.
/// - Speed buttons do not require a button release for the short press to get registered; power and
///   LED buttons do.
/// - The power button *has* a long press which is a no-op. But it does **NOT** trigger a short
///   press!
/// - After a short/long press being triggered, no button presses get registered anymore until all
///   buttons get released. Not even the backlight gets woken up!
struct MonitorContext {
    /// Speed up button monitor.
    speed_up_monitor: SpeedUpButtonMonitor,
    /// Speed down button monitor.
    speed_down_monitor: SpeedDownButtonMonitor,
    /// Power button monitor.
    power_monitor: PowerButtonMonitor,
    /// LED button monitor.
    led_monitor: LedButtonMonitor,
    /// Backlight monitor.
    backlight_monitor: BacklightMonitor,
    /// Tracks the current state of the monitor. See [`MonitorState`] for more details.
    monitor_state: MonitorState,
    /// A bit array of consecutive button presses. The state gets left shifted every time
    /// [`MonitorContext::monitor`] is ran. Helps with tracking whether a short press should
    /// get triggered. The button state is shared between all buttons and **IS NOT** button
    /// independent.
    buttons_state: u64,
    /// The button state history is a simple counter that increments when the `buttons_state` field
    /// reaches max value and is reset. It helps with tracking potential long presses.
    /// Similar to [`buttons_state`], this makes part of the button state shared by all buttons.
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

    /// Run the monitor over the physical components. Meant to be ran exactly once per millisecond.
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
                            self.monitor_state = MonitorState::Focused(MonitorFocusTarget::Power);
                        } else if led_pressed {
                            self.monitor_state = MonitorState::Focused(MonitorFocusTarget::Leds);
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
                        MonitorFocusTarget::Power => {
                            (power_pressed, Some(DeviceState::toggle_power), None)
                        }
                        MonitorFocusTarget::Leds => {
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

    /// Dedicated method that handles the device state changes when a short press gets registered on
    /// a speed button.
    #[inline]
    fn speed_button_pressed<F>(
        shared_state: &mut SharedState,
        backlight_active: bool,
        state_change_fn: F,
        repeat_command: DeviceCommand,
    ) where
        F: FnOnce(&mut DeviceState),
    {
        // The press is completely ignored if the device is powered off.
        if !shared_state.device_state().power_enabled() {
            return;
        }

        if backlight_active {
            // The backlight being active means the device will register the command.
            shared_state.update_device_state(state_change_fn);
        } else {
            // The backlight gets woken up but the command itself gets ignored.
            shared_state.update_device_state(|ds: &mut DeviceState| {
                ds.set_repeat_command(Some(repeat_command))
            });
        }
    }
}

/// Screen backlight monitor.
///
/// The backlight state matters for the speed up and speed down buttons.
/// If the backlight is not active, a button press on these is a no-op which only activates the
/// backlight.
///
/// The backlight times out at around 1300ms.
struct BacklightMonitor {
    /// Physical pin
    pin: Pin<Input<PullUp>, BacklightMonitorPin>,
    /// The last known state of the backlight.
    ///
    /// Helps avoid situations when the backlight is not initially active but a speed button press
    /// wakes it up. In these cases, the backlight pin will report the backlight as active, but
    /// speed commands are not being registered when the backlight was initially off at the
    /// beginning of the press.
    was_active: bool,
}

impl BacklightMonitor {
    #[inline]
    fn new(pin: Pin<Input<PullUp>, BacklightMonitorPin>) -> Self {
        Self {
            was_active: pin.is_low(),
            pin,
        }
    }

    /// Returns whether the backlight is active.
    ///
    /// Consider both whether the backlight was previously active and whether it is currently active
    /// to avoid the race condition where the button press activates the backlight and the read
    /// indicates that it *is* active although it did not use to be.
    #[inline]
    fn is_active(&mut self) -> bool {
        let prev_state = self.was_active;
        self.was_active = self.pin.is_low();
        prev_state && self.was_active
    }
}

/// A physical button monitor.
struct ButtonMonitor<PIN>(Pin<Input<PullUp>, PIN>);

impl<PIN> ButtonMonitor<PIN>
where
    PIN: PinOps,
{
    #[inline]
    fn new(pin: Pin<Input<PullUp>, PIN>) -> Self {
        Self(pin)
    }

    /// Returns whether the button is pressed.
    #[inline]
    fn is_pressed(&self) -> bool {
        self.0.is_low()
    }
}

/// Monitor state enum.
enum MonitorState {
    /// The monitor is active and listening for interactions. This could mean that a button press is
    /// in progress or not.
    Active,
    /// A button press was registered and the monitor is not keeping track of the buttons until all
    /// buttons are released.
    Paused,
    /// A button was pressed long enough to trigger a short press, but the short press gets
    /// triggered on release.
    ///
    /// However, the button need to be monitored more in case a long press gets triggered instead.
    /// That means the monitor is focused on a given button only,
    Focused(MonitorFocusTarget),
}

/// What button the monitor is focused on. This enum contains variants only for buttons that have a
/// long press.
enum MonitorFocusTarget {
    Power,
    Leds,
}
