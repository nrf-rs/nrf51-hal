//! Implementations of the embedded-hal `CountDown` trait.

use core::time::Duration;

use void::Void;

use hal::timer::{CountDown, Periodic};
use nb::{Error, Result};
use nrf51::TIMER0;

use crate::hi_res_timer::{HiResTimer, Nrf51Timer, TimerCc, TimerFrequency, TimerWidth};
use crate::lo_res_timer::{LoResTimer, Nrf51Rtc, RtcCc, RtcFrequency};
use crate::time::{Hfticks, Lfticks};

/// A TIMER peripheral as a `CountDown` provider.
///
/// `CountDownTimer` instances implement the embedded-hal `CountDown` trait.
///
/// `start()` accepts either a `Duration` value or an `Hfticks` value
/// representing a number of nRF51 clock cycles.
///
/// The timer is periodic (it implements the `Periodic` trait).
///
/// Calling `start()` more than once is permitted (whether or not `wait()` has
/// already been called).
///
/// # Panics
///
/// `start()` panics if the requested time requires more ticks at the set
/// frequency than can be represented using the timer's bit-width (32-bit for
/// TIMER0, 16-bit otherwise). See `TimerFrequency` for a table of the
/// effective time limits.
///
/// # Example
/// ```
/// use core::time::Duration;
/// use embedded_hal::timer::CountDown;
/// use nb::block;
/// use nrf51_hal::hi_res_timer::TimerFrequency;
/// use nrf51_hal::timer::CountdownTimer;
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut timer0 = CountDownTimer::new(p.TIMER0, TimerFrequency::Freq1MHz);
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
    /// specified frequency.
    pub fn new(timer: T, frequency: TimerFrequency) -> CountDownTimer<T> {
        let mut hi_res_timer = timer.as_max_width_timer();
        hi_res_timer.set_frequency(frequency);
        hi_res_timer.enable_auto_clear(TimerCc::CC0);
        CountDownTimer { timer: hi_res_timer }
    }

    /// Gives the underlying `nrf51::TIMER`*n* instance back.
    pub fn free(self) -> T {
        self.timer.free()
    }
}

impl<T: Nrf51Timer> CountDown for CountDownTimer<T> {
    type Time = Hfticks;

    fn start<D>(&mut self, count: D)
    where
        D: Into<Self::Time>,
    {
        let hfticks = count.into();
        let ticks = self.timer.frequency().scale(hfticks.0)
            .expect("TIMER compare value overflow");
        let ticks = T::MaxWidth::try_from_u32(ticks).expect("TIMER compare value too wide");
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


/// An RTC peripheral as a `CountDown` provider.
///
/// `CountDownRTC` instances implement the embedded-hal `CountDown` trait.
///
/// `start()` accepts either a `Duration` value or a `Lfticks` value
/// representing a number of cycles of the nRF51's 32.768kHz LFCLK.
///
/// The timer is not periodic (doesn't implement the `Periodic` trait).
///
/// Calling `start()` more than once is permitted (whether or not `wait()` has
/// already been called).
///
/// # Panics
///
/// `start()` panics if the requested time requires 2^24 or more ticks at the
/// set frequency. See `RtcFrequency` for a table of the effective time
/// limits.
///
/// `wait()` panics if it's called twice without an intervening `start()`.
///
/// # Example
/// ```
/// use core::time::Duration;
/// use embedded_hal::timer::CountDown;
/// use nb::block;
/// use nrf51_hal::lo_res_timer::FREQ_1024HZ;
/// use nrf51_hal::timer::CountdownRtc;
/// let p = nrf51::Peripherals::take().unwrap();
/// p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
/// while p.CLOCK.events_lfclkstarted.read().bits() == 0 {}
/// p.CLOCK.events_lfclkstarted.reset();
/// let mut rtc0 = CountDownRtc::new(p.RTC0, FREQ_1024HZ);
/// CountDown::start(&mut rtc0, Duration::from_millis(1500));
/// block!(rtc0.wait());
/// ```
pub struct CountDownRtc<T: Nrf51Rtc> {
    timer: LoResTimer<T>,
    wait_allowed: bool,
}

impl<T: Nrf51Rtc> CountDownRtc<T> {
    /// Returns a new `CountDownRtc` wrapping the passed RTC.
    ///
    /// Takes ownership of the RTC peripheral.
    ///
    /// The RTC is set to the specified frequency.
    pub fn new(timer: T, frequency: RtcFrequency) -> CountDownRtc<T> {
        let mut lo_res_timer = LoResTimer::new(timer);
        lo_res_timer.set_frequency(frequency);
        lo_res_timer.enable_compare_event(RtcCc::CC0);
        CountDownRtc { timer: lo_res_timer, wait_allowed: false}
    }

    /// Gives the underlying `nrf51::RTC`*n* instance back.
    pub fn free(self) -> T {
        self.timer.free()
    }
}

impl<T: Nrf51Rtc> CountDown for CountDownRtc<T> {
    type Time = Lfticks;

    fn start<D>(&mut self, count: D)
    where
        D: Into<Self::Time>,
    {
        let lfticks = count.into();
        let ticks = self.timer.frequency().scale(lfticks.0)
            .expect("RTC compare value overflow");
        // Stop the timer to make sure the event doesn't occur while we're
        // setting things up.
        self.timer.stop();
        self.timer.clear_compare_event(RtcCc::CC0);
        self.timer.set_compare_register(RtcCc::CC0, ticks);
        self.timer.clear();
        self.timer.start();
        self.wait_allowed = true;
    }

    fn wait(&mut self) -> Result<(), Void> {
        assert!(self.wait_allowed, "called wait() twice on nonperiodic timer");
        if self.timer.poll_compare_event(RtcCc::CC0) {
            self.wait_allowed = false;
            self.timer.stop();
            Ok(())
        } else {
            Err(Error::WouldBlock)
        }
    }
}

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
/// `Timer` is provided in addition to `CountDownTimer` for backwards
/// compatibility, and as a simple way to get an implementation of `CountDown`
/// if you don't care about choosing which timer or frequency to use.
///
/// # Panics
///
/// `start()` panics if the requested time exceeds the maximum setting.
pub struct Timer(CountDownTimer<TIMER0>);

impl Timer {
    /// Returns a new `Timer` wrapping TIMER0.
    ///
    /// Takes ownership of the TIMER0 peripheral.
    pub fn new(timer: TIMER0) -> Timer {
        Timer(CountDownTimer::new(timer, TimerFrequency::Freq1MHz))
    }

    /// Gives the underlying `nrf51::TIMER0` instance back.
    pub fn free(self) -> TIMER0 {
        self.0.free()
    }
}

impl CountDown for Timer {
    type Time = Duration;

    fn start<D>(&mut self, count: D)
    where
        D: Into<Self::Time>,
    {
        self.0.start(count.into());
    }

    fn wait(&mut self) -> Result<(), Void> {
        self.0.wait()
    }
}

impl Periodic for Timer {}
