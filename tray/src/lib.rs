mod cooler;

pub use anyhow::Result as AnyResult;
pub use cooler::Cooler;
use gtk::{
    CheckMenuItem, MenuItem,
    traits::{CheckMenuItemExt, WidgetExt},
};

#[derive(Debug)]
pub struct MenuItems {
    pub speed_up_mi: MenuItem,
    pub speed_down_mi: MenuItem,
    pub color_mi: MenuItem,
    pub speed_auto_adjust_mi: CheckMenuItem,
    pub power_mi: CheckMenuItem,
    pub leds_mi: CheckMenuItem,
}

impl MenuItems {
    pub fn set_sensitive(&self, sensitive: bool) {
        if !self.speed_auto_adjust_mi.is_active() {
            self.speed_up_mi.set_sensitive(sensitive);
            self.speed_down_mi.set_sensitive(sensitive);
        }

        self.color_mi.set_sensitive(sensitive);
        self.speed_auto_adjust_mi.set_sensitive(sensitive);
        self.power_mi.set_sensitive(sensitive);
        self.leds_mi.set_sensitive(sensitive);
    }
}
