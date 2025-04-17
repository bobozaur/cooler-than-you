/// Commands that the device can execute.
///
/// While the power and LED buttons are in fact toggles, separating
/// them makes it easier to reason about what to do depending on the
/// current device state.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
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
    // Long press on LEDs button.
    // Turns the leds off after a short delay.
    //
    // This is a separate command because on device unplug
    // a USB suspend gets triggered which sends [`Command::LedsOff`]
    // to the main function.
    //
    // However, power runs out way before the long press is achieved,
    // and a short press is triggered instead, changing LEDs color.
    //
    // So, for the USB suspend code, we use the [`Command::DelayedLedsOff`]
    // so we can wait a bit before triggering the long press, ensuring that
    // if the device is unplugged power runs out before a LED button short
    // press can occur.
    DelayedLedsOff,
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
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Command::SpeedUp),
            1 => Ok(Command::SpeedDown),
            2 => Ok(Command::PowerOn),
            3 => Ok(Command::PowerOff),
            4 => Ok(Command::LedsOn),
            5 => Ok(Command::LedsOff),
            6 => Ok(Command::DelayedLedsOff),
            7 => Ok(Command::LedsColorChange),
            _ => Err(()),
        }
    }
}

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
