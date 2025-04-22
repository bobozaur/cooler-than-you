use core::prelude::v1::derive;

use crate::fan_speed::FanSpeed;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DeviceState {
    fan_speed: FanSpeed,
    power_enabled: bool,
    leds_enabled: bool,
}

impl DeviceState {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            fan_speed: FanSpeed::Speed1,
            power_enabled: true,
            leds_enabled: true,
        }
    }

    #[inline]
    #[must_use]
    pub fn power_enabled(&self) -> bool {
        self.power_enabled
    }

    #[inline]
    #[must_use]
    pub fn leds_enabled(&self) -> bool {
        self.leds_enabled
    }

    #[inline]
    #[must_use]
    pub fn fan_speed(&self) -> FanSpeed {
        self.fan_speed
    }

    #[inline]
    pub fn increase_fan_speed(&mut self) {
        self.fan_speed.increase();
    }

    #[inline]
    pub fn decrease_fan_speed(&mut self) {
        self.fan_speed.decrease();
    }

    #[inline]
    pub fn toggle_power(&mut self) {
        self.power_enabled = !self.power_enabled;
    }

    #[inline]
    pub fn toggle_leds(&mut self) {
        self.leds_enabled = !self.leds_enabled;
    }
}

impl TryFrom<u8> for DeviceState {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let power_enabled = value & 0b1000_0000 != 0;
        let leds_enabled = value & 0b0100_0000 != 0;
        let fan_speed = (value & 0b0011_1111).try_into()?;

        Ok(Self {
            fan_speed,
            power_enabled,
            leds_enabled,
        })
    }
}

impl From<DeviceState> for u8 {
    fn from(state: DeviceState) -> Self {
        let power_enabled = u8::from(state.power_enabled) << 7;
        let leds_enabled = u8::from(state.leds_enabled) << 6;
        let fan_speed = state.fan_speed as u8;

        power_enabled | leds_enabled | fan_speed
    }
}
