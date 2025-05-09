use std::{
    convert::TryInto,
    os::fd::RawFd,
    ptr::NonNull,
    rc::Rc,
    task::{Poll, Waker},
    time::Duration,
};

use gtk::glib::{self, ControlFlow, IOCondition};
use rusb::{
    Context, DeviceHandle, Error, Result, UsbContext,
    constants::{
        LIBUSB_ENDPOINT_DIR_MASK, LIBUSB_ENDPOINT_OUT, LIBUSB_ERROR_INVALID_PARAM,
        LIBUSB_ERROR_NO_DEVICE, LIBUSB_ERROR_NOT_SUPPORTED, LIBUSB_TRANSFER_CANCELLED,
        LIBUSB_TRANSFER_COMPLETED, LIBUSB_TRANSFER_ERROR, LIBUSB_TRANSFER_NO_DEVICE,
        LIBUSB_TRANSFER_OVERFLOW, LIBUSB_TRANSFER_STALL, LIBUSB_TRANSFER_TIMED_OUT,
    },
    ffi,
};

pub struct InterruptTransfer {
    dev_handle: Rc<DeviceHandle<Context>>,
    endpoint: u8,
    ptr: NonNull<ffi::libusb_transfer>,
    buffer: Vec<u8>,
    state: TransferState,
}

impl InterruptTransfer {
    pub fn new(dev_handle: Rc<DeviceHandle<Context>>, endpoint: u8, buffer: Vec<u8>) -> Self {
        // non-isochronous endpoints (e.g. control, bulk, interrupt) specify a value of 0
        // This is step 1 of async API

        let ptr = unsafe {
            NonNull::new(ffi::libusb_alloc_transfer(0)).expect("Could not allocate transfer!")
        };

        Self {
            dev_handle,
            endpoint,
            ptr,
            buffer,
            state: TransferState::MustSubmit,
        }
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

        let user_data = Box::into_raw(Box::new(waker)).cast::<libc::c_void>();

        unsafe {
            ffi::libusb_fill_interrupt_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.endpoint,
                self.buffer.as_mut_ptr(),
                length,
                InterruptTransfer::transfer_cb,
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

    extern "system" fn fd_added_cb(
        fd: libc::c_int,
        events: libc::c_short,
        user_data: *mut libc::c_void,
    ) {
        let context = unsafe { Context::from_raw(user_data.cast()) };
        Self::monitor_pollfd(context, fd, events);
    }

    fn monitor_pollfd(context: Context, fd: RawFd, events: libc::c_short) {
        let condition = IOCondition::from_bits_truncate(events.try_into().unwrap());

        let handle_events_fn = move |_fd, _condition| {
            context.handle_events(Some(Duration::ZERO)).unwrap();
            ControlFlow::Continue
        };
        glib::source::unix_fd_add_local(fd, condition, handle_events_fn);
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

impl Future for InterruptTransfer {
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

impl Drop for InterruptTransfer {
    fn drop(&mut self) {
        match self.state {
            TransferState::MustPoll => self.cancel(),
            TransferState::MustSubmit | TransferState::Completed => unsafe {
                ffi::libusb_free_transfer(self.ptr.as_ptr());
            },
        }
    }
}

enum TransferState {
    MustSubmit,
    MustPoll,
    Completed,
}

pub fn init(context: &Context) {
    unsafe {
        let pollfds_ptr = ffi::libusb_get_pollfds(context.as_raw());

        let mut current = pollfds_ptr;

        while !(*current).is_null() {
            let pollfd = &**current;

            let fd = pollfd.fd;
            let events = pollfd.events;

            InterruptTransfer::monitor_pollfd(context.clone(), fd, events);
            current = current.add(1);
        }

        ffi::libusb_set_pollfd_notifiers(
            context.as_raw(),
            Some(InterruptTransfer::fd_added_cb),
            None,
            context.as_raw().cast(),
        );
    }
}
