use crate::mhz19b::*;

use embedded_hal::serial::*;

impl<R, W> ReadWrite for (R, W) where R: Read<u8>, W: Write<u8> {
    fn mhz19b_write(&mut self, bytes: &[u8; 9]) -> Result<()> {
        for &b in bytes {
            self.1.write(b).map_err(|_| ErrorBase::IoError(0))?;
        }
        self.1.flush().map_err(|_| ErrorBase::IoError(0))?;
        Ok(())
    }

    fn mhz19b_read(&mut self, buffer: &mut [u8; 9]) -> Result<()> {
        for byte in buffer {
            match self.0.read() {
                Ok(b) => *byte = b,
                Err(_) => return Err(ErrorBase::IoError(0))
            }
        }
        Ok(())
    }
}

pub type Error = ErrorBase<u8>;
