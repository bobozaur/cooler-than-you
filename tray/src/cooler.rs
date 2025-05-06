use std::{rc::Rc, time::Duration};

use anyhow::{Context as _, anyhow};
use itertools::Itertools;
use rusb::{Context, DeviceHandle, Direction, TransferType, UsbContext};
use shared::{Command, DeviceState, USB_PID, USB_VID};

use crate::AnyResult;

#[derive(Clone, Debug)]
pub struct Cooler {
    handle: Rc<DeviceHandle<Context>>,
    interface_number: u8,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl Cooler {
    ///
    /// # Errors
    pub fn new() -> AnyResult<Self> {
        let handle = Context::new()
            .context("unable to initialize libusb")?
            .open_device_with_vid_pid(USB_VID, USB_PID)
            .context("unable to open device")
            .map(Rc::new)?;

        let device = handle.device();
        let device_desc = device.device_descriptor().context("device descriptor")?;

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

        let cooler = Self {
            handle,
            interface_number,
            in_endpoint_address,
            out_endpoint_address,
        };

        Ok(cooler)
    }

    ///
    /// # Errors
    pub fn recv_state(&self) -> AnyResult<Option<DeviceState>> {
        // TODO: Figure out why something like this is needed.
        //       Subsequent reads do not succeed otherwise.
        self.handle.clear_halt(self.in_endpoint_address)?;

        let mut buf = [0; 1];

        match self.handle.read_interrupt(
            self.in_endpoint_address,
            &mut buf,
            Duration::from_millis(10),
        ) {
            Ok(1) => Ok(Some(DeviceState::try_from(buf[0])?)),
            Ok(_) => anyhow::bail!("device state not read"),
            Err(rusb::Error::Timeout) => Ok(None),
            Err(e) => Err(e)?,
        }
    }

    ///
    /// # Errors
    pub fn send_command(&self, command: Command) -> AnyResult<()> {
        match self.handle.write_interrupt(
            self.out_endpoint_address,
            &[command.into()],
            Duration::from_millis(500),
        ) {
            Ok(1) => Ok(()),
            Ok(_) => anyhow::bail!("command not written"),
            Err(e) => Err(e)?,
        }
    }
}

impl Drop for Cooler {
    fn drop(&mut self) {
        if let Ok(false) = self.handle.kernel_driver_active(self.interface_number) {
            self.handle.attach_kernel_driver(self.interface_number).ok();
        }
    }
}
