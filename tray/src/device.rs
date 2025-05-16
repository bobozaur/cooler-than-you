use std::{
    sync::Arc,
    task::{Poll, ready},
};

use anyhow::Context as _;
use futures_core::Stream;
use futures_util::FutureExt;
use rusb::{
    Device as RusbDevice, DeviceHandle, Direction, LogCallbackMode, LogLevel, TransferType,
    UsbContext,
};
use rusb_async::{AsyncContext, FdCallbacksEventHandler, InterruptTransfer};
use shared::{DeviceCommand, DeviceState, USB_MANUFACTURER, USB_PID, USB_PRODUCT, USB_VID};
use tracing::instrument;

use crate::{AnyResult, exactly_one::ExactlyOneIter, fd_callbacks::GlibFdCallbacks};

#[derive(Clone, Debug)]
pub struct Device(Arc<DeviceInner>);

impl Device {
    /// Creates a device instance which can be used for reading and writing.
    ///
    /// # Errors
    ///
    /// Returns an error if a matching device is not found, could not be opened or the interface
    /// could not be set up and claimed.
    #[instrument(err(Debug), ret)]
    pub fn new() -> AnyResult<Self> {
        let event_handler = FdCallbacksEventHandler::new(GlibFdCallbacks::default());
        let mut context =
            AsyncContext::new(event_handler).context("could not initialize libusb")?;

        context.set_log_level(LogLevel::Warning);
        context.set_log_callback(Box::new(Self::log_message), LogCallbackMode::Global);

        let handle = context
            .devices()?
            .iter()
            .filter_map(Self::device_filter)
            .exactly_one()
            .map(Arc::new)
            .context("could not find a matching device or open it")?;

        let device = handle.device();
        let device_desc = device
            .device_descriptor()
            .context("failed to read the device descriptor")?;

        let config_desc = (0..device_desc.num_configurations())
            .filter_map(|i| device.config_descriptor(i).ok())
            .exactly_one()
            .context("failed to read the config descriptor")?;

        let config_number = config_desc.number();

        let interface_desc = config_desc
            .interfaces()
            .flat_map(|i| i.descriptors())
            .exactly_one()
            .context("failed to read the interface descriptor")?;

        let interface_number = interface_desc.interface_number();
        let setting_number = interface_desc.setting_number();

        let in_endpoint_address = interface_desc
            .endpoint_descriptors()
            .filter(|edesc| edesc.direction() == Direction::In)
            .filter(|edesc| edesc.transfer_type() == TransferType::Interrupt)
            .exactly_one()
            .context("failed to read the IN interrupt endpoint descriptor")
            .map(|e| e.address())?;

        let out_endpoint_address = interface_desc
            .endpoint_descriptors()
            .filter(|edesc| edesc.direction() == Direction::Out)
            .filter(|edesc| edesc.transfer_type() == TransferType::Interrupt)
            .exactly_one()
            .context("failed to read the OUT interrupt endpoint descriptor")
            .map(|e| e.address())?;

        if handle.kernel_driver_active(interface_number)? {
            tracing::info!("detaching kernel driver");
            handle
                .detach_kernel_driver(interface_number)
                .context("failed to detach kernel driver")?;
        }

        handle
            .set_active_configuration(config_number)
            .context("failed to set config number")?;

        handle
            .claim_interface(interface_number)
            .context("failed to claim interface")?;

        handle
            .set_alternate_setting(interface_number, setting_number)
            .context("failed to choose alternate setting")?;

        let inner = DeviceInner {
            handle,
            interface_number,
            in_endpoint_address,
            out_endpoint_address,
        };

        Ok(Self(Arc::new(inner)))
    }

    /// Creates a [`DeviceStateStream`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`InterruptTransfer::new`] fails.
    #[instrument(skip(self), err(Debug))]
    pub fn state_stream(&self) -> AnyResult<DeviceStateStream> {
        let transfer = InterruptTransfer::new(
            self.0.handle.clone(),
            self.0.in_endpoint_address,
            vec![0; 1],
        )?;

        Ok(DeviceStateStream {
            transfer,
            in_endpoint_address: self.0.in_endpoint_address,
        })
    }

    /// Sends a command to the device.
    ///
    /// # Errors
    ///
    /// Returns an error if [`InterruptTransfer::new`] fails
    /// or if the transfer could not be completed.
    #[instrument(skip(self), err(Debug))]
    pub async fn send_command(&self, command: DeviceCommand) -> AnyResult<()> {
        InterruptTransfer::new(
            self.0.handle.clone(),
            self.0.out_endpoint_address,
            vec![command.into(); 1],
        )?
        .await?;

        Ok(())
    }

    #[expect(clippy::needless_pass_by_value, reason = "used in a `filter_map`")]
    fn device_filter(device: RusbDevice<AsyncContext>) -> Option<DeviceHandle<AsyncContext>> {
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

    #[expect(clippy::needless_pass_by_value, reason = "for context log callback")]
    fn log_message(level: LogLevel, message: String) {
        match level {
            LogLevel::None => (),
            LogLevel::Error => tracing::error!(message),
            LogLevel::Warning => tracing::warn!(message),
            LogLevel::Info => tracing::info!(message),
            LogLevel::Debug => tracing::debug!(message),
        }
    }
}

/// A never ending [`Stream`] that reads and returns the [`DeviceState`].
#[derive(Debug)]
pub struct DeviceStateStream {
    transfer: InterruptTransfer<AsyncContext>,
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
            .exactly_one()?
            .try_into()?;

        let endpoint = self.in_endpoint_address;
        self.transfer.renew(endpoint, vec![0; 1])?;

        Poll::Ready(Some(Ok(state)))
    }
}

#[derive(Clone, Debug)]
struct DeviceInner {
    /// Using an [`Arc`] because that's what the async libusb transfers require.
    handle: Arc<DeviceHandle<AsyncContext>>,
    interface_number: u8,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
}

impl Drop for DeviceInner {
    fn drop(&mut self) {
        if let Ok(false) = self.handle.kernel_driver_active(self.interface_number) {
            if let Err(e) = self.handle.attach_kernel_driver(self.interface_number) {
                tracing::error!("error re-attaching kernel driver: {e}");
            }
        }
    }
}
