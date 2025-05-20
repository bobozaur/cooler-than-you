use gtk::{
    MenuItem,
    traits::{GtkMenuItemExt, WidgetExt},
};
use shared::FanSpeed;

use crate::menu::item::CustomMenuItem;

/// Non-actionable item that displays the current fan speed. This item is purely meant for display
/// and is never UI sensitive.
pub type SpeedLabelItem = CustomMenuItem<MenuItem, SpeedLabel>;

#[derive(Clone, Copy, Debug)]
pub struct SpeedLabel;

/// Poor man's `const_concat`, specialized for generating item labels.
macro_rules! make_label {
    ($bytes:expr) => {{
        const PREFIX: &[u8] = b"Fan speed: ";
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
    pub fn update(&self, fan_speed: FanSpeed) {
        macro_rules! make_speed_label {
            ($speed:expr) => {
                make_label!(&[$speed as u8 + b'0'])
            };
        }

        let label = match fan_speed {
            FanSpeed::Speed1 => make_speed_label!(FanSpeed::Speed1),
            FanSpeed::Speed2 => make_speed_label!(FanSpeed::Speed2),
            FanSpeed::Speed3 => make_speed_label!(FanSpeed::Speed3),
            FanSpeed::Speed4 => make_speed_label!(FanSpeed::Speed4),
            FanSpeed::Speed5 => make_speed_label!(FanSpeed::Speed5),
            FanSpeed::Speed6 => make_speed_label!(FanSpeed::Speed6),
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
