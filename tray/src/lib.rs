mod device;
mod fd_callbacks;
mod indicator;
mod menu;
mod exactly_one;

pub use anyhow::Result as AnyResult;
pub use device::Device;
pub use indicator::Indicator;
pub use menu::item::{quit::QuitItem, speed_label::SpeedLabelItem};
