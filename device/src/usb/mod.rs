mod hid_report;
mod interrupts;
mod suspender;

use arduino_hal::{
    pac::{PLL, USB_DEVICE},
    usb::AvrGenericUsbBus,
};
use avr_device::interrupt;
use hid_report::HidReport;
use shared::{USB_MANUFACTURER, USB_PID, USB_POLL_MS, USB_PRODUCT, USB_VID};
use suspender::Suspender;
use usb_device::{
    LangID,
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_hid::{descriptor::SerializedDescriptor, hid_class::HIDClass};

use crate::{interrupt_cell::InterruptCell, shared_state::SHARED_STATE};

type UsbBus = AvrGenericUsbBus<Suspender>;

static USB_DEVICE: InterruptCell<UsbContext> = InterruptCell::uninit();

pub fn setup_usb(pll: PLL, usb: USB_DEVICE) {
    static USB_BUS: InterruptCell<UsbBusAllocator<UsbBus>> = InterruptCell::uninit();

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

    let usb_bus = USB_BUS.init(UsbBus::with_suspend_notifier(usb, Suspender::new(pll)));

    let strings = StringDescriptors::new(LangID::EN)
        .manufacturer(USB_MANUFACTURER)
        .product(USB_PRODUCT);

    let hid_class = HIDClass::new(usb_bus, HidReport::desc(), USB_POLL_MS);
    let usb_device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(USB_VID, USB_PID))
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

    /// We separate the two interrupt functions because it seems that
    /// using a [`avr_device::interrupt::Mutex`] within the `USB_GEN` interrupt
    /// results in the host not enumerating the device correctly.
    ///
    /// The exact reason is beyond my understanding. It could be a timing issue
    /// but I doubt that because the `USB_COM` interrupt is time sensitive as well.
    ///
    /// It really looks like contention between the two interrupts, which is surprinsing
    /// because nested interrupts have to be explicitly enabled, which I don't do. Maybe
    /// the USB interrupts behave in a slightly different way?
    ///
    /// Nevertheless, we don't want to do any data communication in the `USB_GEN` interrupt
    /// so this approach is more than satisfactory. I just wish I did not spend two days of
    /// my life troubleshooting this only to encounter this behavior that I cannot really explain.
    ///
    /// Simple polling here is enough to provide the report descriptor and handle suspend/resume
    /// behavior.
    #[inline]
    fn poll_gen(&mut self) {
        self.usb_device.poll(&mut [&mut self.hid_class]);
    }

    /// The `USB_COM` interrupt code. Unlike [`UsbContext::poll_gen`], we do send data
    /// in this interrupt so we want to access and serialize the shared state. The
    /// [`avr_device::interrupt::Mutex`] does not cause any issues here.
    #[inline]
    fn poll_com(&mut self) {
        interrupt::free(|cs| {
            let shared_state = &mut SHARED_STATE.borrow(cs).borrow_mut();
            let device_state = *shared_state.device_state();

            if !self.usb_device.poll(&mut [&mut self.hid_class]) {
                return;
            }

            shared_state.if_send_state(|| {
                let res = self.hid_class.push_raw_input(&[device_state.into()]);
                matches!(res, Ok(1))
            });

            let mut report_buf = [0u8; 1];

            if let Ok(1) = self.hid_class.pull_raw_output(&mut report_buf) {
                if let Ok(command) = report_buf[0].try_into() {
                    shared_state.push_command(command);
                }
            }
        });
    }
}
