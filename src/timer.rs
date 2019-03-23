use core::time::Duration;
use core::u32;

use void::Void;

use hal::timer::{CountDown, Periodic};
use nb::{Error, Result};
use nrf51::TIMER0;

pub struct Timer(TIMER0);

impl Timer {
    pub fn new(timer: TIMER0) -> Timer {
        // 32bits @ 1MHz == max delay of ~1 hour 11 minutes
        timer.bitmode.write(|w| w.bitmode()._32bit());
        timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });
        timer.intenset.write(|w| w.compare0().set());
        timer.shorts.write(|w| w.compare0_clear().enabled());

        Timer(timer)
    }
}

impl CountDown for Timer {
    type Time = Duration;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        let duration = count.into();
        assert!(duration.as_secs() < u64::from((u32::MAX - duration.subsec_micros()) / 1_000_000));

        let us = (duration.as_secs() as u32) * 1_000_000 + duration.subsec_micros();
        // Stop the timer to make sure the event doesn't occur while we're
        // setting things up (if start() is called more than once).
        self.0.tasks_stop.write(|w| unsafe { w.bits(1) });
        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.events_compare[0].reset();
        self.0.tasks_clear.write(|w| unsafe { w.bits(1) });
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    fn wait(&mut self) -> Result<(), Void> {
        if self.0.events_compare[0].read().bits() == 1 {
            self.0.events_compare[0].reset();
            Ok(())
        } else {
            Err(Error::WouldBlock)
        }
    }
}

impl Periodic for Timer {}
