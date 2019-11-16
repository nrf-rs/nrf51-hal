#![no_std]
#![allow(non_camel_case_types)]

pub use nrf51;

pub mod adc;
pub mod delay;
pub mod ecb;
pub mod gpio;
pub mod hi_res_timer;
pub mod i2c;
pub mod lo_res_timer;
pub mod prelude;
pub mod rng;
pub mod serial;
pub mod spi;
pub mod temp;
pub mod time;
pub mod timer;
