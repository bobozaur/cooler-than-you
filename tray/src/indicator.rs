use std::{fmt::Debug, rc::Rc};

use gtk::{
    Menu,
    glib::SignalHandlerId,
    traits::{MenuShellExt, WidgetExt},
};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use tracing::instrument;

use crate::{
    AnyResult, Device,
    menu::{MenuItems, item::MenuItemSetup},
};

#[derive(Debug)]
pub struct Indicator {
    inner: InnerIndicator,
    menu: Menu,
    menu_items: Rc<MenuItems>,
}

impl Indicator {
    /// Creates the tray [`Indicator`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if [`gtk::init`] fails.
    #[instrument(err(Debug))]
    pub fn new() -> AnyResult<Self> {
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
        })
    }

    pub fn add_menu_item<MI>(&mut self, menu_item: &MI, device: Device) -> Option<SignalHandlerId>
    where
        MI: MenuItemSetup,
    {
        let (mi, handler_id) = menu_item.setup(self.menu_items.clone(), device);
        self.menu.append(mi);
        handler_id
    }

    #[must_use]
    pub fn menu_items(&self) -> &Rc<MenuItems> {
        &self.menu_items
    }

    /// Blocks the current thread by calling [`gtk::main`] to run the event loop.
    pub fn run(mut self) {
        self.menu.show_all();
        self.inner.0.set_menu(&mut self.menu);

        gtk::main();
    }
}

/// Wrapper used for easier [`Debug`] impl of [`Indicator`].
struct InnerIndicator(AppIndicator);

impl Debug for InnerIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerIndicator").finish()
    }
}
