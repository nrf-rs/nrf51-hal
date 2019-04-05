//! Implementations of the embedded-hal `Delay` trait.

use nrf51::TIMER0;

use hal::blocking::delay::{DelayMs, DelayUs};

use crate::hi_res_timer::{HiResTimer, Nrf51Timer, TimerCc, TimerFrequency, TimerWidth};
use crate::lo_res_timer::{LoResTimer, Nrf51Rtc, RtcCc, RtcFrequency};
use crate::time::{Hfticks, Lfticks};

/// A TIMER peripheral as a delay provider.
///
/// `DelayTimer` instances implement the embedded-hal `DelayMs` and `DelayUs`
/// traits (for `u8`, `u16`, and `u32`).
///
/// # Panics
///
/// `delay_ms()` and `delay_us()` panic if the requested time requires more
/// ticks at the set frequency than can be represented using the timer's
/// bit-width (32-bit for TIMER0, 16-bit otherwise). See `TimerFrequency` for
/// a table of the effective time limits.
///
/// # Example
/// ```
/// use embedded_hal::delay::DelayMs;
/// use nrf51_hal::hi_res_timer::TimerFrequency;
/// use nrf51_hal::delay::DelayTimer;
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut timer0 = DelayTimer::new(p.TIMER0, TimerFrequency::Freq1MHz);
/// timer0.delay_ms(1000);
/// ```
pub struct DelayTimer<T: Nrf51Timer> {
    timer: HiResTimer<T, T::MaxWidth>,
}

impl<T: Nrf51Timer> DelayTimer<T> {
    /// Returns a new `DelayTimer` wrapping the passed TIMER.
    ///
    /// Takes ownership of the TIMER peripheral.
    ///
    /// The TIMER is set to the greatest bit-width it supports, and the
    /// specified frequency.
    pub fn new(timer: T, frequency: TimerFrequency) -> Self {
        let mut hi_res_timer = timer.as_max_width_timer();
        hi_res_timer.set_frequency(frequency);
        hi_res_timer.enable_auto_stop(TimerCc::CC0);
        DelayTimer { timer: hi_res_timer }
    }

    /// Gives the underlying `nrf51::TIMER`*n* instance back.
    pub fn free(self) -> T {
        self.timer.free()
    }

    fn delay(&mut self, hfticks: Hfticks) {
        let ticks = self.timer.frequency().scale(hfticks.0)
            .expect("TIMER compare value overflow");
        let ticks = T::MaxWidth::try_from_u32(ticks).expect("TIMER compare value too wide");
        self.timer.clear();
        self.timer.set_compare_register(TimerCc::CC0, ticks);
        self.timer.start();
        while !self.timer.poll_compare_event(TimerCc::CC0) {}
    }
}

impl<T: Nrf51Timer> DelayMs<u32> for DelayTimer<T> {
    fn delay_ms(&mut self, ms: u32) {
        self.delay(Hfticks::from_ms(ms));
    }
}

impl<T: Nrf51Timer> DelayMs<u16> for DelayTimer<T> {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(u32::from(ms));
    }
}

impl<T: Nrf51Timer> DelayMs<u8> for DelayTimer<T> {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u32::from(ms));
    }
}

impl<T: Nrf51Timer> DelayUs<u32> for DelayTimer<T> {
    fn delay_us(&mut self, us: u32) {
        self.delay(Hfticks::from_us(us));
    }
}

impl<T: Nrf51Timer> DelayUs<u16> for DelayTimer<T> {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(u32::from(us))
    }
}

impl<T: Nrf51Timer> DelayUs<u8> for DelayTimer<T> {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(u32::from(us))
    }
}

/// An RTC peripheral as a delay provider.
///
/// `DelayRTC` instances implement the embedded-hal `DelayMs` and `DelayUs`
/// traits (for `u8`, `u16`, and `u32`).
///
/// # Panics
///
/// `delay_ms()` and `delay_us()` panic if the requested time requires 2^24 or
/// more ticks at the set frequency. See `RtcFrequency` for a table of the
/// effective time limits.
///
/// # Example
/// ```
/// use embedded_hal::delay::DelayMs;
/// use nrf51_hal::lo_res_timer::FREQ_1024HZ;
/// use nrf51_hal::delay::DelayRtc;
/// let p = nrf51::Peripherals::take().unwrap();
/// p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
/// while p.CLOCK.events_lfclkstarted.read().bits() == 0 {}
/// p.CLOCK.events_lfclkstarted.reset();
/// let mut rtc0 = DelayRtc::new(p.RTC0, FREQ_1024HZ);
/// rtc0.delay_ms(1000);
/// ```
pub struct DelayRtc<T: Nrf51Rtc> {
    timer: LoResTimer<T>,
}

impl<T: Nrf51Rtc> DelayRtc<T> {
    /// Returns a new `DelayRtc` wrapping the passed RTC.
    ///
    /// Takes ownership of the RTC peripheral.
    ///
    /// The RTC is set to the specified frequency.
    pub fn new(timer: T, frequency: RtcFrequency) -> Self {
        let mut lo_res_timer = LoResTimer::new(timer);
        lo_res_timer.set_frequency(frequency);
        lo_res_timer.enable_compare_event(RtcCc::CC0);
        DelayRtc { timer: lo_res_timer }
    }

    /// Gives the underlying `nrf51::RTC`*n* instance back.
    pub fn free(self) -> T {
        self.timer.free()
    }

    fn delay(&mut self, lfticks: Lfticks) {
        let ticks = self.timer.frequency().scale(lfticks.0)
            .expect("RTC compare value overflow");
        self.timer.clear();
        self.timer.set_compare_register(RtcCc::CC0, ticks);
        self.timer.start();
        while !self.timer.poll_compare_event(RtcCc::CC0) {}
        self.timer.stop();
    }
}

impl<T: Nrf51Rtc> DelayMs<u32> for DelayRtc<T> {
    fn delay_ms(&mut self, ms: u32) {
        self.delay(Lfticks::from_ms(ms));
    }
}

impl<T: Nrf51Rtc> DelayMs<u16> for DelayRtc<T> {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(u32::from(ms));
    }
}

impl<T: Nrf51Rtc> DelayMs<u8> for DelayRtc<T> {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u32::from(ms));
    }
}

impl<T: Nrf51Rtc> DelayUs<u32> for DelayRtc<T> {
    fn delay_us(&mut self, us: u32) {
        self.delay(Lfticks::from_us(us));
    }
}

impl<T: Nrf51Rtc> DelayUs<u16> for DelayRtc<T> {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(u32::from(us))
    }
}

impl<T: Nrf51Rtc> DelayUs<u8> for DelayRtc<T> {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(u32::from(us))
    }
}

/// System timer `TIMER0` as a delay provider.
///
/// This is a 32-bit timer running at 1MHz, giving a maximum setting of
/// approximately 71 minutes.
///
/// `Delay` instances implement the embedded-hal `DelayMs` and `DelayUs`
/// traits (for `u8`, `u16`, and `u32`).
///
/// `Delay` is provided in addition to `DelayTimer` for backwards
/// compatibility, and as a simple way to get an implementation of the
/// embedded-hal delay traits if you don't care about choosing which timer or
/// frequency to use.
///
/// # Panics
///
/// `delay_ms()` and `delay_us()` panic if the requested time exceeds the
/// maximum setting.
pub struct Delay(DelayTimer<TIMER0>);

impl Delay {
    /// Returns a new `Delay` wrapping TIMER0.
    ///
    /// Takes ownership of the TIMER0 peripheral.
    pub fn new(timer: TIMER0) -> Delay {
        Delay(DelayTimer::new(timer, TimerFrequency::Freq1MHz))
    }

    /// Gives the underlying `nrf51::TIMER0` instance back.
    pub fn free(self) -> TIMER0 {
        self.0.free()
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) { self.0.delay_ms(ms) }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) { self.0.delay_ms(ms) }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) { self.0.delay_ms(ms) }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) { self.0.delay_us(us) }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) { self.0.delay_us(us) }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) { self.0.delay_us(us) }
}
