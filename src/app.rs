use crate::shell::Shell;

use std::io;
use std::sync::{Arc, Mutex};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

pub struct App {
    /// Current value of the input box
    input: String,
    output_vec: Arc<Mutex<Vec<String>>>,
    /// History of inputs
    input_history: Vec<String>,
    // This struct will also hold the shell, which it will get commands from
    shell: Shell,
}

impl App {
    pub fn new(shell: Shell) -> App {
        App {
            input: String::new(),
            output_vec: Arc::new(Mutex::new(Vec::new())),
            input_history: Vec::new(),
            shell,
        }
    }

    fn add_to_history(&mut self, h: String) {
        self.input_history.push(h);
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    app.shell.spawn_listener(app.output_vec.clone());

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Added handling for arrow keys to scroll through input history
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    // Here the input will be sent to the shell, then added to input_history
                    app.add_to_history(app.input.clone());
                    app.shell.parse_external(app.input.drain(..).collect());
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

    // This (probably) should not be done this way, change after testing initial linkup
    // Maybe look for differnces and only change this if need?
    let text: Vec<Spans> = app
        .output_vec
        .lock()
        .unwrap()
        .iter()
        .map(|line| Spans::from(Span::raw(line.clone())))
        .collect();

    let paragraph =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(paragraph, chunks[0]);
}
