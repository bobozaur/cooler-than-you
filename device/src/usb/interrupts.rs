use avr_device::interrupt;

use super::USB_DEVICE;

#[interrupt(atmega32u4)]
fn USB_GEN() {
    USB_DEVICE.as_inner_mut().poll_gen();
}

#[interrupt(atmega32u4)]
fn USB_COM() {
    USB_DEVICE.as_inner_mut().poll_com();
}
