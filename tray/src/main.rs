use anyhow::Context;
use gtk::{
    SeparatorMenuItem,
    glib::{self, ControlFlow},
};
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
    indicator.add_menu_item(&menu_items.change_color);
    indicator.add_menu_item(&SeparatorMenuItem::new());
    let power_handler_id = indicator
        .add_menu_item(&menu_items.power)
        .context("missing power callback handler id")?;
    indicator.add_menu_item(&SeparatorMenuItem::new());
    indicator.add_menu_item(&quit_mi);

    glib::idle_add_local(move || {
        let Ok(Some(device_state)) = device.recv_state() else {
            return ControlFlow::Continue;
        };

        println!("{device_state:?}");

        menu_items
            .power
            .set_active(device_state.power_enabled(), &power_handler_id);
        menu_items
            .leds
            .set_active(device_state.leds_enabled(), &leds_handler_id);

        if let Some(command) = device_state.command_to_repeat() {
            device.send_command(command).ok();
            return ControlFlow::Continue;
        }

        speed_label.update_speed(device_state.fan_speed());

        menu_items.refresh_sensitivity();
        ControlFlow::Continue
    });

    indicator.run();
    Ok(())
}
