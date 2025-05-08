use std::rc::Rc;

use gtk::{
    CheckMenuItem, MenuItem,
    glib::SignalHandlerId,
    traits::{CheckMenuItemExt, GtkMenuItemExt},
};
use shared::DeviceCommand;

use crate::{
    Cooler,
    menu::{
        MenuItems,
        item::{CustomMenuItem, ItemLabel, MenuItemSetup},
    },
};

pub type SpeedUpItem = CustomMenuItem<MenuItem, SpeedUp>;
pub type SpeedDownItem = CustomMenuItem<MenuItem, SpeedDown>;
pub type LedsChangeColorItem = CustomMenuItem<MenuItem, LedsChangeColor>;
pub type LedsToggleItem = CustomMenuItem<CheckMenuItem, LedsToggle>;
pub type PowerToggleItem = CustomMenuItem<CheckMenuItem, PowerToggle>;

#[derive(Clone, Copy, Debug)]
pub struct SpeedUp;
#[derive(Clone, Copy, Debug)]
pub struct SpeedDown;
#[derive(Clone, Copy, Debug)]
pub struct LedsChangeColor;
#[derive(Clone, Copy, Debug)]
pub struct LedsToggle;
#[derive(Clone, Copy, Debug)]
pub struct PowerToggle;

pub trait CommandItem: ItemLabel {
    type MenuItem: Default + GtkMenuItemExt;

    fn command(mi: &Self::MenuItem) -> DeviceCommand;
}

impl ItemLabel for SpeedUp {
    const LABEL: &str = "Increase fan speed";
}

impl CommandItem for SpeedUp {
    type MenuItem = MenuItem;

    fn command(_: &Self::MenuItem) -> DeviceCommand {
        DeviceCommand::SpeedUp
    }
}

impl ItemLabel for SpeedDown {
    const LABEL: &str = "Decrease fan speed";
}

impl CommandItem for SpeedDown {
    type MenuItem = MenuItem;

    fn command(_: &Self::MenuItem) -> DeviceCommand {
        DeviceCommand::SpeedDown
    }
}

impl ItemLabel for LedsChangeColor {
    const LABEL: &str = "Change lights color";
}

impl CommandItem for LedsChangeColor {
    type MenuItem = MenuItem;

    fn command(_: &Self::MenuItem) -> DeviceCommand {
        DeviceCommand::LedsColorChange
    }
}

impl ItemLabel for LedsToggle {
    const LABEL: &str = "Lights";
}

impl CommandItem for LedsToggle {
    type MenuItem = CheckMenuItem;

    fn command(mi: &Self::MenuItem) -> DeviceCommand {
        if mi.is_active() {
            DeviceCommand::LedsOn
        } else {
            DeviceCommand::LedsOff
        }
    }
}

impl ItemLabel for PowerToggle {
    const LABEL: &str = "Power";
}

impl CommandItem for PowerToggle {
    type MenuItem = CheckMenuItem;

    fn command(mi: &Self::MenuItem) -> DeviceCommand {
        if mi.is_active() {
            DeviceCommand::PowerOn
        } else {
            DeviceCommand::PowerOff
        }
    }
}

impl<MI, CMD> MenuItemSetup for CustomMenuItem<MI, CMD>
where
    CMD: CommandItem<MenuItem = MI>,
    MI: GtkMenuItemExt,
{
    type MenuItem = MI;

    fn setup(
        &self,
        menu_items: Rc<MenuItems>,
        device: Cooler,
    ) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        let handler_id = self.inner.connect_activate(move |mi| {
            menu_items.disable();
            if device.send_command(CMD::command(mi)).is_err() {}
        });

        (&self.inner, Some(handler_id))
    }
}
