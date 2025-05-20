use std::{cell::Cell, fmt::Debug, rc::Rc};

use gtk::{
    Menu, SeparatorMenuItem,
    traits::{MenuShellExt, WidgetExt},
};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use shared::FanSpeed;
use tracing::instrument;

use crate::{AnyResult, Device, menu::MenuItems};

/// The system tray icon UI indicator.
///
/// Somewhat equivalent to a [`gtk::Application`], in that it takes care of
/// setting up `gtk` related stuff under the hood and blocks the current thread
/// when ran.
pub struct Indicator(AppIndicator);

impl Indicator {
    /// Creates the tray [`Indicator`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if [`gtk::init`] fails.
    #[instrument(err(Debug))]
    pub fn new() -> AnyResult<Self> {
        gtk::init()?;

        let mut app_indicator = AppIndicator::new("CoolerThanYou tray icon", "");
        app_indicator.set_status(AppIndicatorStatus::Active);
        app_indicator.set_icon_theme_path("");
        app_indicator.set_icon_full("cooler-than-you", "icon");

        Ok(Self(app_indicator))
    }

    /// Blocks the current thread by calling [`gtk::main`] to run the event loop.
    pub fn run(mut self, device: Device) {
        let mut menu = Menu::new();
        let menu_items = MenuItems::new(device.clone());

        menu.append(menu_items.speed_label.as_ref());
        menu.append(&SeparatorMenuItem::new());
        menu.append(menu_items.speed_auto.as_ref());
        menu.append(menu_items.speed_up.as_ref());
        menu.append(menu_items.speed_down.as_ref());
        menu.append(&SeparatorMenuItem::new());
        menu.append(menu_items.leds.as_ref());
        menu.append(menu_items.leds_change_color.as_ref());
        menu.append(&SeparatorMenuItem::new());
        menu.append(menu_items.power.as_ref());
        menu.append(&SeparatorMenuItem::new());
        menu.append(menu_items.quit.as_ref());

        // We send the commands this way so that the time between them being sent and read is
        // minimal and happens as soon as the event loop is started.
        crate::spawn(crate::power_cycle_device(device.clone()));

        // This will self-adjust, we just start with the lowest speed.
        // An [`Rc<Cell<FanSpeed>>`] is used here to share the value between the speed auto task and
        // the main background task because FanSpeed is [`Copy`].
        let fan_speed = Rc::new(Cell::new(FanSpeed::Speed1));

        crate::spawn(crate::speed_auto_task(
            device.clone(),
            menu_items.clone(),
            fan_speed.clone(),
        ));

        crate::spawn(crate::process_device_state(device, menu_items, fan_speed));

        menu.show_all();
        self.0.set_menu(&mut menu);

        gtk::main();
    }
}

impl Debug for Indicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Indicator").finish()
    }
}
