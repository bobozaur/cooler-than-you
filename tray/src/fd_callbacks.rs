use std::{collections::BTreeMap, os::fd::RawFd, sync::Mutex, time::Duration};

use gtk::glib::{self, ControlFlow, IOCondition, SourceId};
use rusb_async::{AsyncUsbContext, FdCallbacks, FdEvents};
use tracing::instrument;

/// File descriptor callback state holding struct, implementor of [`FdCallbacks`].
#[derive(Debug, Default)]
pub struct GlibFdCallbacks {
    // NOTE: `rusb` is being threadsafe, therefore a [`RefCell`] would not suffice.
    fd_sources_map: Mutex<BTreeMap<RawFd, SourceId>>,
}

impl<C> FdCallbacks<C> for GlibFdCallbacks
where
    C: AsyncUsbContext,
{
    #[instrument(skip(self, context))]
    fn fd_added(&self, context: C, fd: RawFd, events: FdEvents) {
        let handle_events_fn = move |_, _| {
            context.handle_events(Some(Duration::ZERO)).unwrap();
            ControlFlow::Continue
        };

        let condition = match events {
            FdEvents::Read => IOCondition::IN,
            FdEvents::Write => IOCondition::OUT,
            FdEvents::ReadWrite => IOCondition::IN.union(IOCondition::OUT),
        };

        tracing::debug!("adding fd {fd} - condition: {condition:?} as source");
        let source_id = glib::source::unix_fd_add(fd, condition, handle_events_fn);
        self.fd_sources_map.lock().unwrap().insert(fd, source_id);
    }

    #[instrument(skip(self))]
    fn fd_removed(&self, fd: RawFd) {
        if let Some(source_id) = self.fd_sources_map.lock().unwrap().remove(&fd) {
            tracing::debug!("removing fd {fd} as source");
            source_id.remove();
        }
    }
}
