use std::convert;

use crate::mhz19b::*;

impl<T> ReadWrite for T where T: std::io::Read + std::io::Write {
    fn mhz19b_write(&mut self, bytes: &[u8; 9]) -> Result<()> {
        self.write_all(bytes)?;
        self.flush()?;
        Ok(())
    }

    fn mhz19b_read(&mut self, buffer: &mut [u8; 9]) -> Result<()> {
        self.read_exact(buffer)?;
        Ok(())
    }
}

pub type Error = ErrorBase<std::io::Error>;

impl convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { ErrorBase::IoError(e) }
}

#[cfg(feature = "serial")]
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
