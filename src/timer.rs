//! Timers and counters

use core::time::Duration;
use core::convert::TryFrom;
use core::u32;
use core::fmt;
use core::ops::Mul;
use core::result::Result;
use void::Void;

pub use hal::timer::CountDown;

use nrf51::{TIMER0, TIMER1, TIMER2, RTC0, RTC1};

/// Microseconds
#[derive(Debug, Clone, Copy)]
pub struct micros(pub u32);

/// Hertz
#[derive(Debug, Clone, Copy)]
pub struct hertz(pub u32);

fn mul_micro_u32(lhs: u32, rhs: u32) -> u32 {
    u32::try_from(u64::from(lhs) * u64::from(rhs) / 1_000_000).unwrap()
}

impl Mul<hertz> for micros {
    type Output = u32;

    fn mul(self, rhs: hertz) -> u32 {
        mul_micro_u32(self.0, rhs.0)
    }
}

impl Mul<micros> for hertz {
    type Output = u32;

    fn mul(self, rhs: micros) -> u32 {
        mul_micro_u32(self.0, rhs.0)
    }
}

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `us`
    fn us(self) -> micros;
    /// Wrap in `hz`
    fn hz(self) -> hertz;
}

impl U32Ext for u32 {
    fn us(self) -> micros {
        micros(self)
    }
    fn hz(self) -> hertz {
        hertz(self)
    }
}

/// Extension trait that adds `Into<micros>` to the `Duration` type
impl Into<micros> for Duration {
    fn into(self) -> micros {
        micros(u32::try_from(self.as_micros()).unwrap())
    }
}

pub enum BitMode {
    _16bit = 0,
    _08bit = 1,
    _24bit = 2,
    _32bit = 3,
}

#[derive(Debug)]
pub struct InvalidValueError<T> {
    value_name: &'static str,
    value: T,
    upper_bound: T,
}

impl<T> fmt::Display for InvalidValueError<T> where
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

const HCLK_FREQ: u32 = 16_000_000;
// const LCLK_FREQ: u32 = 32768;

// fn bitmode_width(bitmode: BitMode) -> u8 {
//     match bitmode {
//         BitMode::_08bit => 8,
//         BitMode::_16bit => 16,
//         BitMode::_24bit => 24,
//         BitMode::_32bit => 32,
//     }
// }

// fn prescaler_from_duration(bitmode: BitMode, duration: Duration) -> u8 {
//     // max_duration = 2^bit_width / clk_freq * 2^prescaler
//     let prescaler = duration.as_micros() / 1_000_000 * u128::from(HCLK_FREQ) / 2u128.pow(bitmode_width(bitmode));
//     let prescaler = f32::from(u16::try_from(prescaler).unwrap()).log2();
//     assert!(prescaler <= 9);
//     u8::try_from(prescaler.ceil()).unwrap()
// }

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
    fn frequency(&mut self) -> hertz;
    /// Set prescaler
    fn set_prescaler(&mut self, prescaler: Self::Prescaler);
    /// Set prescaler, checked
    fn checked_set_prescaler(&mut self, prescaler: Self::Prescaler) -> Result<(), InvalidValueError<Self::Prescaler>>;
    /// Set compare
    fn set_compare(&mut self, idx: usize, compare: Self::Compare);
    /// Set compare, checked
    fn checked_set_compare(&mut self, idx: usize, compare: Self::Compare) -> Result<(), InvalidValueError<Self::Compare>>;
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
    fn set_compare_start(&mut self, idx: usize, compare: Self::Compare) -> Result<(), InvalidValueError<Self::Compare>> {
        
        // Reset compare event
        self.reset_compare_event(idx);

        // Set compare value
        self.checked_set_compare(idx, compare)?;
        
        // Restart counter
        self.restart();

        Ok(())
    }

    /// Simple delay
    fn delay(&mut self, idx: usize, compare: Self::Compare) -> Result<(), InvalidValueError<Self::Compare>> {

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
        fn checked_set_prescaler(&mut self, prescaler: $prescaler_type) -> Result<(), InvalidValueError<$prescaler_type>> {
            
            const MAX_PRESCALER: $prescaler_type = $max_prescaler;

            if prescaler >= MAX_PRESCALER {
                return Err(InvalidValueError{
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

macro_rules! timers_and_counters {
    ( $($TIMCO:ty),+ ) => {
        $(

            impl Timer<$TIMCO> {
                /// Free and return timer
                pub fn free(self) -> $TIMCO {
                    self.timer
                }
            }

            impl CountDown for Timer<$TIMCO> {
                type Time = micros;

                fn start<T>(&mut self, count: T)
                where
                    T: Into<Self::Time>,
                {

                    // Get comparison value
                    let compare: u32 = count.into() * self.frequency();

                    // Set compare event and start counter
                    let r = self.set_compare_start(0, compare);

                    if let Err(_) = r {
                        panic!("Timer value error");
                    }
                }

                fn wait(&mut self) -> nb::Result<(), Void> {

                    // Check for comparison event
                    if self.compare_event(0) {

                        // Reset comparison event
                        self.reset_compare_event(0);
                        Ok(())

                    } else {

                        Err(nb::Error::WouldBlock)
                    }
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
                /// Construct with prescaler
                pub fn new(timer: $TIM, prescaler: u8) -> Self {

                    // Stop timer
                    timer.tasks_stop.write(|w| unsafe { w.bits(1) });

                    timer.bitmode.write(|w| unsafe { w.bits($bitmode as u32) });
                    
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
                fn frequency(&mut self) -> hertz {
                    let prescaler = self.timer.prescaler.read().prescaler().bits();
                    let frequency = HCLK_FREQ.checked_div(2u32.pow(u32::from(prescaler))).unwrap();
                    frequency.hz()
                }

                /// Set comparison value, unchecked
                fn set_compare(&mut self, idx: usize, counter: u32) {
                    // Write countup time
                    self.timer.cc[idx].write(|w| unsafe { w.bits(counter) });
                }

                /// Set comparison value, unchecked
                fn checked_set_compare(&mut self, idx: usize, counter: Self::Compare) -> Result<(), InvalidValueError<Self::Compare>> {

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

                        return Err(InvalidValueError{
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
        $($TIM:ty: $count:expr,)+
    ]) => {
        $(

            impl Timer<$TIM> {
                /// Construct with prescaler
                pub fn new(timer: $TIM, prescaler: u16) -> Self {

                    // Stop timer
                    timer.tasks_stop.write(|w| unsafe { w.bits(1) });
                    
                    timer.prescaler.write(|w| unsafe { w.prescaler().bits(prescaler) });

                    // max_duration = bit_width / 16MHz * 2^prescaler
                    // Set prescaler to 4 so 16MHz / 2^4 = 1MHz timer
                    // 32bits @ 1MHz = ~72 minutes
                    // 24bits @ 1MHz = ~16 seconds
                    // 16bits @ 1MHz = ~67 milliseconds
                    Timer { timer: timer }
                }
            }

            impl TimerCounter for Timer<$TIM> {

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
                fn frequency(&mut self) -> hertz {
                    let prescaler = self.timer.prescaler.read().prescaler().bits();
                    let frequency = HCLK_FREQ.checked_div(2u32.pow(u32::from(prescaler))).unwrap();
                    frequency.hz()
                }

                /// Set comparison value, unchecked
                fn set_compare(&mut self, idx: usize, counter: u32) {
                    // Write countup time
                    self.timer.cc[idx].write(|w| unsafe { w.bits(counter) });
                }

                /// Set comparison value, unchecked
                fn checked_set_compare(&mut self, idx: usize, counter: u32) -> Result<(), InvalidValueError<u32>> {

                    // 2^24
                    const MAX_COUNTER: u32 = 16_777_216;

                    // Assert counter is less than bit width
                    // assert!(counter <= max_size, "counter({}) < {}", counter, max_size);
                    if counter > MAX_COUNTER {

                        return Err(InvalidValueError{
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

        )+
    };
}

rtcs!([
    RTC0: 3,
    RTC1: 4,
]);