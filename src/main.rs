mod app;
mod command;
mod communicator;
mod shell;

use command::Command;
use shell::Shell;
use std::error::Error;
use std::io;
use std::process;
use std::time::Duration;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    // COMMANDS (EXIT) NEED CHANGED TO WORK WITH TUI
    let com_vec: Vec<Command> = vec![
        // Command::new("exit", command::exit_shell),
        Command::new("write-digital", command::write_digital),
        Command::new("write-analog", command::write_analog),
        Command::new("read-digital", command::read_digital),
        Command::new("read-analog", command::read_analog),
        Command::new("lsdev", command::lsdev),
    ];

    let shell = match Shell::new(com_vec) {
        Ok(shell) => shell,
        Err(e) => {
            eprintln!("Shell could not be started: {}", e);
            process::exit(1);
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(100);
    let app = app::App::new(shell);
    let res = app::run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    // shell.run_loop();
    Ok(())
}
