use std::{cell::RefCell, collections::BTreeMap, ffi::c_short, os::fd::RawFd, time::Duration};

use gtk::glib::{self, ControlFlow, IOCondition, SourceId};
use rusb::{Context, UsbContext};
use rusb_async::FdMonitor;

#[derive(Debug)]
pub struct GlibFdHandlerContext {
    context: Context,
    fd_sources_map: RefCell<BTreeMap<RawFd, SourceId>>,
}

impl GlibFdHandlerContext {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            fd_sources_map: RefCell::new(BTreeMap::default()),
        }
    }
}

impl FdMonitor<Context> for GlibFdHandlerContext {
    fn context(&self) -> &Context {
        &self.context
    }

    fn fd_added(&self, fd: RawFd, events: c_short) {
        let context = self.context.clone();
        let handle_events_fn = move |_, _| {
            context.handle_events(Some(Duration::ZERO)).unwrap();
            ControlFlow::Continue
        };

        let condition = IOCondition::from_bits_truncate(events.try_into().unwrap());
        let source_id = glib::source::unix_fd_add_local(fd, condition, handle_events_fn);
        self.fd_sources_map.borrow_mut().insert(fd, source_id);
    }

    fn fd_removed(&self, fd: RawFd) {
        if let Some(source_id) = self.fd_sources_map.borrow_mut().remove(&fd) {
            source_id.remove();
        }
    }
}
