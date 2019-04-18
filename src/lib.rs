pub mod mhz19b {
    use std::{convert, fmt, io};

    pub trait ReadWrite: io::Read + io::Write {}

    impl<T> ReadWrite for T where T: io::Read + io::Write {}

    /// Read a CO2 value (in ppm) from the sensor.
    pub fn read_co2(port: &mut ReadWrite) -> Result<u16> {
        port.write_all(&[0xFFu8, 0x01, 0x86, 0, 0, 0, 0, 0, 0x79])?;

        let mut response = [0u8; 9];
        port.read_exact(&mut response)?;

        check_response(&response, &[0xFF, 0x86])?;

        let co2 = (response[2] as u16) * 256 + (response[3] as u16);
        Ok(co2)
    }

    /// Perform zero calibration. Only call this if the sensor is in a stable 400ppm environment
    /// for over 20 minutes.
    pub fn calibrate_zero(port: &mut ReadWrite) -> Result<()> {
        port.write_all(&[0xFFu8, 0x01, 0x87, 0, 0, 0, 0, 0, 0x78])?;
        Ok(())
    }

    /// Enable or disable automatic baseline correction, which causes the zero point (400ppm)
    /// to be automatically re-calibrated every 24 hours. See the data sheet for more info.
    pub fn set_auto_correction(port: &mut ReadWrite, on: bool) -> Result<()> {
        let mut request = [0xFFu8, 0x01, 0x79, (if on { 0xA0 } else { 0x00 }), 0, 0, 0, 0, 0];
        request[request.len() - 1] = checksum(&request);
        port.write_all(&request)?;

        let mut response = [0u8; 9];
        port.read_exact(&mut response)?;

        check_response(&response, &[0xFF, 0x79, 0x01])?;

        Ok(())
    }

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
            return Err(Error::InvalidResponse);
        }

        if !checksum_ok(&response) {
            return Err(Error::InvalidChecksum);
        }

        Ok(())
    }

    #[derive(Debug)]
    pub enum Error {
        InvalidResponse,
        InvalidChecksum,
        IoError(std::io::Error)
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use std::error::Error as StdError;
            let description = match self {
                Error::InvalidResponse => "Unexpected response (wrong prefix).",
                Error::InvalidChecksum => "Checksum verification failed.",
                Error::IoError(e) => e.description()
            };
            write!(f, "{}", description)
        }
    }

    impl convert::From<io::Error> for Error {
        fn from(e: io::Error) -> Self { Error::IoError(e) }
    }

    pub type Result<T> = std::result::Result<T, Error>;

    #[cfg(feature = "serialport")]
    pub mod serial {
        use std::ffi::OsStr;
        use std::time::Duration;
        use serialport::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
        use serialport::SerialPort;

        pub fn connect<T: AsRef<OsStr> + ?Sized>(port: &T, timeout: Duration) -> serialport::Result<Box<SerialPort>> {
            let settings = SerialPortSettings {
                baud_rate: 9600,
                data_bits: DataBits::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                flow_control: FlowControl::None,
                timeout,
            };

            serialport::open_with_settings(port, &settings)
        }
    }
}

#[cfg(test)]
mod tests {
    use mockstream::MockStream;

    use crate::mhz19b::*;

    #[test]
    fn parses_co2_value() {
        let mut port = MockStream::new();
        port.push_bytes_to_read(&[0xFF, 0x86, 0x01, 0x90, 0, 0, 0, 0, 0xE9]);
        let co2 = read_co2(&mut port);
        assert_eq!(Ok(400), co2);
    }

    #[test]
    fn checksum_doesnt_overflow() {
        let mut port = MockStream::new();
        // This makes the "middle" part of the checksum add up to zero.
        port.push_bytes_to_read(&[0xFF, 0x79, 0x01, 0x86, 0, 0, 0, 0, 0]);
        let result = set_auto_correction(&mut port, true);
        assert_eq!(Ok(()), result)
    }

    impl PartialEq<Error> for Error {
        fn eq(&self, other: &Error) -> bool {
            use crate::mhz19b::Error::*;
            match (self, other) {
                (InvalidChecksum, InvalidChecksum) => true,
                (InvalidResponse, InvalidResponse) => true,
                (IoError(e), IoError(e2)) => e.kind() == e2.kind(),
                _ => false
            }
        }
    }
}
