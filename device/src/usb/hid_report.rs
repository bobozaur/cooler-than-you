use usbd_hid::descriptor::{gen_hid_descriptor, generator_prelude::*};

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
        #[item_settings data,variable,absolute] state=input;
        #[item_settings data,variable,absolute] command=output;

    }
)]
pub struct HidReport {
    state: u8,
    command: u8,
}
