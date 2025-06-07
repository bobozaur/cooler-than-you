mod device;
mod exactly_one;
mod fd_callbacks;
mod indicator;
mod menu;

pub use anyhow::Result as AnyResult;
pub use device::Device;
use futures_util::TryFutureExt;
use gtk::glib::{self, JoinHandle};
pub use indicator::Indicator;

/// Spawns a fallible future on the event loop, quiting it by calling [`gtk::main_quit`] if the
/// future returns an error.
fn spawn_local<F>(fut: F) -> JoinHandle<Result<F::Ok, F::Error>>
where
    F: TryFutureExt + 'static,
{
    glib::spawn_future_local(fut.inspect_err(|_| gtk::main_quit()))
}
