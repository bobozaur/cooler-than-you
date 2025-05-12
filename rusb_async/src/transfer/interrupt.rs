use std::{convert::TryInto, sync::Arc, task::Waker};

use rusb::{
    DeviceHandle, UsbContext,
    constants::{LIBUSB_ENDPOINT_DIR_MASK, LIBUSB_ENDPOINT_OUT},
    ffi,
};

use crate::{
    error::{Error, Result},
    fd::{FdHandler, FdMonitor},
    transfer::{FillTransfer, SingleBufferTransfer, Transfer, TransferState},
};

pub type InterruptTransfer<C> = Transfer<C, Interrupt>;

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Interrupt(());

impl<C> InterruptTransfer<C>
where
    C: UsbContext,
{
    /// # Errors
    pub fn new<M>(
        dev_handle: Arc<DeviceHandle<C>>,
        endpoint: u8,
        buffer: Vec<u8>,
        _fd_handler: &FdHandler<C, M>,
    ) -> Result<Self>
    where
        M: FdMonitor<C>,
    {
        Transfer::alloc(dev_handle, endpoint, buffer, Interrupt(()), 0)
    }

    /// # Errors
    pub fn reuse<M>(
        &mut self,
        endpoint: u8,
        buffer: Vec<u8>,
        _fd_handler: &FdHandler<C, M>,
    ) -> Result<()>
    where
        M: FdMonitor<C>,
    {
        self.endpoint = endpoint;
        self.swap_buffer(buffer)?;
        self.state = TransferState::Allocated;
        Ok(())
    }
}

impl<C> FillTransfer for InterruptTransfer<C>
where
    C: UsbContext,
{
    fn fill(&mut self, waker: Waker) -> Result<()> {
        let length = if self.endpoint & LIBUSB_ENDPOINT_DIR_MASK == LIBUSB_ENDPOINT_OUT {
            // for OUT endpoints: the currently valid data in the buffer
            self.buffer.len()
        } else {
            // for IN endpoints: the full capacity
            self.buffer.capacity()
        };

        let length = length
            .try_into()
            .map_err(|_| Error::Other("Invalid buffer length"))?;

        let user_data = Box::into_raw(Box::new(waker)).cast();

        unsafe {
            ffi::libusb_fill_interrupt_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.endpoint,
                self.buffer.as_mut_ptr(),
                length,
                Self::transfer_cb,
                user_data,
                0,
            );
        }

        Ok(())
    }
}

impl SingleBufferTransfer for Interrupt {}
