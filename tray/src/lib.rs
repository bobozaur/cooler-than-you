mod cooler;
mod indicator;
mod menu;

pub use anyhow::Result as AnyResult;
pub use cooler::Cooler;
pub use indicator::Indicator;
pub use menu::item::{quit::QuitItem, speed_label::SpeedLabelItem};
