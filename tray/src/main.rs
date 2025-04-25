use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use anyhow::Context;
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use shared::Command;
use tray::{AnyResult, Device};

// background thread
// - checks temp and adjusts speed (if enabled) (needs Device)
// -

#[allow(clippy::too_many_lines)]
fn main() -> AnyResult<()> {
    let device = loop {
        if let Ok(device) = Device::new() {
            break device;
        }
    };

    loop {
        device.send_command(Command::SpeedUp).context("write")?;
        thread::sleep(Duration::from_secs(3));
        println!("{:?}", device.recv_state().context("read"));
    }

    device.send_command(Command::PowerOff)?;
    device.send_command(Command::PowerOn)?;

    let mut device_state_opt = None;

    while let Some(state) = device.recv_state()? {
        device_state_opt = Some(state);
    }

    let device_state = Rc::new(RefCell::new(
        device_state_opt.context("device state not received on startup")?,
    ));

    gtk::init()?;

    let speed_up_mi = gtk::MenuItem::with_label("Increase speed");
    let speed_down_mi = gtk::MenuItem::with_label("Decrease speed");
    let color_mi = gtk::MenuItem::with_label("Change color");
    let led_mi = gtk::CheckMenuItem::with_label("Lights");
    let power_mi = gtk::CheckMenuItem::with_label("Power");
    let quit_mi = gtk::MenuItem::with_label("Quit");

    let device_state_ref = device_state.borrow();
    led_mi.set_active(device_state_ref.leds_enabled());
    power_mi.set_active(device_state_ref.power_enabled());

    speed_up_mi.connect_activate({
        let device = device.clone();
        let device_state = device_state.clone();

        move |mi| {
            mi.set_sensitive(false);

            let mut device_state = device_state.borrow_mut();

            loop {
                while device.send_command(Command::SpeedUp).is_err() {}

                let Ok(Some(state)) = device.recv_state() else {
                    continue;
                };

                if *device_state != state {
                    *device_state = state;
                    break;
                }
            }

            mi.set_sensitive(true);
        }
    });

    speed_down_mi.connect_activate({
        let device = device.clone();
        let device_state = device_state.clone();

        move |mi| {
            mi.set_sensitive(false);

            let mut device_state = device_state.borrow_mut();

            loop {
                while device.send_command(Command::SpeedDown).is_err() {}

                let Ok(Some(state)) = device.recv_state() else {
                    continue;
                };

                if *device_state != state {
                    *device_state = state;
                    break;
                }
            }

            mi.set_sensitive(true);
        }
    });

    color_mi.connect_activate({
        let device = device.clone();

        move |mi| {
            mi.set_sensitive(false);
            while device.send_command(Command::LedsColorChange).is_err() {}
            while let Ok(None) | Err(_) = device.recv_state() {}
            mi.set_sensitive(true);
        }
    });

    led_mi.connect_toggled({
        let device = device.clone();
        let device_state = device_state.clone();

        move |mi| {
            mi.set_sensitive(false);

            let command = if mi.is_active() {
                Command::LedsOn
            } else {
                Command::LedsOff
            };

            while device.send_command(command).is_err() {}

            if let Ok(Some(state)) = device.recv_state() {
                *device_state.borrow_mut() = state;
                mi.set_active(!mi.is_active());
            }

            mi.set_sensitive(true);
        }
    });

    power_mi.connect_toggled({
        let device = device.clone();
        let device_state = device_state.clone();

        move |mi| {
            mi.set_sensitive(false);

            let command = if mi.is_active() {
                Command::PowerOn
            } else {
                Command::PowerOff
            };

            while device.send_command(command).is_err() {}

            if let Ok(Some(state)) = device.recv_state() {
                *device_state.borrow_mut() = state;
                mi.set_active(!mi.is_active());
            }

            mi.set_sensitive(true);
        }
    });

    quit_mi.connect_activate(|_| {
        gtk::main_quit();
    });

    let mut indicator = AppIndicator::new("CoolerThanYou tray icon", "");
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_icon_theme_path("");
    indicator.set_icon_full("cooler-than-you", "icon");

    let mut menu = gtk::Menu::new();
    menu.append(&speed_up_mi);
    menu.append(&speed_down_mi);
    menu.append(&color_mi);
    menu.append(&led_mi);
    menu.append(&power_mi);
    menu.append(&quit_mi);
    indicator.set_menu(&mut menu);
    menu.show_all();

    gtk::main();
    Ok(())
}
