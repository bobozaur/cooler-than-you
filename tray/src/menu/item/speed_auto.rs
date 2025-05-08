use std::rc::Rc;

use gtk::{CheckMenuItem, glib::SignalHandlerId, traits::CheckMenuItemExt};

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

    fn setup(&self, _: Rc<MenuItems>, _: Device) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        self.inner.set_active(true);
        (&self.inner, None)
    }
}
