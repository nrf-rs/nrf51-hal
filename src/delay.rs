//! Implementation of the embedded-hal `Delay` trait.

use cast::u32;
use nrf51::TIMER0;

use hal::blocking::delay::{DelayMs, DelayUs};

use crate::hi_res_timer::{HiResTimer, As32BitTimer, TimerCc};

/// System timer `TIMER0` as a delay provider.
///
/// This is a 32-bit timer running at 1MHz, giving a maximum setting of
/// approximately 71 minutes.
///
/// `Delay` instances implement the embedded-hal `DelayMs` and `DelayUs`
/// traits (for `u8`, `u16`, and `u32`).
///
/// # Panics
///
/// `delay_ms()` panics if the requested time exceeds the maximum setting.
pub struct Delay {
    timer: HiResTimer<TIMER0, u32>
}

impl Delay {
    /// Returns a new `Delay` wrapping TIMER0.
    ///
    /// Takes ownership of the TIMER0 peripheral.
    pub fn new(timer: TIMER0) -> Delay {
        let mut hi_res_timer = timer.as_32bit_timer();
        hi_res_timer.enable_auto_stop(TimerCc::CC0);
        Delay{timer: hi_res_timer}
    }

    /// Gives the underlying `nrf51::TIMER0` instance back.
    pub fn free(self) -> TIMER0 {
        self.timer.free()
    }

    fn delay(&mut self, us: u32) {
        self.timer.clear();
        // Default frequency is 1MHz, so we can use microseconds as ticks
        self.timer.set_compare_register(TimerCc::CC0, us);
        self.timer.start();
        while !self.timer.poll_compare_event(TimerCc::CC0) {}
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay(ms.checked_mul(1000).expect("ms delay out of range"));
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay(u32(ms) * 1000);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay(u32(ms) * 1000);
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        self.delay(us);
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay(u32(us));
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay(u32(us));
    }
}
