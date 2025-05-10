mod device;
mod fd_handler;
mod indicator;
mod menu;

pub use anyhow::Result as AnyResult;
pub use device::Device;
pub use indicator::Indicator;
pub use menu::item::{quit::QuitItem, speed_label::SpeedLabelItem};
