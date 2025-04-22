use arduino_hal::port::{
    Pin, PinOps,
    mode::{Input, PullUp},
};

use super::pins::{
    BacklightMonitorPin, LedMonitorPin, PowerMonitorPin, SpeedDownMonitorPin, SpeedUpMonitorPin,
};

pub type SpeedUpButtonMonitor = ButtonMonitor<SpeedUpMonitorPin>;
pub type SpeedDownButtonMonitor = ButtonMonitor<SpeedDownMonitorPin>;
pub type PowerButtonMonitor = ButtonMonitor<PowerMonitorPin>;
pub type LedButtonMonitor = ButtonMonitor<LedMonitorPin>;
pub type BacklightMonitor = Monitor<BacklightMonitorPin>;

pub struct Monitor<PIN>(Pin<Input<PullUp>, PIN>);

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

pub struct ButtonMonitor<PIN>(Monitor<PIN>);

impl<PIN> ButtonMonitor<PIN>
where
    PIN: PinOps,
{
    #[inline]
    pub fn new(pin: Pin<Input<PullUp>, PIN>) -> Self {
        Self(Monitor::new(pin))
    }

    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.0.is_active()
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
