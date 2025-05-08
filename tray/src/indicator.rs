use std::{fmt::Debug, rc::Rc};

use gtk::{
    Menu,
    glib::{self, SignalHandlerId},
    traits::{MenuShellExt, WidgetExt},
};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use shared::DeviceCommand;

use crate::{
    AnyResult, Cooler,
    menu::{MenuItems, item::MenuItemSetup},
};

#[derive(Debug)]
pub struct Indicator {
    inner: InnerIndicator,
    menu: Menu,
    menu_items: Rc<MenuItems>,
    device: Cooler,
}

impl Indicator {
    ///
    /// # Errors
    pub fn new() -> AnyResult<Self> {
        let device = Cooler::new()?;

        gtk::init()?;

        let menu_items = Rc::new(MenuItems::new());
        let menu = Menu::new();

        let mut app_indicator = AppIndicator::new("CoolerThanYou tray icon", "");
        app_indicator.set_status(AppIndicatorStatus::Active);
        app_indicator.set_icon_theme_path("");
        app_indicator.set_icon_full("cooler-than-you", "icon");
        let inner = InnerIndicator(app_indicator);

        Ok(Self {
            inner,
            menu,
            menu_items,
            device,
        })
    }

    pub fn add_menu_item<MI>(&mut self, menu_item: &MI) -> Option<SignalHandlerId>
    where
        MI: MenuItemSetup,
    {
        let (mi, handler_id) = menu_item.setup(self.menu_items.clone(), self.device.clone());
        self.menu.append(mi);
        handler_id
    }

    #[must_use]
    pub fn device(&self) -> &Cooler {
        &self.device
    }

    #[must_use]
    pub fn menu_items(&self) -> &Rc<MenuItems> {
        &self.menu_items
    }

    pub fn run(mut self) {
        self.menu.show_all();
        self.inner.0.set_menu(&mut self.menu);

        // Power cycle the device to ensure it's on.
        // If it's already off, the first command will be a no-op.
        glib::idle_add_local_once({
            let device = self.device.clone();
            move || {
                device.send_command(DeviceCommand::PowerOff).ok();
                device.send_command(DeviceCommand::PowerOn).ok();
            }
        });

        gtk::main();
    }
}

struct InnerIndicator(AppIndicator);

impl Debug for InnerIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InnerIndicator")
    }
}
