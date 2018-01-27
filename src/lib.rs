#![no_std]
#![cfg_attr(feature="rt",feature(global_asm))]
#![cfg_attr(feature="rt",feature(macro_reexport))]
#![cfg_attr(feature="rt",feature(used))]
#![feature(const_fn)]
#![allow(non_camel_case_types)]
#![feature(never_type)]

extern crate cast;
extern crate cortex_m;
extern crate bare_metal;
extern crate embedded_hal as hal;
extern crate nb;
pub extern crate nrf51;

pub mod delay;
pub mod gpio;
pub mod prelude;
pub mod serial;
