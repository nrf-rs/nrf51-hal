//! The RTC (Real Time Counter) peripherals.
//!
//! This module provides a interface to the RTC peripherals, more or less
//! directly corresponding to the underlying registers, tasks, and events.
//!
//! Note that in order for the RTCs to run, you must also start the
//! low-frequency clock:
//! ```ignore
//! let p = nrf51::Peripherals::take().unwrap();
//! p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
//! while p.CLOCK.events_lfclkstarted.read().bits() == 0 {}
//! p.CLOCK.events_lfclkstarted.reset();
//! ```

use core::ops::Deref;

use cast::u32;

use nrf51;

/// The base frequency of a RTC peripheral, in hertz.
///
/// This is the highest available frequency.
pub const LFCLK_HZ: u32 = 32_768;

/// One of the nRF51's low-resolution timers (`nrf51::RTC0` or `nrf51::RTC1`).
pub trait Nrf51Rtc: Deref<Target = nrf51::rtc0::RegisterBlock> + Sized {
    /// Returns true if the specified CC register is present on this RTC.
    ///
    /// RTC1 has all four registers; RTC0 is missing CC3.
    fn has_register(register: RtcCc) -> bool;

    /// Panic if this RTC doesn't have the specified CC register.
    fn validate_register(register: RtcCc) {
        assert!(Self::has_register(register), "register is not present");
    }
}

/// One of the RTC Compare registers.
///
/// (We follow the reference manual in abbreviating these as 'CC', although
/// they're not used for capture.)
///
/// Note that RTC0 doesn't have the CC3 register.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RtcCc {
    CC0 = 0,
    CC1 = 1,
    CC2 = 2,
    CC3 = 3,
}

impl RtcCc {
    /// Returns the CC register with the specified index.
    ///
    /// Note you can use 'as' to go in the other direction.
    pub fn from_index(cc: usize) -> Result<RtcCc, ()> {
        match cc {
            0 => Ok(RtcCc::CC0),
            1 => Ok(RtcCc::CC1),
            2 => Ok(RtcCc::CC2),
            3 => Ok(RtcCc::CC3),
            _ => Err(()),
        }
    }
}

/// A possible RTC frequency.
///
/// The hardware represents the frequency using a 12-bit prescaler value which
/// determines the RTC's frequency as follows:
///
/// *frequency in Hz* = 32768 / (*prescaler* + 1)
///
/// Use `RtcFrequency::from_prescaler()` to create an arbitrary `RtcFrequency`
/// from the corresponding prescaler value.
///
/// The `lo_res_timer::FREQ_`*nnn*`HZ` convenience constants correspond to the
/// power-of-two submultiples of the base 32768Hz frequency:
///
/// Frequency      | Period   | Overflow
/// -------------- | -------- | ---------
/// `FREQ_32768HZ` | ~30.5µs  | 512 seconds
/// `FREQ_16384HZ` | ~61µs    | over 17 minutes
/// `FREQ_8192HZ`  | ~122µs   | over 34 minutes
/// `FREQ_4096HZ`  | ~244µs   | over 68 minutes
/// `FREQ_2048HZ`  | ~488µs   | over 136 minutes
/// `FREQ_1024HZ`  | ~977µs   | over 4.5 hours
/// `FREQ_512HZ`   | ~1.95ms  | over 9 hours
/// `FREQ_256HZ`   | ~3.9ms   | over 18 hours
/// `FREQ_128HZ`   | ~7.8ms   | over 36 hours
/// `FREQ_64HZ`    | 15.625ms | over 3 days
/// `FREQ_32HZ`    | 31.25ms  | over 6 days
/// `FREQ_16HZ`    | 62.5ms   | over 12 days
/// `FREQ_8HZ`     | 125ms    | over 24 days
///
/// # Examples
///
/// ```
/// let frequency = RtcFrequency::new(327); // 99.9Hz
/// let frequency = FREQ_32768HZ;
/// let frequency = FREQ_8HZ;
/// ```
#[derive(Copy, Clone, Debug)]
pub struct RtcFrequency(u16);

impl RtcFrequency {
    /// Returns an `RtcFrequency` with the specified prescaler value.
    ///
    /// # Panics
    ///
    /// Panics if the supplied value is ≥ 2^12 (4096).
    pub fn from_prescaler<T: Into<u32>>(i: T) -> RtcFrequency {
        let i = i.into();
        assert!(i < 1 << 12, "prescaler value out of range");
        RtcFrequency(i as u16)
    }

    const fn const_from_prescaler(i: u16) -> RtcFrequency {
        RtcFrequency(i)
    }

    /// Returns the prescaler value.
    pub fn as_prescaler(self) -> u32 {
        self.0 as u32
    }

    /// Apply the effective prescaler value.
    ///
    /// Converts a number of ticks of the base 32768Hz clock to the equivalent
    /// number of ticks for an RTC using this frequency.
    ///
    /// Rounds down.
    ///
    /// Returns `None` if the result doesn't fit in a u32 (but note that if
    /// the result doesn't fit in 24 bits it can't be used directly).
    pub fn scale(self, base_ticks: u64) -> Option<u32> {
        u32(base_ticks / (self.0 + 1) as u64).ok()
    }
}

/// RTC frequency 32768Hz
pub const FREQ_32768HZ: RtcFrequency = RtcFrequency::const_from_prescaler(0);

/// RTC frequency 16384Hz
pub const FREQ_16384HZ: RtcFrequency = RtcFrequency::const_from_prescaler(1);

/// RTC frequency 8192Hz
pub const FREQ_8192HZ: RtcFrequency = RtcFrequency::const_from_prescaler(3);

/// RTC frequency 4096Hz
pub const FREQ_4096HZ: RtcFrequency = RtcFrequency::const_from_prescaler(7);

/// RTC frequency 2048Hz
pub const FREQ_2048HZ: RtcFrequency = RtcFrequency::const_from_prescaler(15);

/// RTC frequency 1024Hz
pub const FREQ_1024HZ: RtcFrequency = RtcFrequency::const_from_prescaler(31);

/// RTC frequency 512Hz
pub const FREQ_512HZ: RtcFrequency = RtcFrequency::const_from_prescaler(63);

/// RTC frequency 256Hz
pub const FREQ_256HZ: RtcFrequency = RtcFrequency::const_from_prescaler(127);

/// RTC frequency 128Hz
pub const FREQ_128HZ: RtcFrequency = RtcFrequency::const_from_prescaler(255);

/// RTC frequency 64Hz
pub const FREQ_64HZ: RtcFrequency = RtcFrequency::const_from_prescaler(511);

/// RTC frequency 32Hz
pub const FREQ_32HZ: RtcFrequency = RtcFrequency::const_from_prescaler(1023);

/// RTC frequency 16Hz
pub const FREQ_16HZ: RtcFrequency = RtcFrequency::const_from_prescaler(2047);

/// RTC frequency 8Hz
pub const FREQ_8HZ: RtcFrequency = RtcFrequency::const_from_prescaler(4095);

/// A safe wrapper around an nRF51 RTC (Real Time Counter) peripheral.
///
/// Both RTC peripherals have a fixed counter width of 24 bits.
///
/// Note that when a request is made to start or stop the counter it will take
/// effect after a delay of 15 to 46 µs. See the nRF51 Series Reference Manual
/// for details.
///
/// # Panics
///
/// All methods taking a `RtcCc` parameter will panic if `CC3` is passed and
/// the underlying peripheral is RTC0.
///
/// # Examples
///
/// ```ignore
/// use nrf51_hal::lo_res_timer::{LoResTimer, FREQ_1024HZ};
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut rtc0 = LoResTimer::new(p.RTC0);
/// rtc0.set_frequency(FREQ_1024HZ);
/// rtc0.set_compare_register(RtcCc::CC0, 512);
/// rtc0.start();
/// while !rtc0.poll_compare_event(RtcrCc::CC0) {};
/// rtc0.stop();
/// ```
pub struct LoResTimer<T: Nrf51Rtc> {
    rtc: T,
}

impl<T: Nrf51Rtc> LoResTimer<T> {
    /// Takes ownership of the RTC peripheral, returning a safe wrapper.
    ///
    /// Stops the RTC and resets all other configuration to the defaults. In
    /// particular all events are disabled, the counter is reset to zero, and
    /// the prescaler is reset from `FREQ_32768HZ`.
    pub fn new(rtc: T) -> LoResTimer<T> {
        let mut l = LoResTimer { rtc };
        // Note prescaler should be changed only when the timer is stopped.
        l.stop();
        l.clear();
        l.rtc.prescaler.reset();
        l.rtc.cc[0].reset();
        l.rtc.cc[1].reset();
        l.rtc.cc[2].reset();
        if T::has_register(RtcCc::CC3) {
            l.rtc.cc[3].reset();
        }
        l.rtc.events_tick.reset();
        l.rtc.events_ovrflw.reset();
        l.rtc.events_compare[0].reset();
        l.rtc.events_compare[1].reset();
        l.rtc.events_compare[2].reset();
        if T::has_register(RtcCc::CC3) {
            l.rtc.events_compare[3].reset();
        }
        l.rtc.evtenclr.write(|w| {
            let w = w
                .tick()
                .clear()
                .ovrflw()
                .clear()
                .compare0()
                .clear()
                .compare1()
                .clear()
                .compare2()
                .clear();
            if T::has_register(RtcCc::CC3) {
                w.compare3().clear()
            } else {
                w
            }
        });
        l.rtc.intenclr.write(|w| {
            let w = w
                .tick()
                .clear()
                .ovrflw()
                .clear()
                .compare0()
                .clear()
                .compare1()
                .clear()
                .compare2()
                .clear();
            if T::has_register(RtcCc::CC3) {
                w.compare3().clear()
            } else {
                w
            }
        });
        l
    }

    /// Stops the RTC and returns the underlying `nrf51::RTC`*n* instance.
    ///
    /// Other than being stopped, the RTC is left in an unspecified state.
    pub fn free(mut self) -> T {
        self.stop();
        self.rtc
    }

    /// Returns the RTC's current frequency by reading the prescaler register.
    pub fn frequency(&self) -> RtcFrequency {
        RtcFrequency::from_prescaler(self.rtc.prescaler.read().prescaler().bits())
    }

    /// Stops the RTC and sets its frequency.
    ///
    /// Writes the prescaler register.
    pub fn set_frequency(&mut self, frequency: RtcFrequency) {
        self.stop();
        self.rtc
            .prescaler
            .write(|w| unsafe { w.bits(frequency.as_prescaler()) });
    }

    /// Returns the RTC's current counter value.
    pub fn read_counter(&self) -> u32 {
        self.rtc.counter.read().counter().bits()
    }

    /// Stores a value in the specified CC register.
    ///
    /// # Panics
    ///
    /// Panics if `ticks` is ≥ 2^24.
    pub fn set_compare_register(&mut self, register: RtcCc, ticks: u32) {
        T::validate_register(register);
        assert!(ticks < 1 << 24, "register value out of range");
        self.rtc.cc[register as usize].write(|w| unsafe { w.bits(ticks) });
    }

    /// Resets the RTC's counter to zero.
    ///
    /// If the RTC was running it will continue to run.
    pub fn clear(&mut self) {
        self.rtc.tasks_clear.write(|w| unsafe { w.bits(1) });
    }

    /// Starts the RTC.
    pub fn start(&mut self) {
        self.rtc.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    /// Stops the RTC.
    pub fn stop(&mut self) {
        self.rtc.tasks_stop.write(|w| unsafe { w.bits(1) });
    }

    /// Sets the RTC's counter value to 0xFFFFF0.
    ///
    /// This will cause it to overflow shortly afterwards.
    pub fn trigger_overflow(&mut self) {
        self.rtc.tasks_trigovrflw.write(|w| unsafe { w.bits(1) });
    }

    /// Enables the COMPARE event for the specified CC register.
    ///
    /// The event is generated when the counter reaches the value in the CC
    /// register.
    pub fn enable_compare_event(&mut self, register: RtcCc) {
        T::validate_register(register);
        self.rtc.evtenset.write(|w| match register {
            RtcCc::CC0 => w.compare0().set(),
            RtcCc::CC1 => w.compare1().set(),
            RtcCc::CC2 => w.compare2().set(),
            RtcCc::CC3 => w.compare3().set(),
        });
    }

    /// Disables the COMPARE event for the specified CC register.
    pub fn disable_compare_event(&mut self, register: RtcCc) {
        T::validate_register(register);
        self.rtc.evtenclr.write(|w| match register {
            RtcCc::CC0 => w.compare0().clear(),
            RtcCc::CC1 => w.compare1().clear(),
            RtcCc::CC2 => w.compare2().clear(),
            RtcCc::CC3 => w.compare3().clear(),
        });
    }

    /// Enables the TICK event.
    ///
    /// The event is generated each time the RTC's counter increases.
    pub fn enable_tick_event(&mut self) {
        self.rtc.evtenset.write(|w| w.tick().set());
    }

    /// Disables the TICK event.
    pub fn disable_tick_event(&mut self) {
        self.rtc.evtenclr.write(|w| w.tick().clear());
    }

    /// Enables the OVRFLW event.
    ///
    /// The event is generated when the RTC's counter overflows from 0xFFFFFF
    /// to 0.
    pub fn enable_overflow_event(&mut self) {
        self.rtc.evtenset.write(|w| w.ovrflw().set());
    }

    /// Disables the OVRFLW event.
    pub fn disable_overflow_event(&mut self) {
        self.rtc.evtenclr.write(|w| w.ovrflw().clear());
    }

    /// Enables the interrupt for the specified CC register.
    ///
    /// The RTC's interrupt will be signalled when the register's COMPARE
    /// event is generated.
    ///
    /// Note the event must be enabled separately.
    ///
    /// The interrupt handler should clear the event flag.
    pub fn enable_compare_interrupt(&mut self, register: RtcCc) {
        T::validate_register(register);
        self.rtc.intenset.write(|w| match register {
            RtcCc::CC0 => w.compare0().set(),
            RtcCc::CC1 => w.compare1().set(),
            RtcCc::CC2 => w.compare2().set(),
            RtcCc::CC3 => w.compare3().set(),
        });
    }

    /// Disables the interrupt for the specified CC register.
    pub fn disable_compare_interrupt(&mut self, register: RtcCc) {
        T::validate_register(register);
        self.rtc.intenclr.write(|w| match register {
            RtcCc::CC0 => w.compare0().clear(),
            RtcCc::CC1 => w.compare1().clear(),
            RtcCc::CC2 => w.compare2().clear(),
            RtcCc::CC3 => w.compare3().clear(),
        });
    }

    /// Enables the interrupt for the TICK event.
    ///
    /// The RTC's interrupt will be signalled when the TICK event is generated.
    ///
    /// Note the event must be enabled separately.
    ///
    /// The interrupt handler should clear the event flag.
    pub fn enable_tick_interrupt(&mut self) {
        self.rtc.intenset.write(|w| w.tick().set());
    }

    /// Disables the interrupt for the TICK event.
    pub fn disable_tick_interrupt(&mut self) {
        self.rtc.intenclr.write(|w| w.tick().clear());
    }

    /// Enables the interrupt for the OVRFLW event.
    ///
    /// The RTC's interrupt will be signalled when the OVRFLW event is
    /// generated.
    ///
    /// Note the event must be enabled separately.
    ///
    /// The interrupt handler should clear the event flag.
    pub fn enable_overflow_interrupt(&mut self) {
        self.rtc.intenset.write(|w| w.ovrflw().set());
    }

    /// Disables the interrupt for the OVRFLW event.
    pub fn disable_overflow_interrupt(&mut self) {
        self.rtc.intenclr.write(|w| w.ovrflw().clear());
    }

    /// Returns the COMPARE event flag for the specified CC register.
    ///
    /// The RTC sets this flag ("generates the COMPARE event") when the event
    /// is enabled and the counter reaches the value in the CC register.
    pub fn read_compare_event(&self, register: RtcCc) -> bool {
        let event_reg = &self.rtc.events_compare[register as usize];
        event_reg.read().bits() != 0
    }

    /// Clears the COMPARE event flag for the specified CC register.
    pub fn clear_compare_event(&mut self, register: RtcCc) {
        let event_reg = &self.rtc.events_compare[register as usize];
        event_reg.reset();
    }

    /// Checks and clears the COMPARE event flag for the specified CC register.
    ///
    /// Returns true if the counter has reached the value in the CC register
    /// (and the event is enabled) since this method (or
    /// `clear_compare_event()`) was last called for the same register.
    pub fn poll_compare_event(&mut self, register: RtcCc) -> bool {
        T::validate_register(register);
        let fired = self.read_compare_event(register);
        if fired {
            self.clear_compare_event(register)
        }
        fired
    }

    /// Returns the TICK event flag.
    pub fn read_tick_event(&self) -> bool {
        self.rtc.events_tick.read().bits() != 0
    }

    /// Clears the TICK event flag.
    pub fn clear_tick_event(&mut self) {
        self.rtc.events_tick.reset();
    }

    /// Checks and clears the TICK event flag.
    ///
    /// Returns true if the TICK event has occurred (and the event is enabled)
    /// since this method (or `clear_tick_event()`) was last called.
    pub fn poll_tick_event(&mut self) -> bool {
        let fired = self.read_tick_event();
        if fired {
            self.clear_tick_event()
        }
        fired
    }

    /// Returns the OVRFLW event flag.
    pub fn read_overflow_event(&self) -> bool {
        self.rtc.events_ovrflw.read().bits() != 0
    }

    /// Clears the OVRFLW event flag.
    pub fn clear_overflow_event(&mut self) {
        self.rtc.events_ovrflw.reset();
    }

    /// Checks and clears the OVRFLW event flag.
    ///
    /// Returns true if the OVRFLW event has occurred (and the event is
    /// enabled) since this method (or `clear_overflow_event()`) was last
    /// called.
    pub fn poll_overflow_event(&mut self) -> bool {
        let fired = self.read_overflow_event();
        if fired {
            self.clear_overflow_event()
        }
        fired
    }
}

impl Nrf51Rtc for nrf51::RTC0 {
    fn has_register(register: RtcCc) -> bool {
        register != RtcCc::CC3
    }
}

impl Nrf51Rtc for nrf51::RTC1 {
    #[allow(unused_variables)]
    fn has_register(register: RtcCc) -> bool {
        true
    }
}
