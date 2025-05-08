mod pins;

use arduino_hal::{
    delay_ms,
    port::{Pin, PinOps, mode::Output},
};
use pins::{
    LedButtonPin, LongPressPin, PowerButtonPin, ShortPressPin, SpeedDownButtonPin, SpeedUpButtonPin,
};

pub type SpeedUpButton = Button<SpeedUpButtonPin>;
pub type SpeedDownButton = Button<SpeedDownButtonPin>;
pub type PowerButton = Button<PowerButtonPin>;
pub type LedButton = Button<LedButtonPin>;

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
    PIN: ShortPressPin,
{
    #[inline]
    pub fn short_press(&mut self) {
        self.0.set_high();
        delay_ms(PIN::SHORT_PRESS_MS);
        self.0.set_low();
        delay_ms(PIN::POST_PRESS_DELAY);
    }
}

impl<PIN> Button<PIN>
where
    PIN: LongPressPin,
{
    #[inline]
    pub fn long_press(&mut self) {
        self.0.set_high();
        delay_ms(PIN::LONG_PRESS_MS);
        self.0.set_low();
        delay_ms(PIN::POST_PRESS_DELAY);
    }
}
