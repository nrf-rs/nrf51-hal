#![no_std]
#![cfg_attr(feature = "rt", feature(global_asm))]
#![feature(const_fn)]
#![feature(try_from)]
#![feature(duration_as_u128)]
#![allow(non_camel_case_types)]

extern crate cast;
extern crate cortex_m;
pub extern crate embedded_hal as hal;
extern crate void;
#[macro_use(block)]
pub extern crate nb;
pub extern crate nrf51;

pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod prelude;
pub mod rng;
pub mod serial;
pub mod timer;
