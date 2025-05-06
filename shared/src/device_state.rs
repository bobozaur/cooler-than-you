use thiserror::Error as ThisError;

use crate::{
    Command,
    command::CommandConvError,
    fan_speed::{FanSpeed, FanSpeedConvError},
};

#[derive(Clone, Copy, Debug)]
pub struct DeviceState {
    power_enabled: bool,
    leds_enabled: bool,
    fan_speed: FanSpeed,
    command_to_repeat: Option<Command>,
}

impl PartialEq for DeviceState {
    fn eq(&self, other: &Self) -> bool {
        self.power_enabled == other.power_enabled()
            && self.leds_enabled == other.leds_enabled()
            && self.fan_speed == other.fan_speed
    }
}

impl DeviceState {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            power_enabled: true,
            leds_enabled: true,
            fan_speed: FanSpeed::Speed1,
            command_to_repeat: None,
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
    #[must_use]
    pub fn command_to_repeat(&self) -> Option<Command> {
        self.command_to_repeat
    }

    #[inline]
    pub fn toggle_power(&mut self) {
        self.power_enabled = !self.power_enabled;
    }

    #[inline]
    pub fn toggle_leds(&mut self) {
        self.leds_enabled = !self.leds_enabled;
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
    pub fn set_repeat_command(&mut self, command: Option<Command>) {
        self.command_to_repeat = command;
    }
}

impl From<DeviceState> for u8 {
    fn from(state: DeviceState) -> Self {
        let power_enabled = u8::from(state.power_enabled) << 7;
        let leds_enabled = u8::from(state.leds_enabled) << 6;
        let fan_speed = u8::from(state.fan_speed) << 3;
        let command_to_repeat_byte = state.command_to_repeat.map(u8::from).unwrap_or_default();

        power_enabled | leds_enabled | fan_speed | command_to_repeat_byte
    }
}

impl TryFrom<u8> for DeviceState {
    type Error = DeviceStateConvError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let power_enabled = value & 0b1000_0000 != 0;
        let leds_enabled = value & 0b0100_0000 != 0;
        let fan_speed = ((value & 0b0011_1000) >> 3).try_into()?;
        let command_to_repeat_byte = value & 0b0000_0111;

        let repeat_command = (command_to_repeat_byte != 0)
            .then(|| Command::try_from(command_to_repeat_byte))
            .transpose()?;

        Ok(Self {
            power_enabled,
            leds_enabled,
            fan_speed,
            command_to_repeat: repeat_command,
        })
    }
}

#[derive(Clone, Copy, Debug, ThisError)]
#[cfg_attr(test, derive(PartialEq))]
#[error("integer to device state conversion failed")]
pub enum DeviceStateConvError {
    Fan(#[from] FanSpeedConvError),
    Command(#[from] CommandConvError),
}
