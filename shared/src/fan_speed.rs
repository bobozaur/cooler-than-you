#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(strum::EnumIter, PartialEq))]
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
        *self = (*self as u8 + 1).try_into().unwrap_or(FanSpeed::Speed6);
    }

    pub fn decrease(&mut self) {
        *self = (*self as u8 - 1).try_into().unwrap_or(FanSpeed::Speed1);
    }
}

impl From<FanSpeed> for u8 {
    fn from(value: FanSpeed) -> Self {
        value as Self
    }
}

impl TryFrom<u8> for FanSpeed {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FanSpeed::Speed1),
            1 => Ok(FanSpeed::Speed2),
            2 => Ok(FanSpeed::Speed3),
            3 => Ok(FanSpeed::Speed4),
            4 => Ok(FanSpeed::Speed5),
            5 => Ok(FanSpeed::Speed6),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::FanSpeed;

    #[test]
    fn test_fan_speed_conversion() {
        for fan_speed in FanSpeed::iter() {
            assert_eq!((fan_speed as u8).try_into(), Ok(fan_speed));
        }
    }

    #[test]
    fn test_max_fan_speed() {
        for mut fan_speed in [FanSpeed::Speed5, FanSpeed::Speed6] {
            fan_speed.increase();
            assert_eq!(fan_speed, FanSpeed::Speed6);
        }
    }

    #[test]
    fn test_min_fan_speed() {
        for mut fan_speed in [FanSpeed::Speed2, FanSpeed::Speed1] {
            fan_speed.decrease();
            assert_eq!(fan_speed, FanSpeed::Speed1);
        }
    }
}
