/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

extern crate num;

enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq)]
enum TargetMode {
    Daily,
    LongTerm
}

#[derive(Copy, Clone)]
enum Status {
    TODO,
    REJECTED,
    DONE
}

#[derive(Clone)]
struct TodoData {
    message: String,
    status: Status
}

impl Default for TodoData {
    fn default() -> TodoData {
        TodoData {
            message: String::new(),
            status: Status::TODO
        }
    }
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// Wether to write to daily or long term todo's
    target_mode: TargetMode,
    /// Daily todo's
    messages: Vec<TodoData>,
    /// Long term todo's
    long_term_todo : Vec<TodoData>,
    target_row : u32,
    target_column : u32
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            target_mode: TargetMode::Daily,
            messages: vec![TodoData::default()],
            long_term_todo : vec![TodoData::default()],
            target_row : 0,
            target_column : 0
        }
    }
}

impl App {
    fn get_messages(&self) -> Vec<TodoData> {
        match self.target_mode {
            TargetMode::Daily => {
                self.messages.clone()
            },
            TargetMode::LongTerm => {
                self.long_term_todo.clone()
            }
        }
    }

    fn get_messages_mut(&mut self) -> &mut Vec<TodoData> {
        match self.target_mode {
            TargetMode::Daily => {
                &mut self.messages
            },
            TargetMode::LongTerm => {
                &mut self.long_term_todo
            }
        }
    }

    fn push_message(&mut self, new_entry : TodoData) {
        self.input = String::new();
        self.get_messages_mut().push(new_entry);
    }

    fn add_char(&mut self, new_char : char) 
    {
        self.input.push(new_char);
        let target_index = self.target_row;
        self.get_messages_mut()[target_index as usize].message = self.input.clone();
    }

    fn remove_char(&mut self)
    {
        self.input.pop();
        let target_index = self.target_row;
        self.get_messages_mut()[target_index as usize].message = self.input.clone();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
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

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;


        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('t') => {
                        // TODO[rsmekens]: 
                        if app.target_mode == TargetMode::Daily {
                            app.target_mode = TargetMode::LongTerm;
                            app.target_row = 0;
                        }
                        else {
                            app.target_mode = TargetMode::Daily;
                            app.target_row = 0;
                        }
                    }
                    KeyCode::Up => {
                        // Requires casting to i32 here to not get substract overflow because of  u32
                        app.target_row = num::clamp(app.target_row as i32 - 1, 0, app.get_messages().len() as i32 - 1) as u32;
                        println!("{}", app.target_row);
                    }
                    KeyCode::Down => {
                        // Requires casting to i32 here to not get substract overflow because of  u32
                        app.target_row = num::clamp(app.target_row as i32 + 1, 0, app.get_messages().len() as i32 - 1) as u32;
                    }
                    _ => {}
                },

                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        // 1. Push new line to todo queue as we've finished writing current one
                        let new_entry = TodoData { 
                            message : String::new(),
                            status : Status::TODO
                        };
                        app.push_message(new_entry);

                        // 2. Clear input
                        app.input = String::new();

                        // 3. Move to next line
                        app.target_row += 1;

                        // 4. Exit editing mode
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        app.add_char(c);
                    }
                    KeyCode::Backspace => {
                        app.remove_char();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn get_title(app : &App) -> String {
        match app.target_mode {
        TargetMode::Daily => {
            return "Daily".to_string(); 
        },
        TargetMode::LongTerm => {
            return "Long Term".to_string();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
                Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to change todo type")
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let messages_to_display : Vec<String> =app.get_messages() 
        .iter()
        .enumerate()
        .map(|(i, m)| {
            m.message.clone()
        })
        .collect();
    let mut title = get_title(&app);


    let messages: Vec<ListItem> = messages_to_display
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
            ListItem::new(content)
        })
        .collect();

    let messages =
        List::new(messages).block(
            Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(match app.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            }));
    f.render_widget(messages, chunks[2]);

    let x_offset = 4;
    let y_offset = 1 + app.target_row as u16;

    f.set_cursor(
        chunks[2].x + x_offset + app.input.width() as u16,
        chunks[2].y + y_offset 
    )
}