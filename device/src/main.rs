#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod button;
mod command;
mod interrupt_cell;
mod shared_state;
mod timed_monitor;
mod usb;

use arduino_hal::{Pins, delay_ms};
use avr_device::{asm::sleep, interrupt};
use button::{LedButton, PowerButton, SpeedDownButton, SpeedUpButton};
use command::Command;
use panic_halt as _;
use shared::{DeviceCommand, FanSpeed};
use shared_state::SHARED_STATE;
use timed_monitor::setup_timed_monitor;
use usb::setup_usb;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    // Disable the analog comparator
    peripherals.AC.acsr.write(|w| w.acd().set_bit());
    // Disable ADC
    peripherals.ADC.adcsra.write(|w| w.aden().clear_bit());
    peripherals.CPU.prr0.write(|w| w.pradc().set_bit());
    // Disable the on-chip debug system
    peripherals.CPU.mcucr.write(|w| w.jtd().set_bit());
    // Disable TWI
    peripherals.TWI.twcr.write(|w| w.twen().clear_bit());
    peripherals.CPU.prr0.write(|w| w.prtwi().set_bit());
    // Disable SPI
    peripherals.SPI.spcr.write(|w| w.spe().clear_bit());
    peripherals.CPU.prr0.write(|w| w.prspi().set_bit());
    // Disable USART
    peripherals.USART1.ucsr1b.write(|w| w.rxen1().clear_bit());
    peripherals.USART1.ucsr1b.write(|w| w.txen1().clear_bit());
    peripherals.CPU.prr1.write(|w| w.prusart1().set_bit());
    // Disable power to unused timers
    peripherals.CPU.prr0.write(|w| w.prtim1().set_bit());
    peripherals.CPU.prr1.write(|w| w.prtim3().set_bit());
    peripherals.CPU.prr1.write(|w| w.prtim4().set_bit());

    let pll = peripherals.PLL;
    let timer = peripherals.TC0;
    let usb = peripherals.USB_DEVICE;

    let Pins {
        d5: backlight_mon_pin,
        d6: speed_down_mon_pin,
        d7: speed_up_mon_pin,
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
    for _ in 0..FanSpeed::Speed6 as u8 {
        speed_down_btn.short_press();
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
            let shared_state = &mut *SHARED_STATE.borrow(cs).borrow_mut();
            let power_enabled = shared_state.device_state().power_enabled();
            let leds_enabled = shared_state.device_state().leds_enabled();

            // Omit commands that are inconsistent with the current state.
            loop {
                break match shared_state.pop_command() {
                    Some(Command::Device(DeviceCommand::PowerOn)) if power_enabled => continue,
                    Some(Command::Device(DeviceCommand::PowerOff)) if !power_enabled => continue,
                    Some(Command::Device(DeviceCommand::LedsOn)) if leds_enabled => continue,
                    Some(Command::Device(DeviceCommand::LedsOff)) if !leds_enabled => continue,
                    command => command,
                };
            }
        });

        match command {
            Some(Command::Device(DeviceCommand::SpeedUp)) => speed_up_btn.short_press(),
            Some(Command::Device(DeviceCommand::SpeedDown)) => speed_down_btn.short_press(),
            Some(Command::Device(DeviceCommand::PowerOn | DeviceCommand::PowerOff)) => {
                power_btn.short_press()
            }
            Some(Command::Device(DeviceCommand::LedsOn | DeviceCommand::LedsOff)) => {
                led_btn.long_press()
            }
            Some(Command::Device(DeviceCommand::LedsColorChange)) => led_btn.short_press(),
            Some(Command::Delay275Ms) => delay_ms(275),
            None => sleep(),
        }
    }
}
