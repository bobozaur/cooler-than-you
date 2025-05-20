use std::{fmt::Debug, rc::Rc};

use futures_util::TryStreamExt;
use gtk::{
    Menu, SeparatorMenuItem,
    traits::{MenuShellExt, WidgetExt},
};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use shared::DeviceCommand;
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
        crate::spawn_local(Self::power_cycle_device(device.clone()));

        // Spawn background task.
        crate::spawn_local(Self::background_task(device, menu_items));

        menu.show_all();
        self.0.set_menu(&mut menu);

        gtk::main();
    }

    /// Power cycle the device to ensure it's on.
    /// If it's already off, the first command will be a no-op.
    #[instrument(skip_all, err(Debug))]
    async fn power_cycle_device(device: Device) -> AnyResult<()> {
        device.send_command(DeviceCommand::PowerOff).await?;
        device.send_command(DeviceCommand::PowerOn).await?;
        Ok(())
    }

    #[instrument(skip_all, err(Debug))]
    async fn background_task(device: Device, menu_items: Rc<MenuItems>) -> AnyResult<()> {
        let mut state_stream = device.state_stream()?;

        while let Some(device_state) = state_stream.try_next().await? {
            tracing::info!("received state: {device_state:?}");

            let speed = device_state.fan_speed();
            menu_items.speed_label.update_label(speed);
            menu_items.speed_auto.register_speed(speed);

            menu_items.power.set_active(device_state.power_enabled());
            menu_items.leds.set_active(device_state.leds_enabled());

            if let Some(command) = device_state.command_to_repeat() {
                device.send_command(command).await?;
                continue;
            }

            menu_items.refresh_sensitivity();
        }

        Ok(())
    }
}

impl Debug for Indicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Indicator").finish()
    }
}
