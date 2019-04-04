//! Implementation of the embedded-hal `CountDown` trait.

use core::time::Duration;
use core::u32;

use void::Void;

use hal::timer::{CountDown, Periodic};
use nb::{Error, Result};
use nrf51::TIMER0;

use crate::hi_res_timer::{HiResTimer, As32BitTimer, TimerCc};

/// System timer `TIMER0` as a `CountDown` provider.
///
/// This is a 32-bit timer running at 1MHz, giving a maximum setting of
/// approximately 71 minutes.
///
/// `Timer` instances implement the embedded-hal `CountDown` trait.
///
/// `start()` accepts a `Duration` value.
///
/// The timer is periodic (it implements the `Periodic` trait).
///
/// Calling `start()` more than once is permitted (whether or not `wait()` has
/// already been called).
///
/// # Panics
///
/// `start()` panics if the requested time exceeds the maximum setting.
pub struct Timer(HiResTimer<TIMER0, u32>);

impl Timer {
    /// Returns a new `Timer` wrapping TIMER0.
    ///
    /// Takes ownership of the TIMER0 peripheral.
    pub fn new(timer: TIMER0) -> Timer {
        let mut hi_res_timer = timer.as_32bit_timer();
        hi_res_timer.enable_auto_clear(TimerCc::CC0);
        Timer(hi_res_timer)
    }

    /// Gives the underlying `nrf51::TIMER0` instance back.
    pub fn free(self) -> TIMER0 {
        self.0.free()
    }
}

impl CountDown for Timer {
    type Time = Duration;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        let duration = count.into();
        assert!(duration.as_secs() < u64::from((u32::MAX - duration.subsec_micros()) / 1_000_000));
        let us = (duration.as_secs() as u32) * 1_000_000 + duration.subsec_micros();
        // Stop the timer to make sure the event doesn't occur while we're
        // setting things up (if start() is called more than once).
        self.0.stop();
        self.0.clear_compare_event(TimerCc::CC0);
        // Default frequency is 1MHz, so we can use microseconds as ticks
        self.0.set_compare_register(TimerCc::CC0, us);
        self.0.clear();
        self.0.start();
    }

    fn wait(&mut self) -> Result<(), Void> {
        if self.0.poll_compare_event(TimerCc::CC0) {
            Ok(())
        } else {
            Err(Error::WouldBlock)
        }
    }
}

impl Periodic for Timer {}
