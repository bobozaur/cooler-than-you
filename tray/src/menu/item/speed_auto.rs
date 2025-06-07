use std::{
    cell::{Cell, OnceCell},
    rc::{Rc, Weak},
};

use futures_util::StreamExt;
use gtk::{
    CheckMenuItem,
    glib::{self, JoinHandle},
    traits::{CheckMenuItemExt, GtkMenuItemExt},
};
use shared::{DeviceCommand, FanSpeed};
use systemstat::{Platform, System};
use tracing::instrument;

use crate::{
    AnyResult, Device,
    menu::{MenuItems, item::CustomMenuItem},
};

/// Actionable checkbox item that enables/disables the fan speed auto adjustment based on
/// temperature. This item is already active on start-up.
pub type SpeedAutoItem = CustomMenuItem<CheckMenuItem, SpeedAuto>;

#[derive(Clone, Debug)]
pub struct SpeedAuto(Rc<Cell<FanSpeed>>);

impl SpeedAutoItem {
    // NOTE: Used this name to be consistent with the other checkbox items
    //       construction method.
    pub fn new_checkbox(menu_items: Weak<MenuItems>, device: Device, fan_curve: [f32; 5]) -> Self {
        // This will self-adjust, we just start with the lowest speed.
        // An [`Rc<Cell<FanSpeed>>`] is used here to share the value between the speed auto task and
        // the main background task because of `gtk` callbacks trait bounds and because [`FanSpeed`]
        // is [`Copy`].
        let fan_speed = Rc::new(Cell::new(FanSpeed::Speed1));
        let kind = SpeedAuto(fan_speed.clone());

        let inner = CheckMenuItem::with_label("Auto fan speed");
        inner.set_active(true);
        let fut = Self::speed_auto_task(device.clone(), fan_speed.clone(), fan_curve);
        let join_handle: Cell<Option<JoinHandle<_>>> = Cell::new(Some(crate::spawn_local(fut)));
        let cache = OnceCell::new();

        inner.connect_activate(move |mi| {
            match join_handle.replace(None) {
                Some(h) => {
                    tracing::debug!("stopping speed auto task");
                    h.abort();
                }
                // Ensure the task is only spawned on activation.
                None if mi.is_active() => {
                    tracing::debug!("spawning speed auto task");
                    let fut = Self::speed_auto_task(device.clone(), fan_speed.clone(), fan_curve);
                    join_handle.set(Some(crate::spawn_local(fut)));
                }
                _ => tracing::warn!("no task found on item de-activation"),
            }

            // Cache the weak pointer upgrade so as not to do it every time.
            let cache_fn = || menu_items.upgrade().expect("menu items are never dropped");
            cache
                .get_or_init(cache_fn)
                .refresh_speed_items_sensitivity();
        });

        Self { inner, kind }
    }

    pub fn register_speed(&self, speed: FanSpeed) {
        self.kind.0.set(speed);
    }

    #[instrument(skip_all, err(Debug))]
    async fn speed_auto_task(
        device: Device,
        fan_speed: Rc<Cell<FanSpeed>>,
        fan_curve: [f32; 5],
    ) -> AnyResult<()> {
        let system = System::new();
        let mut ticker = glib::interval_stream_seconds(1);

        while let Some(()) = ticker.next().await {
            if let (Ok(temp), fan_speed) = (system.cpu_temp(), fan_speed.get()) {
                let command = match fan_speed {
                    FanSpeed::Speed1 if temp > fan_curve[0] => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed2 if temp > fan_curve[1] => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed3 if temp > fan_curve[2] => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed4 if temp > fan_curve[3] => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed5 if temp > fan_curve[4] => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed6 if temp > fan_curve[4] => None,
                    FanSpeed::Speed6 => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed5 if temp < fan_curve[3] => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed4 if temp < fan_curve[2] => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed3 if temp < fan_curve[1] => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed2 if temp < fan_curve[0] => Some(DeviceCommand::SpeedDown),
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

        Ok(())
    }
}
