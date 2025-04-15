mod hid_report;
mod interrupts;
mod suspender;

use arduino_hal::{
    pac::{PLL, USB_DEVICE},
    usb::AvrGenericUsbBus,
};
use suspender::Suspender;
use usb_device::{
    LangID,
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_hid::{
    descriptor::{SerializedDescriptor, SystemControlKey, SystemControlReport},
    hid_class::HIDClass,
};

use crate::interrupt_cell::InterruptCell;

type UsbBus = AvrGenericUsbBus<Suspender>;

static USB_DEVICE: InterruptCell<UsbContext> = InterruptCell::uninit();

pub fn setup_usb(pll: PLL, usb: USB_DEVICE) {
    // Configure PLL interface
    // prescale 16MHz crystal -> 8MHz
    pll.pllcsr.write(|w| w.pindiv().set_bit());
    // 96MHz PLL output; /1.5 for 64MHz timers, /2 for 48MHz USB
    pll.pllfrq
        .write(|w| w.pdiv().mhz96().plltm().factor_15().pllusb().set_bit());

    // Enable PLL
    pll.pllcsr.modify(|_, w| w.plle().set_bit());

    // Check PLL lock
    while pll.pllcsr.read().plock().bit_is_clear() {}

    static USB_BUS: InterruptCell<UsbBusAllocator<UsbBus>> = InterruptCell::uninit();
    USB_BUS.init(UsbBus::with_suspend_notifier(usb, Suspender::new(pll)));
    let usb_bus = USB_BUS.as_inner_mut();

    let strings = StringDescriptors::new(LangID::EN)
        .manufacturer("Cooler")
        .product("Than You");

    let hid_class = HIDClass::new(usb_bus, SystemControlReport::desc(), 1);
    let usb_device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0xd016, 0xdb08))
        .strings(&[strings])
        .unwrap()
        .max_power(500)
        .unwrap()
        .build();

    USB_DEVICE.init(UsbContext::new(usb_device, hid_class));
}

struct UsbContext {
    usb_device: UsbDevice<'static, UsbBus>,
    hid_class: HIDClass<'static, UsbBus>,
}

impl UsbContext {
    #[inline]
    fn new(usb_device: UsbDevice<'static, UsbBus>, hid_class: HIDClass<'static, UsbBus>) -> Self {
        Self {
            usb_device,
            hid_class,
        }
    }

    fn poll(&mut self) {
        const BLANK_REPORT: SystemControlReport = SystemControlReport {
            usage_id: SystemControlKey::Reserved as u8,
        };

        self.hid_class.push_input(&BLANK_REPORT).ok();

        if self.usb_device.poll(&mut [&mut self.hid_class]) {
            let mut report_buf = [0u8; 1];

            if self.hid_class.pull_raw_output(&mut report_buf).is_ok() {
                // if let Ok(command) = report_buf[0].try_into() {
                //     shared_state.command_queue.push_front(command);
                // }
            }
        }
    }
}
