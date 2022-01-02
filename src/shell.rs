use crate::command::Command;
use crate::communicator::Communicator;
use std::io::{stdin, stdout, Write};
use std::process;
// use std::time::Duration;

pub struct Shell {
    input_buf: String,
    output_buf: String,
    communicator: Communicator,
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
            Ok(c) => c,
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
            self.communicator.get_name()
        );
    }

    pub fn run_loop(&mut self) {
        self.welcome_msg();

        let Self {
            input_buf,
            output_buf,
            communicator,
            com_vec,
        } = self;

        loop {
            if communicator.msg_available() {
                *output_buf = match communicator.get_output() {
                    Ok(str) => str,
                    Err(e) => {
                        eprintln!("Error getting output: {}", e);
                        String::new()
                    }
                };
            }

            if !output_buf.is_empty() {
                println!("{}", output_buf.trim());
                output_buf.clear();
            }
            print!(">> ");
            let _ = stdout().flush();

            match stdin().read_line(input_buf) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    process::exit(1);
                }
            };
            Shell::parse(input_buf, com_vec, communicator);
            input_buf.clear();
            // std::thread::sleep(Duration::from_millis(50));
        }
    }
}

//Commands
impl Shell {
    fn parse(line: &String, com_vec: &Vec<Command>, communicator: &mut Communicator) {
        // let now = Instant::now();
        let line_vec: Vec<&str> = line.split(" ").collect();
        let mut argv: Vec<String> = Vec::new();
        if line_vec.len() > 1 {
            line_vec[1..]
                .iter()
                .for_each(|str| argv.push(String::from(str.clone())))
        }
        for command in com_vec.iter() {
            if line_vec[0].trim() == command.name {
                // println!("Time to parse: {}", now.elapsed().as_micros());
                command.exec(&argv, communicator);
                return;
            }
        }
        //If the command does not match a built in one, it will be writen by the communicator
        match communicator.write(line.trim().as_bytes()) {
            Ok(_) => communicator.wait_for_response(),
            Err(e) => {
                eprintln!("Command error: {}", e);
            }
        };
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
        // let now = Instant::now();
        let argstr: String = argv
            .iter()
            .map(|str| format!("{} ", str).to_string())
            .collect();
        // println!("Time to parse args: {}", now.elapsed().as_micros());
        // let now2 = Instant::now();
        match communicator.write(format!("3 {}", argstr.trim()).as_bytes()) {
            Ok(_) => {
                // println!("Time to write args: {}", now2.elapsed().as_micros());
                ()
            }
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
        communicator.wait_for_response();
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
        communicator.wait_for_response();
    }
}
