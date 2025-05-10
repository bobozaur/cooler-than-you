use std::{ffi::c_short, os::fd::RawFd, time::Duration};

use gtk::glib::{self, ControlFlow, IOCondition};
use rusb::{Context, UsbContext, ffi::libusb_context};
use rusb_async::FdHandlerContext;

#[derive(Debug)]
pub struct GlibFdHandlerContext {
    context: Context,
}

impl GlibFdHandlerContext {
    pub fn new(context: Context) -> Self {
        Self { context }
    }
}

impl FdHandlerContext for GlibFdHandlerContext {
    fn raw_context(&self) -> *mut libusb_context {
        self.context.as_raw()
    }

    fn fd_added(&self, fd: RawFd, events: c_short) {
        let context = self.context.clone();
        let handle_events_fn = move |_, _| {
            context.handle_events(Some(Duration::ZERO)).unwrap();
            ControlFlow::Continue
        };

        let condition = IOCondition::from_bits_truncate(events.try_into().unwrap());
        glib::source::unix_fd_add_local(fd, condition, handle_events_fn);
    }

    fn fd_removed(&self, _fd: RawFd) {}
}
