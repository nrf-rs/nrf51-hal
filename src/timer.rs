//! Timers and counters

use core::time::Duration;
use core::convert::TryFrom;
use core::u32;
use core::fmt;
use core::ops::Mul;
use core::result::Result;
use void::Void;

pub use hal::timer::CountDown;

use nrf51::{TIMER0, TIMER1, TIMER2};

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

// fn bitmode_width(bitmode: BitMode) -> u8 {
//     match bitmode {
//         BitMode::_08bit => 8,
//         BitMode::_16bit => 16,
//         BitMode::_24bit => 24,
//         BitMode::_32bit => 32,
//     }
// }

#[derive(Debug)]
pub struct InvalidRegisterValueError;

impl fmt::Display for InvalidRegisterValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "register value invalid when checked")
    }
}

pub struct Timer<TIM> {
    pub timer: TIM,
}

const HCLK_FREQ: u32 = 16_000_000;
// const LCLK_FREQ: u32 = 32768;

// fn prescaler_from_duration(bitmode: BitMode, duration: Duration) -> u8 {
//     // max_duration = 2^bit_width / clk_freq * 2^prescaler
//     let prescaler = duration.as_micros() / 1_000_000 * u128::from(HCLK_FREQ) / 2u128.pow(bitmode_width(bitmode));
//     let prescaler = f32::from(u16::try_from(prescaler).unwrap()).log2();
//     assert!(prescaler <= 9);
//     u8::try_from(prescaler.ceil()).unwrap()
// }

macro_rules! timers {
    ([
        $($TIM:ty: $bitmode:expr,)+
    ]) => {
        $(
            impl Timer<$TIM> {
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

                /// f = 16MHz / (2^prescaler)
                pub fn frequency(&mut self) -> hertz {
                    let prescaler = self.timer.prescaler.read().prescaler().bits();
                    let frequency = HCLK_FREQ.checked_div(2u32.pow(u32::from(prescaler))).unwrap();
                    frequency.hz()
                }

                /// Set comparison bit width
                /// Set counter bit width = 32bit 24bit 16bit 8bit
                ///             bitmode   = 3     2     0     1
                pub fn bitmode(&mut self, bitmode: BitMode) {
                    self.timer.bitmode.write(|w| unsafe { w.bits(bitmode as u32) });
                }

                /// Set prescaler
                /// f = 16MHz / (2^prescaler)
                /// prescaler = [0, 9]
                pub fn prescaler(&mut self, value: u8) {
                    self.timer.prescaler.write(|w| unsafe { w.prescaler().bits(value) });
                }

                /// Set prescaler, checked
                pub fn checked_prescaler(&mut self, value: u8) -> Result<(), InvalidRegisterValueError> {
                    if value > 9 {
                        return Err(InvalidRegisterValueError);
                    }
                    self.prescaler(value);
                    Ok(())
                }

                /// Set comparison value, unchecked
                pub fn compare(&mut self, idx: usize, counter: u32) {
                    // Write countdown time
                    self.timer.cc[idx].write(|w| unsafe { w.bits(counter) });
                }

                /// Set comparison value, unchecked
                pub fn checked_compare(&mut self, idx: usize, counter: u32) {

                    let bitmode = self.timer.bitmode.read().bitmode();

                    let max_size = match bitmode.bits() {
                        0 => 65_535,
                        1 => 255,
                        2 => 16_777_215,
                        3 => 4_294_967_295,
                        _ => unreachable!(),
                    };

                    // Assert counter is less than bit width
                    assert!(counter <= max_size, "counter({}) < {}", counter, max_size);

                    // Write countdown time
                    self.compare(idx, counter);
                }

                /// Reet comparison value, unchecked
                pub fn reset_compare(&mut self, idx: usize) {
                    // Reset comparison interrupt
                    self.timer.events_compare[idx].reset();
                }

                /// Enable compare interrupt
                fn compare_int_enable(&mut self, idx: usize) {
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
                pub fn compare_int_stop(&mut self, idx: usize) {

                    self.compare_int_enable(idx);
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
                pub fn compare_int_clear(&mut self, idx: usize) {

                    self.compare_int_enable(idx);
                    match idx {
                        0 => self.timer.shorts.write(|w| w.compare0_clear().enabled()),
                        1 => self.timer.shorts.write(|w| w.compare1_clear().enabled()),
                        2 => self.timer.shorts.write(|w| w.compare2_clear().enabled()),
                        3 => self.timer.shorts.write(|w| w.compare3_clear().enabled()),
                        _ => panic!(),
                    }
                }

                /// Clear timer
                /// Effect guaranteed in 1 tick
                pub fn clear(&mut self) {
                    // Clear timer counter
                    self.timer.tasks_clear.write(|w| unsafe { w.bits(1) });
                }

                /// Start timer
                /// No guarantee of delay
                pub fn start(&mut self) {
                    // Clear timer counter
                    self.clear();

                    // Start timer counter
                    self.timer.tasks_start.write(|w| unsafe { w.bits(1) });
                }

                /// Simple delay
                pub fn delay(&mut self, us: micros) {
                    
                    self.reset_compare(0);

                    let compare: u32 = us * self.frequency();

                    self.checked_compare(0, compare);
                    
                    self.start();

                    // Busy wait for event to happen
                    while self.timer.events_compare[0].read().bits() == 0 {}

                    // Stop counting
                    self.timer.tasks_stop.write(|w| unsafe { w.bits(1) });
                }

                /// Free and return timer
                pub fn free(self) -> $TIM {
                    self.timer
                }
            }

            impl CountDown for Timer<$TIM> {
                type Time = micros;

                fn start<T>(&mut self, count: T)
                where
                    T: Into<Self::Time>,
                {

                    // Reset comparison interrupt
                    self.reset_compare(0);

                    // Get comparison value
                    let compare: u32 = count.into() * self.frequency();

                    // Write countdown time
                    self.checked_compare(0, compare);

                    // Start timer
                    self.start();
                }

                fn wait(&mut self) -> nb::Result<(), Void> {
                    if self.timer.events_compare[0].read().bits() == 1 {
                        self.timer.events_compare[0].reset();
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
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