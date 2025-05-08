use std::{cell::Cell, rc::Rc};

use gtk::{
    CheckMenuItem,
    glib::{self, ControlFlow, SignalHandlerId, SourceId},
    traits::{CheckMenuItemExt, GtkMenuItemExt},
};
use shared::DeviceCommand;
use systemstat::{Platform, System};

use crate::{
    Cooler,
    menu::{
        MenuItems,
        item::{CustomMenuItem, ItemLabel, MenuItemSetup},
    },
};

pub type SpeedAutoItem = CustomMenuItem<CheckMenuItem, SpeedAuto>;

#[derive(Clone, Copy, Debug)]
pub struct SpeedAuto;

impl ItemLabel for SpeedAuto {
    const LABEL: &str = "Auto fan speed";
}

impl MenuItemSetup for SpeedAutoItem {
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
