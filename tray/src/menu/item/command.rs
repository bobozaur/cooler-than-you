use std::rc::Weak;

use gtk::{
    CheckMenuItem, MenuItem,
    glib::SignalHandlerId,
    traits::{CheckMenuItemExt, GtkMenuItemExt},
};
use shared::DeviceCommand;

use crate::{
    Device,
    menu::{MenuItems, item::CustomMenuItem},
};

pub type SpeedUpItem = CustomMenuItem<MenuItem, SpeedUp>;
pub type SpeedDownItem = CustomMenuItem<MenuItem, SpeedDown>;
pub type LedsChangeColorItem = CustomMenuItem<MenuItem, LedsChangeColor>;
pub type LedsToggleItem = CustomMenuItem<CheckMenuItem, LedsToggle>;
pub type PowerToggleItem = CustomMenuItem<CheckMenuItem, PowerToggle>;

pub trait CommandItemKind {
    const LABEL: &str;

    type MenuItem: Default + GtkMenuItemExt;

    fn new(handler_id: SignalHandlerId) -> Self;

    fn command(mi: &Self::MenuItem) -> DeviceCommand;
}

#[derive(Clone, Copy, Debug)]
pub struct SpeedUp;

impl CommandItemKind for SpeedUp {
    const LABEL: &str = "Increase fan speed";

    type MenuItem = MenuItem;

    fn new(_: SignalHandlerId) -> Self {
        Self
    }

    fn command(_: &Self::MenuItem) -> DeviceCommand {
        DeviceCommand::SpeedUp
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpeedDown;

impl CommandItemKind for SpeedDown {
    const LABEL: &str = "Decrease fan speed";

    type MenuItem = MenuItem;

    fn new(_: SignalHandlerId) -> Self {
        Self
    }

    fn command(_: &Self::MenuItem) -> DeviceCommand {
        DeviceCommand::SpeedDown
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LedsChangeColor;

impl CommandItemKind for LedsChangeColor {
    const LABEL: &str = "Change lights color";

    type MenuItem = MenuItem;

    fn new(_: SignalHandlerId) -> Self {
        Self
    }

    fn command(_: &Self::MenuItem) -> DeviceCommand {
        DeviceCommand::LedsColorChange
    }
}

#[derive(Debug)]
pub struct LedsToggle(SignalHandlerId);

impl CommandItemKind for LedsToggle {
    const LABEL: &str = "Lights";

    type MenuItem = CheckMenuItem;

    fn new(handler_id: SignalHandlerId) -> Self {
        Self(handler_id)
    }

    fn command(mi: &Self::MenuItem) -> DeviceCommand {
        if mi.is_active() {
            DeviceCommand::LedsOn
        } else {
            DeviceCommand::LedsOff
        }
    }
}

impl AsRef<SignalHandlerId> for LedsToggle {
    fn as_ref(&self) -> &SignalHandlerId {
        &self.0
    }
}

#[derive(Debug)]
pub struct PowerToggle(SignalHandlerId);

impl CommandItemKind for PowerToggle {
    const LABEL: &str = "Power";

    type MenuItem = CheckMenuItem;

    fn new(handler_id: SignalHandlerId) -> Self {
        Self(handler_id)
    }

    fn command(mi: &Self::MenuItem) -> DeviceCommand {
        if mi.is_active() {
            DeviceCommand::PowerOn
        } else {
            DeviceCommand::PowerOff
        }
    }
}

impl AsRef<SignalHandlerId> for PowerToggle {
    fn as_ref(&self) -> &SignalHandlerId {
        &self.0
    }
}

impl<MI, K> CustomMenuItem<MI, K>
where
    K: CommandItemKind<MenuItem = MI>,
    MI: Default + GtkMenuItemExt,
{
    pub fn new(menu_items: Weak<MenuItems>, device: Device) -> Self {
        let inner = MI::default();
        inner.set_label(K::LABEL);

        let signal_handler_id = inner.connect_activate(move |mi| {
            menu_items
                .upgrade()
                .expect("menu items are never dropped")
                .disable();

            let command = K::command(mi);
            let device = device.clone();
            crate::spawn(async move { device.send_command(command).await });
        });

        let kind = K::new(signal_handler_id);
        Self { inner, kind }
    }
}
