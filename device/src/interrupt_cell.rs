use core::{cell::UnsafeCell, mem::MaybeUninit};

pub struct InterruptCell<T>(UnsafeCell<MaybeUninit<T>>);

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
