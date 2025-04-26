mod cooler;

use std::rc::Rc;

pub use anyhow::Result as AnyResult;
pub use cooler::Cooler;
use gtk::{CheckMenuItem, MenuItem, traits::WidgetExt};
use rusb::{Error as RusbError, Result as RusbResult};

#[derive(Debug)]
pub struct MenuItems {
    pub speed_up_mi: MenuItem,
    pub speed_down_mi: MenuItem,
    pub color_mi: MenuItem,
    pub power_mi: CheckMenuItem,
    pub led_mi: CheckMenuItem,
}

impl MenuItems {
    pub fn set_sensitive(&self, sensitive: bool) {
        self.speed_up_mi.set_sensitive(sensitive);
        self.speed_down_mi.set_sensitive(sensitive);
        self.color_mi.set_sensitive(sensitive);
        self.power_mi.set_sensitive(sensitive);
        self.led_mi.set_sensitive(sensitive);
    }
}

pub async fn track_state(cooler: Rc<Cooler>, menu_items: Rc<MenuItems>) {
    loop {
        match cooler.recv_state() {
            Ok(state) => todo!(),
            Err(_) => todo!(),
        }
    }
}
