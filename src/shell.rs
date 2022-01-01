use std::io::{stdin, stdout, Write};
use std::process;
use std::time::Duration;

pub struct Shell {
    input_buf: String,
    output_buf: String,
    port: Box<dyn serialport::SerialPort>,
}

impl Shell {
    pub fn init() -> Result<Self, &'static str> {
        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                eprintln!("Error reading ports: {}", e);
                process::exit(1);
            }
        };

        if ports.is_empty() {
            return Err("No serial devives found".into());
        }

        let port_name = Shell::user_select_port(ports);
        println!("Selected port: {}", &port_name);

        let port = match serialport::new(port_name, 9600)
            .timeout(Duration::from_millis(10))
            .open()
        {
            Ok(port) => port,
            Err(e) => {
                eprintln!("Could not connect: {}", e);
                process::exit(1);
            }
        };
        Ok(Self {
            input_buf: String::new(),
            output_buf: String::new(),
            port,
        })
    }
    //Should propably become a command
    fn user_select_port(port_list: Vec<serialport::SerialPortInfo>) -> String {
        for (i, p) in port_list.iter().enumerate() {
            println!("{}: {}", i + 1, p.port_name);
        }

        loop {
            let mut port_str = String::new();
            print!("Select a port: ");
            let _ = stdout().flush();

            match stdin().read_line(&mut port_str) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    process::exit(1);
                }
            };

            let port_index = match port_str.trim().parse::<usize>() {
                Ok(p) if p != 0 => p - 1,
                _ => {
                    println!("Enter a valid port");
                    continue;
                }
            };

            match port_list.get(port_index) {
                Some(port) => return port.port_name.clone(),
                None => {
                    println!("Select a port listed above");
                    continue;
                }
            }
        }
    }

    pub fn run_loop(&mut self) {
        loop {
            print!(">> ");
            let _ = stdout().flush();

            match stdin().read_line(&mut self.input_buf) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    process::exit(1);
                }
            };

            match self.port.write(&self.input_buf.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error writing: {}", e);
                    process::exit(1);
                }
            };
        }
    }
}
