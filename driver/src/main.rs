use std::{thread, time::Duration};

use rusb::{Context, Error, UsbContext};
use shared::{Command, DeviceState};

fn main() {
    let device = Context::new()
        .expect("unable to initialize libusb")
        .devices()
        .expect("unable to enumerate devices")
        .iter()
        .find_map(|device| {
            let desc = device.device_descriptor().ok()?;
            (desc.vendor_id() == 0xd016 && desc.product_id() == 0xdb08).then_some(device)
        })
        .expect("no matching device found");

    for interface in device.active_config_descriptor().unwrap().interfaces() {
        for idesc in interface.descriptors() {
            println!("Interface descriptor: {idesc:?}");

            for edesc in idesc.endpoint_descriptors() {
                println!(
                    "Endpoint descriptor: {}:{:?}:{:?} - {edesc:?}",
                    edesc.number(),
                    edesc.direction(),
                    edesc.transfer_type()
                );
            }
        }
    }

    let handle = device.open().expect("unable to open device");
    handle.set_auto_detach_kernel_driver(true).unwrap();
    handle.claim_interface(0).unwrap();
    handle.set_alternate_setting(0, 0).unwrap();

    let mut buf = [0; 1];

    loop {
        match handle.read_interrupt(130, &mut buf, Duration::from_millis(500)) {
            Ok(_) => println!("State: {:?}", DeviceState::try_from(buf[0]).unwrap()),
            Err(Error::Timeout) => continue,
            Err(e) => panic!("{e}"),
        }

        match handle.write_interrupt(
            1,
            &[Command::LedsColorChange.into()],
            Duration::from_millis(500),
        ) {
            Ok(n) => println!("Wrote {n} bytes"),
            Err(e) => panic!("{e}"),
        }

        thread::sleep(Duration::from_secs(2));
    }

    // handle.release_interface(0).unwrap();
}
