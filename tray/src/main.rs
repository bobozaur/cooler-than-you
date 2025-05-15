//! TODO:
//! - disable WDT and BOD fuses
//! - check error handling
//! - logging
//! - debian packaging
//! - comments and docs
//! - fancy icon

use std::{cell::Cell, rc::Rc};

use anyhow::Context;
use gtk::{SeparatorMenuItem, glib};
use shared::FanSpeed;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, fmt};
use tray::{AnyResult, Indicator, QuitItem, SpeedLabelItem, process_device_state, speed_auto_task};

fn main() -> AnyResult<()> {
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    LogTracer::init()?;

    let mut indicator = Indicator::new()?;
    let device = indicator.device().clone();
    let menu_items = indicator.menu_items().clone();

    let speed_label = SpeedLabelItem::new();
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
    // An [`Rc<Cell<FanSpeed>>`] is used here to share the value between the speed auto task and
    // the main background task because FanSpeed is [`Copy`].
    let fan_speed = Rc::new(Cell::new(FanSpeed::Speed1));
    glib::spawn_future_local(speed_auto_task(
        device.clone(),
        menu_items.clone(),
        fan_speed.clone(),
    ));
    glib::spawn_future_local(process_device_state(
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
