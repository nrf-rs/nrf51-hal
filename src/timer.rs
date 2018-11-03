//! Timer

pub use hal::timer::{CountDown, Periodic};
pub use timer_counter::{TimCo, TimerCounter, BitMode};
pub use time::{Hfticks, Lfticks, Micros, Millis, Hertz};

use nrf51::{TIMER0, TIMER1, TIMER2, RTC0, RTC1};
use void::Void;

pub struct Timer<TIM>(TimCo<TIM>);


macro_rules! Countdown_wait {
    ($idx:expr) => {
        fn wait(&mut self) -> nb::Result<(), Void> {

            // Check for comparison event
            if self.0.compare_event($idx) {

                // Reset comparison event
                self.0.reset_compare_event($idx);
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

            impl Timer<$TIM> {
                /// Construct TIMER based timer with prescaler
                pub fn new(timer: $TIM, prescaler: u8) -> Self {

                    Timer(TimCo::<$TIM>::new(timer, prescaler))
                }

                /// Set comparison bit width
                /// Set counter bit width = 32bit 24bit 16bit 8bit
                ///             bitmode   = 3     2     0     1
                pub fn set_bitmode(&mut self, bitmode: BitMode) {
                    self.0.set_bitmode(bitmode);
                }
            }

            impl CountDown for Timer<$TIM> {
                type Time = Hfticks;

                fn start<T>(&mut self, count: T)
                where
                    T: Into<Self::Time>,
                {

                    // Get comparison value
                    let compare: u32 = count
                        .into()
                        .checked_mul(self.0.frequency())
                        .expect("Timer count value error");

                    // Set periodic
                    self.0.set_compare_int_clear(0);

                    // Set compare event and start counter
                    self.0.set_compare_start(0, compare)
                        .expect("Timer comparison value error");
                }

                Countdown_wait!(0);
            }

            impl Periodic for Timer<$TIM> {}

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

            impl Timer<$RTC> {
                /// Construct RTC based timer with prescaler
                /// *WARNING* The LFCLK needs to be activated first, e.g.
                /// ```
                /// p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
                /// ```
                pub fn new(timer: $RTC, prescaler: u16) -> Self {

                    Timer(TimCo::<$RTC>::new(timer, prescaler))
                }
            }

            impl CountDown for Timer<$RTC> {
                type Time = Lfticks;

                fn start<T>(&mut self, count: T)
                where
                    T: Into<Self::Time>,
                {

                    // Get comparison value
                    let compare: u32 = count
                        .into()
                        .checked_mul(self.0.frequency())
                        .expect("Timer count value error");

                    // Set compare event and start counter
                    self.0.set_compare_start(0, compare)
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