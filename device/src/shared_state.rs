use core::cell::RefCell;

use avr_device::interrupt::Mutex;
use circular_buffer::CircularBuffer;
use shared::{Command, DeviceState, FanSpeed};

pub static SHARED_STATE: Mutex<RefCell<SharedState>> = Mutex::new(RefCell::new(SharedState::new()));

pub struct SharedState {
    pub device_state: DeviceState,
    pub send_state: bool,
    pub command_queue: CircularBuffer<32, Command>,
}

impl SharedState {
    #[inline]
    pub const fn new() -> Self {
        Self {
            device_state: DeviceState {
                fan_speed: FanSpeed::Speed6,
                power_enabled: true,
                leds_enabled: true,
            },
            send_state: true,
            command_queue: CircularBuffer::new(),
        }
    }
}
