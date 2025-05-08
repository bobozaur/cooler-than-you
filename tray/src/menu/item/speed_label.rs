use std::rc::Rc;

use gtk::{
    MenuItem,
    glib::SignalHandlerId,
    traits::{GtkMenuItemExt, WidgetExt},
};
use shared::FanSpeed;

use crate::{
    Cooler,
    menu::{MenuItems, item::MenuItemSetup},
};

#[derive(Debug)]
pub struct SpeedLabelItem {
    inner: MenuItem,
    buf: String,
}

impl SpeedLabelItem {
    #[must_use]
    pub fn new() -> Self {
        let buf = String::from("Fan speed: 0");
        let inner = MenuItem::with_label(&buf);
        inner.set_sensitive(false);
        Self { inner, buf }
    }

    pub fn update_speed(&mut self, fan_speed: FanSpeed) {
        self.buf.pop();
        self.buf.push(char::from(fan_speed));
        self.inner.set_label(&self.buf);
    }
}

impl Default for SpeedLabelItem {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuItemSetup for SpeedLabelItem {
    type MenuItem = MenuItem;

    fn setup(&self, _: Rc<MenuItems>, _: Cooler) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        (&self.inner, None)
    }
}
