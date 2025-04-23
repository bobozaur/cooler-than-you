use std::{thread, time::Duration};

use anyhow::{Context as _, Result as AnyResult, anyhow, bail};
use itertools::Itertools;
use rusb::{Context, Direction, Error, TransferType, UsbContext};
use shared::{Command, DeviceState, USB_PID, USB_VID};

fn main() -> AnyResult<()> {
    let (device, device_desc) = Context::new()
        .context("unable to initialize libusb")?
        .devices()
        .context("unable to enumerate devices")?
        .iter()
        .filter_map(|device| device.device_descriptor().ok().map(|desc| (device, desc)))
        .filter(|(_, desc)| desc.vendor_id() == USB_VID && desc.product_id() == USB_PID)
        .exactly_one()
        .map_err(|err| anyhow!("found {} device descriptors", err.count()))?;

    let config_desc = (0..device_desc.num_configurations())
        .filter_map(|i| device.config_descriptor(i).ok())
        .exactly_one()
        .map_err(|err| anyhow!("found {} config descriptors", err.count()))?;

    let interface_desc = config_desc
        .interfaces()
        .flat_map(|i| i.descriptors())
        .exactly_one()
        .map_err(|err| anyhow!("found {} interface descriptors", err.count()))?;

    let in_endpoint_desc = interface_desc
        .endpoint_descriptors()
        .filter(|edesc| edesc.direction() == Direction::In)
        .filter(|edesc| edesc.transfer_type() == TransferType::Interrupt)
        .exactly_one()
        .map_err(|err| anyhow!("found {} IN interrupt endpoint descriptors", err.count()))?;

    let out_endpoint_desc = interface_desc
        .endpoint_descriptors()
        .filter(|edesc| edesc.direction() == Direction::Out)
        .filter(|edesc| edesc.transfer_type() == TransferType::Interrupt)
        .exactly_one()
        .map_err(|err| anyhow!("found {} OUT interrupt endpoint descriptors", err.count()))?;

    let handle = device.open()?;
    handle.set_auto_detach_kernel_driver(true)?;

    let mut buf = [0; 1];

    loop {
        handle.claim_interface(interface_desc.interface_number())?;
        handle.set_alternate_setting(
            interface_desc.interface_number(),
            interface_desc.setting_number(),
        )?;

        match handle.read_interrupt(
            in_endpoint_desc.address(),
            &mut buf,
            Duration::from_millis(500),
        ) {
            Ok(1) => {
                let device_state = DeviceState::try_from(buf[0])?;
                println!("State: {device_state:?}");
            }
            Ok(n) => bail!("unexpected amount of {n} bytes read"),
            Err(Error::Timeout) => (),
            Err(e) => bail!(e),
        }

        match handle.write_interrupt(
            out_endpoint_desc.address(),
            &[Command::LedsColorChange.into()],
            Duration::from_millis(500),
        ) {
            Ok(n) => println!("Wrote {n} bytes"),
            Err(e) => panic!("{e}"),
        }

        handle.release_interface(0).unwrap();

        thread::sleep(Duration::from_millis(500));
    }
}
