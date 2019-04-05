//! Time conversions for the high frequency clock.

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
