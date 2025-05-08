mod item;

pub use item::{
    AddableMenuItem, LedsChangeColorItem, LedsToggleItem, PowerToggleItem, QuitItem, SpeedAutoItem,
    SpeedDownItem, SpeedLabelItem, SpeedUpItem,
};

#[derive(Debug, Default)]
pub struct MenuItems {
    pub speed_auto: SpeedAutoItem,
    pub speed_up: SpeedUpItem,
    pub speed_down: SpeedDownItem,
    pub leds: LedsToggleItem,
    pub change_color: LedsChangeColorItem,
    pub power: PowerToggleItem,
}

impl MenuItems {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refresh_sensitivity(&self) {
        self.set_sensitive(true);
    }

    pub fn disable(&self) {
        self.set_sensitive(false);
    }

    fn set_sensitive(&self, flag: bool) {
        let enable_speed_ctrl = flag && self.power.is_active() && !self.speed_auto.is_active();

        self.speed_auto.set_sensitive(flag);
        self.speed_up.set_sensitive(enable_speed_ctrl);
        self.speed_down.set_sensitive(enable_speed_ctrl);

        self.leds.set_sensitive(flag);
        let enable_leds_ctrl = flag && self.leds.is_active();
        self.change_color.set_sensitive(enable_leds_ctrl);

        self.power.set_sensitive(flag);
    }
}
