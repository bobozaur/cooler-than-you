use std::{
    ffi::{c_int, c_short, c_void},
    marker::PhantomData,
    os::fd::RawFd,
    ptr::{self, NonNull},
};

use rusb::{UsbContext, ffi};

#[derive(Debug)]
pub struct FdHandler<C, M>
where
    C: UsbContext,
    M: FdMonitor<C>,
{
    fd_monitor_ptr: *mut M,
    context: PhantomData<fn() -> C>,
}

impl<C, M> FdHandler<C, M>
where
    C: UsbContext,
    M: FdMonitor<C>,
{
    pub fn new(fd_monitor: M) -> Self {
        let context = fd_monitor.context().as_raw();

        unsafe {
            let pollfds_opt_ptr = NonNull::new(ffi::libusb_get_pollfds(context).cast_mut());
            if let Some(mut pollfds_ptr) = pollfds_opt_ptr {
                while let Some(pollfd) = NonNull::new(*pollfds_ptr.as_ptr()) {
                    let fd = pollfd.as_ref().fd;
                    let events = pollfd.as_ref().events;

                    fd_monitor.fd_added(fd, events);
                    pollfds_ptr = pollfds_ptr.add(1);
                }
            }

            let fd_monitor_ptr = Box::into_raw(Box::new(fd_monitor));
            let user_data = fd_monitor_ptr.cast();

            ffi::libusb_set_pollfd_notifiers(
                context,
                Some(Self::fd_added_cb),
                Some(Self::fd_removed_cb),
                user_data,
            );

            Self {
                fd_monitor_ptr,
                context: PhantomData,
            }
        }
    }

    extern "system" fn fd_added_cb(fd: c_int, events: c_short, user_data: *mut c_void) {
        unsafe { &*user_data.cast::<M>() }.fd_added(fd, events);
    }

    extern "system" fn fd_removed_cb(fd: c_int, user_data: *mut c_void) {
        unsafe { &*user_data.cast::<M>() }.fd_removed(fd);
    }
}

impl<C, M> Drop for FdHandler<C, M>
where
    C: UsbContext,
    M: FdMonitor<C>,
{
    fn drop(&mut self) {
        unsafe {
            let fd_monitor = Box::from_raw(self.fd_monitor_ptr);

            ffi::libusb_set_pollfd_notifiers(
                fd_monitor.context().as_raw(),
                None,
                None,
                ptr::null_mut(),
            );
        }
    }
}

pub trait FdMonitor<C>
where
    C: UsbContext,
{
    fn context(&self) -> &C;

    fn fd_added(&self, fd: RawFd, events: c_short);

    fn fd_removed(&self, fd: RawFd);
}
