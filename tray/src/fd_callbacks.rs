use std::{collections::BTreeMap, os::fd::RawFd, sync::Mutex, time::Duration};

use gtk::glib::{self, ControlFlow, IOCondition, SourceId};
use rusb_async::{AsyncUsbContext, FdCallbacks, FdEvents};

#[derive(Debug, Default)]
pub struct GlibFdCallbacks {
    fd_sources_map: Mutex<BTreeMap<RawFd, SourceId>>,
}

impl<C> FdCallbacks<C> for GlibFdCallbacks
where
    C: AsyncUsbContext,
{
    fn fd_added(&self, context: C, fd: RawFd, events: FdEvents) {
        let handle_events_fn = move |_, _| {
            context.handle_events(Some(Duration::ZERO)).unwrap();
            ControlFlow::Continue
        };

        let condition = match events {
            FdEvents::Read => IOCondition::IN,
            FdEvents::Write => IOCondition::OUT,
            FdEvents::ReadWrite => IOCondition::IN.union(IOCondition::OUT),
            FdEvents::Other => return,
        };

        let source_id = glib::source::unix_fd_add_local(fd, condition, handle_events_fn);
        self.fd_sources_map.lock().unwrap().insert(fd, source_id);
    }

    fn fd_removed(&self, fd: RawFd) {
        if let Some(source_id) = self.fd_sources_map.lock().unwrap().remove(&fd) {
            source_id.remove();
        }
    }
}
