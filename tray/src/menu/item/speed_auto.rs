use std::{cell::OnceCell, rc::Weak};

use gtk::{
    CheckMenuItem,
    traits::{CheckMenuItemExt, GtkMenuItemExt},
};

use crate::menu::{MenuItems, item::CustomMenuItem};

/// Actionable checkbox item that enables/disables the fan speed auto adjustment based on
/// temperature. This item is already active on start-up.
pub type SpeedAutoItem = CustomMenuItem<CheckMenuItem, SpeedAuto>;

#[derive(Clone, Copy, Debug)]
pub struct SpeedAuto;

impl SpeedAutoItem {
    // NOTE: Used this name to be consistent with the other checkbox items
    //       construction method.
    pub fn new_checkbox(menu_items: Weak<MenuItems>) -> Self {
        let inner = CheckMenuItem::with_label("Auto fan speed");
        inner.set_active(true);
        let cache = OnceCell::new();

        inner.connect_activate(move |_| {
            // Cache the weak pointer upgrade so as not to do it every time.
            let cache_fn = || menu_items.upgrade().expect("menu items are never dropped");
            cache
                .get_or_init(cache_fn)
                .refresh_speed_items_sensitivity();
        });

        Self {
            inner,
            kind: SpeedAuto,
        }
    }
}
