use usbd_hid::descriptor::{gen_hid_descriptor, generator_prelude::*};
/// USB HID report.
///
/// The device can only send its state when it changes (packed in a single byte) and can only
/// receive a byte representing a command to execute.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = 0x0B) = {
        (usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
            #[item_settings data,variable,absolute] state=input;
        };
        (usage_page = VENDOR_DEFINED_START, usage = 0x02) = {
            #[item_settings data,variable,absolute] command=output;
        };

    }
)]
pub struct HidReport {
    state: u8,
    command: u8,
}
