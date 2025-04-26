use std::time::Duration;

use anyhow::{Context as _, anyhow};
use itertools::Itertools;
use rusb::{Context, DeviceHandle, Direction, TransferType, UsbContext};
use shared::{Command, DeviceState, USB_PID, USB_VID};

use crate::AnyResult;

#[derive(Debug)]
pub struct Cooler {
    handle: DeviceHandle<Context>,
    interface_number: u8,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl Cooler {
    ///
    /// # Errors
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

        let config_number = config_desc.number();

        let interface_desc = config_desc
            .interfaces()
            .flat_map(|i| i.descriptors())
            .exactly_one()
            .map_err(|err| anyhow!("{err}"))
            .context("interface descriptor")?;

        let interface_number = interface_desc.interface_number();
        let setting_number = interface_desc.setting_number();

        let in_endpoint_address = interface_desc
            .endpoint_descriptors()
            .filter(|edesc| edesc.direction() == Direction::In)
            .filter(|edesc| edesc.transfer_type() == TransferType::Interrupt)
            .exactly_one()
            .map_err(|err| anyhow!("{err}"))
            .context("IN interrupt endpoint descriptor")
            .map(|e| e.address())?;

        let out_endpoint_address = interface_desc
            .endpoint_descriptors()
            .filter(|edesc| edesc.direction() == Direction::Out)
            .filter(|edesc| edesc.transfer_type() == TransferType::Interrupt)
            .exactly_one()
            .map_err(|err| anyhow!("{err}"))
            .context("OUT interrupt endpoint descriptor")
            .map(|e| e.address())?;

        let handle = device.open()?;

        if handle.kernel_driver_active(interface_number)? {
            handle.detach_kernel_driver(interface_number)?;
        }

        handle
            .set_active_configuration(config_number)
            .context("setting config number")?;

        handle
            .claim_interface(interface_number)
            .context("claiming interface")?;
        handle
            .set_alternate_setting(interface_number, setting_number)
            .context("choosing alternate setting")?;

        Ok(Self {
            handle,
            interface_number,
            in_endpoint_address,
            out_endpoint_address,
        })
    }

    ///
    /// # Errors
    pub fn recv_state(&self) -> AnyResult<DeviceState> {
        let mut buf = [0; 1];

        let read = self.handle.read_interrupt(
            self.in_endpoint_address,
            &mut buf,
            Duration::from_millis(1),
        )?;

        if read != 1 {
            anyhow::bail!("device state not read");
        }

        DeviceState::try_from(buf[0]).map_err(From::from)
    }

    ///
    /// # Errors
    pub fn send_command(&self, command: Command) -> AnyResult<()> {
        let written = self.handle.write_interrupt(
            self.out_endpoint_address,
            &[command.into()],
            Duration::from_millis(500),
        )?;

        if written != 1 {
            anyhow::bail!("command not written");
        }

        Ok(())
    }
}

impl Drop for Cooler {
    fn drop(&mut self) {
        if let Ok(false) = self.handle.kernel_driver_active(self.interface_number) {
            self.handle.attach_kernel_driver(self.interface_number).ok();
        }
    }
}
