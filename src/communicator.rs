use std::{error::Error, time::Duration};

pub struct Communicator {
    port: Box<dyn serialport::SerialPort>,
}

impl Communicator {
    pub fn new(addr: String, baudrate: u32) -> Result<Self, Box<dyn Error>> {
        let port = match serialport::new(addr, baudrate)
            .timeout(Duration::from_millis(10))
            .open()
        {
            Ok(port) => port,
            Err(e) => return Err(e.into()),
        };

        Ok(Self { port })
    }

    pub fn change_port(&mut self, addr: String, baudrate: u32) {
        let port = match serialport::new(addr, baudrate)
            .timeout(Duration::from_millis(10))
            .open()
        {
            Ok(port) => port,
            Err(e) => {
                eprintln!("Could not change port: {}", e);
                return;
            }
        };
        self.port = port;
    }

    pub fn msg_available(&self) -> bool {
        if self.port.bytes_to_read().unwrap() != 0 {
            return true;
        }
        false
    }

    fn bytes_avail(&self) -> u32 {
        match self.port.bytes_to_read() {
            Ok(n) => n,
            Err(e) => panic!("Error reading port: {}", e),
        }
    }

    pub fn wait_for_response(&self) {
        let mut ba = self.bytes_avail();
        loop {
            if self.msg_available() {
                let cba = self.bytes_avail();
                if ba < cba {
                    ba = cba;
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                break;
            }
        }
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Box<dyn Error>> {
        match self.port.write(buffer) {
            Ok(n) => Ok(n),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_output(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buf = [0 as u8; 1000];
        let bytes_read = match self.port.read(&mut buf) {
            Ok(br) => br,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => 0,
            Err(e) => return Err(e.into()),
        };

        if bytes_read == 0 {
            return Ok(String::new());
        }

        match String::from_utf8(buf[..bytes_read].to_vec()) {
            Ok(str) => Ok(str),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_name(&self) -> String {
        self.port.name().unwrap()
    }
}
