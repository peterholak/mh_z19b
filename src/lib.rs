#![cfg_attr(not(feature = "std"), no_std)]

pub mod mhz19b {
    use core::fmt;

    #[cfg(feature = "hal")]
    mod hal;

    #[cfg(feature = "hal")]
    use hal as platform;

    #[cfg(feature = "std")]
    mod std_io;

    #[cfg(feature = "std")]
    use std_io as platform;

    pub use platform::Error;

    /// Read a CO2 value (in ppm) from the sensor.
    pub fn read_co2(port: &mut ReadWrite) -> Result<u16> {
        port.mhz19b_write(&[0xFFu8, 0x01, 0x86, 0, 0, 0, 0, 0, 0x79])?;

        let mut response = [0u8; 9];
        port.mhz19b_read(&mut response)?;

        check_response(&response, &[0xFF, 0x86])?;

        let co2 = (response[2] as u16) * 256 + (response[3] as u16);
        Ok(co2)
    }

    /// Perform zero calibration. Only call this if the sensor is in a stable 400ppm environment
    /// for over 20 minutes.
    pub fn calibrate_zero(port: &mut ReadWrite) -> Result<()> {
        port.mhz19b_write(&[0xFFu8, 0x01, 0x87, 0, 0, 0, 0, 0, 0x78])?;
        Ok(())
    }

    /// Enable or disable automatic baseline correction, which causes the zero point (400ppm)
    /// to be automatically re-calibrated every 24 hours. See the data sheet for more info.
    pub fn set_auto_correction(port: &mut ReadWrite, on: bool) -> Result<()> {
        let mut request = [0xFFu8, 0x01, 0x79, (if on { 0xA0 } else { 0x00 }), 0, 0, 0, 0, 0];
        request[request.len() - 1] = checksum(&request);
        port.mhz19b_write(&request)?;

        let mut response = [0u8; 9];
        port.mhz19b_read(&mut response)?;

        check_response(&response, &[0xFF, 0x79, 0x01])?;

        Ok(())
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[cfg(feature = "serial")]
    pub use std_io::serial;

    fn checksum(packet: &[u8]) -> u8 {
        let middle = packet[1..8].iter()
            .fold(0u16, |acc, c| (acc + *c as u16) % 256);

        ((0xffu16 - middle + 1u16) % 256) as u8
    }

    fn checksum_ok(response: &[u8; 9]) -> bool {
        response[8] == checksum(response)
    }

    fn check_response(response: &[u8; 9], expected_start: &[u8]) -> Result<()> {
        if !response.starts_with(expected_start) {
            return Err(ErrorBase::InvalidResponse);
        }

        if !checksum_ok(&response) {
            return Err(ErrorBase::InvalidChecksum);
        }

        Ok(())
    }

    #[derive(Debug)]
    pub enum ErrorBase<IO> where IO: fmt::Display {
        InvalidResponse,
        InvalidChecksum,
        IoError(IO),
    }

    pub trait ReadWrite {
        fn mhz19b_write(&mut self, bytes: &[u8; 9]) -> Result<()>;
        fn mhz19b_read(&mut self, buffer: &mut [u8; 9]) -> Result<()>;
    }

    impl<T> fmt::Display for ErrorBase<T> where T: fmt::Display {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ErrorBase::InvalidResponse => write!(f, "Unexpected response (wrong prefix)."),
                ErrorBase::InvalidChecksum => write!(f, "Checksum verification failed."),
                ErrorBase::IoError(e) => write!(f, "{}", e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mhz19b::*;

    struct MockIo { read_buffer: [u8; 9] }

    impl MockIo {
        fn new() -> MockIo { MockIo { read_buffer: [0u8; 9] } }
        fn set_read_buffer(&mut self, bytes: &[u8]) {
            self.read_buffer.copy_from_slice(bytes)
        }
    }

    impl ReadWrite for MockIo {
        fn mhz19b_write(&mut self, _bytes: &[u8; 9]) -> Result<()> {
            Ok(())
        }

        fn mhz19b_read(&mut self, buffer: &mut [u8; 9]) -> Result<()> {
            buffer.copy_from_slice(&self.read_buffer);
            Ok(())
        }
    }

    #[test]
    fn parses_co2_value() {
        let mut port = MockIo::new();
        port.set_read_buffer(&[0xFF, 0x86, 0x01, 0x90, 0, 0, 0, 0, 0xE9]);
        let co2 = read_co2(&mut port);
        assert_eq!(Ok(400), co2);
    }

    #[test]
    fn checksum_doesnt_overflow() {
        let mut port = MockIo::new();
        // This makes the "middle" part of the checksum add up to zero.
        port.set_read_buffer(&[0xFF, 0x79, 0x01, 0x86, 0, 0, 0, 0, 0]);
        let result = set_auto_correction(&mut port, true);
        assert_eq!(Ok(()), result)
    }

    impl PartialEq<Error> for Error {
        fn eq(&self, other: &Error) -> bool {
            use crate::mhz19b::ErrorBase::*;
            match (self, other) {
                (InvalidChecksum, InvalidChecksum) => true,
                (InvalidResponse, InvalidResponse) => true,
                (IoError(_), IoError(_)) => true,
                _ => false
            }
        }
    }
}
