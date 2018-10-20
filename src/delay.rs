//! Delays

use cast::u32;
use nrf51::{TIMER0, TIMER1, TIMER2};

use hal::blocking::delay::{DelayMs, DelayUs};

pub use timer::{Timer, BitMode};
pub use timer::{micros, hertz};

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
                    self.delay_ms(u32(ms));
                }
            }

            impl DelayMs<u8> for Timer<$TIM> {
                fn delay_ms(&mut self, ms: u8) {
                    self.delay_ms(u32(ms));
                }
            }

            impl DelayUs<u32> for Timer<$TIM> {
                fn delay_us(&mut self, us: u32) {
                    
                    self.delay(micros(us));
                }
            }

            impl DelayUs<u16> for Timer<$TIM> {
                fn delay_us(&mut self, us: u16) {
                    self.delay_us(u32(us))
                }
            }

            impl DelayUs<u8> for Timer<$TIM> {
                fn delay_us(&mut self, us: u8) {
                    self.delay_us(u32(us))
                }
            }

        )+
    };
}

delay!(TIMER0, TIMER1, TIMER2);
