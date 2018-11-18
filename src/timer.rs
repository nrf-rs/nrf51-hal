//! Timer

pub use timer_counter::{Timer, TimerCounter, Generic};
pub use timer_counter::{BitMode, Countdown};
pub use time::{Hfticks, Lfticks, Micros, Millis, Hertz};

use hal::timer;
use nrf51::{TIMER0, TIMER1, TIMER2, RTC0, RTC1};
use void::Void;


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

macro_rules! timers {
    ([
        $($TIM:ty: $bitmode:expr,)+
    ]) => {
        $(

            impl timer::CountDown for Timer<Countdown, $TIM> {
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

            impl timer::Periodic for Timer<Countdown, $TIM> {}

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

            impl timer::CountDown for Timer<Countdown, $RTC> {
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