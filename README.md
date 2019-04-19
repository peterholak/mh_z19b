MH-Z19B for Rust

This implements part of the protocol used to communicate
with the [MH-Z19B](https://www.winsen-sensor.com/d/files/infrared-gas-sensor/mh-z19b-co2-ver1_0.pdf) CO2 sensor.

Depending on the features selected, the serial communication is done either via
the [serialport](https://crates.io/crates/serialport) crate (requires std) or
via [embedded-hal](https://crates.io/crates/embedded-hal) (can be used with no_std).

See the example for details.
