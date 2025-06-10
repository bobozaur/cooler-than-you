use avr_device::interrupt;

use crate::timed_monitor::MONITOR_CTX;

#[interrupt(atmega32u4)]
fn TIMER0_COMPA() {
    MONITOR_CTX.as_inner_mut().monitor();
}
