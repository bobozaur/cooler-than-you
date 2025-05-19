//! TODO:
//! - debian packaging
//! - comments and docs
//! - fancy icon

use std::{cell::Cell, rc::Rc};

use anyhow::Context;
use gtk::SeparatorMenuItem;
use shared::FanSpeed;
use tracing_subscriber::{EnvFilter, fmt};
use tray::{
    AnyResult, Device, Indicator, QuitItem, SpeedLabelItem, power_cycle_device,
    process_device_state, speed_auto_task,
};

fn main() -> AnyResult<()> {
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let device = Device::new()?;
    let mut indicator = Indicator::new()?;
    let menu_items = indicator.menu_items().clone();

    let speed_label = SpeedLabelItem::new();
    let quit_mi = QuitItem::new();

    indicator.add_menu_item(&speed_label, device.clone());
    indicator.add_menu_item(&SeparatorMenuItem::new(), device.clone());
    indicator.add_menu_item(&menu_items.speed_auto, device.clone());
    indicator.add_menu_item(&menu_items.speed_up, device.clone());
    indicator.add_menu_item(&menu_items.speed_down, device.clone());
    indicator.add_menu_item(&SeparatorMenuItem::new(), device.clone());
    let leds_handler_id = indicator
        .add_menu_item(&menu_items.leds, device.clone())
        .context("missing lights callback handler id")?;
    indicator.add_menu_item(&menu_items.leds_change_color, device.clone());
    indicator.add_menu_item(&SeparatorMenuItem::new(), device.clone());
    let power_handler_id = indicator
        .add_menu_item(&menu_items.power, device.clone())
        .context("missing power callback handler id")?;
    indicator.add_menu_item(&SeparatorMenuItem::new(), device.clone());
    indicator.add_menu_item(&quit_mi, device.clone());

    // We send the commands this way so that the time between them being sent and read is minimal
    // and happens as soon as the event loop is started.
    tray::spawn(power_cycle_device(device.clone()));

    // This will self-adjust, we just start with the lowest speed.
    // An [`Rc<Cell<FanSpeed>>`] is used here to share the value between the speed auto task and
    // the main background task because FanSpeed is [`Copy`].
    let fan_speed = Rc::new(Cell::new(FanSpeed::Speed1));

    tray::spawn(speed_auto_task(
        device.clone(),
        menu_items.clone(),
        fan_speed.clone(),
    ));

    tray::spawn(process_device_state(
        device,
        menu_items,
        fan_speed,
        speed_label,
        power_handler_id,
        leds_handler_id,
    ));

    indicator.run();
    Ok(())
}
