#![doc = include_str!("../README.md")]
#![no_std]

mod device_command;
mod device_state;
mod fan_speed;

pub use device_command::DeviceCommand;
pub use device_state::DeviceState;
pub use fan_speed::FanSpeed;

pub const USB_VID: u16 = 0x16C0;
pub const USB_PID: u16 = 0x05df;
pub const USB_MANUFACTURER: &str = "mirceapetrebogdan@gmail.com";
pub const USB_PRODUCT: &str = "Cooler Than You";
