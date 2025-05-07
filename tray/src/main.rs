use std::{cell::RefCell, rc::Rc};

use gtk::{
    CheckMenuItem, MenuItem, SeparatorMenuItem,
    glib::{self, ControlFlow, SourceId},
    prelude::*,
};
use shared::Command;
use systemstat::{Platform, System};
use tray::{AnyResult, Cooler, Indicator, MenuItems};

#[allow(clippy::too_many_lines)]
fn main() -> AnyResult<()> {
    let cooler = Cooler::new()?;

    gtk::init()?;

    let speed_up_mi = MenuItem::with_label("Increase speed");
    let speed_down_mi = MenuItem::with_label("Decrease speed");
    let color_mi = MenuItem::with_label("Change color");
    let speed_auto_adjust_mi = CheckMenuItem::with_label("Auto-adjust speed");
    let power_mi = CheckMenuItem::with_label("Power");
    let leds_mi = CheckMenuItem::with_label("Lights");
    let quit_mi = MenuItem::with_label("Quit");

    let menu_items = Rc::new(MenuItems {
        speed_up_mi,
        speed_down_mi,
        color_mi,
        speed_auto_adjust_mi,
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
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |_| {
            menu_items.set_sensitive(false);
            if cooler.send_command(Command::LedsColorChange).is_err() {
                menu_items.set_sensitive(true);
            }
        }
    });

    menu_items.speed_auto_adjust_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();
        let source_id: RefCell<Option<SourceId>> = RefCell::new(None);

        move |mi| {
            let source_id = &mut *source_id.borrow_mut();

            match source_id.take() {
                Some(id) if !mi.is_active() => {
                    menu_items.speed_up_mi.set_sensitive(true);
                    menu_items.speed_down_mi.set_sensitive(true);
                    id.remove();
                }
                None if mi.is_active() => {
                    menu_items.speed_up_mi.set_sensitive(false);
                    menu_items.speed_down_mi.set_sensitive(false);
                    source_id.replace(glib::timeout_add_seconds_local(5, {
                        let cooler = cooler.clone();
                        move || {
                            let system = System::new();
                            system.cpu_temp().ok();
                            cooler.send_command(Command::SpeedUp).ok();
                            ControlFlow::Continue
                        }
                    }));
                }
                _ => (),
            }

            mi.set_sensitive(true);
        }
    });

    let power_sigh_id = menu_items.power_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |mi| {
            menu_items.set_sensitive(false);
            let command = if mi.is_active() {
                Command::PowerOn
            } else {
                Command::PowerOff
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
                Command::LedsOn
            } else {
                Command::LedsOff
            };

            if cooler.send_command(command).is_err() {
                menu_items.set_sensitive(true);
            }
        }
    });

    quit_mi.connect_activate(|_| gtk::main_quit());

    let mut indicator = Indicator::new()?;

    indicator.add_menu_item(&menu_items.speed_up_mi);
    indicator.add_menu_item(&menu_items.speed_down_mi);
    indicator.add_menu_item(&menu_items.color_mi);
    indicator.add_menu_item(&menu_items.speed_auto_adjust_mi);
    indicator.add_menu_item(&menu_items.leds_mi);
    indicator.add_menu_item(&menu_items.power_mi);
    indicator.add_menu_item(&SeparatorMenuItem::new());
    indicator.add_menu_item(&quit_mi);

    // Power cycle the device to ensure it's on.
    // If it's already off, the first command will be a no-op.
    glib::idle_add_local_once({
        let cooler = cooler.clone();
        move || {
            cooler.send_command(Command::PowerOff).ok();
            cooler.send_command(Command::PowerOn).ok();
        }
    });

    glib::idle_add_local(move || {
        let Ok(Some(device_state)) = cooler.recv_state() else {
            return ControlFlow::Continue;
        };

        println!("{device_state:?}");

        menu_items.power_mi.block_signal(&power_sigh_id);
        menu_items.power_mi.set_active(device_state.power_enabled());
        menu_items.power_mi.unblock_signal(&power_sigh_id);

        menu_items.leds_mi.block_signal(&leds_sigh_id);
        menu_items.leds_mi.set_active(device_state.leds_enabled());
        menu_items.leds_mi.unblock_signal(&leds_sigh_id);

        if let Some(command) = device_state.command_to_repeat() {
            cooler.send_command(command).ok();
            return ControlFlow::Continue;
        }

        menu_items.set_sensitive(true);
        ControlFlow::Continue
    });

    indicator.run();
    Ok(())
}
