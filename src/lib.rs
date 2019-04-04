#![no_std]
#![allow(non_camel_case_types)]

extern crate cast;
extern crate cortex_m;
extern crate fpa;
pub extern crate embedded_hal as hal;
extern crate void;
#[macro_use(block)]
pub extern crate nb;
pub extern crate nrf51;

pub mod delay;
pub mod ecb;
pub mod gpio;
pub mod hi_res_timer;
pub mod i2c;
pub mod prelude;
pub mod rng;
pub mod serial;
pub mod timer;
pub mod spi;
pub mod temp;
