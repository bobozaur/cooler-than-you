use std::{
    convert::TryInto,
    ptr::NonNull,
    sync::Arc,
    task::{Poll, Waker},
};

use rusb::{
    DeviceHandle, Error, Result, UsbContext,
    constants::{
        LIBUSB_ENDPOINT_DIR_MASK, LIBUSB_ENDPOINT_OUT, LIBUSB_ERROR_INVALID_PARAM,
        LIBUSB_ERROR_NO_DEVICE, LIBUSB_ERROR_NOT_SUPPORTED, LIBUSB_TRANSFER_CANCELLED,
        LIBUSB_TRANSFER_COMPLETED, LIBUSB_TRANSFER_ERROR, LIBUSB_TRANSFER_NO_DEVICE,
        LIBUSB_TRANSFER_OVERFLOW, LIBUSB_TRANSFER_STALL, LIBUSB_TRANSFER_TIMED_OUT,
    },
    ffi,
};

use crate::{FdHandler, FdMonitor};

#[derive(Debug)]
pub struct InterruptTransfer<C>
where
    C: UsbContext,
{
    dev_handle: Arc<DeviceHandle<C>>,
    endpoint: u8,
    ptr: NonNull<ffi::libusb_transfer>,
    buffer: Vec<u8>,
    state: TransferState,
}

impl<C> InterruptTransfer<C>
where
    C: UsbContext,
{
    /// # Errors
    pub fn new<M>(
        dev_handle: Arc<DeviceHandle<C>>,
        endpoint: u8,
        buffer: Vec<u8>,
        _fd_handler: &FdHandler<C, M>,
    ) -> Result<Self>
    where
        M: FdMonitor<C>,
    {
        // non-isochronous endpoints (e.g. control, bulk, interrupt) specify a value of 0
        // This is step 1 of async API

        let Some(ptr) = NonNull::new(unsafe { ffi::libusb_alloc_transfer(0) }) else {
            return Err(Error::Other);
        };

        Ok(Self {
            dev_handle,
            endpoint,
            ptr,
            buffer,
            state: TransferState::MustSubmit,
        })
    }

    // Step 3 of async API
    fn submit(&mut self, waker: Waker) -> Result<()> {
        let length = if self.endpoint & LIBUSB_ENDPOINT_DIR_MASK == LIBUSB_ENDPOINT_OUT {
            // for OUT endpoints: the currently valid data in the buffer
            self.buffer.len()
        } else {
            // for IN endpoints: the full capacity
            self.buffer.capacity()
        }
        .try_into()
        .unwrap();

        let user_data = Box::into_raw(Box::new(waker)).cast();

        unsafe {
            ffi::libusb_fill_interrupt_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.endpoint,
                self.buffer.as_mut_ptr(),
                length,
                Self::transfer_cb,
                user_data,
                0,
            );
        }

        let errno = unsafe { ffi::libusb_submit_transfer(self.ptr.as_ptr()) };

        match errno {
            0 => Ok(()),
            LIBUSB_ERROR_NO_DEVICE => Err(Error::NoDevice),
            LIBUSB_ERROR_NOT_SUPPORTED => Err(Error::NotSupported),
            LIBUSB_ERROR_INVALID_PARAM => Err(Error::InvalidParam),
            _ => Err(Error::Other),
        }
    }

    // Part of step 4 of async API the transfer is finished being handled when
    // `poll()` is called.
    extern "system" fn transfer_cb(transfer: *mut ffi::libusb_transfer) {
        // Safety: transfer is still valid because libusb just completed
        // it but we haven't told anyone yet. user_data remains valid
        // because it is freed only with the transfer.
        // After the store to completed, these may no longer be valid if
        // the polling thread freed it after seeing it completed.
        unsafe {
            let transfer = &mut *transfer;

            if transfer.status == LIBUSB_TRANSFER_CANCELLED {
                ffi::libusb_free_transfer(transfer);
            } else {
                Box::from_raw(transfer.user_data.cast::<Waker>()).wake();
            }
        };
    }

    /// Prerequisite: self.buffer ans self.ptr are both correctly set
    fn take_buffer(&mut self) -> Vec<u8> {
        debug_assert!(self.transfer().length >= self.transfer().actual_length);
        unsafe {
            let len = self.transfer().actual_length.try_into().unwrap();
            self.buffer.set_len(len);
        }

        let transfer_struct = unsafe { self.ptr.as_mut() };

        let data = std::mem::take(&mut self.buffer);

        // Update transfer struct for new buffer
        transfer_struct.actual_length = 0; // TODO: Is this necessary?
        transfer_struct.buffer = self.buffer.as_mut_ptr();
        transfer_struct.length = self.buffer.capacity().try_into().unwrap();

        data
    }

    fn transfer(&self) -> &ffi::libusb_transfer {
        // Safety: transfer remains valid as long as self
        unsafe { self.ptr.as_ref() }
    }

    fn cancel(&mut self) {
        unsafe { ffi::libusb_cancel_transfer(self.ptr.as_ptr()) };
    }

    fn handle_completed(&mut self) -> Result<Vec<u8>> {
        let err = match self.transfer().status {
            LIBUSB_TRANSFER_COMPLETED => return Ok(self.take_buffer()),
            LIBUSB_TRANSFER_CANCELLED => Error::Interrupted,
            LIBUSB_TRANSFER_NO_DEVICE => Error::NoDevice,
            LIBUSB_TRANSFER_OVERFLOW => Error::Overflow,
            LIBUSB_TRANSFER_TIMED_OUT => Error::Timeout,
            LIBUSB_TRANSFER_STALL => Error::Pipe,
            LIBUSB_TRANSFER_ERROR => Error::Io,
            _ => Error::Other,
        };
        Err(err)
    }
}

impl<C> Future for InterruptTransfer<C>
where
    C: UsbContext,
{
    type Output = Result<Vec<u8>>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.state {
            TransferState::MustSubmit => {
                self.submit(cx.waker().clone())?;
                self.state = TransferState::MustPoll;
                Poll::Pending
            }
            TransferState::MustPoll => {
                self.state = TransferState::Completed;
                Poll::Ready(self.handle_completed())
            }
            TransferState::Completed => Poll::Ready(Err(Error::Other)),
        }
    }
}

impl<C> Drop for InterruptTransfer<C>
where
    C: UsbContext,
{
    fn drop(&mut self) {
        match self.state {
            TransferState::MustPoll => self.cancel(),
            TransferState::MustSubmit | TransferState::Completed => unsafe {
                ffi::libusb_free_transfer(self.ptr.as_ptr());
            },
        }
    }
}

#[derive(Debug)]
enum TransferState {
    MustSubmit,
    MustPoll,
    Completed,
}
