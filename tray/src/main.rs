use anyhow::Context;
use gtk::{
    SeparatorMenuItem,
    glib::{self, ControlFlow},
};
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

    let system = System::new();
    // This will self-adjust, we just start with the lowest speed.
    let mut current_fan_speed = FanSpeed::Speed1;

    // Frequency with which this is called is determined by the timeout value in
    // [`Device::recv_state`].
    glib::idle_add_local(move || {
        let device_state_opt = device.recv_state().ok().flatten();

        if menu_items.speed_auto.is_active() {
            if let (Ok(temp), fan_speed) = (system.cpu_temp(), current_fan_speed) {
                let command = match fan_speed {
                    FanSpeed::Speed1 if temp > 60.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed2 if temp > 65.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed3 if temp > 70.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed4 if temp > 75.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed5 if temp > 80.0 => Some(DeviceCommand::SpeedUp),
                    FanSpeed::Speed6 if temp > 80.0 => None,
                    FanSpeed::Speed6
                    | FanSpeed::Speed5
                    | FanSpeed::Speed4
                    | FanSpeed::Speed3
                    | FanSpeed::Speed2 => Some(DeviceCommand::SpeedDown),
                    FanSpeed::Speed1 => None,
                };

                command.and_then(|c| device.send_command(c).ok());
            }
        }

        let Some(device_state) = device_state_opt else {
            return ControlFlow::Continue;
        };

        let fan_speed = device_state.fan_speed();
        current_fan_speed = fan_speed;
        speed_label.update_speed(fan_speed);

        menu_items
            .power
            .set_active(device_state.power_enabled(), &power_handler_id);
        menu_items
            .leds
            .set_active(device_state.leds_enabled(), &leds_handler_id);

        println!("{device_state:?}");

        if let Some(command) = device_state.command_to_repeat() {
            device.send_command(command).ok();
            return ControlFlow::Continue;
        }

        menu_items.refresh_sensitivity();
        ControlFlow::Continue
    });

    indicator.run();
    Ok(())
}
