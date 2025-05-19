//! TODO:
//! - debian packaging
//! - comments and docs
//! - fancy icon

use tracing_subscriber::{EnvFilter, fmt};
use tray::{AnyResult, Device, Indicator};

fn main() -> AnyResult<()> {
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Indicator::new()?.run(Device::new()?);
    Ok(())
}
