use std::{sync::Arc, time::Duration};

use anyhow::{Context as _, anyhow};
use itertools::Itertools;
use rusb::{Context, DeviceHandle, Direction, Error as RusbError, TransferType, UsbContext};
use shared::{Command, DeviceState, USB_PID, USB_VID};

use crate::AnyResult;

#[derive(Clone, Debug)]
pub struct Device {
    handle: Arc<DeviceHandle<Context>>,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl Device {
    pub fn new() -> AnyResult<Self> {
        let (device, device_desc) = Context::new()
            .context("unable to initialize libusb")?
            .devices()
            .context("unable to enumerate devices")?
            .iter()
            .filter_map(|device| device.device_descriptor().ok().map(|desc| (device, desc)))
            .filter(|(_, desc)| desc.vendor_id() == USB_VID && desc.product_id() == USB_PID)
            .exactly_one()
            .map_err(|err| anyhow!("{err}"))
            .context("device descriptor")?;

        let config_desc = (0..device_desc.num_configurations())
            .filter_map(|i| device.config_descriptor(i).ok())
            .exactly_one()
            .map_err(|err| anyhow!("{err}"))
            .context("config descriptor")?;

        let interface_desc = config_desc
            .interfaces()
            .flat_map(|i| i.descriptors())
            .exactly_one()
            .map_err(|err| anyhow!("{err}"))
            .context("interface descriptor")?;

        let in_endpoint_desc = interface_desc
            .endpoint_descriptors()
            .filter(|edesc| edesc.direction() == Direction::In)
            .filter(|edesc| edesc.transfer_type() == TransferType::Interrupt)
            .exactly_one()
            .map_err(|err| anyhow!("{err}"))
            .context("IN interrupt endpoint descriptor")?;

        let out_endpoint_desc = interface_desc
            .endpoint_descriptors()
            .filter(|edesc| edesc.direction() == Direction::Out)
            .filter(|edesc| edesc.transfer_type() == TransferType::Interrupt)
            .exactly_one()
            .map_err(|err| anyhow!("{err}"))
            .context("OUT interrupt endpoint descriptor")?;

        let handle = device.open()?;
        handle.set_auto_detach_kernel_driver(true)?;
        handle.unconfigure()?;
        handle.set_active_configuration(config_desc.number())?;
        handle.claim_interface(interface_desc.interface_number())?;
        handle.set_alternate_setting(
            interface_desc.interface_number(),
            interface_desc.setting_number(),
        )?;

        Ok(Self {
            handle: Arc::new(handle),
            in_endpoint_address: in_endpoint_desc.address(),
            out_endpoint_address: out_endpoint_desc.address(),
        })
    }

    pub fn recv_state(&self) -> AnyResult<Option<DeviceState>> {
        let mut buf = [0; 1];

        let res = self.handle.read_interrupt(
            self.in_endpoint_address,
            &mut buf,
            Duration::from_millis(500),
        );

        let state = match res {
            Ok(_) => DeviceState::try_from(buf[0])?,
            Err(RusbError::Timeout) => return Ok(None),
            Err(e) => Err(e)?,
        };

        Ok(Some(state))
    }

    pub fn send_commnad(&self, command: Command) -> AnyResult<()> {
        self.handle.write_interrupt(
            self.out_endpoint_address,
            &[command.into()],
            Duration::from_millis(500),
        )?;

        Ok(())
    }
}
