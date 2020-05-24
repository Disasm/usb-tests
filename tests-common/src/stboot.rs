use std::io::{self, Error, ErrorKind};
use serialport::{SerialPort, ClearBuffer};
use std::time::Duration;
use std::convert::TryInto;

const ACK: u8 = 0x79;
const NACK: u8 = 0x1F;


pub struct Bootloader {
    serial: Box<dyn SerialPort>,
    supported_commands: Vec<u8>,
}

impl Bootloader {
    pub fn new(serial: Box<dyn SerialPort>) -> Self {
        Self {
            serial,
            supported_commands: vec![0x00]
        }
    }

    fn check_suppored_command(&self, command: u8) -> io::Result<()> {
        if self.supported_commands.iter().find(|v| **v == command).is_some() {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!("Unsupported command: 0x{:02x}", command)
            ))
        }
    }

    fn recv_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.serial.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn check_result(&mut self) -> io::Result<()> {
        self.serial.set_timeout(Duration::from_secs(1))?;

        let byte = self.recv_byte()?;
        match byte {
            ACK => Ok(()),
            NACK => Err(Error::new(ErrorKind::InvalidData,
                                   "Unexpected NAK")),
            v => Err(Error::new(ErrorKind::InvalidData,
                                format!("Unexpected result code: {:02x}", v))),
        }
    }

    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.serial.write_all(data)?;
        self.check_result()?;
        Ok(())
    }

    pub fn init(&mut self) -> io::Result<()> {
        self.serial.clear(ClearBuffer::All)?;
        self.serial.set_timeout(Duration::from_millis(10))?;
        while self.recv_byte().is_ok() {}

        self.serial.set_timeout(Duration::from_millis(100))?;
        self.serial.write_all(&[0x7f])?;
        match self.recv_byte() {
            Ok(ACK) => return Ok(()),
            Ok(NACK) => return Ok(()), // the interface wasn't closed properly, but still ok
            Ok(b) => return Err(Error::new(ErrorKind::InvalidData,
                                    format!("Unexpected result code: {:02x}", b))),
            Err(e) if e.kind() != ErrorKind::TimedOut => return Err(e),
            Err(_) => {} // timeout, continue
        }

        // Send a new byte, we should get back a NACK.
        self.serial.write_all(&[0x7f])?;
        if self.recv_byte()? == NACK {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Failed to init device"
            ))
        }
    }

    pub fn cmd_get(&mut self) -> io::Result<()> {
        self.send(&[0x00, 0xff])?;
        let len = self.recv_byte()?;

        let mut buf = vec![0; len as usize + 1];
        self.serial.read_exact(&mut buf)?;
        self.check_result()?;

        self.supported_commands.clear();
        self.supported_commands.extend_from_slice(&buf[1..]);

        Ok(())
    }

    pub fn get_device_id(&mut self) -> io::Result<u16> {
        self.check_suppored_command(0x02)?;
        self.send(&[0x02, 0xfd])?;
        let len = self.recv_byte()?;

        let mut buf = vec![0; len as usize + 1];
        self.serial.read_exact(&mut buf)?;
        self.check_result()?;

        if buf.len() == 2 {
            let id = u16::from_be_bytes((&buf as &[u8]).try_into().unwrap());
            Ok(id)
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                "Unsupported response length"
            ))
        }
    }
}
