//! Implementation of the embedded-hal `CountDown` trait.

use core::time::Duration;
use core::u32;

use void::Void;

use hal::timer::{CountDown, Periodic};
use nb::{Error, Result};
use nrf51::TIMER0;

use crate::hi_res_timer::{HiResTimer, Nrf51Timer, TimerCc, TimerWidth};

/// A TIMER peripheral as a `CountDown` provider.
///
/// `CountDownTimer` instances implement the embedded-hal `CountDown` trait.
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
/// `start()` panics if the requested time requires more 1mHz ticks than can
/// be represented using the timer's bit-width (32-bit for TIMER0, giving
/// approximately 71 minutes; 16-bit otherwise, giving approximately 65ms).
///
/// # Example
/// ```
/// use core::time::Duration;
/// use embedded_hal::timer::CountDown;
/// use nb::block;
/// use nrf51_hal::timer::CountdownTimer;
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut timer0 = CountDownTimer::new(p.TIMER0);
/// CountDown::start(&mut timer0, Duration::from_millis(1500));
/// block!(timer0.wait());
/// ```
pub struct CountDownTimer<T: Nrf51Timer> {
    timer: HiResTimer<T, T::MaxWidth>,
}

impl<T: Nrf51Timer> CountDownTimer<T> {
    /// Returns a new `CountDownTimer` wrapping the passed TIMER.
    ///
    /// Takes ownership of the TIMER peripheral.
    ///
    /// The TIMER is set to the greatest bit-width it supports, and the
    /// default 1MHz frequency.
    pub fn new(timer: T) -> CountDownTimer<T> {
        let mut hi_res_timer = timer.as_max_width_timer();
        hi_res_timer.enable_auto_clear(TimerCc::CC0);
        CountDownTimer { timer: hi_res_timer }
    }

    /// Gives the underlying `nrf51::TIMER`*n* instance back.
    pub fn free(self) -> T {
        self.timer.free()
    }
}

impl<T: Nrf51Timer> CountDown for CountDownTimer<T> {
    type Time = Duration;

    fn start<D>(&mut self, count: D)
    where
        D: Into<Self::Time>,
    {
        let duration = count.into();
        assert!(duration.as_secs() < u64::from((u32::MAX - duration.subsec_micros()) / 1_000_000));
        let us = (duration.as_secs() as u32) * 1_000_000 + duration.subsec_micros();
        // Default frequency is 1MHz, so we can use microseconds as ticks
        let ticks = T::MaxWidth::try_from_u32(us).expect("TIMER compare value too wide");
        // Stop the timer to make sure the event doesn't occur while we're
        // setting things up (if start() is called more than once).
        self.timer.stop();
        self.timer.clear_compare_event(TimerCc::CC0);
        self.timer.set_compare_register(TimerCc::CC0, ticks);
        self.timer.clear();
        self.timer.start();
    }

    fn wait(&mut self) -> Result<(), Void> {
        if self.timer.poll_compare_event(TimerCc::CC0) {
            Ok(())
        } else {
            Err(Error::WouldBlock)
        }
    }
}

impl<T: Nrf51Timer> Periodic for CountDownTimer<T> {}


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
pub type Timer = CountDownTimer<TIMER0>;
