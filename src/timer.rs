//! Timers and counters

use core::u32;
use core::fmt;
use core::result::Result;

use cast::{u32, Error};
use void::Void;

pub use hal::timer::{CountDown, Periodic};

use nrf51::{TIMER0, TIMER1, TIMER2, RTC0, RTC1};


const HFCLK_FREQ: u32 = 16_000_000;
const LFCLK_FREQ: u32 = 32_768;

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
        // (64-lhs0s) + (32-rhs0s) - 10         <= 32
        if (64 - self.0.leading_zeros()) + (32 - rhs.0.leading_zeros()) - 10 > 32 {
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
    fn checked_mul(self, rhs: Hertz) -> Option<u32> {
        let p = self.0.checked_mul(rhs.0)?;
        Some(u32(p / LFCLK_FREQ))
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

pub enum BitMode {
    _16bit = 0,
    _08bit = 1,
    _24bit = 2,
    _32bit = 3,
}

/// Error to represent values which are outside upper bounds
#[derive(Debug)]
pub struct OverValueError<T> {
    value_name: &'static str,
    value: T,
    upper_bound: T,
}

impl<T> fmt::Display for OverValueError<T> where
    T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "value invalid when checked: {}({}) < {}",
            self.value_name, self.value, self.upper_bound
        )
    }
}

pub struct Timer<TIM> {
    pub timer: TIM,
}

pub trait TimerCounter {
    /// Type for prescaler
    type Prescaler;
    /// Type for compare, unused as same for TIMER and RTC
    type Compare;

    /// Start task 
    fn task_start(&mut self);
    /// Stop task 
    fn task_stop(&mut self);
    /// Clear task 
    fn task_clear(&mut self);
    /// Frequency of counter, should read prescaler register 
    fn frequency(&mut self) -> Hertz;
    /// Set prescaler
    fn set_prescaler(&mut self, prescaler: Self::Prescaler);
    /// Set prescaler, checked
    fn checked_set_prescaler(&mut self, prescaler: Self::Prescaler) -> Result<(), OverValueError<Self::Prescaler>>;
    /// Set compare
    fn set_compare(&mut self, idx: usize, compare: Self::Compare);
    /// Set compare, checked
    fn checked_set_compare(&mut self, idx: usize, compare: Self::Compare) -> Result<(), OverValueError<Self::Compare>>;
    /// Check compare event
    fn compare_event(&mut self, idx: usize) -> bool;
    /// Reset compare event
    fn reset_compare_event(&mut self, idx: usize);

    /// Restart timer
    fn restart(&mut self) {
        
        // Clear timer counter
        self.task_clear();

        // Start timer counter
        self.task_start();
    }

    /// Set compare and start timer
    fn set_compare_start(&mut self, idx: usize, compare: Self::Compare) -> Result<(), OverValueError<Self::Compare>> {
        
        // Reset compare event
        self.reset_compare_event(idx);

        // Set compare value
        self.checked_set_compare(idx, compare)?;
        
        // Restart counter
        self.restart();

        Ok(())
    }

    /// Simple delay
    fn delay(&mut self, idx: usize, compare: Self::Compare) -> Result<(), OverValueError<Self::Compare>> {

        // Set compare event and start counter
        self.set_compare_start(idx, compare)?;

        // Busy wait for event to happen
        while !self.compare_event(idx) {}

        // Stop counting
        self.task_stop();

        Ok(())
    }
}

macro_rules! task_start {
    () => {
        fn task_start(&mut self) {
            // Start
            self.timer.tasks_start.write(|w| unsafe { w.bits(1) });
        }
    };
}

macro_rules! task_stop {
    () => {
        fn task_stop(&mut self) {
            // Stop
            self.timer.tasks_stop.write(|w| unsafe { w.bits(1) });
        }
    };
}

macro_rules! task_clear {
    () => {
        fn task_clear(&mut self) {
            // Clear
            self.timer.tasks_clear.write(|w| unsafe { w.bits(1) });
        }
    };
}

macro_rules! compare_event {
    () => {
        fn compare_event(&mut self, idx: usize) -> bool {
            // Check comparison event
            self.timer.events_compare[idx].read().bits() == 1
        }
    };
}

macro_rules! reset_compare_event {
    () => {
        fn reset_compare_event(&mut self, idx: usize) {
            // Clear comparison event
            self.timer.events_compare[idx].reset();
        }
    };
}

macro_rules! set_prescaler {
    ($prescaler_type:ty) => {
        fn set_prescaler(&mut self, prescaler: $prescaler_type) {
            // Set prescaler value
            self.timer.prescaler.write(|w| unsafe { w.prescaler().bits(prescaler) });
        }
    };
}

macro_rules! checked_set_prescaler {
    ($prescaler_type:ty, $max_prescaler:expr) => {
        fn checked_set_prescaler(&mut self, prescaler: $prescaler_type) -> Result<(), OverValueError<$prescaler_type>> {
            
            const MAX_PRESCALER: $prescaler_type = $max_prescaler;

            if prescaler >= MAX_PRESCALER {
                return Err(OverValueError{
                    value_name: "prescaler",
                    value: prescaler,
                    upper_bound: MAX_PRESCALER
                });
            }

            self.set_prescaler(prescaler);

            Ok(())
        }
    };
}

macro_rules! Countdown_wait {
    ($idx:expr) => {
        fn wait(&mut self) -> nb::Result<(), Void> {

            // Check for comparison event
            if self.compare_event($idx) {

                // Reset comparison event
                self.reset_compare_event($idx);
                Ok(())

            } else {

                Err(nb::Error::WouldBlock)
            }
        }
    };
}

macro_rules! timers_and_counters {
    ( $($TIMCO:ty),+ ) => {
        $(

            impl Timer<$TIMCO> {
                /// Free and return timer
                pub fn free(self) -> $TIMCO {
                    self.timer
                }
            }

            

        )+
    };
}

timers_and_counters!(TIMER0, TIMER1, TIMER2, RTC0, RTC1);

macro_rules! timers {
    ([
        $($TIM:ty: $bitmode:expr,)+
    ]) => {
        $(

            impl Timer<$TIM> {
                /// Construct TIMER based timer with prescaler
                pub fn new(timer: $TIM, prescaler: u8) -> Self {

                    // Stop timer
                    timer.tasks_stop.write(|w| unsafe { w.bits(1) });

                    // Set bitmode
                    timer.bitmode.write(|w| unsafe { w.bits($bitmode as u32) });
                    
                    // Set prescaler
                    timer.prescaler.write(|w| unsafe { w.prescaler().bits(prescaler) });

                    // max_duration = bit_width / 16MHz * 2^prescaler
                    // Set prescaler to 4 so 16MHz / 2^4 = 1MHz timer
                    // 32bits @ 1MHz = ~72 minutes
                    // 24bits @ 1MHz = ~16 seconds
                    // 16bits @ 1MHz = ~67 milliseconds
                    Timer { timer: timer }
                }

                /// Set comparison bit width
                /// Set counter bit width = 32bit 24bit 16bit 8bit
                ///             bitmode   = 3     2     0     1
                pub fn set_bitmode(&mut self, bitmode: BitMode) {
                    self.timer.bitmode.write(|w| unsafe { w.bits(bitmode as u32) });
                }
            }

            impl TimerCounter for Timer<$TIM> {

                type Prescaler = u8;
                type Compare = u32;

                /// Start timer
                /// Unknown start jitter
                task_start!();

                /// Stop task
                /// jitter <=1 HFCLK cycle
                task_stop!();

                /// Clear task
                /// jitter <=1 HFCLK cycle
                task_clear!();

                /// Compare event
                /// Returns true if event triggered
                compare_event!();

                /// Reset compare event
                /// Event only triggered again on rising edge
                reset_compare_event!();

                /// Set prescaler
                /// f = 16MHz / (2^prescaler)
                /// prescaler = [0, 9]
                set_prescaler!(u8);
                
                checked_set_prescaler!(u8, 9);

                /// Get frequency
                /// f = 16MHz / (2^prescaler)
                fn frequency(&mut self) -> Hertz {
                    let prescaler = self.timer.prescaler.read().prescaler().bits();
                    let frequency = HFCLK_FREQ.checked_div(2u32.pow(u32::from(prescaler))).unwrap();
                    frequency.hz()
                }

                /// Set comparison value, unchecked
                fn set_compare(&mut self, idx: usize, counter: u32) {
                    // Write countup time
                    self.timer.cc[idx].write(|w| unsafe { w.bits(counter) });
                }

                /// Set comparison value, unchecked
                fn checked_set_compare(&mut self, idx: usize, counter: Self::Compare) -> Result<(), OverValueError<Self::Compare>> {

                    let bitmode = self.timer.bitmode.read().bitmode();

                    let max_size = match bitmode.bits() {
                        0 => 65_535,
                        1 => 255,
                        2 => 16_777_215,
                        3 => 4_294_967_295,
                        _ => unreachable!(),
                    };

                    // Assert counter is less than bit width
                    // assert!(counter <= max_size, "counter({}) < {}", counter, max_size);
                    if counter > max_size {

                        return Err(OverValueError{
                            value_name: "compare",
                            value: counter,
                            upper_bound: max_size,
                        });
                    }

                    // Write countdown time
                    self.set_compare(idx, counter);

                    Ok(())
                }
            }

            impl Timer<$TIM> {

                /// Enable compare interrupt
                fn enable_compare_int(&mut self, idx: usize) {

                    match idx {
                        0 => self.timer.intenset.write(|w| w.compare0().set()),
                        1 => self.timer.intenset.write(|w| w.compare1().set()),
                        2 => self.timer.intenset.write(|w| w.compare2().set()),
                        3 => self.timer.intenset.write(|w| w.compare3().set()),
                        _ => panic!(),
                    }
                }

                /// Enable shortcut to stop on compare interrupt
                /// Timer stops on compare
                pub fn set_compare_int_stop(&mut self, idx: usize) {

                    self.enable_compare_int(idx);

                    match idx {
                        0 => self.timer.shorts.write(|w| w.compare0_stop().enabled()),
                        1 => self.timer.shorts.write(|w| w.compare1_stop().enabled()),
                        2 => self.timer.shorts.write(|w| w.compare2_stop().enabled()),
                        3 => self.timer.shorts.write(|w| w.compare3_stop().enabled()),
                        _ => panic!(),
                    }
                }

                /// Enable shortcut to clear on compare interrupt
                /// Timer is periodic
                pub fn set_compare_int_clear(&mut self, idx: usize) {

                    self.enable_compare_int(idx);

                    match idx {
                        0 => self.timer.shorts.write(|w| w.compare0_clear().enabled()),
                        1 => self.timer.shorts.write(|w| w.compare1_clear().enabled()),
                        2 => self.timer.shorts.write(|w| w.compare2_clear().enabled()),
                        3 => self.timer.shorts.write(|w| w.compare3_clear().enabled()),
                        _ => panic!(),
                    }
                }
            }

            impl CountDown for Timer<$TIM> {
                type Time = Hfticks;

                fn start<T>(&mut self, count: T)
                where
                    T: Into<Self::Time>,
                {

                    // Get comparison value
                    let compare: u32 = count
                        .into()
                        .checked_mul(self.frequency())
                        .expect("Timer count value error");

                    // Set periodic
                    self.set_compare_int_clear(0);

                    // Set compare event and start counter
                    self.set_compare_start(0, compare)
                        .expect("Timer comparison value error");
                }

                Countdown_wait!(0);
            }

            impl Periodic for Timer<$TIM> {}

            // Cancel has not been implemented as an nrf51::TIMER's status cannot be read directly.
            // This is needed as Cancel must throw an error if the timer is stopped.

        )+
    };
}

timers!([
    TIMER0: BitMode::_32bit,
    TIMER1: BitMode::_16bit,
    TIMER2: BitMode::_16bit,
]);

macro_rules! rtcs {
    ([
        $($RTC:ty: $count:expr,)+
    ]) => {
        $(

            impl Timer<$RTC> {
                /// Construct RTC based timer with prescaler
                /// *WARNING* The LFCLK needs to be activated first, e.g.
                /// ```
                /// p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
                /// ```
                pub fn new(timer: $RTC, prescaler: u16) -> Self {

                    // Start LFCLK, should be done before
                    // clock.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });

                    // Stop timer
                    timer.tasks_stop.write(|w| unsafe { w.bits(1) });
                    
                    // Set prescaler
                    timer.prescaler.write(|w| unsafe { w.prescaler().bits(prescaler) });

                    // max_duration = 24bits / 32.768kHz * (prescaler+1)
                    // 24bits @ 32.768kHz = 512 seconds
                    Timer { timer: timer }
                }
            }

            impl TimerCounter for Timer<$RTC> {

                type Prescaler = u16;
                type Compare = u32;

                /// Start timer
                /// first count:
                /// - with rising edge after first falling edge
                /// - after 30.5μs +/- 15μs
                task_start!();

                /// Stop task
                /// rising edge after first falling edge
                /// [15μs, 46μs]
                task_stop!();

                /// Clear task
                /// rising edge after first falling edge
                /// [15μs, 46μs]
                task_clear!();

                /// Compare event
                /// Returns true if event triggered
                compare_event!();

                /// Reset compare event
                /// Event only triggered again on rising edge
                reset_compare_event!();

                /// Set prescaler
                /// f = 32.768kHz / (prescaler + 1)
                /// prescaler = [0, 2^12)
                set_prescaler!(u16);

                checked_set_prescaler!(u16, 4096);

                /// Get frequency
                /// f = 32.768kHz / (prescaler + 1)
                fn frequency(&mut self) -> Hertz {
                    let prescaler = self.timer.prescaler.read().prescaler().bits();
                    let frequency = LFCLK_FREQ.checked_div(u32::from(prescaler) + 1).unwrap();
                    frequency.hz()
                }

                /// Set comparison value, unchecked
                fn set_compare(&mut self, idx: usize, counter: u32) {

                    // Enable comparison event
                    // Yes, this will often be redundant
                    // No, I could not think of a better simple way of doing this
                    // This code is free, free as in destined for landfill
                    match idx {
                        0 => self.timer.evten.write(|w| w.compare0().enabled()),
                        1 => self.timer.evten.write(|w| w.compare1().enabled()),
                        2 => self.timer.evten.write(|w| w.compare2().enabled()),
                        3 => self.timer.evten.write(|w| w.compare3().enabled()),
                        _ => panic!("Invalid set_compare idx: {}", idx),
                    }

                    // Write countup time
                    self.timer.cc[idx].write(|w| unsafe { w.bits(counter) });
                }

                /// Set comparison value, unchecked
                fn checked_set_compare(&mut self, idx: usize, counter: u32) -> Result<(), OverValueError<u32>> {

                    // 2^24
                    const MAX_COUNTER: u32 = 16_777_216;

                    // Assert counter is less than bit width
                    // assert!(counter <= max_size, "counter({}) < {}", counter, max_size);
                    if counter > MAX_COUNTER {

                        return Err(OverValueError{
                            value_name: "compare",
                            value: counter,
                            upper_bound: MAX_COUNTER,
                        });
                    }

                    // Write countdown time
                    self.set_compare(idx, counter);

                    Ok(())
                }
            }

            impl CountDown for Timer<$RTC> {
                type Time = Lfticks;

                fn start<T>(&mut self, count: T)
                where
                    T: Into<Self::Time>,
                {

                    // Get comparison value
                    let compare: u32 = count
                        .into()
                        .checked_mul(self.frequency())
                        .expect("Timer count value error");

                    // Set compare event and start counter
                    self.set_compare_start(0, compare)
                        .expect("Timer comparison value error");
                }

                Countdown_wait!(0);
            }

            // Cancel has not been implemented as an nrf51::RTC's status cannot be read directly.
            // This is needed as Cancel must throw an error if the timer is stopped.

        )+
    };
}

rtcs!([
    RTC0: 3,
    RTC1: 4,
]);