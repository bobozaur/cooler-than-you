#[repr(u8)]
#[derive(Copy, Clone)]
pub enum FanSpeed {
    Speed1,
    Speed2,
    Speed3,
    Speed4,
    Speed5,
    Speed6,
}

impl FanSpeed {
    pub fn increase(&mut self) {
        *self = match self {
            Self::Speed1 => Self::Speed2,
            Self::Speed2 => Self::Speed3,
            Self::Speed3 => Self::Speed4,
            Self::Speed4 => Self::Speed5,
            Self::Speed5 | FanSpeed::Speed6 => Self::Speed6,
        };
    }

    pub fn decrease(&mut self) {
        *self = match self {
            Self::Speed1 | Self::Speed2 => Self::Speed1,
            Self::Speed3 => Self::Speed2,
            Self::Speed4 => Self::Speed3,
            Self::Speed5 => Self::Speed4,
            Self::Speed6 => Self::Speed5,
        };
    }
}
