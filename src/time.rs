//! Time definitions

use cast::{u32, Error};
use core::time::Duration;

pub const HFCLK_FREQ: u32 = 16_000_000;
pub const LFCLK_FREQ: u32 = 32_768;

/// High frequency tick
/// one sixteenth of a microsecond, the time for one nrf51 clock cycle
/// Max: 2^32 / (16MHz / 2^9) = 2^37 seconds = ~38 hours = 2^41 hfticks
#[derive(Debug, Clone, Copy)]
pub struct Hfticks(pub u64);

/// Low frequency tick
/// 32.768 kHz clock cycle
/// Max: 2^16 / (32.768kHz / 2^12) = 8192seconds = ~137 minutes = 2^28 lfticks
#[derive(Debug, Clone, Copy)]
pub struct Lfticks(pub u32);

/// Microsecond
/// 
#[derive(Debug, Clone, Copy)]
pub struct Micros(pub u64);

/// Milliseconds
#[derive(Debug, Clone, Copy)]
pub struct Millis(pub u32);

/// Hertz
/// Maximum is ~2^24
#[derive(Debug, Clone, Copy)]
pub struct Hertz(pub u32);


impl Hfticks {
    /// Checked multiplication
    pub fn checked_mul(self, rhs: Hertz) -> Result<u32, cast::Error> {
        // Size check
        // lhs        * rhs        / 16_000_000 <= u32::max()
        // (64-lhs0s) + (32-rhs0s) - 24         <= 32
        if (64 - self.0.leading_zeros()) + (32 - rhs.0.leading_zeros()) - 24 >= 32 {
            Err(Error::Overflow)
        } else {
            let p = self.0.checked_mul(u64::from(rhs.0)).ok_or(Error::Overflow)?;
            u32(p / u64::from(HFCLK_FREQ))
        }
    }
}

impl Micros {
    /// Checked multiplication
    pub fn checked_mul(self, rhs: Hertz) -> Result<u32, cast::Error> {
        let p = self.0.checked_mul(u64::from(rhs.0)).ok_or(Error::Overflow)?;
        u32(p / 1_000_000)
    }
}

impl From<Duration> for Hfticks {
    fn from(duration: Duration) -> Self {
        Hfticks(
            u64::from(duration.subsec_nanos()) * 16 / 1000 +
            duration.as_secs().checked_mul(16_000_000).unwrap()
        )
    }
}

impl From<Micros> for Hfticks {
    fn from(micros: Micros) -> Self {
        Hfticks(micros.0 * 16)
    }
}

impl From<Millis> for Hfticks {
    fn from(millis: Millis) -> Self {
        Hfticks(u64::from(millis.0) * 16_000)
    }
}

impl Lfticks {
    pub fn checked_mul(self, rhs: Hertz) -> Option<u32> {
        match self.0.checked_mul(rhs.0) {
            Some(val) => Some(val / LFCLK_FREQ),
            None => (self.0 / LFCLK_FREQ).checked_mul(rhs.0),
        }
    }
}

impl From<Duration> for Lfticks {
    fn from(duration: Duration) -> Self {
        let subsec_part = u64::from(duration.subsec_nanos()).checked_mul(32768).unwrap()  / 1_000_000_000;
        let sec_part = duration.as_secs().checked_mul(32768).unwrap();
        Lfticks(u32(subsec_part + sec_part).unwrap())
    }
}

impl From<Millis> for Lfticks {
    fn from(millis: Millis) -> Self {
        Lfticks(millis.0 * LFCLK_FREQ / 1000)
    }
}

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `ms`
    fn ms(self) -> Millis;
    /// Wrap in `hz`
    fn hz(self) -> Hertz;
}

impl U32Ext for u32 {
    fn ms(self) -> Millis {
        Millis(self)
    }
    fn hz(self) -> Hertz {
        Hertz(self)
    }
}

/// Extension trait that adds convenience methods to the `u64` type
pub trait U64Ext {
    /// Wrap in `us`
    fn us(self) -> Micros;
}

impl U64Ext for u64 {
    fn us(self) -> Micros {
        Micros(self)
    }
}
