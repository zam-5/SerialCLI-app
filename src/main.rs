use std::io::{stdin, stdout, Write};
// use std::io::BufRead;
use std::process;
use std::time::Duration;

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

fn main() {
    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(e) => {
            eprintln!("No serial ports found: {}", e);
            process::exit(1);
        }
    };
    let port_name = user_select_port(ports);
    println!("Selected port: {}", &port_name);

    let mut port = match serialport::new(port_name, 9600)
        .timeout(Duration::from_millis(10))
        .open()
    {
        Ok(port) => port,
        Err(e) => {
            eprintln!("Could not connect: {}", e);
            process::exit(1);
        }
    };
    loop {
        let mut com = String::new();
        print!("> ");
        let _ = stdout().flush();

        match stdin().read_line(&mut com) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                process::exit(1);
            }
        };

        match port.write(&com.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error writing: {}", e);
                process::exit(1);
            }
        };
    }
}
