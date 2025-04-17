use arduino_hal::{
    hal::port::{PB4, PB5, PC6, PD7, PE6},
    port::PinOps,
};

use crate::shared_state::SharedState;

pub type SpeedUpMonitorPin = PE6;
pub type SpeedDownMonitorPin = PD7;
pub type PowerMonitorPin = PB5;
pub type LedMonitorPin = PB4;
pub type BacklightMonitorPin = PC6;

pub trait ShortPressPin: PinOps {
    fn short_press_state_update(shared_state: &mut SharedState);
}

pub trait LongPressPin: ShortPressPin {
    fn long_press_state_update(shared_state: &mut SharedState);
}

/// Short press on `+` button.
///
/// Has no effect if power is disabled.
impl ShortPressPin for SpeedUpMonitorPin {
    fn short_press_state_update(shared_state: &mut SharedState) {
        if shared_state.power_enabled {
            shared_state.fan_speed.increase();
        }
    }
}

/// Short press on `-` button.
///
/// Has no effect if power is disabled.
impl ShortPressPin for SpeedDownMonitorPin {
    fn short_press_state_update(shared_state: &mut SharedState) {
        if shared_state.power_enabled {
            shared_state.fan_speed.decrease();
        }
    }
}

/// Short press on power button.
///
/// Works regardless of power state.
impl ShortPressPin for PowerMonitorPin {
    fn short_press_state_update(shared_state: &mut SharedState) {
        shared_state.power_enabled = !shared_state.power_enabled;
    }
}

/// Long press on power button.
///
/// It's a no-op, but still prevents other commands
/// until released.
impl LongPressPin for PowerMonitorPin {
    fn long_press_state_update(_: &mut SharedState) {}
}

/// A short press of the LEDs button cycles through the LEDs
/// colors, but has no impact on the state.
///
/// See [`SharedState`] for why we do not track the LED colors.
impl ShortPressPin for LedMonitorPin {
    fn short_press_state_update(_: &mut SharedState) {}
}

/// Long press on LEDs button.
///
/// Works regardless of power state.
impl LongPressPin for LedMonitorPin {
    fn long_press_state_update(shared_state: &mut SharedState) {
        shared_state.leds_enabled = !shared_state.leds_enabled;
    }
}
