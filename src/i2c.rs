use gpio::gpio::PIN;
use gpio::{Input, OpenDrain};
use nrf51::twi0::frequency;
use nrf51::TWI1;

use hal::blocking::i2c::{Write, WriteRead};

/// I2C abstraction
pub struct I2c<I2C> {
    i2c: I2C,
    sdapin: PIN<Input<OpenDrain>>,
    sclpin: PIN<Input<OpenDrain>>,
}

#[derive(Debug)]
pub enum Error {
    OVERRUN,
    NACK,
}

pub enum Frequency {
    K100,
    K250,
    K400,
}

impl Into<frequency::FREQUENCYW> for Frequency {
    fn into(self) -> frequency::FREQUENCYW {
        match self {
            Frequency::K100 => frequency::FREQUENCYW::K100,
            Frequency::K250 => frequency::FREQUENCYW::K250,
            Frequency::K400 => frequency::FREQUENCYW::K400,
        }
    }
}

impl I2c<TWI1> {
    pub fn i2c1(i2c: TWI1, sdapin: PIN<Input<OpenDrain>>, sclpin: PIN<Input<OpenDrain>>) -> Self {
        Self::i2c1_with_frequency(i2c, sdapin, sclpin, Frequency::K250)
    }

    pub fn i2c1_with_frequency(
        i2c: TWI1,
        sdapin: PIN<Input<OpenDrain>>,
        sclpin: PIN<Input<OpenDrain>>,
        frequency: Frequency,
    ) -> Self {
        /* Tell I2C controller which pins to use for sending and receiving */
        i2c.pselscl
            .write(|w| unsafe { w.bits(sclpin.get_id().into()) });
        i2c.pselsda
            .write(|w| unsafe { w.bits(sdapin.get_id().into()) });

        /* Set master clock frequency */
        i2c.frequency
            .write(|w| w.frequency().variant(frequency.into()));

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

    fn send_start(&self) -> Result<(), Error> {
        let twi = &self.i2c;

        /* Start data transmission */
        twi.tasks_starttx.write(|w| unsafe { w.bits(1) });
        Ok(())
    }

    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        let twi = &self.i2c;

        /* Clear sent event */
        twi.events_txdsent.write(|w| unsafe { w.bits(0) });

        /* Copy data into the send buffer */
        twi.txd.write(|w| unsafe { w.bits(u32::from(byte)) });

        /* Wait until transmission was confirmed */
        while twi.events_txdsent.read().bits() == 0 {
            /* Bail out if we get an error instead */
            if twi.events_error.read().bits() != 0 {
                twi.events_error.write(|w| unsafe { w.bits(0) });
                return Err(Error::NACK);
            }
        }

        /* Clear sent event */
        twi.events_txdsent.write(|w| unsafe { w.bits(0) });

        Ok(())
    }

    fn recv_byte(&self) -> Result<u8, Error> {
        let twi = &self.i2c;

        /* Clear reception event */
        twi.events_rxdready.write(|w| unsafe { w.bits(0) });

        /* Start data reception */
        twi.tasks_startrx.write(|w| unsafe { w.bits(1) });

        /* Wait until something ended up in the buffer */
        while twi.events_rxdready.read().bits() == 0 {
            /* Bail out if it's an error instead of data */
            if twi.events_error.read().bits() != 0 {
                twi.events_error.write(|w| unsafe { w.bits(0) });
                return Err(Error::OVERRUN);
            }
        }

        /* Read out data */
        let out = twi.rxd.read().bits() as u8;

        /* Clear reception event */
        twi.events_rxdready.write(|w| unsafe { w.bits(0) });

        Ok(out)
    }

    fn send_stop(&self) -> Result<(), Error> {
        let twi = &self.i2c;

        /* Clear stopped event */
        twi.events_stopped.write(|w| unsafe { w.bits(0) });

        /* Start stop condition */
        twi.tasks_stop.write(|w| unsafe { w.bits(1) });

        /* Wait until stop was sent */
        while twi.events_stopped.read().bits() == 0 {
            /* Bail out if we get an error instead */
            if twi.events_error.read().bits() != 0 {
                twi.events_error.write(|w| unsafe { w.bits(0) });
                return Err(Error::NACK);
            }
        }

        Ok(())
    }
}

impl WriteRead for I2c<TWI1> {
    type Error = Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        let twi = &self.i2c;

        /* Make sure all previously used shortcuts are disabled */
        twi.shorts
            .write(|w| w.bb_stop().disabled().bb_suspend().disabled());

        /* Request data */
        twi.address.write(|w| unsafe { w.address().bits(addr) });

        self.send_start()?;

        /* Send out all bytes in the outgoing buffer */
        for out in bytes {
            self.send_byte(*out)?;
        }

        /* Turn around to read data */
        if let Some((last, before)) = buffer.split_last_mut() {
            /* If we want to read multiple bytes we need to use the suspend mode */
            if !before.is_empty() {
                twi.shorts.write(|w| w.bb_suspend().enabled());
            }

            for in_ in &mut before.into_iter() {
                *in_ = self.recv_byte()?;

                twi.tasks_resume.write(|w| unsafe { w.bits(1) });
            }

            twi.shorts
                .write(|w| w.bb_suspend().disabled().bb_stop().enabled());

            *last = self.recv_byte()?;
        }

        self.send_stop()?;
        Ok(())
    }
}

impl Write for I2c<TWI1> {
    type Error = Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        let twi = &self.i2c;

        /* Make sure all previously used shortcuts are disabled */
        twi.shorts
            .write(|w| w.bb_stop().disabled().bb_suspend().disabled());

        /* Set Slave I2C address */
        twi.address.write(|w| unsafe { w.address().bits(addr) });

        /* Send start condition */
        self.send_start()?;

        /* Clock out all bytes */
        for in_ in bytes {
            self.send_byte(*in_)?;
        }

        /* Send stop */
        self.send_stop()?;
        Ok(())
    }
}
