mod cooler;
mod indicator;

pub use anyhow::Result as AnyResult;
pub use cooler::Cooler;
use gtk::{
    CheckMenuItem, MenuItem,
    traits::{CheckMenuItemExt, WidgetExt},
};
pub use indicator::Indicator;

#[derive(Debug)]
pub struct MenuItems {
    pub speed_auto_adjust_mi: CheckMenuItem,
    pub speed_up_mi: MenuItem,
    pub speed_down_mi: MenuItem,
    pub leds_mi: CheckMenuItem,
    pub color_mi: MenuItem,
    pub power_mi: CheckMenuItem,
}

impl MenuItems {
    #[must_use]
    pub fn new() -> Self {
        let speed_auto_adjust_mi = CheckMenuItem::with_label("Auto-adjust speed");
        let speed_up_mi = MenuItem::with_label("Increase speed");
        let speed_down_mi = MenuItem::with_label("Decrease speed");
        let leds_mi = CheckMenuItem::with_label("Lights");
        let color_mi = MenuItem::with_label("Change color");
        let power_mi = CheckMenuItem::with_label("Power");

        Self {
            speed_auto_adjust_mi,
            speed_up_mi,
            speed_down_mi,
            leds_mi,
            color_mi,
            power_mi,
        }
    }

    pub fn refresh_sensitivity(&self) {
        self.set_sensitive(true);
    }

    pub fn disable(&self) {
        self.set_sensitive(false);
    }

    fn set_sensitive(&self, flag: bool) {
        let enable_speed_ctrl =
            flag && self.power_mi.is_active() && !self.speed_auto_adjust_mi.is_active();

        self.speed_auto_adjust_mi.set_sensitive(flag);
        self.speed_up_mi.set_sensitive(enable_speed_ctrl);
        self.speed_down_mi.set_sensitive(enable_speed_ctrl);

        self.leds_mi.set_sensitive(flag);
        let enable_leds_ctrl = flag && self.leds_mi.is_active();
        self.color_mi.set_sensitive(enable_leds_ctrl);

        self.power_mi.set_sensitive(flag);
    }
}

impl Default for MenuItems {
    fn default() -> Self {
        Self::new()
    }
}
