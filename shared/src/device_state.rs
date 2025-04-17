use core::prelude::v1::derive;

use crate::fan_speed::FanSpeed;

#[derive(Copy, Clone, Debug)]
pub struct DeviceState {
    pub fan_speed: FanSpeed,
    pub power_enabled: bool,
    pub leds_enabled: bool,
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
