use arduino_hal::{hal::port::{PB1, PB2, PB3, PB6}, port::PinOps};

pub type SpeedUpButtonPin = PB3;
pub type SpeedDownButtonPin = PB1;
pub type PowerButtonPin = PB6;
pub type LedButtonPin = PB2;

pub trait ShortPressPin: PinOps {}

pub trait LongPressPin: ShortPressPin {}

impl ShortPressPin for SpeedUpButtonPin {}
impl ShortPressPin for SpeedDownButtonPin {}
impl ShortPressPin for PowerButtonPin {}
impl ShortPressPin for LedButtonPin {}

impl LongPressPin for LedButtonPin {}