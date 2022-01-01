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
}
