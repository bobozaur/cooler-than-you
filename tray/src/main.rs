use std::{cell::RefCell, rc::Rc};

use gtk::{
    MenuItem, SeparatorMenuItem,
    glib::{self, ControlFlow, SourceId},
    prelude::*,
};
use shared::Command;
use systemstat::{Platform, System};
use tray::{AnyResult, Cooler, Indicator, MenuItems};

#[allow(clippy::too_many_lines)]
fn main() -> AnyResult<()> {
    let cooler = Cooler::new()?;
    let mut indicator = Indicator::new()?;

    let menu_items = Rc::new(MenuItems::new());

    menu_items.speed_auto_adjust_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();
        let source_id: RefCell<Option<SourceId>> = RefCell::new(None);

        move |mi| {
            let source_id = &mut *source_id.borrow_mut();
            menu_items.refresh_sensitivity();

            match source_id.take() {
                Some(id) if !mi.is_active() => {
                    id.remove();
                }
                None if mi.is_active() => {
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
        }
    });

    menu_items.speed_up_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |_| {
            menu_items.disable();

            if cooler.send_command(Command::SpeedUp).is_err() {}
        }
    });

    menu_items.speed_down_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |_| {
            menu_items.disable();

            if cooler.send_command(Command::SpeedDown).is_err() {}
        }
    });

    let leds_sigh_id = menu_items.leds_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |mi| {
            menu_items.disable();

            let command = if mi.is_active() {
                Command::LedsOn
            } else {
                Command::LedsOff
            };

            if cooler.send_command(command).is_err() {}
        }
    });

    menu_items.color_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |_| {
            menu_items.disable();

            if cooler.send_command(Command::LedsColorChange).is_err() {}
        }
    });

    let power_sigh_id = menu_items.power_mi.connect_activate({
        let menu_items = menu_items.clone();
        let cooler = cooler.clone();

        move |mi| {
            menu_items.disable();

            let command = if mi.is_active() {
                Command::PowerOn
            } else {
                Command::PowerOff
            };

            if cooler.send_command(command).is_err() {}
        }
    });

    let speed_label_mi = MenuItem::with_label("Fan speed: N/A");
    speed_label_mi.set_sensitive(false);

    let quit_mi = MenuItem::with_label("Quit");
    quit_mi.connect_activate(|_| gtk::main_quit());

    indicator.add_menu_item(&speed_label_mi);
    indicator.add_menu_item(&SeparatorMenuItem::new());
    indicator.add_menu_item(&menu_items.speed_auto_adjust_mi);
    indicator.add_menu_item(&menu_items.speed_up_mi);
    indicator.add_menu_item(&menu_items.speed_down_mi);
    indicator.add_menu_item(&SeparatorMenuItem::new());
    indicator.add_menu_item(&menu_items.leds_mi);
    indicator.add_menu_item(&menu_items.color_mi);
    indicator.add_menu_item(&SeparatorMenuItem::new());
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

        let speed = u8::from(device_state.fan_speed()) + 1;
        speed_label_mi.set_label(&format!("Fan speed: {speed}"));

        menu_items.refresh_sensitivity();
        ControlFlow::Continue
    });

    indicator.run();
    Ok(())
}
