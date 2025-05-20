mod device;
mod exactly_one;
mod fd_callbacks;
mod indicator;
mod menu;

use std::{cell::Cell, rc::Rc, time::Duration};

pub use anyhow::Result as AnyResult;
pub use device::Device;
use futures_util::{TryFutureExt, TryStreamExt};
use gtk::glib;
pub use indicator::Indicator;
use shared::{DeviceCommand, FanSpeed};
use systemstat::{Platform, System};
use tracing::instrument;

use crate::menu::MenuItems;

/// Spawns a fallible future on the event loop, quiting it by calling
/// [`gtk::main_quit`] if the future returns an error.
pub fn spawn<F>(fut: F)
where
    F: TryFutureExt + 'static,
{
    glib::spawn_future_local(fut.map_err(|_| gtk::main_quit()));
}

/// Power cycle the device to ensure it's on.
/// If it's already off, the first command will be a no-op.
#[instrument(skip_all, err(Debug))]
pub async fn power_cycle_device(device: Device) -> AnyResult<()> {
    device.send_command(DeviceCommand::PowerOff).await?;
    device.send_command(DeviceCommand::PowerOn).await?;
    Ok(())
}

#[instrument(skip_all, err(Debug))]
pub async fn process_device_state(
    device: Device,
    menu_items: Rc<MenuItems>,
    fan_speed: Rc<Cell<FanSpeed>>,
) -> AnyResult<()> {
    {
        let mut state_stream = device.state_stream()?;

        while let Some(device_state) = state_stream.try_next().await? {
            tracing::info!("received state: {device_state:?}");

            let speed = device_state.fan_speed();
            fan_speed.replace(speed);
            menu_items.speed_label.update(speed);

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

#[instrument(skip_all, err(Debug))]
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
                tracing::info!("CPU temp: {temp}, fan speed: {fan_speed:?}");
                device.send_command(command).await?;
            }
        }
    }
}
