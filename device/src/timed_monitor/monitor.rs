use arduino_hal::port::{
    Pin, PinOps,
    mode::{Input, PullUp},
};

use crate::timed_monitor::pins::{
    BacklightMonitorPin, LedMonitorPin, PowerMonitorPin, SpeedDownMonitorPin, SpeedUpMonitorPin,
};

pub type SpeedUpButtonMonitor = ButtonMonitor<SpeedUpMonitorPin>;
pub type SpeedDownButtonMonitor = ButtonMonitor<SpeedDownMonitorPin>;
pub type PowerButtonMonitor = ButtonMonitor<PowerMonitorPin>;
pub type LedButtonMonitor = ButtonMonitor<LedMonitorPin>;

pub struct BacklightMonitor {
    pin: Pin<Input<PullUp>, BacklightMonitorPin>,
    state: bool,
}

impl BacklightMonitor {
    #[inline]
    pub fn new(pin: Pin<Input<PullUp>, BacklightMonitorPin>) -> Self {
        Self {
            state: pin.is_low(),
            pin,
        }
    }

    /// Consider both whether the backlight was previously active and whether it is currently active
    /// to avoid the race condition where the button press activates the backlight and the read
    /// indicates that it *is* active although it did not use to be.
    #[inline]
    pub fn is_active(&mut self) -> bool {
        let prev_state = self.state;
        self.state = self.pin.is_low();
        prev_state && self.state
    }
}

pub struct ButtonMonitor<PIN>(Pin<Input<PullUp>, PIN>);

impl<PIN> ButtonMonitor<PIN>
where
    PIN: PinOps,
{
    #[inline]
    pub fn new(pin: Pin<Input<PullUp>, PIN>) -> Self {
        Self(pin)
    }

    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.0.is_low()
    }
}

pub enum MonitorState {
    Active,
    Paused,
    Focused(MonitorFocusKind),
}

pub enum MonitorFocusKind {
    Power,
    Leds,
}
