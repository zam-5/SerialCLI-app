mod command;
mod communicator;
mod shell;

use shell::Shell;
use std::process;

fn main() {
    let mut shell = match Shell::init() {
        Ok(shell) => shell,
        Err(e) => {
            eprintln!("Shell could not be started: {}", e);
            process::exit(1);
        }
    };

    shell.run_loop();
}
