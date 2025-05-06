#![no_std]

mod command;
mod device_state;
mod fan_speed;

pub use command::Command;
pub use device_state::DeviceState;
pub use fan_speed::FanSpeed;

pub const USB_VID: u16 = 0xD016;
pub const USB_PID: u16 = 0xDB08;
pub const USB_POLL_MS: u8 = 10;
