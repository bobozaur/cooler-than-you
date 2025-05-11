use std::{sync::Arc, task::Waker};

use rusb::{DeviceHandle, UsbContext, constants::LIBUSB_CONTROL_SETUP_SIZE, ffi};

use crate::{
    Error, FdHandler, FdMonitor, Result,
    transfer::{FillTransfer, Transfer},
};

pub type ControlTransfer<C> = Transfer<C, Control>;
pub type RawControlTransfer<C> = Transfer<C, RawControl>;

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Control {
    request_type: u8,
    request: u8,
    value: u16,
    index: u16,
}

impl<C> ControlTransfer<C>
where
    C: UsbContext,
{
    /// # Errors
    pub fn new<M>(
        dev_handle: Arc<DeviceHandle<C>>,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
        _fd_handler: &FdHandler<C, M>,
    ) -> Result<Self>
    where
        M: FdMonitor<C>,
    {
        let buffer = Vec::with_capacity(data.len() + LIBUSB_CONTROL_SETUP_SIZE);
        let kind = Control {
            request_type,
            request,
            value,
            index,
        };

        Transfer::alloc(dev_handle, 0, buffer, kind, 0)
    }
}

impl<C> FillTransfer for ControlTransfer<C>
where
    C: UsbContext,
{
    fn fill(&mut self, waker: Waker) -> Result<()> {
        let length = self.buffer.capacity() - LIBUSB_CONTROL_SETUP_SIZE;
        let length = length
            .try_into()
            .map_err(|_| Error::Other("Invalid buffer size"))?;

        let user_data = Box::into_raw(Box::new(waker)).cast();

        unsafe {
            ffi::libusb_fill_control_setup(
                self.buffer.as_mut_ptr(),
                self.kind.request_type,
                self.kind.request,
                self.kind.value,
                self.kind.index,
                length,
            );

            ffi::libusb_fill_control_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.buffer.as_mut_ptr(),
                Self::transfer_cb,
                user_data,
                0,
            );
        }

        Ok(())
    }
}

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct RawControl(());

impl<C> RawControlTransfer<C>
where
    C: UsbContext,
{
    /// # Errors
    pub fn new<M>(
        dev_handle: Arc<DeviceHandle<C>>,
        buffer: Vec<u8>,
        _fd_handler: &FdHandler<C, M>,
    ) -> Result<Self>
    where
        M: FdMonitor<C>,
    {
        Transfer::alloc(dev_handle, 0, buffer, RawControl(()), 0)
    }
}

impl<C> FillTransfer for RawControlTransfer<C>
where
    C: UsbContext,
{
    fn fill(&mut self, waker: Waker) -> Result<()> {
        let user_data = Box::into_raw(Box::new(waker)).cast();

        unsafe {
            ffi::libusb_fill_control_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.buffer.as_mut_ptr(),
                Self::transfer_cb,
                user_data,
                0,
            );
        }

        Ok(())
    }
}
