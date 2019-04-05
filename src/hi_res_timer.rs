//! The TIMER peripherals.
//!
//! This module provides a interface to the TIMER peripherals, more or less
//! directly corresponding to the underlying registers, tasks, and events.
//!
//! At present only timer mode (as opposed to counter mode) is supported.
//!
//! TIMER0's 24-bit mode isn't supported.

use core::marker::PhantomData;
use core::ops::Deref;

use cast::{u8, u16};

use nrf51;

/// The base frequency of a TIMER, in megahertz.
///
/// This is the highest available frequency.
pub const HFCLK_MHZ: u32 = 16;

/// An unsigned integer type used as the bit-width of a TIMER.
pub trait TimerWidth: Into<u32> {
    #[doc(hidden)]
    /// Set the specified timer to this type's width.
    fn program_timer<T: Nrf51Timer>(timer: &T);

    #[doc(hidden)]
    /// Checked conversion of a u32 to this type.
    fn try_from_u32(val: u32) -> Result<Self, ()>;
}

impl TimerWidth for u8 {
    #[doc(hidden)]
    fn program_timer<T: Nrf51Timer>(timer: &T) {
        timer.bitmode.write(|w| w.bitmode()._08bit());
    }

    #[doc(hidden)]
    fn try_from_u32(val: u32) -> Result<u8, ()> {
        u8(val).map_err(|_| ())
    }
}

impl TimerWidth for u16 {
    #[doc(hidden)]
    fn program_timer<T: Nrf51Timer>(timer: &T) {
        timer.bitmode.write(|w| w.bitmode()._16bit());
    }

    #[doc(hidden)]
    fn try_from_u32(val: u32) -> Result<u16, ()> {
        u16(val).map_err(|_| ())
    }
}

impl TimerWidth for u32 {
    #[doc(hidden)]
    fn program_timer<T: Nrf51Timer>(timer: &T) {
        timer.bitmode.write(|w| w.bitmode()._32bit());
    }

    #[doc(hidden)]
    fn try_from_u32(val: u32) -> Result<u32, ()> {
        Ok(val)
    }
}

/// One of the nRF51's high-resolution timers (`nrf51::TIMER0`, `nrf51::TIMER1`,
/// or `nrf51::TIMER2`)
pub trait Nrf51Timer: Deref<Target = nrf51::timer0::RegisterBlock> + Sized {
    /// An unsigned integer type with the maximum bit-width supported by this
    /// TIMER.
    type MaxWidth: TimerWidth;

    /// Takes ownership of the TIMER peripheral, returning a safe wrapper.
    ///
    /// Returns a `HiResTimer` with the maximum available bit-width.
    ///
    /// That is, 32-bit for TIMER0, 16-bit for TIMER1 or TIMER2.
    fn as_max_width_timer(self) -> HiResTimer<Self, Self::MaxWidth>;
}

/// One of the TIMER Capture/Compare registers.
///
/// Each TIMER has four CC registers.
#[derive(Copy, Clone, Debug)]
pub enum TimerCc {
    CC0 = 0,
    CC1 = 1,
    CC2 = 2,
    CC3 = 3,
}

impl TimerCc {
    /// Returns the CC register with the specified index.
    ///
    /// Note you can use 'as' to go in the other direction.
    pub fn from_index(cc: usize) -> Result<TimerCc, ()> {
        match cc {
            0 => Ok(TimerCc::CC0),
            1 => Ok(TimerCc::CC1),
            2 => Ok(TimerCc::CC2),
            3 => Ok(TimerCc::CC3),
            _ => Err(()),
        }
    }
}

/// One of the possible TIMER frequencies.
///
/// All three TIMERs support all these frequencies.
///
/// TimerFrequency | Period | 16-bit overflow | 32-bit overflow
/// -------------- | ------ | --------------- | ---------------
///  `Freq16MHz`   | 62.5ns |    >   4 ms     |  >  4 minutes
///  `Freq8MHz`    |  125ns |    >   8 ms     |  >  8 minutes
///  `Freq4MHz`    |  250ns |    >  16 ms     |  > 17 minutes
///  `Freq2MHz`    |  500ns |    >  32 ms     |  > 35 minutes
///  `Freq1MHz`    |    1µs |    >  65 ms     |  > 71 minutes
///  `Freq500kHz`  |    2µs |    > 131 ms     |  >  2 hours
///  `Freq250kHz`  |    4µs |    > 262 ms     |  >  4 hours
///  `Freq125kHz`  |    8µs |    > 524 ms     |  >  9 hours
///  `Freq62500Hz` |   16µs |    >   1 s      |  > 19 hours
///  `Freq31250Hz` |   32µs |    >   2 s      |  > 38 hours

#[derive(Copy, Clone, Debug)]
// Labels imitated from Nordic SDK nrf_timer_frequency_t
pub enum TimerFrequency {
    Freq16MHz = 0,
    Freq8MHz = 1,
    Freq4MHz = 2,
    Freq2MHz = 3,
    Freq1MHz = 4,
    Freq500kHz = 5,
    Freq250kHz = 6,
    Freq125kHz = 7,
    Freq62500Hz = 8,
    Freq31250Hz = 9,
}

impl TimerFrequency {
    /// Returns the value used in the PRESCALER register for this frequency.
    ///
    /// This is in the range 0 ..= 9.
    pub fn as_prescaler(self) -> u32 {
        self as u32
    }

    /// Returns a `TimerFrequency` given a value for the PRESCALER register.
    ///
    /// # Panics
    ///
    /// Panics if `prescaler` is greater than 9.
    pub fn from_prescaler(prescaler: u8) -> TimerFrequency {
        match prescaler {
            0 => TimerFrequency::Freq16MHz,
            1 => TimerFrequency::Freq8MHz,
            2 => TimerFrequency::Freq4MHz,
            3 => TimerFrequency::Freq2MHz,
            4 => TimerFrequency::Freq1MHz,
            5 => TimerFrequency::Freq500kHz,
            6 => TimerFrequency::Freq250kHz,
            7 => TimerFrequency::Freq125kHz,
            8 => TimerFrequency::Freq62500Hz,
            9 => TimerFrequency::Freq31250Hz,
            _ => panic!("prescaler out of range"),
        }
    }
}

/// A safe wrapper around an nRF51 TIMER in timer mode, with a specific
/// counter bit-width.
///
/// To create a `HiResTimer` from one of the `nrf51::TIMER`*n* instances, use
/// an appropriate trait and call one of the `as_max_width_timer()`,
/// `as_8bit_timer()`, `as_16bit_timer()`, or `as_32bit_timer()` methods.
///
/// Creating a `HiResTimer` in this way stops the TIMER, sets the requested
/// bit-width, and resets all other configuration to the defaults. In
/// particular the counter is reset to zero and the frequency is reset to 1MHz
/// (`Freq1MHz`).
///
/// Note that only TIMER0 supports 32-bit mode; the maximum bit-width for
/// TIMER1 and TIMER2 is 16-bit.
///
/// `HiResTimer` doesn't support TIMER0's 24-bit mode.
///
/// # Examples
///
/// ```ignore
/// use nrf51_hal::hi_res_timer::Nrf51Timer;
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut timer0 = p.TIMER0.as_max_width_timer(); // 32-bit
/// let mut timer1 = p.TIMER1.as_max_width_timer(); // 16-bit
/// let mut timer2 = p.TIMER2.as_max_width_timer(); // 16-bit
/// ```
///
/// ```ignore
/// use nrf51_hal::hi_res_timer::As8BitTimer;
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut timer1 = p.TIMER1.as_8bit_timer();
/// ```
///
/// ```ignore
/// use nrf51_hal::hi_res_timer::As16BitTimer;
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut timer2 = p.TIMER2.as_16bit_timer();
/// ```
///
/// ```ignore
/// use nrf51_hal::hi_res_timer::As32BitTimer;
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut timer0 = p.TIMER0.as_32bit_timer();
/// ```
///
/// ```ignore
/// use nrf51_hal::hi_res_timer::{As16BitTimer, As32BitTimer};
/// let p = nrf51::Peripherals::take().unwrap();
/// let mut timer0_32 = p.TIMER0.as_32bit_timer();
/// timer0_32.set_frequency(TimerFrequency::Freq31250Hz);
/// timer0_32.set_compare_register(TimerCc::CC0, 2_700_000_000);
/// timer0_32.start();
/// while !timer0_32.poll_compare_event(TimerCc::CC0) {};
/// timer0_32.stop();
/// let mut timer0_16 = timer0_32.free().as_16bit_timer();
/// ```

pub struct HiResTimer<T: Nrf51Timer, Width: TimerWidth> {
    timer: T,
    _width: PhantomData<Width>,
}

impl<T: Nrf51Timer, Width: TimerWidth> HiResTimer<T, Width> {
    // Private so that TIMER1 and TIMER2 can't be constructed with width u32.
    fn new(timer: T) -> HiResTimer<T, Width> {
        let mut h = HiResTimer {timer, _width: PhantomData};
        // Note prescaler and bitmode must be changed only when the timer is
        // stopped.
        h.stop();
        h.clear();
        h.timer.mode.reset();
        Width::program_timer(&h.timer);
        h.timer.prescaler.reset();
        h.timer.shorts.reset();
        h.timer.cc[0].reset();
        h.timer.cc[1].reset();
        h.timer.cc[2].reset();
        h.timer.cc[3].reset();
        h.timer.events_compare[0].reset();
        h.timer.events_compare[1].reset();
        h.timer.events_compare[2].reset();
        h.timer.events_compare[3].reset();
        h.timer.intenclr.write(|w| {
            w.compare0().clear()
             .compare1().clear()
             .compare2().clear()
             .compare3().clear()
        });
        h
    }

    /// Stops the timer and returns the underlying `nrf51::TIMER`*n* instance.
    ///
    /// Other than being stopped, the TIMER is left in an unspecified state.
    pub fn free(mut self) -> T {
        self.stop();
        self.timer
    }

    /// Returns the TIMER's current frequency.
    pub fn frequency(&self) -> TimerFrequency {
        let prescaler = self.timer.prescaler.read().prescaler().bits();
        TimerFrequency::from_prescaler(prescaler)
    }

    /// Stops the TIMER and sets its frequency.
    pub fn set_frequency(&mut self, frequency: TimerFrequency) {
        let prescaler = frequency.as_prescaler();
        self.stop();
        self.timer.prescaler.write(|w| unsafe { w.bits(prescaler) });
    }

    /// Resets the TIMER's counter to zero.
    ///
    /// If the TIMER was running it will continue to run.
    pub fn clear(&mut self) {
        self.timer.tasks_clear.write(|w| unsafe { w.bits(1) });
    }

    /// Starts the TIMER.
    pub fn start(&mut self) {
        self.timer.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    /// Stops the TIMER.
    pub fn stop(&mut self) {
        self.timer.tasks_stop.write(|w| unsafe { w.bits(1) });
    }

    /// Stops the TIMER and shuts down its power.
    ///
    /// This lowers power consumption, but increases the latency of the next
    /// start.
    pub fn shut_down(&mut self) {
        self.timer.tasks_shutdown.write(|w| unsafe { w.bits(1) });
    }

    /// Stores a value in the specified CC register.
    pub fn set_compare_register(&mut self, register: TimerCc, ticks: Width) {
        self.timer.cc[register as usize].write(|w| unsafe { w.bits(ticks.into()) });
    }

    /// Stores the current counter value in the specified CC register.
    pub fn capture(&mut self, register: TimerCc) {
        self.timer.tasks_capture[register as usize].write(|w| unsafe { w.bits(1) });
    }

    /// Returns the value currently stored in the specified CC register.
    ///
    /// Returns a u32. The value is expected to be small enough to fit in this
    /// timer's bit-width.
    pub fn captured_counter(&mut self, register: TimerCc) -> u32 {
        self.timer.cc[register as usize].read().bits()
    }

    /// Returns the COMPARE event flag for the specified CC register.
    ///
    /// The timer sets this flag ("generates the COMPARE event") when the
    /// counter reaches the value in the CC register.
    pub fn read_compare_event(&self, register: TimerCc) -> bool {
        let event_reg = &self.timer.events_compare[register as usize];
        event_reg.read().bits() != 0
    }

    /// Clears the COMPARE event flag for the specified CC register.
    pub fn clear_compare_event(&mut self, register: TimerCc) {
        let event_reg = &self.timer.events_compare[register as usize];
        event_reg.reset();
    }

    /// Checks and clears the COMPARE event flag for the specified CC register.
    ///
    /// Returns true if the counter has reached the value in the CC register
    /// since this method (or `clear_compare_event()`) was last called for the
    /// same register.
    pub fn poll_compare_event(&mut self, register: TimerCc) -> bool {
        let fired = self.read_compare_event(register);
        if fired { self.clear_compare_event(register) }
        fired
    }

    /// Enables the interrupt for the specified CC register.
    ///
    /// The timer's interrupt will be signalled when the register's COMPARE
    /// event is generated.
    ///
    /// The interrupt handler should clear the event flag.
    pub fn enable_compare_interrupt(&mut self, register: TimerCc) {
        self.timer.intenset.write(|w| match register {
            TimerCc::CC0 => w.compare0().set(),
            TimerCc::CC1 => w.compare1().set(),
            TimerCc::CC2 => w.compare2().set(),
            TimerCc::CC3 => w.compare3().set(),
        });
    }

    /// Disables the interrupt for the specified CC register.
    pub fn disable_compare_interrupt(&mut self, register: TimerCc) {
        self.timer.intenclr.write(|w| match register {
            TimerCc::CC0 => w.compare0().clear(),
            TimerCc::CC1 => w.compare1().clear(),
            TimerCc::CC2 => w.compare2().clear(),
            TimerCc::CC3 => w.compare3().clear(),
        });
    }

    /// Enables the shortcut between a COMPARE event and the CLEAR task.
    ///
    /// When the counter reaches the value in the specified CC register, the
    /// counter will be automatically reset to zero.
    pub fn enable_auto_clear(&mut self, register: TimerCc) {
        self.timer.shorts.write(|w| match register {
            TimerCc::CC0 => w.compare0_clear().enabled(),
            TimerCc::CC1 => w.compare1_clear().enabled(),
            TimerCc::CC2 => w.compare2_clear().enabled(),
            TimerCc::CC3 => w.compare3_clear().enabled(),
        });
    }

    /// Disables the shortcut between a COMPARE event and the CLEAR task.
    pub fn disable_auto_clear(&mut self, register: TimerCc) {
        self.timer.shorts.write(|w| match register {
            TimerCc::CC0 => w.compare0_clear().disabled(),
            TimerCc::CC1 => w.compare1_clear().disabled(),
            TimerCc::CC2 => w.compare2_clear().disabled(),
            TimerCc::CC3 => w.compare3_clear().disabled(),
        });
    }

    /// Enables the shortcut between a COMPARE event and the STOP task.
    ///
    /// When the counter reaches the value in the specified CC register, the
    /// counter will be automatically stopped.
    pub fn enable_auto_stop(&mut self, register: TimerCc) {
        self.timer.shorts.write(|w| match register {
            TimerCc::CC0 => w.compare0_stop().enabled(),
            TimerCc::CC1 => w.compare1_stop().enabled(),
            TimerCc::CC2 => w.compare2_stop().enabled(),
            TimerCc::CC3 => w.compare3_stop().enabled(),
        });
    }

    /// Disables the shortcut between a COMPARE event and the STOP task.
    pub fn disable_auto_stop(&mut self, register: TimerCc) {
        self.timer.shorts.write(|w| match register {
            TimerCc::CC0 => w.compare0_stop().disabled(),
            TimerCc::CC1 => w.compare1_stop().disabled(),
            TimerCc::CC2 => w.compare2_stop().disabled(),
            TimerCc::CC3 => w.compare3_stop().disabled(),
        });
    }
}

/// One of the nRF51's high-resolution timers, supporting an 8-bit counter
/// (which all of them do).
pub trait As8BitTimer: Nrf51Timer {
    /// Takes ownership of the TIMER peripheral, returning a safe wrapper.
    ///
    /// Returns a `HiResTimer` in 8-bit mode.
    fn as_8bit_timer(self) -> HiResTimer<Self, u8> {
        HiResTimer::new(self)
    }
}

/// One of the nRF51's high-resolution timers, supporting a 16-bit counter
/// (which all of them do).
pub trait As16BitTimer: Nrf51Timer {
    /// Takes ownership of the TIMER peripheral, returning a safe wrapper.
    ///
    /// Returns a `HiResTimer` in 16-bit mode.
    fn as_16bit_timer(self) -> HiResTimer<Self, u16> {
        HiResTimer::new(self)
    }
}

/// One of the nRF51's high-resolution timers, supporting a 32-bit counter
/// (ie, only `nrf51::TIMER0`).
pub trait As32BitTimer: Nrf51Timer {
    /// Takes ownership of the TIMER peripheral, returning a safe wrapper.
    ///
    /// Returns a `HiResTimer` in 32-bit mode.
    fn as_32bit_timer(self) -> HiResTimer<Self, u32> {
        HiResTimer::new(self)
    }
}

impl As8BitTimer for nrf51::TIMER0 {}
impl As8BitTimer for nrf51::TIMER1 {}
impl As8BitTimer for nrf51::TIMER2 {}
impl As16BitTimer for nrf51::TIMER0 {}
impl As16BitTimer for nrf51::TIMER1 {}
impl As16BitTimer for nrf51::TIMER2 {}
impl As32BitTimer for nrf51::TIMER0 {}

impl Nrf51Timer for nrf51::TIMER0 {
    type MaxWidth = u32;
    fn as_max_width_timer(self) -> HiResTimer<nrf51::TIMER0, u32> {
        self.as_32bit_timer()
    }
}

impl Nrf51Timer for nrf51::TIMER1 {
    type MaxWidth = u16;
    fn as_max_width_timer(self) -> HiResTimer<nrf51::TIMER1, u16> {
        self.as_16bit_timer()
    }
}

impl Nrf51Timer for nrf51::TIMER2 {
    type MaxWidth = u16;
    fn as_max_width_timer(self) -> HiResTimer<nrf51::TIMER2, u16> {
        self.as_16bit_timer()
    }
}
