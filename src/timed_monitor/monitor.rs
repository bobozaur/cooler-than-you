use arduino_hal::port::{
    Pin, PinOps,
    mode::{Input, PullUp},
};

use super::pins::{
    BacklightMonitorPin, LedMonitorPin, PowerMonitorPin, SpeedDownMonitorPin, SpeedUpMonitorPin,
};
use crate::{
    shared_state::SharedState,
    timed_monitor::pins::{LongPressPin, ShortPressPin},
};

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
    pub fn all_buttons_released(&self) -> bool {
        !self.speed_up_monitor.is_pressed()
            && !self.speed_down_monitor.is_pressed()
            && !self.power_monitor.is_pressed()
            && !self.led_monitor.is_pressed()
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
    pub fn speed_buttons_enabled(&self) -> bool {
        self.buttons_enabled && self.backlight_monitor.is_active()
    }
}

pub trait ButtonMonitorOps {
    fn monitor(&mut self, shared_state: &mut SharedState, monitor_state: &mut MonitorState);
}

impl<PIN> ButtonMonitorOps for ButtonMonitor<PIN>
where
    PIN: ShortPressPin,
{
    fn monitor(&mut self, shared_state: &mut SharedState, monitor_state: &mut MonitorState) {
        self.state = (self.state << 1) ^ self.is_pressed() as u64;

        if monitor_state.speed_buttons_enabled() && self.state == 0x000000FFFFFFFFFF {
            monitor_state.buttons_enabled = false;
            PIN::short_press_state_update(shared_state)
        }
    }
}

impl<PIN> ButtonMonitorOps for ExtendedButtonMonitor<PIN>
where
    PIN: LongPressPin,
{
    fn monitor(&mut self, shared_state: &mut SharedState, monitor_state: &mut MonitorState) {
        let button_pressed = self.is_pressed();
        self.state = (self.state << 1) ^ button_pressed as u64;

        // TODO: document order of operations
        if self.history < 21 {
            // Long press pending
            if self.state == u64::MAX {
                self.state = 0;
                self.history += 1;
            }
            // Short press
            else if monitor_state.buttons_enabled
                && (self.state << 23 == 0xFFFFFFFFFF000000 || (self.history > 0 && !button_pressed))
            {
                self.history = 0;
                monitor_state.buttons_enabled = false;
                PIN::short_press_state_update(shared_state)
            }
        } else {
            // Long press released
            if self.state << 7 == 0xFFFFFFFFFFFFFF00 {
                self.history = 0;
            }
            // Long press triggered
            else if monitor_state.buttons_enabled && self.state == 0x00FFFFFFFFFFFFFF {
                PIN::long_press_state_update(shared_state)
            }
        }
    }
}

impl ButtonMonitorOps for MonitorButtons {
    fn monitor(&mut self, shared_state: &mut SharedState, monitor_state: &mut MonitorState) {
        self.speed_up_monitor.monitor(shared_state, monitor_state);
        self.speed_down_monitor.monitor(shared_state, monitor_state);
        self.power_monitor.monitor(shared_state, monitor_state);
        self.led_monitor.monitor(shared_state, monitor_state);

        // If buttons are disabled (a command was issued), check if all buttons are released.
        // As long as a button is still pressed, no other button presses are registered.
        monitor_state.buttons_enabled |= self.all_buttons_released();
    }
}
