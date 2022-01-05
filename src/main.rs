mod command;
mod communicator;
mod shell;

use command::Command;
use shell::Shell;
use std::error::Error;
use std::io;
use std::process;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// History of recorded messages
    messages: Vec<String>,
    /// History of inputs
    input_history: Vec<String>,
    // This struct will also hold the shell, which it will get commands from
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            messages: Vec::new(),
            input_history: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

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

    shell.run_loop();
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Added handling for arrow keys to scroll through input history
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    // Here the input will be sent to the shell, then added to input_history
                    app.messages.push(app.input.drain(..).collect());
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Esc => return Ok(()),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.size());

    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);

    f.set_cursor(
        // Put cursor past the end of the input text
        chunks[1].x + app.input.width() as u16 + 1,
        // Move one line down, from the border to the input line
        chunks[1].y + 1,
    );

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| {
            let content = vec![Spans::from(Span::raw(format!("{}", m)))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[0]);
}
