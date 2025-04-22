use core::cell::RefCell;

use avr_device::interrupt::Mutex;
use circular_buffer::CircularBuffer;
use shared::{Command, DeviceState};

pub static SHARED_STATE: Mutex<RefCell<SharedState>> = Mutex::new(RefCell::new(SharedState::new()));

pub struct SharedState {
    device_state: DeviceState,
    send_state: bool,
    command_queue: CircularBuffer<32, Command>,
}

impl SharedState {
    pub const fn new() -> Self {
        Self {
            device_state: DeviceState::new(),
            send_state: true,
            command_queue: CircularBuffer::new(),
        }
    }

    #[inline]
    pub fn write_device_state<F>(&mut self, f: F)
    where
        F: FnOnce(&mut DeviceState),
    {
        let previous_device_state = self.device_state;
        f(&mut self.device_state);
        self.send_state = previous_device_state != self.device_state;
    }

    #[inline]
    pub fn device_state(&self) -> &DeviceState {
        &self.device_state
    }

    #[inline]
    pub fn send_state(&self) -> bool {
        self.send_state
    }

    #[inline]
    pub fn mark_state_as_sent(&mut self) {
        self.send_state = false;
    }

    #[inline]
    pub fn push_command(&mut self, command: Command) {
        self.command_queue.push_front(command);
    }

    #[inline]
    pub fn pop_command(&mut self) -> Option<Command> {
        self.command_queue.pop_back()
    }
}
