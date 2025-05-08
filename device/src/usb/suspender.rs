use arduino_hal::{pac::PLL, usb::SuspendNotifier};
use avr_device::interrupt;
use shared::DeviceCommand;

use crate::{command::Command, shared_state::SHARED_STATE};

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
