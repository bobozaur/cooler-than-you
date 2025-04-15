use core::cell::RefCell;

use avr_device::interrupt::Mutex;
use circular_buffer::CircularBuffer;

use crate::{command::Command, fan_speed::FanSpeed};

pub static SHARED_STATE: Mutex<RefCell<SharedState>> = Mutex::new(RefCell::new(SharedState::new()));

pub struct SharedState {
    pub fan_speed: FanSpeed,
    pub power_enabled: bool,
    pub leds_enabled: bool,
    pub command_queue: CircularBuffer<32, Command>,
}

impl SharedState {
    #[inline]
    pub const fn new() -> Self {
        Self {
            fan_speed: FanSpeed::Speed6,
            power_enabled: true,
            leds_enabled: true,
            command_queue: CircularBuffer::new(),
        }
    }
}
