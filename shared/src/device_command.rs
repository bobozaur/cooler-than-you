use thiserror::Error as ThisError;

/// Commands that the device can execute.
///
/// While the power and LED buttons are in fact toggles, separating
/// them makes it easier to reason about what to do depending on the
/// current device state.
///
/// We start the enum variant indexing at `1` for the sake of
/// having 0 represent no command to repeat in the [`crate::device_state::DeviceState`].
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(strum::EnumIter, PartialEq))]
pub enum DeviceCommand {
    // Short press on `+` button.
    // Increases fan speed.
    SpeedUp = 1,
    // Short press on `-` button.
    // Decreases fan speed.
    SpeedDown,
    // Short press on power button.
    // Turns the cooler on.
    PowerOn,
    // Short press on power button.
    // Turns the cooler off.
    PowerOff,
    // Long press on LEDs button.
    // Turns the leds on.
    LedsOn,
    // Long press on LEDs button.
    // Turns the leds off.
    LedsOff,
    // Short press on LEDs button.
    // Changes LEDs color.
    LedsColorChange,
}

impl From<DeviceCommand> for u8 {
    fn from(value: DeviceCommand) -> Self {
        value as Self
    }
}

impl TryFrom<u8> for DeviceCommand {
    type Error = CommandConvError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DeviceCommand::SpeedUp),
            2 => Ok(DeviceCommand::SpeedDown),
            3 => Ok(DeviceCommand::PowerOn),
            4 => Ok(DeviceCommand::PowerOff),
            5 => Ok(DeviceCommand::LedsOn),
            6 => Ok(DeviceCommand::LedsOff),
            7 => Ok(DeviceCommand::LedsColorChange),
            _ => Err(CommandConvError),
        }
    }
}

#[derive(Clone, Copy, Debug, ThisError)]
#[cfg_attr(test, derive(PartialEq))]
#[error("integer to command conversion failed")]
pub struct CommandConvError;

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::DeviceCommand;

    const MAX_BITS: usize = 3;

    #[test]
    fn test_command_conversion() {
        for command in DeviceCommand::iter() {
            assert_eq!(command as u8 >> MAX_BITS, 0);
            assert_eq!((command as u8).try_into(), Ok(command));
        }
    }
}
