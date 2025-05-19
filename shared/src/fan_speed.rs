use thiserror::Error as ThisError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum FanSpeed {
    Speed1 = 1,
    Speed2,
    Speed3,
    Speed4,
    Speed5,
    Speed6,
}

impl FanSpeed {
    pub fn increase(&mut self) {
        *self = Self::try_from(*self as u8 + 1).unwrap_or(Self::Speed6);
    }

    pub fn decrease(&mut self) {
        *self = Self::try_from(*self as u8 - 1).unwrap_or(Self::Speed1);
    }
}

impl From<FanSpeed> for u8 {
    fn from(value: FanSpeed) -> Self {
        value as Self
    }
}

impl TryFrom<u8> for FanSpeed {
    type Error = FanSpeedConvError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            n if FanSpeed::Speed1 as u8 == n => Ok(FanSpeed::Speed1),
            n if FanSpeed::Speed2 as u8 == n => Ok(FanSpeed::Speed2),
            n if FanSpeed::Speed3 as u8 == n => Ok(FanSpeed::Speed3),
            n if FanSpeed::Speed4 as u8 == n => Ok(FanSpeed::Speed4),
            n if FanSpeed::Speed5 as u8 == n => Ok(FanSpeed::Speed5),
            n if FanSpeed::Speed6 as u8 == n => Ok(FanSpeed::Speed6),
            _ => Err(FanSpeedConvError),
        }
    }
}

#[derive(Clone, Copy, Debug, ThisError)]
#[cfg_attr(test, derive(PartialEq))]
#[error("integer to fan speed conversion failed")]
pub struct FanSpeedConvError;

#[cfg(test)]
mod tests {
    use core::cmp;

    use strum::IntoEnumIterator;

    use crate::FanSpeed;

    const MAX_BITS: usize = 3;

    #[test]
    fn test_fan_speed_conversion() {
        for fan_speed in FanSpeed::iter() {
            assert_eq!(fan_speed as u8 >> MAX_BITS, 0);
            assert_eq!((fan_speed as u8).try_into(), Ok(fan_speed));
        }
    }

    #[test]
    fn test_fan_speed_increase() {
        for mut fan_speed in FanSpeed::iter() {
            let value = fan_speed as u8;
            fan_speed.increase();
            assert_eq!(
                cmp::min(value.saturating_add(1), FanSpeed::Speed6 as u8),
                fan_speed as u8
            );
        }
    }

    #[test]
    fn test_fan_speed_decrease() {
        for mut fan_speed in FanSpeed::iter() {
            let value = fan_speed as u8;
            fan_speed.decrease();
            assert_eq!(
                cmp::max(value.saturating_sub(1), FanSpeed::Speed1 as u8),
                fan_speed as u8
            );
        }
    }
}
