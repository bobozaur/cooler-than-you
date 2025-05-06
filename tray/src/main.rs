use std::{rc::Rc, thread, time::Duration};

use gtk::{
    CheckMenuItem, MenuItem,
    glib::{self, ControlFlow},
    prelude::*,
};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use shared::{Command, USB_POLL_MS};
use tray::{AnyResult, Cooler, MenuItems, track_state};

#[allow(clippy::too_many_lines)]
fn main() -> AnyResult<()> {
    let mut attempts: u8 = 10;
    let cooler = loop {
        match Cooler::new() {
            Ok(cooler) => break cooler,
            Err(e) if attempts == 0 => Err(e)?,
            Err(_) => {
                thread::sleep(Duration::from_secs(1));
                attempts -= 1;
            }
        }
    };

    // Power cycle the device to ensure it's on.
    // If it's already off, the first command will be a no-op.
    cooler.send_command(Command::PowerOff)?;
    cooler.send_command(Command::PowerOn)?;

    gtk::init()?;

    let speed_up_mi = MenuItem::with_label("Increase speed");
    let speed_down_mi = MenuItem::with_label("Decrease speed");
    let color_mi = MenuItem::with_label("Change color");
    let power_mi = CheckMenuItem::with_label("Power");
    let leds_mi = CheckMenuItem::with_label("Lights");
    let quit_mi = MenuItem::with_label("Quit");

    let menu_items = Rc::new(MenuItems {
        speed_up_mi,
        speed_down_mi,
        color_mi,
        power_mi,
        leds_mi,
    });

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
        let cooler = cooler.clone();

        move |_| {
            cooler.send_command(Command::LedsColorChange).ok();
        }
    });
    let power_sigh_id = menu_items.power_mi.connect_activate({
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

    let leds_sigh_id = menu_items.leds_mi.connect_activate({
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
    menu.append(&menu_items.leds_mi);
    menu.append(&menu_items.power_mi);
    menu.append(&quit_mi);
    indicator.set_menu(&mut menu);
    menu.show_all();

    glib::timeout_add_local(Duration::from_millis(USB_POLL_MS.into()), move || {
        track_state(&cooler, &menu_items, &power_sigh_id, &leds_sigh_id);
        ControlFlow::Continue
    });

    gtk::main();
    Ok(())
}
