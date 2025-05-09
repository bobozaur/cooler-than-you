use std::rc::Rc;

use gtk::{
    CheckMenuItem,
    glib::SignalHandlerId,
    traits::{CheckMenuItemExt, GtkMenuItemExt},
};

use crate::{
    Device,
    menu::{
        MenuItems,
        item::{CustomMenuItem, ItemLabel, MenuItemSetup},
    },
};

pub type SpeedAutoItem = CustomMenuItem<CheckMenuItem, SpeedAuto>;

#[derive(Clone, Copy, Debug)]
pub struct SpeedAuto;

impl ItemLabel for SpeedAuto {
    const LABEL: &str = "Auto fan speed";
}

impl MenuItemSetup for SpeedAutoItem {
    type MenuItem = CheckMenuItem;

    fn setup(
        &self,
        menu_items: Rc<MenuItems>,
        _: Device,
    ) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        self.inner.set_active(true);

        let handler_id = self
            .inner
            .connect_activate(move |_| menu_items.refresh_speed_items_sensitivity());

        (&self.inner, Some(handler_id))
    }
}
