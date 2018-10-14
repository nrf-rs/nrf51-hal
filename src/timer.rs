use core::time::Duration;
use core::u32;

use void::Void;

use hal::timer::{CountDown, Periodic};
use nb::{Error, Result};
use nrf51::{TIMER0, TIMER1, TIMER2};

pub struct Timer<TIM> {
    timer: TIM,
}

macro_rules! timer {
    ($($TIM:ty),+) => {
        $(
            impl Timer<$TIM> {
                pub fn new(timer: $TIM) -> Self {
                    // 32bits @ 1MHz == max delay of ~1 hour 11 minutes
                    timer.bitmode.write(|w| w.bitmode()._32bit());
                    timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });
                    timer.intenset.write(|w| w.compare0().set());
                    timer.shorts.write(|w| w.compare0_clear().enabled());

                    Timer { timer: timer }
                }
            }

            impl CountDown for Timer<$TIM> {
                type Time = Duration;

                fn start<T>(&mut self, count: T)
                where
                    T: Into<Self::Time>,
                {
                    let duration = count.into();
                    assert!(duration.as_secs() < ((u32::MAX - duration.subsec_micros()) / 1_000_000) as u64);

                    let us = (duration.as_secs() as u32) * 1_000_000 + duration.subsec_micros();
                    self.timer.cc[0].write(|w| unsafe { w.bits(us) });

                    self.timer.events_compare[0].reset();
                    self.timer.tasks_clear.write(|w| unsafe { w.bits(1) });
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

            impl Periodic for Timer<$TIM> {}
        )+
    };
}

timer!{
    TIMER0,
    TIMER1,
    TIMER2
}
