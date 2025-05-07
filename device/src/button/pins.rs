use arduino_hal::{
    hal::port::{PB1, PB2, PB3, PB6},
    port::PinOps,
};

pub type SpeedUpButtonPin = PB3;
pub type SpeedDownButtonPin = PB1;
pub type PowerButtonPin = PB6;
pub type LedButtonPin = PB2;

pub trait ShortPressPin: PinOps {
    const POST_PRESS_DELAY: u16 = 5;
    const SHORT_PRESS_MS: u16 = 45;
}

pub trait LongPressPin: ShortPressPin {
    const LONG_PRESS_MS: u16 = 1425;
}

impl ShortPressPin for SpeedUpButtonPin {}
impl ShortPressPin for SpeedDownButtonPin {}
impl ShortPressPin for PowerButtonPin {}
impl ShortPressPin for LedButtonPin {}

impl LongPressPin for LedButtonPin {}
