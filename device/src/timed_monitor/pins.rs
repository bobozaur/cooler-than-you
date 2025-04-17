use arduino_hal::{
    hal::port::{PB4, PB5, PC6, PD7, PE6},
    port::PinOps,
};
use shared::DeviceState;

use crate::{shared_state::SharedState, timed_monitor::MonitorState};

pub type SpeedUpMonitorPin = PE6;
pub type SpeedDownMonitorPin = PD7;
pub type PowerMonitorPin = PB5;
pub type LedMonitorPin = PB4;
pub type BacklightMonitorPin = PC6;

pub trait ShortPressPin: PinOps {
    fn short_press_state_update(shared_state: &mut DeviceState);

    fn register_short_press(shared_state: &mut SharedState, monitor_state: &mut MonitorState) {
        register_press(shared_state, monitor_state, Self::short_press_state_update);
    }
}

pub trait LongPressPin: ShortPressPin {
    fn long_press_state_update(shared_state: &mut DeviceState);

    fn register_long_press(shared_state: &mut SharedState, monitor_state: &mut MonitorState) {
        register_press(shared_state, monitor_state, Self::long_press_state_update);
    }
}

/// Short press on `+` button.
///
/// Has no effect if power is disabled, but we do not
/// check against that because turning off the power results
/// in the backlight being turned off as well.
///
/// A check against the backlight is done prior to getting here.
impl ShortPressPin for SpeedUpMonitorPin {
    fn short_press_state_update(shared_state: &mut DeviceState) {
        shared_state.fan_speed.increase();
    }
}

/// Short press on `-` button.
///
/// Has no effect if power is disabled, but we do not
/// check against that because turning off the power results
/// in the backlight being turned off as well.
///
/// A check against the backlight is done prior to getting here.
impl ShortPressPin for SpeedDownMonitorPin {
    fn short_press_state_update(shared_state: &mut DeviceState) {
        shared_state.fan_speed.decrease();
    }
}

/// Short press on power button.
///
/// Works regardless of power state.
impl ShortPressPin for PowerMonitorPin {
    fn short_press_state_update(shared_state: &mut DeviceState) {
        shared_state.power_enabled = !shared_state.power_enabled;
    }
}

/// Long press on power button.
///
/// It's a no-op, but still prevents other commands
/// until released.
impl LongPressPin for PowerMonitorPin {
    fn long_press_state_update(_: &mut DeviceState) {}
}

/// A short press of the LEDs button cycles through the LEDs
/// colors, but has no impact on the state.
///
/// See [`DeviceState`] for why we do not track the LED colors.
impl ShortPressPin for LedMonitorPin {
    fn short_press_state_update(_: &mut DeviceState) {}
}

/// Long press on LEDs button.
///
/// Works regardless of power state.
impl LongPressPin for LedMonitorPin {
    fn long_press_state_update(shared_state: &mut DeviceState) {
        shared_state.leds_enabled = !shared_state.leds_enabled;
    }
}

fn register_press<F>(shared_state: &mut SharedState, monitor_state: &mut MonitorState, f: F)
where
    F: FnOnce(&mut DeviceState),
{
    if monitor_state.buttons_enabled {
        monitor_state.buttons_enabled = false;
        shared_state.send_state = true;
        f(&mut shared_state.device_state);
    }
}
