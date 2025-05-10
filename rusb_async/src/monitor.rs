use std::{
    ffi::{c_int, c_short, c_void},
    os::fd::RawFd,
    ptr::{self, NonNull},
};

use rusb::ffi::{self, libusb_context};

#[derive(Debug)]
pub struct FdHandler<T>(*mut T)
where
    T: FdHandlerContext;

impl<T> FdHandler<T>
where
    T: FdHandlerContext,
{
    pub fn new(handler_context: T) -> Self {
        let context = handler_context.raw_context();

        unsafe {
            if let Some(mut pollfds_ptr) = NonNull::new(ffi::libusb_get_pollfds(context).cast_mut())
            {
                while let Some(pollfd) = NonNull::new(*pollfds_ptr.as_ptr()) {
                    let fd = pollfd.as_ref().fd;
                    let events = pollfd.as_ref().events;

                    handler_context.fd_added(fd, events);
                    pollfds_ptr = pollfds_ptr.add(1);
                }
            }

            let handler_context = Box::into_raw(Box::new(handler_context));
            let user_data = handler_context.cast();

            ffi::libusb_set_pollfd_notifiers(
                context,
                Some(Self::fd_added_cb),
                Some(Self::fd_removed_cb),
                user_data,
            );

            Self(handler_context)
        }
    }

    extern "system" fn fd_added_cb(fd: c_int, events: c_short, user_data: *mut c_void)
    where
        T: FdHandlerContext,
    {
        unsafe { &*user_data.cast::<T>() }.fd_added(fd, events);
    }

    extern "system" fn fd_removed_cb(fd: c_int, user_data: *mut c_void)
    where
        T: FdHandlerContext,
    {
        unsafe { &*user_data.cast::<T>() }.fd_removed(fd);
    }
}

impl<T> Drop for FdHandler<T>
where
    T: FdHandlerContext,
{
    fn drop(&mut self) {
        unsafe {
            let handler_context = Box::from_raw(self.0);

            ffi::libusb_set_pollfd_notifiers(
                handler_context.raw_context(),
                None,
                None,
                ptr::null_mut(),
            );
        }
    }
}

pub trait FdHandlerContext {
    fn raw_context(&self) -> *mut libusb_context;

    fn fd_added(&self, fd: RawFd, events: c_short);

    fn fd_removed(&self, fd: RawFd);
}
