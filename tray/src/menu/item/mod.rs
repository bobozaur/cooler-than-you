pub mod command;
pub mod quit;
pub mod speed_auto;
pub mod speed_label;

use gtk::{
    CheckMenuItem,
    glib::{ObjectExt, SignalHandlerId},
    traits::{CheckMenuItemExt, WidgetExt},
};

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
        self.inner.block_signal(self.kind.as_ref());
        self.inner.set_active(is_active);
        self.inner.unblock_signal(self.kind.as_ref());
    }
}

impl<MI, K> AsRef<MI> for CustomMenuItem<MI, K> {
    fn as_ref(&self) -> &MI {
        &self.inner
    }
}
