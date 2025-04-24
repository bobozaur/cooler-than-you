use core::cell::RefCell;

use avr_device::interrupt::Mutex;
use circular_buffer::CircularBuffer;
use shared::{Command, DeviceState};

pub static SHARED_STATE: Mutex<RefCell<SharedState>> = Mutex::new(RefCell::new(SharedState::new()));
const COMMAND_QUEUE_SIZE: usize = 64;

pub struct SharedState {
    device_state: DeviceState,
    send_state: bool,
    command_queue: CircularBuffer<COMMAND_QUEUE_SIZE, Command>,
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
    pub fn update_device_state<F>(&mut self, f: F)
    where
        F: FnOnce(&mut DeviceState),
    {
        f(&mut self.device_state);
        self.send_state = true;
    }

    #[inline]
    pub fn if_send_state<F>(&mut self, f: F)
    where
        F: FnOnce() -> bool,
    {
        if self.send_state && f() {
            self.send_state = false;
        }
    }

    #[inline]
    pub fn device_state(&self) -> &DeviceState {
        &self.device_state
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
