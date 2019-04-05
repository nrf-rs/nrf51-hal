//! Time conversions for the high frequency clock.

use core::time::Duration;

use cast::u32;

use crate::hi_res_timer::HFCLK_MHZ;


/// A number of ticks of the nRF51 high-frequency clock (HFCLK).
///
/// The clock frequency is 16MHz, so each tick is 62.5ns.
///
/// All TIMER frequencies are a multiple of this base frequency.
///
/// Holds a 64-bit number.
#[derive(Debug, Clone, Copy)]
pub struct Hfticks(pub u64);

impl Hfticks {
    /// Converts a time in milliseconds to an exact number of ticks of the
    /// high-frequency clock.
    pub fn from_ms(ms: u32) -> Hfticks {
        Hfticks(ms as u64 * HFCLK_MHZ as u64 * 1000)
    }

    /// Converts a time in microseconds to an exact number of ticks of the
    /// high-frequency clock.
    pub fn from_us(us: u32) -> Hfticks {
        Hfticks(us as u64 * HFCLK_MHZ as u64)
    }
}

/// Converts a core::time::Duration to a number of ticks of the high-frequency
/// clock.
///
/// Rounds down.
///
/// # Panics
///
/// Panics if the duration is longer than 2^32-1 seconds
impl From<Duration> for Hfticks {
    fn from(duration: Duration) -> Self {
        let secs = u32(duration.as_secs()).expect("duration too long");
        Hfticks(
            duration.subsec_nanos() as u64 * HFCLK_MHZ as u64 / 1000 +
            secs as u64 * HFCLK_MHZ as u64 * 1_000_000
        )
    }
}
