mod device;
mod exactly_one;
mod fd_callbacks;
mod indicator;
mod menu;

use std::{cell::Cell, rc::Rc, time::Duration};

pub use anyhow::Result as AnyResult;
pub use device::Device;
use futures_util::TryStreamExt;
pub use glib::spawn_future_local as spawn;
use gtk::glib::{self, SignalHandlerId};
pub use indicator::Indicator;
pub use menu::item::{quit::QuitItem, speed_label::SpeedLabelItem};
use shared::{DeviceCommand, FanSpeed};
use systemstat::{Platform, System};
use tracing::instrument;

use crate::menu::MenuItems;

#[instrument(err(Debug))]
pub async fn process_device_state(
    device: Device,
    menu_items: Rc<MenuItems>,
    fan_speed: Rc<Cell<FanSpeed>>,
    mut speed_label: SpeedLabelItem,
    power_handler_id: SignalHandlerId,
    leds_handler_id: SignalHandlerId,
) -> AnyResult<()> {
    {
        let mut state_stream = device.state_stream()?;

        while let Some(device_state) = state_stream.try_next().await? {
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
                device.send_command(command).await?;
                continue;
            }

            menu_items.refresh_sensitivity();
        }

        Ok(())
    }
}

#[instrument(err(Debug))]
pub async fn speed_auto_task(
    device: Device,
    menu_items: Rc<MenuItems>,
    fan_speed: Rc<Cell<FanSpeed>>,
) -> AnyResult<()> {
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
                device.send_command(command).await?;
            }
        }
    }
}
