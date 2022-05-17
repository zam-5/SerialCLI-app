use crate::communicator::Communicator;
use std::io::{stdin, stdout, Write};
use std::process;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Command {
    pub name: String,
    exec_func: fn(&Vec<String>, &mut Communicator),
}

impl Command {
    pub fn new(name: &str, exec_func: fn(&Vec<String>, &mut Communicator)) -> Self {
        Self {
            name: name.to_string(),
            exec_func,
        }
    }

    pub fn exec(&self, argv: &Vec<String>, communicator: Arc<Mutex<Communicator>>) {
        let mut comm_lock = communicator.lock().unwrap();

        (self.exec_func)(argv, &mut comm_lock);
    }
}

//Commands Start Here

pub fn lsdev(_argv: &Vec<String>, _communicator: &mut Communicator) {
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

pub fn user_select_port(port_list: Vec<serialport::SerialPortInfo>) -> String {
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

pub fn exit_shell(_argv: &Vec<String>, _communicator: &mut Communicator) {
    println!("Exiting...");
    process::exit(0);
}

pub fn write_digital(argv: &Vec<String>, communicator: &mut Communicator) {
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

pub fn write_analog(argv: &Vec<String>, communicator: &mut Communicator) {
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

pub fn read_digital(argv: &Vec<String>, communicator: &mut Communicator) {
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

pub fn read_analog(argv: &Vec<String>, communicator: &mut Communicator) {
    let argstr: String = argv
        .iter()
        .map(|str| format!("{} ", str).to_string())
        .collect();
    match communicator.write(format!("0 {}", argstr.trim()).as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Command write error: {}", e);
        }
    };
}
