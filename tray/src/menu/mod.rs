pub mod item;

use std::rc::Rc;

use crate::{
    Device, QuitItem, SpeedLabelItem,
    menu::item::{
        command::{
            LedsChangeColorItem, LedsToggleItem, PowerToggleItem, SpeedDownItem, SpeedUpItem,
        },
        speed_auto::SpeedAutoItem,
    },
};

#[derive(Debug)]
pub struct MenuItems {
    pub speed_label: SpeedLabelItem,
    pub speed_auto: SpeedAutoItem,
    pub speed_up: SpeedUpItem,
    pub speed_down: SpeedDownItem,
    pub leds: LedsToggleItem,
    pub leds_change_color: LedsChangeColorItem,
    pub power: PowerToggleItem,
    pub quit: QuitItem,
}

impl MenuItems {
    pub fn new(device: Device) -> Rc<Self> {
        Rc::new_cyclic(move |menu_items| Self {
            speed_label: SpeedLabelItem::default(),
            speed_auto: SpeedAutoItem::new(menu_items.clone()),
            speed_up: SpeedUpItem::new(menu_items.clone(), device.clone()),
            speed_down: SpeedDownItem::new(menu_items.clone(), device.clone()),
            leds: LedsToggleItem::new(menu_items.clone(), device.clone()),
            leds_change_color: LedsChangeColorItem::new(menu_items.clone(), device.clone()),
            power: PowerToggleItem::new(menu_items.clone(), device),
            quit: QuitItem::default(),
        })
    }

    pub fn refresh_speed_items_sensitivity(&self) {
        self.set_speed_items_sensitive(true);
    }

    pub fn refresh_sensitivity(&self) {
        self.set_sensitive(true);
    }

    pub fn disable(&self) {
        self.set_sensitive(false);
    }

    fn set_speed_items_sensitive(&self, flag: bool) {
        let enable_speed_ctrl = flag && self.power.is_active() && !self.speed_auto.is_active();

        self.speed_auto.set_sensitive(flag);
        self.speed_up.set_sensitive(enable_speed_ctrl);
        self.speed_down.set_sensitive(enable_speed_ctrl);
    }

    fn set_sensitive(&self, flag: bool) {
        self.set_speed_items_sensitive(flag);

        self.leds.set_sensitive(flag);
        let enable_leds_ctrl = flag && self.leds.is_active();
        self.leds_change_color.set_sensitive(enable_leds_ctrl);

        self.power.set_sensitive(flag);
    }
}
