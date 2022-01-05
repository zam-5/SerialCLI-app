mod command;
mod communicator;
mod shell;

use command::Command;
use shell::Shell;
use std::process;

fn main() {
    let com_vec: Vec<Command> = vec![
        Command::new("exit", command::exit_shell),
        Command::new("write-digital", command::write_digital),
        Command::new("write-analog", command::write_analog),
        Command::new("read-digital", command::read_digital),
        Command::new("read-analog", command::read_analog),
        Command::new("lsdev", command::lsdev),
    ];

    let mut shell = match Shell::new(com_vec) {
        Ok(shell) => shell,
        Err(e) => {
            eprintln!("Shell could not be started: {}", e);
            process::exit(1);
        }
    };

    shell.run_loop();
}
