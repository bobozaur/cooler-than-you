# device

CoolerThanYou device code, developed for an Arduino Pro Micro with an ATmega32u4 running at 5V.
The microcontroller performs the following tasks:

- sets a known initial device state on startup
- emulates button presses in software (through transistors soldered in parallel to the push buttons)
- every 1ms monitors the buttons and updates the device state if needed
- when changed, sends the device state to the host through USB
- receives commands to execute through USB
- turns off/on the cooler on host suspend/resume

## Hardware description

After deciding to go futher from simple suspend/resume behavior, the Pro Micro was chosen because it has hardware USB and the cooler circuit is using 5V and this was the only 5V microcontroller I had laying around.

### Arduino Pro Micro

<img src="https://cdn.sparkfun.com/r/600-600/assets/9/c/3/c/4/523a1765757b7f5c6e8b4567.png" style="max-width: 100%;" width="400" />

### Laptop cooler

<img src="https://raw.githubusercontent.com/bobozaur/cooler-than-you/main/device/images/cooler.jpeg" style="max-width: 100%;" width="400" />

The control panel with its four buttons (left to right): speed up, speed down, LED, power. Note that the screen backlight is _ON_ in the picture.

<img src="https://raw.githubusercontent.com/bobozaur/cooler-than-you/main/device/images/cooler_buttons.jpeg" style="max-width: 100%;" width="400" />

Observed cooler behavior:

- The `Speed up` button increases fan speed (max speed 6) when pressed down at least 40ms
- The `Speed down` button decreases fan speed (min speed 1) when pressed down at least 40ms
- The `LED` button changes the LED strip color when pressed at least 40ms and then released
- The `LED` button turns on/off the LED strip when pressed at least 1400ms
- The `Power` button turns on/off the power when pressed at least 40ms and then released
- The `Power` button triggers a no-op when pressed at least 1400ms
- Screen backlight times out after around 13000ms
- `Speed up` and `Speed down` buttons will not take effect if the backlight is off and will turn it on instead
- Once an action is registered, all buttons are disabled until all of them are released
- The buttons share their state, in the sense that it's not necessary for the _same_ button to be pressed for 40ms
- If multiple buttons are pressed after 40ms, the following priority is enforced: `Speed Up > Speed Down > Power > LED`
- During a long press, after 40ms of being pressed, the _same_ button must remain pressed

Hardware components used:

- TIMER0: used for triggerring interrupts every 1ms to execute the monitoring code
- Pin 5 as input: used for backlight monitoring
- Pin 6, 7, 8, 9 as input: used for push button monitoring
- Pins 10, 16, 14, 15 as output: connected to the base of BC547 transistors with 5.7k Ohms resistors in between; used to emulate button presses
- USB & PLL: used for the USB interface
- WDT: used to enter bootloader mode by repurposing the long press on the power button

The back of the cooler PCB is where the hardware connections were soldered.

<img src="https://raw.githubusercontent.com/bobozaur/cooler-than-you/main/device/images/board_back.jpeg" style="max-width: 100%;" width="400" />

Cooler PCB front:

<img src="https://raw.githubusercontent.com/bobozaur/cooler-than-you/main/device/images/board_front.jpeg" style="max-width: 100%;" width="400" />

The Arduino Pro Micro is connected through a cut micro USB cable directly to the pins of the USB-A port that the cooler itself is powered from. The USB-A port lives on a small daughter board. This makes it operational whenever the cooler would be. Unfortunately forgot to take pictures before putting it all together, but it's just four wires soldered to the VCC, D-, D+ and GND pins of the through-hole USB port on the daughter board.

## Build Instructions

1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).

2. Run `cargo run` to flash the firmware to a connected board. `ravedude` looks by default at the `/dev/ttyACM0` device.
   If `ravedude` fails to detect your board, check its documentation at <https://crates.io/crates/ravedude>.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

## Alternative approaches

Please note that I'm not saying that the employed approach is the recommended way to go. As primarily a software engineer it was easier for me to deal with more software and less hardware and this came with the opportunity to reverse-engineer how the cooler works and the challenge to monitor and replicate that. I treated this as a learning experience :).

There were multiple ways to go about this project, but my design decisions were mostly driven by one thing: I wanted to be as least invasive as possible with the hardware even if it comes at the cost of software complexity.

Things would've been much simpler to tackle software-wise if I would've cut the push button traces and re-routed them completely. That way, any button press would go through the Pro Micro. But mistakes there would've been much harder to correct so I decided against it.

### Button press emulation

I have considered using output pins to sink current from the cooler's MCU pins directly instead of soldering transistors in parallel to the push buttons, but I was initially worried that connecting a pin from the cooler's MCU to the Arduino Pro Micro migth cause issues and decided to play it safe. I've done more research since then and I think it would be perfectly fine, actually. The cooler's MCU has some pull-up input pins that should not source much current. The output pins on the Arduino Pro Micro should be able to sink that without a problem. I'll probably revisit this at some point and get rid of the transistors.

### Button press monitoring

In retrospect, a better idea than monitoring buttons every 1ms would've been connecting the Pro Micro to one of the unused connectors of the fan grid and reading the voltage using an analog input pin. There's also a second unused connector for the LED strip which could be used to check if the lights are ON/OFF. This would in fact be more precise too because there's no guessing. If the voltage changes, something **definitely** happened. The current button monitoring approach, while seemingly reliable, cannot guarantee that the cooler's state and the state that the Arduino Pro Micro tracks are really in sync.

However, by the time this idea struck me, I had already put the cooler back together and implemented most of the code so I decided to roll with it. Moreover, some functionality would be lost, such as a long press on the power button triggerring a reset to enter bootloader mode or detecting a LED strip color change through a short press on the LED button. Nevertheless, I might revisit this part of the project and do it this way at a later time. For now, the button monitoring works wonders.
