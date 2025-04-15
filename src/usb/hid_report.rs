use usbd_hid::descriptor::{gen_hid_descriptor, generator_prelude::*};

use crate::shared_state::SharedState;

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

impl From<&SharedState> for HidReport {
    fn from(shared_state: &SharedState) -> Self {
        let power_enabled = (shared_state.power_enabled as u8) << 4;
        let leds_enabled = (shared_state.leds_enabled as u8) << 3;
        let fan_speed = shared_state.fan_speed as u8;

        Self {
            state: power_enabled | leds_enabled | fan_speed,
            command: 0,
        }
    }
}
