use crate::command::Command;
use crate::communicator::Communicator;

use std::io::{stdin, stdout, Write};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Shell {
    input_buf: String,
    output_buf: String,
    communicator: Arc<Mutex<Communicator>>,
    com_vec: Vec<Command>,
}

impl Shell {
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => return Err(format!("Error reading ports: {}", e).into()),
        };

        if ports.is_empty() {
            return Err("No serial devices found".into());
        }

        let port_name = Shell::user_select_port(ports);
        let communicator = match Communicator::new(port_name, 9600) {
            Ok(c) => Arc::new(Mutex::new(c)),
            Err(e) => {
                return Err(format!("Error connecting: {}", e).into());
            }
        };

        let mut com_vec: Vec<Command> = Vec::new();
        com_vec.push(Command::new("exit", Shell::exit_shell));
        com_vec.push(Command::new("write-digital", Shell::write_digital));
        com_vec.push(Command::new("write-analog", Shell::write_analog));
        com_vec.push(Command::new("read-digital", Shell::read_digital));
        com_vec.push(Command::new("read-analog", Shell::read_analog));
        com_vec.push(Command::new("lsdev", Shell::lsdev));
        com_vec.push(Command::new("chdev", Shell::chdev));

        Ok(Self {
            input_buf: String::new(),
            output_buf: String::new(),
            communicator,
            com_vec,
        })
    }

    fn welcome_msg(&self) {
        println!(
            "\nSerialCLI v{}\nConnected to: {}",
            env!("CARGO_PKG_VERSION"),
            self.communicator.lock().unwrap().get_name()
        );
    }

    pub fn run_loop(&mut self) {
        self.welcome_msg();

        let comm_clone = self.communicator.clone();

        thread::spawn(move || loop {
            if comm_clone.lock().unwrap().msg_available() {
                let mut comm = comm_clone.lock().unwrap();
                comm.wait_for_response();
                let output = match comm.get_output() {
                    Ok(str) => str,
                    Err(e) => {
                        eprintln!("Error reading serial port: {}", e);
                        process::exit(1);
                    }
                };
                print!("\r{}\n>> ", output);
                let _ = stdout().flush();
            } else {
                thread::sleep(Duration::from_millis(30));
            }
        });

        loop {
            if !self.output_buf.is_empty() {
                println!("{}", &self.output_buf.trim());
                self.output_buf.clear();
            }
            print!(">> ");
            let _ = stdout().flush();

            match stdin().read_line(&mut self.input_buf) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    process::exit(1);
                }
            };
            self.parse();
            self.input_buf.clear();
        }
    }
}

//Commands
//Most of these can be moved out of the shell struct as they don't ever use &self
impl Shell {
    fn parse(&mut self) {
        let line_vec: Vec<&str> = self.input_buf.split(" ").collect();
        let mut argv: Vec<String> = Vec::new();
        if line_vec.len() > 1 {
            line_vec[1..]
                .iter()
                .for_each(|str| argv.push(String::from(str.clone())))
        }
        for command in self.com_vec.iter() {
            if line_vec[0].trim() == command.name {
                let mut comm = self.communicator.lock().unwrap();
                command.exec(&argv, &mut comm);
                return;
            }
        }
        //If the command does not match a built in one, it will be writen by the communicator
        let mut comm = self.communicator.lock().unwrap();
        match comm.write(self.input_buf.trim().as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Command error: {}", e);
            }
        };
    }

    fn lsdev(_argv: &Vec<String>, _communicator: &mut Communicator) {
        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                eprintln!("Error reading ports: {}", e);
                return;
            }
        };
        println!("Serial Ports found:");
        for (i, p) in ports.iter().enumerate() {
            println!("{}: {}", i + 1, p.port_name);
        }
    }

    fn chdev(_argv: &Vec<String>, communicator: &mut Communicator) {
        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                eprintln!("Error reading ports: {}", e);
                return;
            }
        };
        println!("Serial Ports found:");
        for (i, p) in ports.iter().enumerate() {
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

            let addr = match ports.get(port_index) {
                Some(port) => port.port_name.clone(),
                None => {
                    println!("Select a port listed above");
                    continue;
                }
            };
            communicator.change_port(addr, 9600);
            break;
        }
    }

    fn user_select_port(port_list: Vec<serialport::SerialPortInfo>) -> String {
        println!("Serial Ports found:");
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

    fn exit_shell(_argv: &Vec<String>, _communicator: &mut Communicator) {
        println!("Exiting...");
        process::exit(0);
    }

    fn write_digital(argv: &Vec<String>, communicator: &mut Communicator) {
        let argstr: String = argv
            .iter()
            .map(|str| format!("{} ", str).to_string())
            .collect();

        match communicator.write(format!("3 {}", argstr.trim()).as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Command error: {}", e);
            }
        };
    }

    fn write_analog(argv: &Vec<String>, communicator: &mut Communicator) {
        let argstr: String = argv
            .iter()
            .map(|str| format!("{} ", str).to_string())
            .collect();
        match communicator.write(format!("2 {}", argstr.trim()).as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Command error: {}", e);
            }
        };
    }

    fn read_digital(argv: &Vec<String>, communicator: &mut Communicator) {
        let argstr: String = argv
            .iter()
            .map(|str| format!("{} ", str).to_string())
            .collect();
        match communicator.write(format!("1 {}", argstr.trim()).as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Command error: {}", e);
            }
        };
    }

    fn read_analog(argv: &Vec<String>, communicator: &mut Communicator) {
        let argstr: String = argv
            .iter()
            .map(|str| format!("{} ", str).to_string())
            .collect();
        match communicator.write(format!("0 {}", argstr.trim()).as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Command error: {}", e);
            }
        };
    }
}
