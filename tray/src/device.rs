use std::{rc::Rc, time::Duration};

use anyhow::{Context as _, anyhow};
use itertools::Itertools;
use rusb::{Context, DeviceHandle, Direction, Error as RusbError, TransferType, UsbContext};
use shared::{Command, DeviceState, USB_PID, USB_VID};

use crate::AnyResult;

#[derive(Clone, Debug)]
pub struct Device {
    handle: Rc<DeviceHandle<Context>>,
    interface_number: u8,
    setting_number: u8,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl Device {
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

        if handle.kernel_driver_active(interface_desc.interface_number())? {
            handle.detach_kernel_driver(interface_desc.interface_number())?;
        }

        handle
            .set_active_configuration(config_desc.number())
            .context("setting config number")?;

        Ok(Self {
            handle: Rc::new(handle),
            interface_number: interface_desc.interface_number(),
            setting_number: interface_desc.setting_number(),
            in_endpoint_address: in_endpoint_desc.address(),
            out_endpoint_address: out_endpoint_desc.address(),
        })
    }

    ///
    /// # Errors
    pub fn recv_state(&self) -> AnyResult<Option<DeviceState>> {
        self.operate(|| self.recv_state_impl())
    }

    ///
    /// # Errors
    pub fn send_command(&self, command: Command) -> AnyResult<()> {
        self.operate(|| self.send_command_impl(command))
    }

    fn recv_state_impl(&self) -> AnyResult<Option<DeviceState>> {
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

    fn send_command_impl(&self, command: Command) -> AnyResult<()> {
        self.handle.write_interrupt(
            self.out_endpoint_address,
            &[command.into()],
            Duration::from_millis(500),
        )?;

        Ok(())
    }

    fn operate<F, T>(&self, f: F) -> AnyResult<T>
    where
        F: FnOnce() -> AnyResult<T>,
    {
        self.handle
            .claim_interface(self.interface_number)
            .context("claiming interface")?;
        self.handle
            .set_alternate_setting(self.interface_number, self.setting_number)
            .context("choosing alternate setting")?;

        let res = f();

        self.handle
            .release_interface(self.interface_number)
            .context("releasing interface")?;

        res
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if let Ok(false) = self.handle.kernel_driver_active(self.interface_number) {
            self.handle.attach_kernel_driver(self.interface_number).ok();
        }
    }
}
