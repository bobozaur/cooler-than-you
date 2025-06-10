use core::cell::RefCell;

use avr_device::interrupt::Mutex;
use circular_buffer::CircularBuffer;
use shared::DeviceState;

use crate::command::Command;

/// Mutex locked shared device state across the entire program.
pub static SHARED_STATE: Mutex<RefCell<SharedState>> = Mutex::new(RefCell::new(SharedState::new()));
/// Arbitrarily chosen to just be big enough to provide some command backlog when under high load.
/// [`Command`] is one byte, so this isn't too much out of the total 2560 RAM available.
const COMMAND_QUEUE_SIZE: usize = 64;

/// Shared state struct.
pub struct SharedState {
    /// The current device state.
    device_state: DeviceState,
    /// Whether the device state must be sent to the host, either due to an update or a retry.
    send_state: bool,
    /// FIFO command queue backed by a [`CircularBuffer`] of length [`COMMAND_QUEUE_SIZE`].
    /// Acts as a command backlog when under high load.
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

    /// Updates the device state and sets [`SharedState::send_state`] so it gets sent on next USB
    /// poll.
    #[inline]
    pub fn update_device_state<F>(&mut self, f: F)
    where
        F: FnOnce(&mut DeviceState),
    {
        self.device_state.set_repeat_command(None);
        f(&mut self.device_state);
        self.send_state = true;
    }

    /// Executes the closure if [`SharedState::send_state`] is `true` and, if the closure returns
    /// `true`, sets [`SharedState::send_state`] to false.
    #[inline]
    pub fn if_send_state<F>(&mut self, f: F)
    where
        F: FnOnce() -> bool,
    {
        if self.send_state && f() {
            self.send_state = false;
        }
    }

    /// Returns the current [`DeviceState`].
    #[inline]
    pub fn device_state(&self) -> &DeviceState {
        &self.device_state
    }

    /// Pushes a [`Command`] to the front of the queue.
    #[inline]
    pub fn push_command(&mut self, command: Command) {
        self.command_queue.push_front(command);
    }

    /// Pops a [`Command`] from the back of the queue.
    #[inline]
    pub fn pop_command(&mut self) -> Option<Command> {
        self.command_queue.pop_back()
    }
}
