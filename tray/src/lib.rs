mod cooler;

pub use anyhow::Result as AnyResult;
pub use cooler::Cooler;
use gtk::{
    CheckMenuItem, MenuItem,
    glib::{ObjectExt, SignalHandlerId},
    traits::{CheckMenuItemExt, WidgetExt},
};

#[derive(Debug)]
pub struct MenuItems {
    pub speed_up_mi: MenuItem,
    pub speed_down_mi: MenuItem,
    pub color_mi: MenuItem,
    pub power_mi: CheckMenuItem,
    pub leds_mi: CheckMenuItem,
}

impl MenuItems {
    pub fn set_sensitive(&self, sensitive: bool) {
        self.speed_up_mi.set_sensitive(sensitive);
        self.speed_down_mi.set_sensitive(sensitive);
        self.color_mi.set_sensitive(sensitive);
        self.power_mi.set_sensitive(sensitive);
        self.leds_mi.set_sensitive(sensitive);
    }
}

pub fn track_state(
    cooler: &Cooler,
    menu_items: &MenuItems,
    power_sigh_id: &SignalHandlerId,
    leds_sigh_id: &SignalHandlerId,
) {
    let Ok(Some(device_state)) = cooler.recv_state() else {
        return;
    };

    println!("{device_state:?}");

    menu_items.power_mi.block_signal(power_sigh_id);
    menu_items.power_mi.set_active(device_state.power_enabled());
    menu_items.power_mi.unblock_signal(power_sigh_id);

    menu_items.leds_mi.block_signal(leds_sigh_id);
    menu_items.leds_mi.set_active(device_state.leds_enabled());
    menu_items.leds_mi.unblock_signal(leds_sigh_id);

    menu_items.set_sensitive(true);
}
