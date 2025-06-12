use shared::DeviceCommand;

/// A command that the device can execute.
#[derive(Clone, Copy, Debug)]
pub enum Command {
    /// Commands that map to physical button actions.
    Device(DeviceCommand),
    EnterBootloader,
    /// Artifical command.
    ///
    /// This is here because on device unplug a USB suspend gets triggered which sends
    /// [`DeviceCommand::LedsOff`] to the main function.
    ///
    /// However, power runs out way before the long press (~1400ms) on the device is achieved,
    /// effectivelly triggering a short press instead, changing LEDs color.
    ///
    /// So, to work around this issue, the suspend code sends a delay command before
    /// the actual device commands, giving a chance to residual power to wear off before
    /// executing anything.
    Delay275Ms,
}
