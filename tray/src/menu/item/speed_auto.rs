use std::rc::Weak;

use gtk::{
    CheckMenuItem,
    traits::{CheckMenuItemExt, GtkMenuItemExt},
};

use crate::menu::{MenuItems, item::CustomMenuItem};

pub type SpeedAutoItem = CustomMenuItem<CheckMenuItem, SpeedAuto>;

#[derive(Clone, Copy, Debug)]
pub struct SpeedAuto;

impl SpeedAutoItem {
    pub fn new(menu_items: Weak<MenuItems>) -> Self {
        let inner = CheckMenuItem::with_label("Auto fan speed");
        inner.set_active(true);

        inner.connect_activate(move |_| {
            menu_items
                .upgrade()
                .expect("menu items are never dropped")
                .refresh_speed_items_sensitivity();
        });

        Self {
            inner,
            kind: SpeedAuto,
        }
    }
}
