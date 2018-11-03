//! Delays

use nrf51::{TIMER0, TIMER1, TIMER2, RTC0, RTC1};
use hal::blocking::delay::{DelayMs, DelayUs};

pub use timer::{TimCo, TimerCounter};
pub use time::{Micros, Millis, Hertz};

pub struct Delay<TIM>(TimCo<TIM>);

macro_rules! delay {
    ([
        $($TIM:ty: $prescaler_type:ty,)+
    ]) => {
        $(

            impl Delay<$TIM> {
                /// Construct TIMER based timer with prescaler
                pub fn new(timer: $TIM, prescaler: $prescaler_type) -> Self {

                    Delay(TimCo::<$TIM>::new(timer, prescaler))
                }
            }

            impl DelayMs<u32> for Delay<$TIM> {
                fn delay_ms(&mut self, ms: u32) {
                    self.delay_us(ms * 1_000);
                }
            }

            impl DelayMs<u16> for Delay<$TIM> {
                fn delay_ms(&mut self, ms: u16) {
                    self.delay_ms(u32::from(ms));
                }
            }

            impl DelayMs<u8> for Delay<$TIM> {
                fn delay_ms(&mut self, ms: u8) {
                    self.delay_ms(u32::from(ms));
                }
            }

            impl DelayUs<u32> for Delay<$TIM> {
                fn delay_us(&mut self, us: u32) {
                    
                    let compare: u32 = Micros(u64::from(us))
                        .checked_mul(self.0.frequency())
                        .expect("Invalid delay time: {}μs");

                    self.0.delay(0, compare).unwrap();
                }
            }

            impl DelayUs<u16> for Delay<$TIM> {
                fn delay_us(&mut self, us: u16) {
                    self.delay_us(u32::from(us))
                }
            }

            impl DelayUs<u8> for Delay<$TIM> {
                fn delay_us(&mut self, us: u8) {
                    self.delay_us(u32::from(us))
                }
            }

        )+
    };
}

delay!([
    TIMER0: u8, 
    TIMER1: u8, 
    TIMER2: u8, 
    RTC0: u16, 
    RTC1: u16,
]);
