use gtk::{
    MenuItem,
    traits::{GtkMenuItemExt, WidgetExt},
};
use shared::FanSpeed;

use crate::menu::item::CustomMenuItem;

pub type SpeedLabelItem = CustomMenuItem<MenuItem, SpeedLabel>;

#[derive(Clone, Copy, Debug)]
pub struct SpeedLabel;

macro_rules! make_label {
    ($bytes:expr) => {{
        const PREFIX: &[u8] = SpeedLabelItem::LABEL_PREFIX.as_bytes();
        const LEN: usize = PREFIX.len() + $bytes.len();
        const ARR: [u8; LEN] = {
            let mut arr: [u8; LEN] = [0; LEN];
            let (prefix, custom) = arr.split_at_mut(PREFIX.len());
            prefix.copy_from_slice(PREFIX);
            custom.copy_from_slice($bytes);
            arr
        };

        const LABEL: &str = match str::from_utf8(&ARR) {
            Ok(s) => s,
            Err(_) => panic!("invalid label"),
        };

        LABEL
    }};
}

impl SpeedLabelItem {
    const LABEL_PREFIX: &str = "Fan speed: ";

    pub fn update(&self, fan_speed: FanSpeed) {
        let label = match fan_speed {
            FanSpeed::Speed1 => make_label!(&[FanSpeed::Speed1 as u8 + b'0']),
            FanSpeed::Speed2 => make_label!(&[FanSpeed::Speed2 as u8 + b'0']),
            FanSpeed::Speed3 => make_label!(&[FanSpeed::Speed3 as u8 + b'0']),
            FanSpeed::Speed4 => make_label!(&[FanSpeed::Speed4 as u8 + b'0']),
            FanSpeed::Speed5 => make_label!(&[FanSpeed::Speed5 as u8 + b'0']),
            FanSpeed::Speed6 => make_label!(&[FanSpeed::Speed6 as u8 + b'0']),
        };

        self.inner.set_label(label);
    }
}

impl Default for SpeedLabelItem {
    fn default() -> Self {
        let inner = MenuItem::with_label(make_label!(b"N/A"));
        inner.set_sensitive(false);
        Self {
            inner,
            kind: SpeedLabel,
        }
    }
}
