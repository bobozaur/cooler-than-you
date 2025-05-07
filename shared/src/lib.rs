//! VID and PID used from <https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt>.
//! Discrimination is done by serial number.

#![no_std]

mod command;
mod device_state;
mod fan_speed;

pub use command::Command;
pub use device_state::DeviceState;
pub use fan_speed::FanSpeed;

pub const USB_VID: u16 = 0x16C0;
pub const USB_PID: u16 = 0x27D8;
pub const USB_POLL_MS: u8 = 40;
