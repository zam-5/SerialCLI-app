use crate::command::{self, Command};
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

        let port_name = command::user_select_port(ports);
        let communicator = match Communicator::new(port_name, 9600) {
            Ok(c) => Arc::new(Mutex::new(c)),
            Err(e) => {
                return Err(format!("Error connecting: {}", e).into());
            }
        };

        let mut com_vec: Vec<Command> = Vec::new();
        com_vec.push(Command::new("exit", command::exit_shell));
        com_vec.push(Command::new("write-digital", command::write_digital));
        com_vec.push(Command::new("write-analog", command::write_analog));
        com_vec.push(Command::new("read-digital", command::read_digital));
        com_vec.push(Command::new("read-analog", command::read_analog));
        com_vec.push(Command::new("lsdev", command::lsdev));
        com_vec.push(Command::new("mon-analog", command::monitor_analog));

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

    fn parse(&mut self) {
        let line_vec: Vec<&str> = self.input_buf.split(" ").collect();
        let mut argv: Vec<String> = Vec::new();
        if line_vec.len() > 1 {
            line_vec[1..]
                .iter()
                .for_each(|str| argv.push(String::from(str.clone())))
        }

        let comm_clone = self.communicator.clone();

        for command in self.com_vec.iter() {
            if line_vec[0].trim() == command.name {
                // let mut comm = comm_clone.lock().unwrap();
                let command_copy = (*command).clone();
                thread::spawn(move || {
                    command_copy.exec(&argv, comm_clone);
                });

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
                print!("\r{}  \n>> ", output);
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
