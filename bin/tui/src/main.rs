use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};
use rusqlite::Connection;
use std::{
    fmt::Display,
    io::{self, stdout},
};
use tui_input::{backend::crossterm::EventHandler, Input};

enum InputMode {
    View,
    Insert,
}

enum CurrentlyViewing {
    Search { search_type: ResultType },
    Editing { table: String, entry: Result },
}
enum FocusedWidget {
    SearchBar,
    Results,
}

/// Holds the application state
struct App {
    /// Current value of the search box
    search: Input,
    /// Current input mode
    input_mode: InputMode,
    currently_viewing: CurrentlyViewing,
    currently_focused: FocusedWidget,

    results: ResultsList,
}

struct ResultsList {
    state: ListState,
    items: Vec<Result>,
}
#[derive(Clone)]
struct Result {
    name: String,
    id: i64,
    rtype: ResultType,
}
#[derive(Clone)]
enum ResultType {
    Tag,
    Namespace,
    Image,
    Group,
}

impl Default for Result {
    fn default() -> Self {
        Result {
            name: "".to_string(),
            id: -1,
            rtype: ResultType::Tag,
        }
    }
}

impl Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let app = App {
        search: Input::default(),
        input_mode: InputMode::View,
        currently_viewing: CurrentlyViewing::Search {
            search_type: ResultType::Tag,
        },
        currently_focused: FocusedWidget::SearchBar,
        results: ResultsList {
            state: ListState::default(),
            items: vec![],
        },
    };
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;
    terminal.show_cursor()?;

    match res {
        Ok(_) => {}
        Err(err) => {
            eprint!("App failed with: \n{}", err);
        }
    }
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> io::Result<()> {
    let conn = Connection::open("base.db").unwrap();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.currently_viewing {
                CurrentlyViewing::Search { ref search_type } => {
                    match app.input_mode {
                        InputMode::View => match key.code {
                            KeyCode::Char('i') => match app.currently_focused {
                                FocusedWidget::SearchBar => {
                                    app.input_mode = InputMode::Insert;
                                }
                                _ => {}
                            },
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Char('e') => match app.currently_focused {
                                FocusedWidget::Results => {
                                    app.currently_viewing = CurrentlyViewing::Editing {
                                        table: "tags".to_string(),
                                        entry: app.results.items
                                            [app.results.state.selected().unwrap_or(0)]
                                        .clone(),
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Tab => match app.currently_focused {
                                FocusedWidget::SearchBar => {
                                    app.currently_focused = FocusedWidget::Results
                                }
                                FocusedWidget::Results => {
                                    app.currently_focused = FocusedWidget::SearchBar
                                }
                            },
                            KeyCode::Up => match app.currently_focused {
                                FocusedWidget::Results => app.results.state.select_previous(),
                                _ => {}
                            },
                            KeyCode::Down => match app.currently_focused {
                                FocusedWidget::Results => app.results.state.select_next(),
                                _ => {}
                            },
                            _ => {}
                        },
                        InputMode::Insert => match key.code {
                            KeyCode::Enter => {
                                app.search.reset();
                            }
                            KeyCode::Tab => {
                                app.search.handle(tui_input::InputRequest::DeletePrevWord);

                                for c in app
                                    .results
                                    .items
                                    .first()
                                    .unwrap_or(&Result::default())
                                    .name
                                    .chars()
                                {
                                    app.search.handle(tui_input::InputRequest::InsertChar(c));
                                }
                            }
                            KeyCode::Esc => {
                                app.input_mode = InputMode::View;
                            }
                            _ => {
                                app.search.handle_event(&Event::Key(key));
                                // you could do a cool loading thing and wait for a thread to finish the
                                // sql query
                                let last_word = app.search.value().split(' ').last().unwrap_or("");
                                app.results.items = db::tags::get_tags_with(last_word, &conn)
                                    .iter()
                                    .map(|tag| Result {
                                        name: tag.1.clone(),
                                        id: tag.0,
                                        rtype: search_type.clone(),
                                    })
                                    .collect();
                            }
                        },
                    }
                }
                CurrentlyViewing::Editing { ref entry, .. } => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => app.currently_viewing = CurrentlyViewing::Search { search_type: entry.rtype.clone() },
                    _ => {}
                },
            }
        }
    }
}
fn ui(f: &mut Frame, app: &mut App) {
    match &app.currently_viewing {
        CurrentlyViewing::Search { .. } => tabs::render_search(f, app),
        CurrentlyViewing::Editing { .. } => tabs::render_editing(f, app),
    }
}
mod tabs {
    use super::*;
    pub fn render_search(f: &mut Frame, app: &mut App) {
        let c = core::cell::OnceCell::new();
        let chunks = c.get_or_init(|| {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Min(3),
                    ]
                    .as_ref(),
                )
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
                    Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to complete tags, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to send the query"),
                ],
                Style::default(),
            ),
        };
        let help_msg = Paragraph::new(Text::from(Line::from(msg)).style(style));
        f.render_widget(help_msg, chunks[0]);

        let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor

        let scroll = app.search.visual_scroll(width as usize);
        let search = Paragraph::new(app.search.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(match app.currently_focused {
                        FocusedWidget::SearchBar => Color::Green,
                        _ => Color::White,
                    })
                    .title("Search"),
            )
            .scroll((0, scroll as u16));
        f.render_widget(search, chunks[1]);
        match app.input_mode {
            InputMode::View => {}
            InputMode::Insert => {
                f.set_cursor(
                    chunks[1].x + app.search.visual_cursor() as u16 + 1, // +1 to offset the border
                    chunks[1].y + 1, // also +1 to offset the border
                );
            }
        }

        let output =
            List::new(
                app.results
                    .items
                    .iter()
                    .map(|line| &line.name[..])
                    .collect::<Vec<&str>>(),
            )
            .highlight_style(Color::Yellow)
            .block(Block::default().borders(Borders::ALL).border_style(
                match app.currently_focused {
                    FocusedWidget::Results => Color::Green,
                    _ => Color::White,
                },
            ));
        f.render_stateful_widget(output, chunks[2], &mut app.results.state);
    }
    pub fn render_editing(f: &mut Frame, app: &App) {
        let (table, entry) = match &app.currently_viewing {
            CurrentlyViewing::Editing { table, entry } => (table, entry),
            _ => return,
        };
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(3)].as_ref())
            .split(f.size());
        #[allow(unused)]
        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(outer_layout[1]);


        f.render_widget(
            Paragraph::new(format!("Editing entry '{}' in table {}", entry.name, table)),
            outer_layout[0],
        );

        f.render_widget(
            Paragraph::new("")
            .block(
                Block::default()
                .borders(Borders::ALL)
                .title("Data")
            ), inner_layout[0],
        );
        f.render_widget(
            Paragraph::new("")
            .block(
                Block::default()
                .borders(Borders::ALL)
                .title("Options")
            ), inner_layout[1],
        );
    }
}
