//! CoolerThanYou device code.
//!
//! The code was developed for an Arduino Pro Micro with an ATmega32u4 running at 5V.
//! Hardware components used:
//! - TIMER0
//! - Pins: PB1, PB2, PB3, PB4, PB5, PB6, PC6, PD7, PE6,
//! - USB
//! - PLL
//! - WDT

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod button;
mod command;
mod interrupt_cell;
mod monitor;
mod shared_state;
mod usb;

use arduino_hal::{
    Pins, delay_ms,
    hal::{Wdt, wdt::Timeout},
};
use avr_device::{asm::sleep, interrupt};
use button::{LedButton, PowerButton, SpeedDownButton, SpeedUpButton};
use command::Command;
use monitor::setup_timed_monitor;
use panic_halt as _;
use shared::{DeviceCommand, FanSpeed};
use shared_state::SHARED_STATE;
use usb::setup_usb;

fn enter_bootloader(mut watchdog: Wdt) -> ! {
    /// Magic value that tells the bootloader to remain in bootloader mode on watchdog resets.
    /// Taken from <https://github.com/arduino/ArduinoCore-avr/blob/c8c514c9a19602542bc32c7033f48fecbbda4401/bootloaders/caterina/Caterina.c#L68>
    const BOOT_KEY: u16 = 0x7777;
    /// Pointer to the address where the bootloader looks for the [`BOOT_KEY`].
    /// Taken from <https://github.com/arduino/ArduinoCore-avr/blob/c8c514c9a19602542bc32c7033f48fecbbda4401/bootloaders/caterina/Caterina.c#L69>
    const BOOT_KEY_PTR: *mut u16 = 0x0800 as *mut u16;

    /// Write the magic value
    unsafe { core::ptr::write_volatile(BOOT_KEY_PTR, BOOT_KEY) };

    /// Set the lowest possible time value for the watchdog.
    watchdog.start(Timeout::Ms16).ok();

    /// Loop until the watchdog reset happens.
    loop {}
}

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
    let wdt = peripherals.WDT;

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

    // Create buttons
    let mut speed_up_btn = SpeedUpButton::new(speed_up_btn_pin.into_output());
    let mut speed_down_btn = SpeedDownButton::new(speed_down_btn_pin.into_output());
    let mut power_btn = PowerButton::new(power_btn_pin.into_output());
    let mut led_btn = LedButton::new(led_btn_pin.into_output());

    // Create the watchdog instance
    let watchdog = Wdt::new(wdt, &peripherals.CPU.mcusr);

    // Setup the timed monitor
    setup_timed_monitor(
        &timer,
        speed_up_mon_pin.into_pull_up_input(),
        speed_down_mon_pin.into_pull_up_input(),
        power_mon_pin.into_pull_up_input(),
        led_mon_pin.into_pull_up_input(),
        backlight_mon_pin.into_pull_up_input(),
    );

    // Setup USB.
    //
    // For reasons beyond my understanding the USB must get setup AFTER the timer or it won't work
    // correctly.
    setup_usb(pll, usb);

    // Do some speed down button presses to always ensure a consistent lowest fan speed on startup.
    for _ in 0..FanSpeed::Speed6 as u8 {
        speed_down_btn.short_press();
    }

    // Enable interrupts globally.
    unsafe { interrupt::enable() };

    loop {
        // Check if a command has been received.
        //
        // NOTE: We try to keep the critical section as short as possible here and do the
        //       actual button presses, with their inherent delays, afterwards.
        let command = interrupt::free(|cs| {
            let shared_state = &mut *SHARED_STATE.borrow(cs).borrow_mut();
            let power_enabled = shared_state.device_state().power_enabled();
            let leds_enabled = shared_state.device_state().leds_enabled();

            loop {
                // Ignore commands that are inconsistent with the current state.
                break match shared_state.pop_command() {
                    Some(Command::Device(DeviceCommand::PowerOn)) if power_enabled => continue,
                    Some(Command::Device(DeviceCommand::PowerOff)) if !power_enabled => continue,
                    Some(Command::Device(DeviceCommand::LedsOn)) if leds_enabled => continue,
                    Some(Command::Device(DeviceCommand::LedsOff)) if !leds_enabled => continue,
                    command => command,
                };
            }
        });

        // Execute the command outside of the critical section.
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
            Some(Command::EnterBootloader) => enter_bootloader(watchdog),
            None => sleep(),
        }
    }
}
