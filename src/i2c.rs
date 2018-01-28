use nrf51::TWI1;
use gpio::gpio::PIN;
use gpio::{Input, OpenDrain};

use hal::blocking::i2c::{Write, WriteRead};

/// I2C abstraction
pub struct I2c<I2C> {
    i2c: I2C,
    sdapin: PIN<Input<OpenDrain>>,
    sclpin: PIN<Input<OpenDrain>>,
}

#[derive(Debug)]
pub enum Error {}

impl I2c<TWI1> {
    pub fn i2c1(i2c: TWI1, sdapin: PIN<Input<OpenDrain>>, sclpin: PIN<Input<OpenDrain>>) -> Self {
        /* Tell I2C controller which pins to use for sending and receiving */
        i2c.pselscl
            .write(|w| unsafe { w.bits(sclpin.get_id().into()) });
        i2c.pselsda
            .write(|w| unsafe { w.bits(sdapin.get_id().into()) });

        /* Enable i2c function */
        i2c.enable.write(|w| w.enable().enabled());

        I2c {
            i2c,
            sdapin,
            sclpin,
        }
    }

    pub fn release(self) -> (TWI1, PIN<Input<OpenDrain>>, PIN<Input<OpenDrain>>) {
        (self.i2c, self.sdapin, self.sclpin)
    }
}

impl WriteRead for I2c<TWI1> {
    type Error = Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        let twi = &self.i2c;

        /* Request data */
        twi.address.write(|w| unsafe { w.address().bits(addr) });
        twi.tasks_starttx.write(|w| unsafe { w.bits(1) });

        for out in bytes {
            twi.txd.write(|w| unsafe { w.bits(*out as u32) });
            while twi.events_txdsent.read().bits() == 0 {}
            twi.events_txdsent.write(|w| unsafe { w.bits(0) });
        }

        /* Turn around to read data */
        twi.shorts.write(|w| w.bb_suspend().enabled());
        twi.tasks_startrx.write(|w| unsafe { w.bits(1) });

        if let Some((last, before)) = buffer.split_last_mut() {
            for in_ in &mut before.into_iter() {
                while twi.events_rxdready.read().bits() == 0 {}
                *in_ = twi.rxd.read().bits() as u8;
                twi.events_rxdready.write(|w| unsafe { w.bits(0) });
                twi.tasks_resume.write(|w| unsafe { w.bits(1) });
            }

            twi.shorts.write(|w| w.bb_stop().enabled());
            twi.tasks_resume.write(|w| unsafe { w.bits(1) });

            while twi.events_rxdready.read().bits() == 0 {}
            *last = twi.rxd.read().bits() as u8;
            twi.events_rxdready.write(|w| unsafe { w.bits(0) });
        }

        twi.tasks_stop.write(|w| unsafe { w.bits(1) });
        Ok(())
    }
}

impl Write for I2c<TWI1> {
    type Error = Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        let twi = &self.i2c;

        twi.address.write(|w| unsafe { w.address().bits(addr) });
        twi.tasks_starttx.write(|w| unsafe { w.bits(1) });

        for in_ in bytes.into_iter() {
            twi.txd.write(|w| unsafe { w.bits(*in_ as u32) });
            while twi.events_txdsent.read().bits() == 0 {}
            twi.events_txdsent.write(|w| unsafe { w.bits(0) });
        }

        twi.tasks_stop.write(|w| unsafe { w.bits(1) });

        Ok(())
    }
}
