use control_tray::{AnyResult, Device};
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use shared::Command;

fn main() -> AnyResult<()> {
    let device = Device::new()?;

    let device_state = loop {
        if let Some(state) = device.recv_state()? {
            break state;
        }
    };

    gtk::init()?;

    let mut indicator = AppIndicator::new("libappindicator test application", "");
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_icon_theme_path("");
    indicator.set_icon_full("rust-logo", "icon");
    let mut menu = gtk::Menu::new();

    let speed_up_mi = gtk::MenuItem::with_label("Increase speed");
    speed_up_mi.connect_activate({
        let device = device.clone();
        move |_| {
            device.send_commnad(Command::SpeedUp).ok();
        }
    });

    menu.append(&speed_up_mi);

    let speed_down_mi = gtk::MenuItem::with_label("Decrease speed");
    speed_down_mi.connect_activate({
        let device = device.clone();
        move |_| {
            device.send_commnad(Command::SpeedDown).ok();
        }
    });

    menu.append(&speed_down_mi);

    let color_mi = gtk::MenuItem::with_label("Change color");
    color_mi.connect_activate({
        let device = device.clone();
        move |_| {
            device.send_commnad(Command::LedsColorChange).ok();
        }
    });

    menu.append(&color_mi);

    let led_mi = gtk::CheckMenuItem::with_label("Lights");
    led_mi.set_active(device_state.leds_enabled());
    led_mi.connect_toggled({
        let device = device.clone();
        move |mi| {
            let command = if mi.is_active() {
                Command::LedsOn
            } else {
                Command::LedsOff
            };

            if device.send_commnad(command).is_err() {
                mi.set_active(!mi.is_active());
            }
        }
    });

    menu.append(&led_mi);

    let power_mi = gtk::CheckMenuItem::with_label("Power");
    power_mi.set_active(device_state.power_enabled());
    power_mi.connect_toggled({
        let device = device.clone();
        move |mi| {
            let command = if mi.is_active() {
                Command::PowerOn
            } else {
                Command::PowerOff
            };

            if device.send_commnad(command).is_err() {
                mi.set_active(!mi.is_active());
            }
            // mi.set_sensitive(sensitive);
        }
    });

    menu.append(&power_mi);

    let quit_mi = gtk::MenuItem::with_label("Quit");
    quit_mi.connect_activate(|_| {
        gtk::main_quit();
    });

    menu.append(&quit_mi);

    indicator.set_menu(&mut menu);
    menu.show_all();

    gtk::main();

    Ok(())
}
