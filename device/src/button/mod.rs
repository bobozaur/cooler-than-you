mod pins;

use arduino_hal::{
    delay_ms,
    port::{Pin, PinOps, mode::Output},
};
use pins::{LedButtonPin, PowerButtonPin, SpeedDownButtonPin, SpeedUpButtonPin};

pub type SpeedUpButton = Button<SpeedUpButtonPin>;
pub type SpeedDownButton = Button<SpeedDownButtonPin>;
pub type PowerButton = Button<PowerButtonPin>;
pub type LedButton = Button<LedButtonPin>;

/// Generic button struct that emulates button presses.
pub struct Button<PIN>(Pin<Output, PIN>);

impl<PIN> Button<PIN>
where
    PIN: PinOps,
{
    #[inline]
    pub fn new(pin: Pin<Output, PIN>) -> Self {
        Self(pin)
    }
}

impl<PIN> Button<PIN>
where
    PIN: PinOps,
{
    pub const POST_PRESS_DELAY: u16 = 10;
    pub const SHORT_PRESS_MS: u16 = 45;

    /// Performs a short press on the button.
    /// A short press has the duration of 40ms, but [`Self::SHORT_PRESS_MS`] is used to ensure the
    /// button press gets registered.
    ///
    /// A delay of [`Self::POST_PRESS_DELAY`] is used after the button press as a boundary between
    /// subsequent presses.
    #[inline]
    pub fn short_press(&mut self) {
        self.0.set_high();
        delay_ms(Self::SHORT_PRESS_MS);
        self.0.set_low();
        delay_ms(Self::POST_PRESS_DELAY);
    }
}

impl Button<LedButtonPin> {
    pub const LONG_PRESS_MS: u16 = 1425;

    /// Performs a long press on the button.
    /// A long press has the duration of 1400ms, but [`Self::LONG_PRESS_MS`] is used to ensure the
    /// button press gets registered.
    ///
    /// A delay of [`Self::POST_PRESS_DELAY`] is used after the button press as a boundary between
    /// subsequent presses.
    #[inline]
    pub fn long_press(&mut self) {
        self.0.set_high();
        delay_ms(Self::LONG_PRESS_MS);
        self.0.set_low();
        delay_ms(Self::POST_PRESS_DELAY);
    }
}
