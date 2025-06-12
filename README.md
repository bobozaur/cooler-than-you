# cooler_than_you

Toy project for turning a laptop cooler into a "smart" cooler by incorporating an Arduino Pro Micro that presents itself as a USB HID device and communicating with it through USB. Naturally, the code here is highly specific to the laptop cooler used!

## Workspace layout
- [device](device/README.md): the Arduino Pro Micro embedded code to monitor & control the laptop cooler through USB
- [shared](shared/README.md): shared code between `device` and `tray`.
- [tray](tray/README.md): a `libindicator` based system tray driver that uses `rusb` to communicate with the device.

Please check out each crate's `README` for more details.

## Project goals

My biggest pet peeve with laptop coolers is that they are not communicating with the host in any way and that makes them inherently dumb. This bothers me the most when it comes to the host going into sleep while the cooler stays on, like when closing a laptop's lid. The initial motivation for this project was thus to accomplish one simple task: turn off the cooler when the host is suspended and turn it back on when the host resumes. I've actually done that before for my old cooler using a Digispark ATtiny85 board and [V-USB](https://github.com/obdev/v-usb) through SoF (Start of Frame) packet counting.

However, the new cooler I got and used in this project had all these additional buttons and features and so I figured I might go further and replicate the hardware controls into software and write some sort of driver to control the device from the host.

## Invoking `cargo` from workspace root

The `device` crate uses custom `cargo` configuration therefore building it must traditionally be done from the package's directory. Alternatively, invoking `cargo` from the workspace root can be done using unstable flags `cargo -Z unstable-options -C device build`. 

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
