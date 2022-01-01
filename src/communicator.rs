use std::time::Duration;

pub struct Communicator {
    port: Box<dyn serialport::SerialPort>,
}

impl Communicator {
    pub fn new(addr: String, baudrate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let port = match serialport::new(addr, baudrate)
            .timeout(Duration::from_millis(10))
            .open()
        {
            Ok(port) => port,
            Err(e) => return Err(e.into()),
        };

        Ok(Self { port })
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        match self.port.write(buffer) {
            Ok(n) => Ok(n),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_output(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = [0 as u8; 1000];
        let bytes_read = match self.port.read(&mut buf) {
            Ok(br) => br,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => 0,
            Err(e) => return Err(e.into()),
        };

        if bytes_read == 0 {
            return Ok(String::new())
        }

        match String::from_utf8(buf[..bytes_read].to_vec()) {
            Ok(str) => Ok(str),
            Err(e) => Err(e.into()),
        }
    }
}
