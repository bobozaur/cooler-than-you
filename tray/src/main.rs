//! TODO:
//! - comments and docs
//!
//! System tray icon taken from: <https://www.svgrepo.com/svg/503337/fan-circled>.
//! - 64x64px
//! - stored in /usr/share/icons/hicolor/64x64/status/cooler-than-you-symbolic.svg, chmod 644, chown
//!   root

use anyhow::anyhow;
use clap::{Parser, builder::ValueParser};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};
use tray::{AnyResult, Device, Indicator};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, bin_name = "cooler-than-you")]
struct Opts {
    /// Comma-separated list of temperatures for the fan curve
    #[arg(default_value = "60,65,70,75,80")]
    #[arg(value_parser = ValueParser::new(Opts::parse_fan_curve))]
    fan_curve: [f32; 5],
}

impl Opts {
    fn parse_fan_curve(arg: &str) -> AnyResult<[f32; 5]> {
        arg.split(',')
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| anyhow!("expecting 5 fan curve temperature parameters"))
    }
}

fn main() -> AnyResult<()> {
    let Opts { fan_curve } = Opts::parse();

    let journald_layer = tracing_journald::Layer::new()?
        .with_syslog_identifier("cooler-than-you".to_owned())
        .with_filter(EnvFilter::from_default_env());
    tracing_subscriber::registry().with(journald_layer).init();

    Indicator::new(fan_curve)?.run(Device::new()?);

    Ok(())
}
