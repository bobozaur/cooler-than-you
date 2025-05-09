mod hid_report;
mod interrupts;
mod suspender;

use arduino_hal::{
    pac::{PLL, USB_DEVICE},
    usb::AvrGenericUsbBus,
};
use avr_device::interrupt;
use hid_report::HidReport;
use shared::{USB_MANUFACTURER, USB_PID, USB_PRODUCT, USB_VID};
use suspender::Suspender;
use usb_device::{
    LangID,
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_hid::{descriptor::SerializedDescriptor, hid_class::HIDClass};

use crate::{command::Command, interrupt_cell::InterruptCell, shared_state::SHARED_STATE};

type UsbBus = AvrGenericUsbBus<Suspender>;

/// Since any action on the device would require at least 40 ms, I assume
/// there's no reason to poll it much more frequently than that.
const USB_POLL_MS: u8 = 40;
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

    /// The USB interrupt code.
    #[inline]
    fn poll(&mut self) {
        // Because this code gets called from both USB interrupts, we want to continue
        // regardless of what this function returns. Otherwise, failing to access the
        // [`SHARED_STATE`] would result in no polling being performed.
        self.usb_device.poll(&mut [&mut self.hid_class]);

        interrupt::free(|cs| {
            // For reasons beyond my understanding, the two USB interrupts seem to contend
            // on the [`avr_device::interrupt::Mutex`] although, as far as I know, nested interrupts
            // have to be enabled explicitly, which we do not do. Because of that, the panicking
            // variant [`std::cell::RefCell::borrow_mut`] crashes the device if used in
            // both interrupts.
            //
            // Initially, I settled on accessing the [`SharedState`] only in the `USB_COM`
            // interrupt, leaving a simple poll in `USB_GEN`, but that results in USB
            // reads only working after a write. We therefore only proceed if we
            // were able to obtain a mutable reference and return early otherwise.
            let Ok(shared_state) = &mut SHARED_STATE.borrow(cs).try_borrow_mut() else {
                return;
            };

            let device_state = *shared_state.device_state();

            shared_state.if_send_state(|| {
                let res = self.hid_class.push_raw_input(&[device_state.into()]);
                matches!(res, Ok(1))
            });

            let mut report_buf = [0u8; 1];

            if let Ok(1) = self.hid_class.pull_raw_output(&mut report_buf) {
                if let Ok(command) = report_buf[0].try_into() {
                    shared_state.push_command(Command::Device(command));
                }
            }
        });
    }
}
