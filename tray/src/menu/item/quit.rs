use std::rc::Rc;

use gtk::{MenuItem, glib::SignalHandlerId, traits::GtkMenuItemExt};

use crate::{
    Cooler,
    menu::{MenuItems, item::MenuItemSetup},
};

#[derive(Debug)]
pub struct QuitItem(MenuItem);

impl QuitItem {
    #[must_use]
    pub fn new() -> Self {
        Self(MenuItem::with_label("Quit"))
    }
}

impl Default for QuitItem {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuItemSetup for QuitItem {
    type MenuItem = MenuItem;

    fn setup(&self, _: Rc<MenuItems>, _: Cooler) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        (&self.0, Some(self.0.connect_activate(|_| gtk::main_quit())))
    }
}
