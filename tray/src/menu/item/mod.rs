mod cmd;
mod quit;
mod speed_auto;
mod speed_label;

pub use cmd::{LedsChangeColorItem, LedsToggleItem, PowerToggleItem, SpeedDownItem, SpeedUpItem};
use gtk::{
    CheckMenuItem,
    glib::{ObjectExt, SignalHandlerId},
    traits::{CheckMenuItemExt, WidgetExt},
};
pub use quit::QuitItem;
pub use speed_auto::SpeedAutoItem;
pub use speed_label::SpeedLabelItem;

/// A custom menu item that wraps a `gtk` menu item and further specializes its behavior based on
/// the provided kind. Lean towards using the provided aliases rather than interacting with this
/// type directly.
#[derive(Debug)]
pub struct CustomMenuItem<MI, K> {
    inner: MI,
    kind: K,
}

impl<MI, K> CustomMenuItem<MI, K>
where
    MI: WidgetExt,
{
    pub fn set_sensitive(&self, flag: bool) {
        self.inner.set_sensitive(flag);
    }
}

impl<K> CustomMenuItem<CheckMenuItem, K> {
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }
}

impl<K> CustomMenuItem<CheckMenuItem, K>
where
    K: AsRef<SignalHandlerId>,
{
    pub fn set_active(&self, is_active: bool) {
        // We must block the callback signal before tweaking the state to avoid unwantedly
        // triggering it.
        self.inner.block_signal(self.kind.as_ref());
        self.inner.set_active(is_active);
        // Now that the state was changed, we can unblock the callback signal.
        self.inner.unblock_signal(self.kind.as_ref());
    }
}

impl<MI, K> AsRef<MI> for CustomMenuItem<MI, K> {
    fn as_ref(&self) -> &MI {
        &self.inner
    }
}
