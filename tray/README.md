# tray

System tray icon for the `CoolerThanYou` device. SVG icon freely available at: https://www.svgrepo.com/svg/503337/fan-circled.

## Overview

The system tray acts as a software control panel in the form of a `libusb` device driver. The tray UI is built using `libappindicator` and `gtk-rs` and runs in a single thread. Async `rusb` calls are also hooked in the same `glib` event loop, allowing the entire app to run in a single thread. Apart from the emulated hardware buttons, the tray also provides automatic fan speed adjustmenting based on the CPU temperature.

## Build Instructions

1. Install `libgtk-3-dev` and `libayatana-appindicator3-1`.

2. Run `cargo run` to run the system tray and control the device.

3. Install `cargo-deb` through `cargo install cargo-deb`.

4. Run `cargo deb` to create an installable Debian package for the system tray.
