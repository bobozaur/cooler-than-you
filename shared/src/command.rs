use thiserror::Error as ThisError;

/// Commands that the device can execute.
///
/// While the power and LED buttons are in fact toggles, separating
/// them makes it easier to reason about what to do depending on the
/// current device state.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(strum::EnumIter, PartialEq))]
pub enum Command {
    // Short press on `+` button.
    // Increases fan speed.
    SpeedUp,
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

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        value as Self
    }
}

impl TryFrom<u8> for Command {
    type Error = CommandConvError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Command::SpeedUp),
            1 => Ok(Command::SpeedDown),
            2 => Ok(Command::PowerOn),
            3 => Ok(Command::PowerOff),
            4 => Ok(Command::LedsOn),
            5 => Ok(Command::LedsOff),
            6 => Ok(Command::LedsColorChange),
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

    use super::Command;

    #[test]
    fn test_command_parsing() {
        for command in Command::iter() {
            assert_eq!((command as u8).try_into(), Ok(command));
        }
    }
}
