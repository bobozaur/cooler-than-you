use std::{sync::Arc, task::Waker};

use rusb::{
    DeviceHandle, UsbContext,
    constants::{LIBUSB_ENDPOINT_DIR_MASK, LIBUSB_ENDPOINT_OUT},
    ffi,
};

use crate::{
    Error, FdHandler, FdMonitor, Result,
    transfer::{FillTransfer, Transfer},
};

pub type IsochronousTransfer<C> = Transfer<C, Isochronous>;

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Isochronous {
    iso_packets: i32,
}

impl<C> IsochronousTransfer<C>
where
    C: UsbContext,
{
    pub fn new<M>(
        dev_handle: Arc<DeviceHandle<C>>,
        endpoint: u8,
        buffer: Vec<u8>,
        iso_packets: i32,
        _fd_handler: &FdHandler<C, M>,
    ) -> Result<Self>
    where
        M: FdMonitor<C>,
    {
        Self::alloc(
            dev_handle,
            endpoint,
            buffer,
            Isochronous { iso_packets },
            iso_packets,
        )
    }
}

impl<C> FillTransfer for IsochronousTransfer<C>
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

        let length: i32 = length
            .try_into()
            .map_err(|_| Error::Other("Invalid buffer length"))?;

        let packet_lengths = (length / self.kind.iso_packets)
            .try_into()
            .map_err(|_| Error::Other("Invalid iso packets length"))?;

        let user_data = Box::into_raw(Box::new(waker)).cast();

        unsafe {
            ffi::libusb_fill_iso_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.endpoint,
                self.buffer.as_mut_ptr(),
                length,
                self.kind.iso_packets,
                Self::transfer_cb,
                user_data,
                0,
            );

            ffi::libusb_set_iso_packet_lengths(self.ptr.as_ptr(), packet_lengths);
        }

        Ok(())
    }
}
