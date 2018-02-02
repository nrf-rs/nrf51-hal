nrf51-hal
=========

_nrf51-hal_ contains the (slight) hardware abstraction on top of the peripheral
access API for the Nordic Semiconductor NRF51 series microcontroller.

This crate relies on my [nrf51][] crate to provide appropriate register
definitions and implements a partial set of the [embedded-hal][] traits.

This implementation was developped and tested against the fabolous
[BBC micro:bit][] board for which also a [microbit crate][] is
available.

[nrf51]: https://github.com/therealprof/nrf51.git
[embedded-hal]: https://github.com/japaric/embedded-hal.git
[BBC micro:bit]: https://microbit.org
[microbit crate]: https://github.com/therealprof/microbit.git

License
-------

[0-clause BSD license](LICENSE-0BSD.txt).
