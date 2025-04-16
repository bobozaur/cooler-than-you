use arduino_hal::{pac::PLL, usb::SuspendNotifier};
use avr_device::interrupt;

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
            shared_state.command_queue.push_front(Command::DelayedLedsOff);
            shared_state.command_queue.push_front(Command::PowerOff);
        });
    }

    fn resume(&self) {
        self.0.resume();

        interrupt::free(|cs| {
            let mut shared_state = SHARED_STATE.borrow(cs).borrow_mut();
            shared_state.command_queue.push_front(Command::LedsOn);
            shared_state.command_queue.push_front(Command::PowerOn);
        });
    }
}
