use std::time::Duration;

use rusb::{Context, UsbContext};

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

    let handle = device.open().expect("unable to open device");
    handle.set_auto_detach_kernel_driver(true).unwrap();
    handle.claim_interface(0).unwrap();

    println!(
        "Active configuration: {}",
        handle.active_configuration().unwrap()
    );

    println!(
        "Kernel driver active: {}",
        handle.kernel_driver_active(0).unwrap()
    );

    println!(
        "Config descriptor: {:?}",
        device.active_config_descriptor().unwrap()
    );

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

    let mut buf = [0; 1];
    handle
        .read_interrupt(130, &mut buf, Duration::from_millis(10))
        .unwrap();

    println!("Data read: {buf:?}");
}
