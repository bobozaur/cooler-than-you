#![no_std]
#![feature(abi_avr_interrupt)]

use arduino_hal::hal::{wdt::Timeout, Wdt};
use core::{cell::{RefCell, UnsafeCell}, mem::MaybeUninit};

use avr_device::interrupt::Mutex;
use circular_buffer::CircularBuffer;
use shared::DeviceState;

use crate::command::Command;

pub mod button;
pub mod command;
pub mod monitor;
pub mod usb;

/// Mutex locked shared device state across the entire program.
pub static SHARED_STATE: Mutex<RefCell<SharedState>> = Mutex::new(RefCell::new(SharedState::new()));

/// Triggers a watch dog reset that will leave the device in bootloader mode.
pub fn enter_bootloader(mut watchdog: Wdt) -> ! {
    /// Magic value that tells the bootloader to remain in bootloader mode on watchdog resets.
    /// Taken from <https://github.com/arduino/ArduinoCore-avr/blob/c8c514c9a19602542bc32c7033f48fecbbda4401/bootloaders/caterina/Caterina.c#L68>
    const BOOT_KEY: u16 = 0x7777;
    /// Pointer to the address where the bootloader looks for the [`BOOT_KEY`].
    /// Taken from <https://github.com/arduino/ArduinoCore-avr/blob/c8c514c9a19602542bc32c7033f48fecbbda4401/bootloaders/caterina/Caterina.c#L69>
    const BOOT_KEY_PTR: *mut u16 = 0x0800 as *mut u16;

    // Write the magic value
    unsafe { core::ptr::write_volatile(BOOT_KEY_PTR, BOOT_KEY) };

    // Set the lowest possible time value for the watchdog.
    watchdog.start(Timeout::Ms16).ok();

    // Loop until the watchdog reset happens.
    loop {}
}

/// Shared state struct.
#[derive(Debug)]
pub struct SharedState {
    /// The current device state.
    device_state: DeviceState,
    /// Whether the device state must be sent to the host, either due to an update or a retry.
    send_state: bool,
    /// FIFO command queue backed by a [`CircularBuffer`] of length [`SharedState::COMMAND_QUEUE_SIZE`].
    /// Acts as a command backlog when under high load.
    command_queue: CircularBuffer<{ Self::COMMAND_QUEUE_SIZE }, Command>,
}

impl SharedState {
    /// Arbitrarily chosen to just be big enough to provide some command backlog when under high load.
    /// [`Command`] is one byte, so this isn't too much out of the total 2560 RAM available.
    const COMMAND_QUEUE_SIZE: usize = 64;

    const fn new() -> Self {
        Self {
            device_state: DeviceState::new(),
            send_state: true,
            command_queue: CircularBuffer::new(),
        }
    }

    /// Returns the current [`DeviceState`].
    #[inline]
    pub fn device_state(&self) -> &DeviceState {
        &self.device_state
    }

    /// Pops a [`Command`] from the back of the queue.
    #[inline]
    pub fn pop_command(&mut self) -> Option<Command> {
        self.command_queue.pop_back()
    }

    /// Pushes a [`Command`] to the front of the queue.
    #[inline]
    fn push_command(&mut self, command: Command) {
        self.command_queue.push_front(command);
    }

    /// Updates the device state and sets [`SharedState::send_state`] so it gets sent on next USB
    /// poll.
    #[inline]
    fn update_device_state<F>(&mut self, f: F)
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
    fn if_send_state<F>(&mut self, f: F)
    where
        F: FnOnce() -> bool,
    {
        if self.send_state && f() {
            self.send_state = false;
        }
    }
}

/// Wrapper type for [`UnsafeCell`] that implements [`Sync`] and provides convenience methods for
/// dealing with the underlying type.
///
/// The purpose of this cell is to initialize statics that will get used exclusively in interrupts.
struct InterruptCell<T>(UnsafeCell<MaybeUninit<T>>);

/// This implementation does not rely on `T: Sync` as well because of
/// [`usb_device::bus::UsbBusAllocator`], which is not sync.
///
/// See <https://github.com/rust-embedded-community/usb-device/pull/162>.
unsafe impl<T> Sync for InterruptCell<T> {}

impl<T> InterruptCell<T> {
    const fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    #[allow(clippy::mut_from_ref)]
    fn init(&self, inner: T) -> &mut T {
        unsafe { (*self.0.get()).write(inner) }
    }

    #[allow(clippy::mut_from_ref)]
    fn as_inner_mut(&self) -> &mut T {
        unsafe { (*self.0.get()).assume_init_mut() }
    }
}
