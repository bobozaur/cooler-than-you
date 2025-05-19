use gtk::{MenuItem, traits::GtkMenuItemExt};

use crate::menu::item::CustomMenuItem;

pub type QuitItem = CustomMenuItem<MenuItem, Quit>;

#[derive(Clone, Copy, Debug)]
pub struct Quit;

impl Default for QuitItem {
    fn default() -> Self {
        let inner = MenuItem::with_label("Quit");
        inner.connect_activate(|_| gtk::main_quit());
        Self { inner, kind: Quit }
    }
}
