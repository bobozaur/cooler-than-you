use std::{cell::Cell, marker::PhantomData, rc::Rc};

use gtk::{
    CheckMenuItem, MenuItem, SeparatorMenuItem,
    glib::{self, ControlFlow, IsA, ObjectExt, SignalHandlerId, SourceId},
    traits::{CheckMenuItemExt, GtkMenuItemExt, WidgetExt},
};
use shared::{DeviceCommand, FanSpeed};
use systemstat::{Platform, System};

use super::MenuItems;
use crate::Cooler;

pub type SpeedAutoItem = Item<CheckMenuItem, SpeedAuto>;
pub type SpeedUpItem = Item<MenuItem, SpeedUp>;
pub type SpeedDownItem = Item<MenuItem, SpeedDown>;
pub type LedsChangeColorItem = Item<MenuItem, LedsChangeColor>;
pub type LedsToggleItem = Item<CheckMenuItem, LedsToggle>;
pub type PowerToggleItem = Item<CheckMenuItem, PowerToggle>;

pub trait AddableMenuItem {
    type MenuItem: IsA<MenuItem>;

    fn setup(
        &self,
        menu_items: Rc<MenuItems>,
        device: Cooler,
    ) -> (&Self::MenuItem, Option<SignalHandlerId>);
}

impl AddableMenuItem for SeparatorMenuItem {
    type MenuItem = Self;

    fn setup(&self, _: Rc<MenuItems>, _: Cooler) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        (self, None)
    }
}

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

impl AddableMenuItem for SpeedLabelItem {
    type MenuItem = MenuItem;

    fn setup(&self, _: Rc<MenuItems>, _: Cooler) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        (&self.inner, None)
    }
}

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

impl AddableMenuItem for QuitItem {
    type MenuItem = MenuItem;

    fn setup(&self, _: Rc<MenuItems>, _: Cooler) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        (&self.0, Some(self.0.connect_activate(|_| gtk::main_quit())))
    }
}

impl AddableMenuItem for SpeedAutoItem {
    type MenuItem = CheckMenuItem;

    fn setup(
        &self,
        menu_items: Rc<MenuItems>,
        device: Cooler,
    ) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        let source_id: Cell<Option<SourceId>> = Cell::new(None);

        let handler_id = self.inner.connect_activate(move |mi| {
            menu_items.refresh_sensitivity();

            match source_id.replace(None) {
                Some(id) if !mi.is_active() => {
                    id.remove();
                }
                None if mi.is_active() => {
                    let callback = glib::timeout_add_seconds_local(5, {
                        let device = device.clone();

                        move || {
                            let system = System::new();
                            system.cpu_temp().ok();
                            device.send_command(DeviceCommand::SpeedUp).ok();
                            ControlFlow::Continue
                        }
                    });
                    source_id.replace(Some(callback));
                }
                _ => (),
            }
        });

        (&self.inner, Some(handler_id))
    }
}

#[derive(Debug)]
pub struct Item<MI, CMD> {
    inner: MI,
    marker: PhantomData<fn() -> CMD>,
}

impl<MI, CMD> Item<MI, CMD>
where
    CMD: ItemLabel,
    MI: Default + GtkMenuItemExt,
{
    pub fn new() -> Self {
        let inner = MI::default();
        inner.set_label(CMD::LABEL);

        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<MI, CMD> Item<MI, CMD>
where
    MI: WidgetExt,
{
    pub fn set_sensitive(&self, flag: bool) {
        self.inner.set_sensitive(flag);
    }
}

impl<CMD> Item<CheckMenuItem, CMD> {
    pub fn set_active(&self, is_active: bool, handler_id: &SignalHandlerId) {
        self.inner.block_signal(handler_id);
        self.inner.set_active(is_active);
        self.inner.unblock_signal(handler_id);
    }

    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }
}

impl<MI, CMD> Default for Item<MI, CMD>
where
    CMD: ItemLabel,
    MI: Default + GtkMenuItemExt,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<MI, CMD> AddableMenuItem for Item<MI, CMD>
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

#[derive(Clone, Copy, Debug)]
pub struct SpeedAuto;
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

pub trait ItemLabel {
    const LABEL: &str;
}

pub trait CommandItem: ItemLabel {
    type MenuItem: Default + GtkMenuItemExt;

    fn command(mi: &Self::MenuItem) -> DeviceCommand;
}

impl ItemLabel for SpeedAuto {
    const LABEL: &str = "Auto fan speed";
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
