#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![allow(internal_features)]
#![feature(panic_internals)]
#![feature(const_format_args)]

mod button;
mod command;
mod fan_speed;
mod interrupt_cell;
mod shared_state;
mod timed_monitor;
mod usb;

use arduino_hal::{Pins, delay_ms};
use avr_device::{asm::sleep, interrupt};
use button::{LedButton, PowerButton, SpeedDownButton, SpeedUpButton};
use command::Command;
use panic_halt as _;
use shared_state::SHARED_STATE;
use timed_monitor::setup_timed_monitor;
use usb::setup_usb;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pll = peripherals.PLL;
    let timer = peripherals.TC0;
    let usb = peripherals.USB_DEVICE;
    let Pins {
        d5: backlight_mon_pin,
        d6: speed_up_mon_pin,
        d7: speed_down_mon_pin,
        d8: led_mon_pin,
        d9: power_mon_pin,
        d10: power_btn_pin,
        mosi: led_btn_pin,
        miso: speed_up_btn_pin,
        sck: speed_down_btn_pin,
        ..
    } = arduino_hal::pins!(peripherals);

    let mut speed_up_btn = SpeedUpButton::new(speed_up_btn_pin.into_output());
    let mut speed_down_btn = SpeedDownButton::new(speed_down_btn_pin.into_output());
    let mut power_btn = PowerButton::new(power_btn_pin.into_output());
    let mut led_btn = LedButton::new(led_btn_pin.into_output());

    setup_timed_monitor(
        &timer,
        speed_up_mon_pin.into_pull_up_input(),
        speed_down_mon_pin.into_pull_up_input(),
        power_mon_pin.into_pull_up_input(),
        led_mon_pin.into_pull_up_input(),
        backlight_mon_pin.into_pull_up_input(),
    );

    // For reasons beyond my understanding the USB must get setup AFTER
    // the timer or it won't work correctly.
    setup_usb(pll, usb);

    // Always ensure highest speed is set on startup.
    for _ in 0..5 {
        speed_up_btn.short_press();
    }

    // Enable interrupts globally.
    unsafe { interrupt::enable() };

    // Button command priority
    // Speed Up > Speed Down > Power > LED
    //
    // Speed controls don't work with backlight off
    // Power and LED, however, do.

    loop {
        let command = interrupt::free(|cs| {
            let mut shared_state = SHARED_STATE.borrow(cs).borrow_mut();

            // Omit commands that are inconsistent with the current state.
            loop {
                break match shared_state.command_queue.pop_back() {
                    Some(Command::PowerOn) if shared_state.power_enabled => continue,
                    Some(Command::PowerOff) if !shared_state.power_enabled => continue,
                    Some(Command::LedsOn) if shared_state.leds_enabled => continue,
                    Some(Command::LedsOff) if !shared_state.leds_enabled => continue,
                    command => command,
                };
            }
        });

        match command {
            Some(Command::SpeedUp) => speed_up_btn.short_press(),
            Some(Command::SpeedDown) => speed_down_btn.short_press(),
            Some(Command::PowerOn | Command::PowerOff) => power_btn.short_press(),
            Some(Command::LedsOn | Command::LedsOff) => led_btn.long_press(),
            Some(Command::LedsColorChange) => led_btn.short_press(),
            // Used in the USB suspend code. Check the variant docs
            // for more info.
            Some(Command::DelayedLedsOff) => {
                delay_ms(200);
                led_btn.long_press();
            }
            None => sleep(),
        }
    }
}
