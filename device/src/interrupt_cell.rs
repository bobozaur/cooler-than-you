use core::{cell::UnsafeCell, mem::MaybeUninit};

/// Wrapper type for [`UnsafeCell`] that implements [`Sync`] and provides convenience methods for
/// dealing with the underlying type.
///
/// The purpose of this cell is to initialize statics that will get used exclusively in interrupts.
pub struct InterruptCell<T>(UnsafeCell<MaybeUninit<T>>);

/// This implementation does not rely on `T: Sync` as well because of
/// [`usb_device::bus::UsbBusAllocator`], which is not sync.
///
/// See <https://github.com/rust-embedded-community/usb-device/pull/162>.
unsafe impl<T> Sync for InterruptCell<T> {}

impl<T> InterruptCell<T> {
    pub const fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    #[allow(clippy::mut_from_ref)]
    pub fn init(&self, inner: T) -> &mut T {
        unsafe { (*self.0.get()).write(inner) }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_inner_mut(&self) -> &mut T {
        unsafe { (*self.0.get()).assume_init_mut() }
    }
}
