use arduino_hal::port::{
    Pin, PinOps,
    mode::{Input, PullUp},
};
use shared::DeviceState;

use super::pins::{
    BacklightMonitorPin, LedMonitorPin, PowerMonitorPin, SpeedDownMonitorPin, SpeedUpMonitorPin,
};
use crate::timed_monitor::pins::{LongPressPin, ShortPressPin};

type ExtendedButtonMonitor<PIN> = ButtonMonitor<PIN, u8>;

type SpeedUpButtonMonitor = ButtonMonitor<SpeedUpMonitorPin>;
type SpeedDownButtonMonitor = ButtonMonitor<SpeedDownMonitorPin>;
type PowerButtonMonitor = ExtendedButtonMonitor<PowerMonitorPin>;
type LedButtonMonitor = ExtendedButtonMonitor<LedMonitorPin>;
type BacklightMonitor = Monitor<BacklightMonitorPin>;

struct Monitor<PIN>(Pin<Input<PullUp>, PIN>);

impl<PIN> Monitor<PIN>
where
    PIN: PinOps,
{
    #[inline]
    pub fn new(pin: Pin<Input<PullUp>, PIN>) -> Self {
        Self(pin)
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.0.is_low()
    }
}

struct ButtonMonitor<PIN, HIST = ()> {
    /// Pin used
    monitor: Monitor<PIN>,
    /// State bits
    state: u64,
    /// State history
    history: HIST,
}

impl<PIN, HIST> ButtonMonitor<PIN, HIST>
where
    PIN: ShortPressPin,
    HIST: Default,
{
    #[inline]
    fn new(pin: Pin<Input<PullUp>, PIN>) -> Self {
        Self {
            monitor: Monitor::new(pin),
            state: 0,
            history: HIST::default(),
        }
    }

    #[inline]
    fn is_pressed(&self) -> bool {
        self.monitor.is_active()
    }
}
impl<PIN, HIST> ButtonMonitor<PIN, HIST>
where
    Self: ButtonStateUpdate,
    PIN: ShortPressPin,
    HIST: Default,
{
    /// Monitors the button and returns whether it was pressed.
    fn monitor(
        &mut self,
        shared_state: &mut DeviceState,
        monitor_state: &mut MonitorState,
    ) -> bool {
        let is_pressed = self.is_pressed();
        self.state = (self.state << 1) ^ u64::from(is_pressed);
        self.update_state(shared_state, monitor_state);
        is_pressed
    }
}

trait ButtonStateUpdate {
    /// Tracks and updates the button state, registering presses if necessary.
    fn update_state(&mut self, shared_state: &mut DeviceState, monitor_state: &mut MonitorState);
}

impl<PIN> ButtonStateUpdate for ButtonMonitor<PIN>
where
    PIN: ShortPressPin,
{
    #[inline]
    fn update_state(&mut self, shared_state: &mut DeviceState, monitor_state: &mut MonitorState) {
        // If a speed button is pressed when the backlight is **NOT** active then
        // buttons get disabled until all of them are released. The physical button
        // press however activates the backlight, thus a subsequent press will be correctly
        // registered.
        if self.state & 1 == 1 && !monitor_state.backlight_monitor.is_active() {
            monitor_state.buttons_enabled = false;
        }

        // Speed buttons short press require the pin to be low for 40ms. A button release is not
        // necessary for the press to be registered. We therefore look for a sequence of a 0
        // bit followed by 40 `1` bits.
        if self.state << 23 == 0x7FFF_FFFF_FF80_0000 {
            PIN::register_short_press(shared_state, monitor_state);
        }
    }
}

impl<PIN> ButtonStateUpdate for ExtendedButtonMonitor<PIN>
where
    PIN: LongPressPin,
{
    #[inline]
    fn update_state(&mut self, shared_state: &mut DeviceState, monitor_state: &mut MonitorState) {
        // TODO: document order of operations
        if self.history < 21 {
            // Long press pending
            if self.state == u64::MAX {
                self.state = 0;
                self.history += 1;
            }
            // Short press
            else if self.state << 23 == 0xFFFF_FFFF_FF00_0000
                || (self.history > 0 && self.state & 1 == 0)
            {
                self.history = 0;
                PIN::register_short_press(shared_state, monitor_state);
            }
        } else {
            // Long press released
            if self.state << 7 == 0xFFFF_FFFF_FFFF_FF00 {
                self.history = 0;
            }
            // Long press triggered
            else if self.state == 0x00FF_FFFF_FFFF_FFFF {
                PIN::register_long_press(shared_state, monitor_state);
            }
        }
    }
}

#[allow(clippy::struct_field_names)]
pub struct MonitorButtons {
    speed_up_monitor: SpeedUpButtonMonitor,
    speed_down_monitor: SpeedDownButtonMonitor,
    power_monitor: PowerButtonMonitor,
    led_monitor: LedButtonMonitor,
}

impl MonitorButtons {
    #[inline]
    pub fn new(
        speed_up_mon_pin: Pin<Input<PullUp>, SpeedUpMonitorPin>,
        speed_down_mon_pin: Pin<Input<PullUp>, SpeedDownMonitorPin>,
        power_mon_pin: Pin<Input<PullUp>, PowerMonitorPin>,
        led_mon_pin: Pin<Input<PullUp>, LedMonitorPin>,
    ) -> Self {
        Self {
            speed_up_monitor: SpeedUpButtonMonitor::new(speed_up_mon_pin),
            speed_down_monitor: SpeedDownButtonMonitor::new(speed_down_mon_pin),
            power_monitor: PowerButtonMonitor::new(power_mon_pin),
            led_monitor: LedButtonMonitor::new(led_mon_pin),
        }
    }

    #[inline]
    pub fn monitor(&mut self, device_state: &mut DeviceState, monitor_state: &mut MonitorState) {
        // If buttons are disabled (a command was issued), check if all buttons are released.
        // As long as a button is still pressed, no other button presses are registered.
        let any_button_pressed = self.speed_up_monitor.monitor(device_state, monitor_state)
            || self.speed_down_monitor.monitor(device_state, monitor_state)
            || self.power_monitor.monitor(device_state, monitor_state)
            || self.led_monitor.monitor(device_state, monitor_state);

        // The monitor state is modified when a press completes.
        // We only want to update it if a command was registered and
        // buttons were disabled.
        monitor_state.buttons_enabled |= !any_button_pressed;
    }
}

pub struct MonitorState {
    backlight_monitor: BacklightMonitor,
    buttons_enabled: bool,
}

impl MonitorState {
    #[inline]
    pub fn new(backlight_mon_pin: Pin<Input<PullUp>, BacklightMonitorPin>) -> Self {
        Self {
            backlight_monitor: BacklightMonitor::new(backlight_mon_pin),
            buttons_enabled: true,
        }
    }

    #[inline]
    pub fn buttons_enabled(&self) -> bool {
        self.buttons_enabled
    }

    #[inline]
    pub fn set_buttons_enabled(&mut self, enabled: bool) {
        self.buttons_enabled = enabled;
    }
}
