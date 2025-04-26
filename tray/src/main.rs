use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use gtk::{CheckMenuItem, MenuItem, glib::MainContext, prelude::*};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use shared::Command;
use tray::{AnyResult, Cooler, MenuItems, track_state};

#[allow(clippy::too_many_lines)]
fn main() -> AnyResult<()> {
    let mut attempts: u8 = 10;
    let cooler = loop {
        match Cooler::new() {
            Ok(cooler) => break Rc::new(cooler),
            Err(e) if attempts == 0 => Err(e)?,
            Err(_) => {
                thread::sleep(Duration::from_secs(1));
                attempts -= 1;
            }
        }
    };

    cooler.send_command(Command::PowerOn)?;
    cooler.send_command(Command::PowerOff)?;

    let mut attempts: u8 = 10;
    let device_state = loop {
        match cooler.recv_state() {
            Ok(state) => break state,
            Err(e) if attempts == 0 => Err(e)?,
            Err(_) => {
                thread::sleep(Duration::from_millis(100));
                attempts -= 1;
            }
        }
    };

    gtk::init()?;

    let speed_up_mi = MenuItem::with_label("Increase speed");
    let speed_down_mi = MenuItem::with_label("Decrease speed");
    let color_mi = MenuItem::with_label("Change color");
    let power_mi = CheckMenuItem::with_label("Power");
    let led_mi = CheckMenuItem::with_label("Lights");
    let quit_mi = MenuItem::with_label("Quit");

    let menu_items = Rc::new(MenuItems {
        speed_up_mi,
        speed_down_mi,
        color_mi,
        power_mi,
        led_mi,
    });

    menu_items.led_mi.set_active(device_state.leds_enabled());
    menu_items.power_mi.set_active(device_state.power_enabled());

    menu_items.speed_up_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |_| {
            menu_items.set_sensitive(false);
            if cooler.send_command(Command::SpeedUp).is_err() {
                menu_items.set_sensitive(true);
            }
        }
    });
    menu_items.speed_down_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |_| {
            menu_items.set_sensitive(false);
            if cooler.send_command(Command::SpeedDown).is_err() {
                menu_items.set_sensitive(true);
            }
        }
    });
    menu_items.color_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |_| {
            menu_items.set_sensitive(false);
            if cooler.send_command(Command::LedsColorChange).is_err() {
                menu_items.set_sensitive(true);
            }
        }
    });
    menu_items.power_mi.connect_toggled({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |mi| {
            menu_items.set_sensitive(false);
            let command = if mi.is_active() {
                Command::PowerOff
            } else {
                Command::PowerOn
            };

            if cooler.send_command(command).is_err() {
                menu_items.set_sensitive(true);
            }
        }
    });
    menu_items.led_mi.connect_toggled({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |mi| {
            menu_items.set_sensitive(false);
            let command = if mi.is_active() {
                Command::LedsOff
            } else {
                Command::LedsOn
            };

            if cooler.send_command(command).is_err() {
                menu_items.set_sensitive(true);
            }
        }
    });

    quit_mi.connect_activate(|_| gtk::main_quit());

    let mut indicator = AppIndicator::new("CoolerThanYou tray icon", "");
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_icon_theme_path("");
    indicator.set_icon_full("cooler-than-you", "icon");

    let mut menu = gtk::Menu::new();
    menu.append(&menu_items.speed_up_mi);
    menu.append(&menu_items.speed_down_mi);
    menu.append(&menu_items.color_mi);
    menu.append(&menu_items.led_mi);
    menu.append(&menu_items.power_mi);
    menu.append(&quit_mi);
    indicator.set_menu(&mut menu);
    menu.show_all();

    let context = MainContext::default();
    context.spawn_local(track_state(cooler, menu_items.clone()));

    gtk::main();
    Ok(())
}
