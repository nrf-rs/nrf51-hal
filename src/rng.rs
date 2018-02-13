use nrf51::RNG;

/// Blocking read
pub trait Read {
    /// Error type
    type Error;

    /// Reads enough bytes from hardware random number generator to fill `buffer`
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum Error {}

/// System random number generator `RNG` as a random number provider
pub struct Rng {
    rng: RNG,
}

impl Rng {
    pub fn new(rng: RNG) -> Self {
        /* Enable error correction for better values */
        rng.config.write(|w| w.dercen().enabled());

        /* Enable random number generation */
        rng.tasks_start.write(|w| unsafe { w.bits(1) });

        Rng { rng }
    }

    pub fn free(self) -> RNG {
        self.rng
    }
}

impl Read for Rng {
    type Error = Error;

    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        for in_ in &mut buffer.into_iter() {
            /* Let's wait until we have a new random value */
            while self.rng.events_valrdy.read().bits() == 0 {}

            /* Write fetched random number into provided buffer */
            *in_ = self.rng.value.read().bits() as u8;

            /* Clear event for next random number value */
            self.rng.events_valrdy.write(|w| unsafe { w.bits(0) });
        }

        Ok(())
    }
}
