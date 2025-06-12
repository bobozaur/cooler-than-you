use arduino_hal::{pac::PLL, usb::SuspendNotifier};
use avr_device::interrupt;
use shared::DeviceCommand;

use crate::{SHARED_STATE, command::Command};

/// Implementor of [`SuspendNotifier`] whose job is to turn the LEDs and power off when the device
/// is suspended and turn them on when the device is resumed.
/// NOTE: The suspend/resume behavior is irrespective of the device state.
pub struct Suspender(PLL);

impl Suspender {
    #[inline]
    pub fn new(pll: PLL) -> Self {
        Self(pll)
    }
}

impl SuspendNotifier for Suspender {
    fn suspend(&self) {
        self.0.suspend();

        interrupt::free(|cs| {
            let mut shared_state = SHARED_STATE.borrow(cs).borrow_mut();
            // Delay the execution of the commands to turn off the LEDs and power to guard against
            // the situation when the device gets unplugged, which also triggers a suspend.
            //
            // The suspend itself is not the problem, but rather the fact that turning the LEDs off
            // is a long press which takes at least 1400ms. But if the device runs out
            // of power as the press is happening, a short press might get triggered
            // instead if there's enough left over power for at least 40ms.
            //
            // Delaying the command execution allows for the left over power to deplete, and avoid
            // initiating a long press to turn the LEDs off that will not complete.
            shared_state.push_command(Command::Delay275Ms);
            shared_state.push_command(Command::Device(DeviceCommand::LedsOff));
            shared_state.push_command(Command::Device(DeviceCommand::PowerOff));
        });
    }

    fn resume(&self) {
        self.0.resume();

        interrupt::free(|cs| {
            let mut shared_state = SHARED_STATE.borrow(cs).borrow_mut();
            shared_state.push_command(Command::Device(DeviceCommand::LedsOn));
            shared_state.push_command(Command::Device(DeviceCommand::PowerOn));
        });
    }
}
