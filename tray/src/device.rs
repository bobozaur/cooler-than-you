use std::{
    rc::Rc,
    sync::Arc,
    task::{Poll, ready},
};

use anyhow::{Context as _, anyhow};
use futures_core::Stream;
use futures_util::FutureExt;
use itertools::Itertools;
use rusb::{
    Context, Device as RusbDevice, DeviceHandle, Direction, LogCallbackMode, LogLevel,
    TransferType, UsbContext,
};
use rusb_async::{fd::FdHandler, transfer::InterruptTransfer};
use shared::{DeviceCommand, DeviceState, USB_MANUFACTURER, USB_PID, USB_PRODUCT, USB_VID};

use crate::{AnyResult, fd_handler::GlibFdHandlerContext};

#[derive(Clone, Debug)]
pub struct Device {
    /// Using an [`Arc`] because that's what the async libusb transfers
    /// required on construction.
    handle: Arc<DeviceHandle<Context>>,
    fd_handler: Rc<FdHandler<Context, GlibFdHandlerContext>>,
    interface_number: u8,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl Device {
    ///
    /// # Errors
    pub fn new() -> AnyResult<Self> {
        let mut context = Context::new().context("unable to initialize libusb")?;

        let log_fn = Box::new(|_, message| eprintln!("{message}"));
        context.set_log_level(LogLevel::Warning);
        context.set_log_callback(log_fn, LogCallbackMode::Global);

        let fd_handler = Rc::new(FdHandler::new(GlibFdHandlerContext::new(context.clone())));

        let handle = context
            .devices()?
            .iter()
            .filter_map(Self::device_filter)
            .exactly_one()
            .map(Arc::new)
            .map_err(|e| anyhow!("{e}"))
            .context("opening device")?;

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
            fd_handler,
            interface_number,
            in_endpoint_address,
            out_endpoint_address,
        };

        Ok(cooler)
    }

    /// # Errors
    pub fn state_stream(&self) -> AnyResult<DeviceStateStream> {
        let transfer = InterruptTransfer::new(
            self.handle.clone(),
            self.in_endpoint_address,
            vec![0; 1],
            &self.fd_handler,
        )?;

        Ok(DeviceStateStream {
            transfer,
            fd_handler: self.fd_handler.clone(),
            in_endpoint_address: self.in_endpoint_address,
        })
    }

    ///
    /// # Errors
    pub async fn send_command(&self, command: DeviceCommand) -> AnyResult<()> {
        InterruptTransfer::new(
            self.handle.clone(),
            self.out_endpoint_address,
            vec![command.into(); 1],
            &self.fd_handler,
        )?
        .await?;

        Ok(())
    }

    #[expect(clippy::needless_pass_by_value, reason = "used in a `filter_map`")]
    fn device_filter(device: RusbDevice<Context>) -> Option<DeviceHandle<Context>> {
        let desc = device.device_descriptor().ok()?;

        if desc.vendor_id() != USB_VID || desc.product_id() != USB_PID {
            return None;
        }

        let handle = device.open().ok()?;
        let manufacturer = handle.read_manufacturer_string_ascii(&desc).ok()?;
        let product = handle.read_product_string_ascii(&desc).ok()?;

        if manufacturer != USB_MANUFACTURER || product != USB_PRODUCT {
            return None;
        }

        Some(handle)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if let Ok(false) = self.handle.kernel_driver_active(self.interface_number) {
            self.handle.attach_kernel_driver(self.interface_number).ok();
        }
    }
}

#[derive(Debug)]
pub struct DeviceStateStream {
    transfer: InterruptTransfer<Context>,
    fd_handler: Rc<FdHandler<Context, GlibFdHandlerContext>>,
    in_endpoint_address: u8,
}

impl Stream for DeviceStateStream {
    type Item = AnyResult<DeviceState>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let state = ready!(self.transfer.poll_unpin(cx))?
            .into_iter()
            .exactly_one()
            .map_err(|e| anyhow!("{e}"))?
            .try_into()?;

        let fd_handler = self.fd_handler.clone();
        let endpoint = self.in_endpoint_address;
        self.transfer.reuse(endpoint, vec![0; 1], &fd_handler)?;

        Poll::Ready(Some(Ok(state)))
    }
}
