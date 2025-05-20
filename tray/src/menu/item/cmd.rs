use std::{cell::OnceCell, rc::Weak};

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

/// Actionable item that sends a [`DeviceCommand::SpeedUp`] when clicked.
///
/// Disabled when [`crate::menu::item::SpeedAutoItem`] is active or when
/// [`PowerToggleItem`] is inactive.
pub type SpeedUpItem = CustomMenuItem<MenuItem, SpeedUp>;
/// Actionable item that sends a [`DeviceCommand::SpeedDown`] when clicked.
///
/// Disabled when [`crate::menu::item::SpeedAutoItem`] is active or when
/// [`PowerToggleItem`] is inactive.
pub type SpeedDownItem = CustomMenuItem<MenuItem, SpeedDown>;
/// Actionable item that sends a [`DeviceCommand::LedsColorChange`] when clicked.
///
/// Disabled when [`LedsToggleItem`] is inactive.
pub type LedsChangeColorItem = CustomMenuItem<MenuItem, LedsChangeColor>;
/// Actionable checkbox item that sends a [`DeviceCommand::LedsOn`]/[`DeviceCommand::LedsOff`]
/// if checked/unchecked, respectively.
pub type LedsToggleItem = CustomMenuItem<CheckMenuItem, LedsToggle>;
/// Actionable checkbox item that sends a [`DeviceCommand::PowerOn`]/[`DeviceCommand::PowerOff`]
/// if checked/unchecked, respectively.
pub type PowerToggleItem = CustomMenuItem<CheckMenuItem, PowerToggle>;

/// Trait used for abstracting away the custom behavior of each command-issuing menu item kind.
pub trait CommandItemKind {
    /// Menu item text label.
    const LABEL: &str;
    /// Device command to issue when clicked.
    const COMMAND: DeviceCommand;
    const THIS: Self;
}

#[derive(Clone, Copy, Debug)]
pub struct SpeedUp;

impl CommandItemKind for SpeedUp {
    const LABEL: &str = "Increase fan speed";
    const COMMAND: DeviceCommand = DeviceCommand::SpeedUp;
    const THIS: Self = Self;
}

#[derive(Clone, Copy, Debug)]
pub struct SpeedDown;

impl CommandItemKind for SpeedDown {
    const LABEL: &str = "Decrease fan speed";
    const COMMAND: DeviceCommand = DeviceCommand::SpeedDown;
    const THIS: Self = Self;
}

#[derive(Clone, Copy, Debug)]
pub struct LedsChangeColor;

impl CommandItemKind for LedsChangeColor {
    const LABEL: &str = "Change lights color";
    const COMMAND: DeviceCommand = DeviceCommand::LedsColorChange;
    const THIS: Self = Self;
}

impl<MI, K> CustomMenuItem<MI, K>
where
    K: CommandItemKind,
    MI: Default + GtkMenuItemExt,
{
    pub fn new(menu_items: Weak<MenuItems>, device: Device) -> Self {
        let inner = MI::default();
        inner.set_label(K::LABEL);
        let cache = OnceCell::new();

        inner.connect_activate(move |_| {
            // Cache the weak pointer upgrade so as not to do it every time.
            let cache_fn = || menu_items.upgrade().expect("menu items are never dropped");
            cache.get_or_init(cache_fn).disable();

            let device = device.clone();
            crate::spawn(async move { device.send_command(K::COMMAND).await });
        });

        let kind = K::THIS;
        Self { inner, kind }
    }
}

/// Trait used for abstracting away the custom behavior of each command-issuing menu item kind.
pub trait CheckedCommandItemKind: From<SignalHandlerId> {
    /// Menu item text label.
    const LABEL: &str;

    /// Device command to issue when checked.
    const ACTIVE_COMMAND: DeviceCommand;
    /// Device command to issue when unchecked.
    const INACTIVE_COMMAND: DeviceCommand;
}

#[derive(Debug)]
pub struct LedsToggle(SignalHandlerId);

impl CheckedCommandItemKind for LedsToggle {
    const LABEL: &str = "Lights";
    const ACTIVE_COMMAND: DeviceCommand = DeviceCommand::LedsOn;
    const INACTIVE_COMMAND: DeviceCommand = DeviceCommand::LedsOff;
}

impl From<SignalHandlerId> for LedsToggle {
    fn from(value: SignalHandlerId) -> Self {
        Self(value)
    }
}

impl AsRef<SignalHandlerId> for LedsToggle {
    fn as_ref(&self) -> &SignalHandlerId {
        &self.0
    }
}

#[derive(Debug)]
pub struct PowerToggle(SignalHandlerId);

impl CheckedCommandItemKind for PowerToggle {
    const LABEL: &str = "Power";
    const ACTIVE_COMMAND: DeviceCommand = DeviceCommand::PowerOn;
    const INACTIVE_COMMAND: DeviceCommand = DeviceCommand::PowerOff;
}

impl From<SignalHandlerId> for PowerToggle {
    fn from(value: SignalHandlerId) -> Self {
        Self(value)
    }
}

impl AsRef<SignalHandlerId> for PowerToggle {
    fn as_ref(&self) -> &SignalHandlerId {
        &self.0
    }
}

impl<MI, K> CustomMenuItem<MI, K>
where
    K: CheckedCommandItemKind,
    MI: Default + GtkMenuItemExt + CheckMenuItemExt,
{
    // NOTE: A different name has be to used to avoid conflicts with the method
    //       constructing simple items (without a checkbox) defined in this module.
    pub fn new_checkbox(menu_items: Weak<MenuItems>, device: Device) -> Self {
        let inner = MI::default();
        inner.set_label(K::LABEL);
        let cache = OnceCell::new();

        let signal_handler_id = inner.connect_activate(move |mi| {
            // Cache the weak pointer upgrade so as not to do it every time.
            let cache_fn = || menu_items.upgrade().expect("menu items are never dropped");
            cache.get_or_init(cache_fn).disable();

            let command = if mi.is_active() {
                K::ACTIVE_COMMAND
            } else {
                K::INACTIVE_COMMAND
            };

            let device = device.clone();
            crate::spawn(async move { device.send_command(command).await });
        });

        let kind = K::from(signal_handler_id);
        Self { inner, kind }
    }
}
