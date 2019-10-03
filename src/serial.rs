use core::fmt::{Result, Write};
use core::marker::PhantomData;

use nb::block;

use crate::gpio::gpio::PIN;
use crate::gpio::{Floating, Input, Output, PushPull};
use nrf51::UART0;
use void::Void;

pub use nrf51::uart0::baudrate::BAUDRATE_A::{self, *};

/// Serial abstraction
pub struct Serial<UART> {
    uart: UART,
    txpin: PIN<Output<PushPull>>,
    rxpin: PIN<Input<Floating>>,
}

/// Serial receiver
pub struct Rx<UART> {
    _uart: PhantomData<UART>,
}

/// Serial transmitter
pub struct Tx<UART> {
    _uart: PhantomData<UART>,
}

#[derive(Debug)]
pub enum Error {}

impl Serial<UART0> {
    pub fn uart0(
        uart: UART0,
        txpin: PIN<Output<PushPull>>,
        rxpin: PIN<Input<Floating>>,
        speed: BAUDRATE_A,
    ) -> Self {
        // Fill register with dummy data to trigger txd event
        uart.txd.write(|w| unsafe { w.bits(0) });

        // Set output TXD and RXD pins
        uart.pseltxd
            .write(|w| unsafe { w.bits(txpin.get_id().into()) });
        uart.pselrxd
            .write(|w| unsafe { w.bits(rxpin.get_id().into()) });

        // Set baud rate
        uart.baudrate.write(|w| w.baudrate().variant(speed));

        // Enable UART function
        uart.enable.write(|w| w.enable().enabled());

        // Fire up transmitting and receiving task
        uart.tasks_starttx.write(|w| unsafe { w.bits(1) });
        uart.tasks_startrx.write(|w| unsafe { w.bits(1) });

        Serial { uart, txpin, rxpin }
    }

    pub fn release(self) -> (UART0, PIN<Output<PushPull>>, PIN<Input<Floating>>) {
        (self.uart, self.txpin, self.rxpin)
    }

    pub fn split(self) -> (Tx<UART0>, Rx<UART0>) {
        (Tx { _uart: PhantomData }, Rx { _uart: PhantomData })
    }
}

impl embedded_hal::serial::Read<u8> for Rx<UART0> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let uart = unsafe { &*UART0::ptr() };
        match uart.events_rxdrdy.read().bits() {
            0 => Err(nb::Error::WouldBlock),
            _ => {
                // Reset ready for receive event
                uart.events_rxdrdy.reset();

                // Read one 8bit value
                let byte = uart.rxd.read().bits() as u8;

                Ok(byte)
            }
        }
    }
}

impl embedded_hal::serial::Write<u8> for Tx<UART0> {
    type Error = Void;

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        let uart = unsafe { &*UART0::ptr() };
        // Are we ready for sending out next byte?
        if uart.events_txdrdy.read().bits() == 1 {
            // Reset ready for transmit event
            uart.events_txdrdy.reset();

            // Send byte
            uart.txd.write(|w| unsafe { w.bits(u32::from(byte)) });

            Ok(())
        } else {
            // We're not ready, tell application to try again
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<UART> Write for Tx<UART>
where
    Tx<UART>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> Result {
        use embedded_hal::serial::Write;
        let _ = s.as_bytes().iter().map(|c| block!(self.write(*c))).last();
        Ok(())
    }
}
