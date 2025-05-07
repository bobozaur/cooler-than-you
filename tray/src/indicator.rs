use gtk::{
    Menu, MenuItem,
    glib::IsA,
    traits::{MenuShellExt, WidgetExt},
};
use libappindicator::{AppIndicator, AppIndicatorStatus};

use crate::AnyResult;

#[allow(missing_debug_implementations)]
pub struct Indicator {
    inner: AppIndicator,
    menu: Menu,
}

impl Indicator {
    ///
    /// # Errors
    pub fn new() -> AnyResult<Self> {
        gtk::init()?;

        let menu = Menu::new();
        let mut inner = AppIndicator::new("CoolerThanYou tray icon", "");
        inner.set_status(AppIndicatorStatus::Active);
        inner.set_icon_theme_path("");
        inner.set_icon_full("cooler-than-you", "icon");

        Ok(Self { inner, menu })
    }

    pub fn add_menu_item(&mut self, menu_item: &impl IsA<MenuItem>) {
        self.menu.append(menu_item);
    }

    pub fn run(mut self) {
        self.menu.show_all();
        self.inner.set_menu(&mut self.menu);
        gtk::main();
    }
}
