//! Delays

use nrf51::{TIMER0, TIMER1, TIMER2, RTC0, RTC1};
use hal::blocking::delay::{DelayMs, DelayUs};

pub use timer::{Timer, TimerCounter};
pub use timer::{Micros, Millis, Hertz};

pub type Delay<TIM> = Timer<TIM>;

macro_rules! delay {
    ( $($TIM:ty),+ ) => {
        $(

            impl DelayMs<u32> for Timer<$TIM> {
                fn delay_ms(&mut self, ms: u32) {
                    self.delay_us(ms * 1_000);
                }
            }

            impl DelayMs<u16> for Timer<$TIM> {
                fn delay_ms(&mut self, ms: u16) {
                    self.delay_ms(u32::from(ms));
                }
            }

            impl DelayMs<u8> for Timer<$TIM> {
                fn delay_ms(&mut self, ms: u8) {
                    self.delay_ms(u32::from(ms));
                }
            }

            impl DelayUs<u32> for Timer<$TIM> {
                fn delay_us(&mut self, us: u32) {
                    
                    let compare: u32 = Micros(u64::from(us))
                        .checked_mul(self.frequency())
                        .expect("Invalid delay time: {}μs");

                    self.delay(0, compare).unwrap();
                }
            }

            impl DelayUs<u16> for Timer<$TIM> {
                fn delay_us(&mut self, us: u16) {
                    self.delay_us(u32::from(us))
                }
            }

            impl DelayUs<u8> for Timer<$TIM> {
                fn delay_us(&mut self, us: u8) {
                    self.delay_us(u32::from(us))
                }
            }

        )+
    };
}

delay!(TIMER0, TIMER1, TIMER2, RTC0, RTC1);
