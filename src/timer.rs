use core::time::Duration;
use core::convert::TryFrom;
use core::{u16, u32};

use void::Void;

use hal::timer::{CountDown, Periodic};
use nb::{Error, Result};
use nrf51::{TIMER0, TIMER1, TIMER2};

pub struct Timer<TIM> {
    timer: TIM,
}

macro_rules! timer {
    ([
        $($TIM:ty: ($bit_mode:expr, $bit_width:expr),)+
    ]) => {
        $(
            impl Timer<$TIM> {
                pub fn new(timer: $TIM) -> Self {

                    // Stop timer
                    timer.tasks_stop.write(|w| unsafe { w.bits(1) });

                    // 32bits @ 1MHz == max delay of ~1 hour 11 minutes
                    timer.bitmode.write(|w| unsafe { w.bits($bit_mode) });

                    // Set prescaler to 4 so 16MHz / 2^4 == 1MHz timer
                    // TODO: Remove hardcoded scaling
                    timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });

                    // Enable interrupt
                    timer.intenset.write(|w| w.compare0().set());

                    // Enable shortcut to clear on compare interrupt
                    // timer.shorts.write(|w| w.compare0_clear().enabled());

                    // Enable shortcut to clear on compare interrupt
                    // timer.shorts.write(|w| w.compare0_clear().enabled());
                    // Enable shortcut to stop on compare interrupt
                    timer.shorts.write(|w| w.compare0_stop().enabled());

                    // 32bits @ 1MHz == ~72 minutes
                    // 16bits @ 1MHz == ~67 milliseconds
                    Timer { timer: timer }
                }

                pub fn free(self) -> $TIM {
                    self.timer
                }
            }

            impl CountDown for Timer<$TIM> {
                type Time = Duration;

                fn start<T>(&mut self, count: T)
                where
                    T: Into<Self::Time>,
                {
                    let duration = count.into();

                    // Check for overflow and convert to us: u32
                    let us = u32::try_from(duration.as_micros()).unwrap();

                    // Assert us is less than allowed bit width
                    assert!(us <= u32::from($bit_width));

                    // Write countdown time
                    self.timer.cc[0].write(|w| unsafe { w.bits(us) });

                    // Reset comparison interrupt
                    self.timer.events_compare[0].reset();

                    // Clear timer
                    self.timer.tasks_clear.write(|w| unsafe { w.bits(1) });

                    // Start timer
                    self.timer.tasks_start.write(|w| unsafe { w.bits(1) });
                }

                fn wait(&mut self) -> Result<(), Void> {
                    if self.timer.events_compare[0].read().bits() == 1 {
                        self.timer.events_compare[0].reset();
                        Ok(())
                    } else {
                        Err(Error::WouldBlock)
                    }
                }
            }

            // Mark this timer as periodic
            impl Periodic for Timer<$TIM> {}
        )+
    };
}

timer!([
    TIMER0: (0, u32::MAX),
    TIMER1: (3, u16::MAX),
    TIMER2: (3, u16::MAX),
]);