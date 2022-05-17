use crate::shell::Shell;

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};

use tui::widgets::{List, ListItem};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

pub struct App<'a> {
    /// Current value of the input box
    input: String,
    output_vec: Arc<Mutex<Vec<String>>>,
    pub output_list: Vec<ListItem<'a>>,
    /// History of inputs
    input_history: Vec<String>,
    // This struct will also hold the shell, which it will get commands from
    shell: Shell,
}

impl<'a> App<'a> {
    pub fn new(shell: Shell) -> App<'a> {
        App {
            input: String::new(),
            output_vec: Arc::new(Mutex::new(Vec::new())),
            output_list: Vec::new(),
            input_history: Vec::new(),
            shell,
        }
    }

    fn on_tick(&mut self) {
        let ov = self.output_vec.lock().unwrap();
        if ov.len() > self.output_list.len() {
            self.output_list
                .push(ListItem::new(ov.last().unwrap().clone()))
        }
    }

    fn add_output(&mut self, output: String) {
        self.output_vec.lock().unwrap().push(output);
    }

    fn add_to_history(&mut self, h: String) {
        self.input_history.push(h);
    }
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    app.shell.spawn_listener(app.output_vec.clone());
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Added handling for arrow keys to scroll through input history
        if crossterm::event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        // Here the input will be sent to the shell, then added to input_history
                        app.add_to_history(app.input.clone());
                        app.add_output(app.input.clone());
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

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
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
    // Or hold this as a list in the app state, then just add new items to the list
    // let text: Vec<Spans> = app
    //     .output_vec
    //     .lock()
    //     .unwrap()
    //     .iter()
    //     .map(|line| Spans::from(Span::raw(line.clone())))
    //     .collect();

    // let paragraph =
    //     Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Messages"));
    // let items: Vec<ListItem> = app
    //     .output_vec
    //     .lock()
    //     .unwrap()
    //     .iter()
    //     .map(|line| ListItem::new(line.clone()))
    //     .collect();
    let list = List::new(app.output_list.clone())
        .block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(list, chunks[0]);
}
