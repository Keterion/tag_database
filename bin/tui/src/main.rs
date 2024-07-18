use std::io::{self, stdout};
use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, EnableMouseCapture, DisableMouseCapture},
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};
use tui_input::{
    backend::crossterm::EventHandler,
    Input,
};
use rusqlite::Connection;

enum InputMode {
    View,
    Insert,
}

/// Holds the application state
struct App {
    /// Current value of the search box
    search: Input,
    /// Current input mode
    input_mode: InputMode,
    text: Vec<String>,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    
    let app = App {
        search: Input::default(),
        input_mode: InputMode::View,
        text: vec![],
    };
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, mut app: App) -> io::Result<()> {
    let conn = Connection::open("base.db").unwrap();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::View => match key.code {
                    KeyCode::Char('i') => {
                        app.input_mode = InputMode::Insert;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    },
                    _ => {}
                },
                InputMode::Insert => match key.code {
                    KeyCode::Enter => {
                        app.search.reset();
                    },
                    KeyCode::Tab => {
                        //let res = db::tags::get_tags_with(app.search.value(), &conn).iter().fold(String::new(), |acc, tag| {
                        //    acc + &tag.1 + "\n"
                        //});
                        app.search.handle(tui_input::InputRequest::DeletePrevWord);
                        
                        for c in app.text.first().unwrap_or(&"".to_owned()).chars() {
                            app.search.handle(tui_input::InputRequest::InsertChar(c));
                        }
                        // replace incomplete search query term thing with the completion
                    },
                    KeyCode::Esc => {
                        app.input_mode = InputMode::View;
                    },
                    _ => {
                        app.search.handle_event(&Event::Key(key));
                        // you could do a cool loading thing and wait for a thread to finish the
                        // sql query
                        let last_word = app.search.value().split(' ').last().unwrap_or("");
                        app.text = db::tags::get_tags_with(last_word, &conn).iter().map(|tag| tag.1.clone()).collect();
                    }
                },
            }
        }
    }
}
fn ui(f: &mut Frame, app: &App) {
    use std::cell::OnceCell;
    let c = OnceCell::new();
    let chunks = c.get_or_init(|| {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(3),
            ].as_ref())
            .split(f.size())
    });
    let (msg, style) = match app.input_mode {
        InputMode::View => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to insert into the query."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Insert => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to enter view, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to send the query"),
            ],
            Style::default(),
        ),
    };
    let help_msg = Paragraph::new(
        Text::from(Line::from(msg)).style(style)
    );
    f.render_widget(help_msg, chunks[0]);

    let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor

    let scroll = app.search.visual_scroll(width as usize);
    let search = Paragraph::new(app.search.value())
        .style(match app.input_mode{
            InputMode::View => Style::default(),
            InputMode::Insert => Style::default().fg(Color::Yellow),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search, chunks[1]);
    match app.input_mode {
        InputMode::View => {},
        InputMode::Insert => {
            f.set_cursor(
                chunks[1].x + app.search.visual_cursor() as u16 + 1, // +1 to offset the border
                chunks[1].y + 1, // also +1 to offset the border
            );
        }
    }

    let output = Paragraph::new(app.text.iter().fold(String::new(), |acc, tag| acc + &tag + "\n"))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(output, chunks[2]);
}

