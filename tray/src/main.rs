//! TODO:
//! - disable device unused hardware
//! - check error handling
//! - logging
//! - debian packaging
//! - comments and docs
//! - fancy icon

use std::{cell::Cell, rc::Rc, time::Duration};

use anyhow::Context;
use futures_util::StreamExt;
use gtk::{SeparatorMenuItem, glib};
use shared::{DeviceCommand, FanSpeed};
use systemstat::{Platform, System};
use tray::{AnyResult, Indicator, QuitItem, SpeedLabelItem};

fn main() -> AnyResult<()> {
    let mut indicator = Indicator::new()?;
    let device = indicator.device().clone();
    let menu_items = indicator.menu_items().clone();

    let mut speed_label = SpeedLabelItem::new();
    let quit_mi = QuitItem::new();

    indicator.add_menu_item(&speed_label);
    indicator.add_menu_item(&SeparatorMenuItem::new());
    indicator.add_menu_item(&menu_items.speed_auto);
    indicator.add_menu_item(&menu_items.speed_up);
    indicator.add_menu_item(&menu_items.speed_down);
    indicator.add_menu_item(&SeparatorMenuItem::new());
    let leds_handler_id = indicator
        .add_menu_item(&menu_items.leds)
        .context("missing lights callback handler id")?;
    indicator.add_menu_item(&menu_items.leds_change_color);
    indicator.add_menu_item(&SeparatorMenuItem::new());
    let power_handler_id = indicator
        .add_menu_item(&menu_items.power)
        .context("missing power callback handler id")?;
    indicator.add_menu_item(&SeparatorMenuItem::new());
    indicator.add_menu_item(&quit_mi);

    // This will self-adjust, we just start with the lowest speed.
    // An [`Rc<Cell<FanSpeed>>`] is used here to share the value
    // between the temperature monitor task and the main background task
    // because FanSpeed is [`Copy`].
    let fan_speed = Rc::new(Cell::new(FanSpeed::Speed1));

    {
        let fan_speed = fan_speed.clone();
        let device = device.clone();
        let menu_items = menu_items.clone();

        // Frequency with which this is called is determined by the timeout value in
        // [`Device::recv_state`].
        glib::spawn_future_local(async move {
            let mut state_stream = device.state_stream().unwrap();

            while let Some(state_opt) = state_stream.next().await {
                let device_state = state_opt.unwrap();

                let speed = device_state.fan_speed();
                fan_speed.replace(speed);
                speed_label.update_speed(speed);

                menu_items
                    .power
                    .set_active(device_state.power_enabled(), &power_handler_id);
                menu_items
                    .leds
                    .set_active(device_state.leds_enabled(), &leds_handler_id);

                if let Some(command) = device_state.command_to_repeat() {
                    device.send_command(command).await.unwrap();
                    continue;
                }

                menu_items.refresh_sensitivity();
            }
        });
    }

    glib::spawn_future_local(async move {
        let system = System::new();

        loop {
            glib::timeout_future(Duration::from_secs(1)).await;

            if !menu_items.speed_auto.is_active() {
                continue;
            }

            if let (Ok(temp), fan_speed) = (system.cpu_temp(), fan_speed.get()) {
                let command = match fan_speed {
                    FanSpeed::Speed1 if temp > 60.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed2 if temp > 65.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed3 if temp > 70.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed4 if temp > 75.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed5 if temp > 80.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed6 if temp > 80.0 => None,
                    FanSpeed::Speed6 => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed5 if temp < 75.0 => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed4 if temp < 70.0 => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed3 if temp < 65.0 => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed2 if temp < 60.0 => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed5
                    | FanSpeed::Speed4
                    | FanSpeed::Speed3
                    | FanSpeed::Speed2
                    | FanSpeed::Speed1 => None,
                };

                if let Some(command) = command {
                    device.send_command(command).await.unwrap();
                }
            }
        }
    });

    indicator.run();
    Ok(())
}
