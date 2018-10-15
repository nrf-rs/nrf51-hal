//! Delays

use cast::u32;
use nrf51::{TIMER0, TIMER1, TIMER2};

use hal::blocking::delay::{DelayMs, DelayUs};

/// System timer `TIMERX` as a delay provider
pub struct Delay<TIM> {
    timer: TIM,
}

macro_rules! delay {
    ([
        $($TIM:ty: $bit_mode:expr,)+
    ]) => {
        $(
            impl Delay<$TIM> {

                /// Configures the TIMER0 as a delay provider
                pub fn new(timer: $TIM) -> Self {

                    // Stop timer
                    timer.tasks_stop.write(|w| unsafe { w.bits(1) });

                    // Set counter to 32bit or 16bit mode
                    timer.bitmode.write(|w| unsafe { w.bits($bit_mode) });

                    // Set prescaler to 4 so 16MHz / 2^4 == 1MHz timer
                    // TODO: Remove hardcoded scaling
                    timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });

                    // 32bits @ 1MHz == max delay of ~72 minutes
                    // 16bits @ 1MHz == max delay of ~67 milliseconds
                    Delay { timer }
                }

                pub fn free(self) -> $TIM {
                    self.timer
                }
            }

            impl DelayMs<u32> for Delay<$TIM> {
                fn delay_ms(&mut self, ms: u32) {
                    self.delay_us(ms * 1_000);
                }
            }

            impl DelayMs<u16> for Delay<$TIM> {
                fn delay_ms(&mut self, ms: u16) {
                    self.delay_ms(u32(ms));
                }
            }

            impl DelayMs<u8> for Delay<$TIM> {
                fn delay_ms(&mut self, ms: u8) {
                    self.delay_ms(u32(ms));
                }
            }

            impl DelayUs<u32> for Delay<$TIM> {
                fn delay_us(&mut self, us: u32) {
                    // Clear event in case it was used before
                    self.timer.events_compare[0].write(|w| unsafe { w.bits(0) });

                    // Program counter compare register with value
                    self.timer.cc[0].write(|w| unsafe { w.bits(us) });

                    // Clear current counter value
                    self.timer.tasks_clear.write(|w| unsafe { w.bits(1) });

                    // Start counting
                    self.timer.tasks_start.write(|w| unsafe { w.bits(1) });

                    // Busy wait for event to happen
                    while self.timer.events_compare[0].read().bits() == 0 {}

                    // Stop counting
                    self.timer.tasks_stop.write(|w| unsafe { w.bits(1) });
                }
            }

            impl DelayUs<u16> for Delay<$TIM> {
                fn delay_us(&mut self, us: u16) {
                    self.delay_us(u32(us))
                }
            }

            impl DelayUs<u8> for Delay<$TIM> {
                fn delay_us(&mut self, us: u8) {
                    self.delay_us(u32(us))
                }
            }

        )+
    };
}

delay!([
    TIMER0: 0,
    TIMER1: 3,
    TIMER2: 3,
]);
