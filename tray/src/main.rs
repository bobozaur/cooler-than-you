//! TODO:
//! - comments and docs
//! - check if things like udevadm have to be run on install
//!
//! System tray icon taken from: <https://www.svgrepo.com/svg/503337/fan-circled>.
//! - 64x64px
//! - stored in /usr/share/icons/hicolor/64x64/status/cooler-than-you-symbolic.svg, chmod 644, chown
//!   root

use tracing_subscriber::{EnvFilter, fmt};
use tray::{AnyResult, Device, Indicator};

fn main() -> AnyResult<()> {
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Indicator::new()?.run(Device::new()?);
    Ok(())
}
