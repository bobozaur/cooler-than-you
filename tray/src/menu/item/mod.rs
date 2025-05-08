pub mod command;
pub mod quit;
pub mod speed_auto;
pub mod speed_label;

use std::{marker::PhantomData, rc::Rc};

use gtk::{
    CheckMenuItem, MenuItem, SeparatorMenuItem,
    glib::{IsA, ObjectExt, SignalHandlerId},
    traits::{CheckMenuItemExt, GtkMenuItemExt, WidgetExt},
};

use crate::{Cooler, menu::MenuItems};

pub trait ItemLabel {
    const LABEL: &str;
}

pub trait MenuItemSetup {
    type MenuItem: IsA<MenuItem>;

    fn setup(
        &self,
        menu_items: Rc<MenuItems>,
        device: Cooler,
    ) -> (&Self::MenuItem, Option<SignalHandlerId>);
}

impl MenuItemSetup for SeparatorMenuItem {
    type MenuItem = Self;

    fn setup(&self, _: Rc<MenuItems>, _: Cooler) -> (&Self::MenuItem, Option<SignalHandlerId>) {
        (self, None)
    }
}

#[derive(Debug)]
pub struct CustomMenuItem<MI, CMD> {
    inner: MI,
    marker: PhantomData<fn() -> CMD>,
}

impl<MI, CMD> CustomMenuItem<MI, CMD>
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

impl<MI, CMD> CustomMenuItem<MI, CMD>
where
    MI: WidgetExt,
{
    pub fn set_sensitive(&self, flag: bool) {
        self.inner.set_sensitive(flag);
    }
}

impl<CMD> CustomMenuItem<CheckMenuItem, CMD> {
    pub fn set_active(&self, is_active: bool, handler_id: &SignalHandlerId) {
        self.inner.block_signal(handler_id);
        self.inner.set_active(is_active);
        self.inner.unblock_signal(handler_id);
    }

    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }
}

impl<MI, CMD> Default for CustomMenuItem<MI, CMD>
where
    CMD: ItemLabel,
    MI: Default + GtkMenuItemExt,
{
    fn default() -> Self {
        Self::new()
    }
}
