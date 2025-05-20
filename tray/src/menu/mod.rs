pub mod item;

use std::rc::Rc;

use crate::{
    Device,
    menu::item::{
        LedsChangeColorItem, LedsToggleItem, PowerToggleItem, QuitItem, SpeedAutoItem,
        SpeedDownItem, SpeedLabelItem, SpeedUpItem,
    },
};

/// Collection of actionable menu items used in the UI.
///
/// Items have to reference other items so they interact with each other,
/// and this type makes sharing the menu items easy and convenient.
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
    // Ensures this struct cannot be constructed from scratch.
    _private: (),
}

impl MenuItems {
    /// Creates an [`Rc<MenuItems>`].
    ///
    /// The struct is wrapped because it is self referential and meant to be shared and cloned,
    /// since the items' activation callbacks alter the state of other items.
    pub fn new(device: Device) -> Rc<Self> {
        // Not particularly fond of this, but a compromise had to be made:
        // - The cyclic definition allows for items to be valid on construction and for those that
        //   need to store their callback [`SignalHandlerId`] to be able to do so.
        // - Constructing the [`MenuItems`] first and then setting the callbacks can also be done,
        //   but then that trades the self referencing problem with convenience to use and construct
        //   the items and introduces room for mistakes.
        Rc::new_cyclic(move |menu_items| Self {
            speed_label: SpeedLabelItem::default(),
            speed_auto: SpeedAutoItem::new_checkbox(menu_items.clone()),
            speed_up: SpeedUpItem::new(menu_items.clone(), device.clone()),
            speed_down: SpeedDownItem::new(menu_items.clone(), device.clone()),
            leds: LedsToggleItem::new_checkbox(menu_items.clone(), device.clone()),
            leds_change_color: LedsChangeColorItem::new(menu_items.clone(), device.clone()),
            power: PowerToggleItem::new_checkbox(menu_items.clone(), device),
            quit: QuitItem::default(),
            _private: (),
        })
    }

    /// Resets the sensitivity for fan speed items only.
    pub fn refresh_speed_items_sensitivity(&self) {
        self.set_speed_items_sensitive(true);
    }

    /// Resets the sensitivity for all menu items.
    pub fn refresh_sensitivity(&self) {
        self.set_sensitive(true);
    }

    /// Disables all menu items.
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

        let enable_leds_ctrl = flag && self.leds.is_active();
        self.leds.set_sensitive(flag);
        self.leds_change_color.set_sensitive(enable_leds_ctrl);

        self.power.set_sensitive(flag);
    }
}
