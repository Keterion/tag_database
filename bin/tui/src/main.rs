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
    Search {
        search_type: ResultType,
    },
    Editing {
        table: String,
        entry: Result,
        options: SelectorList<EditOption>,
    },
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

    search_results: SelectorList<Result>,

    connection: Connection,
    
    user_input: Option<UserInput>,
}
struct UserInput {
    wanted_by: EditOption,
    influences: i64,
    data: Input,
}

struct SelectorList<T> {
    state: ListState,
    items: Vec<T>,
}

#[derive(Clone)]
struct Result {
    name: String,
    id: i64,
    rtype: ResultType,
}
#[derive(Clone, Debug)]
enum ResultType {
    Tag,
    Namespace,
    Image,
    Group,
}

enum EditOption {
    Rename,
    Delete,
    AddTag,
    RemoveTag,
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
        search_results: SelectorList {
            state: ListState::default(),
            items: vec![],
        },
        connection: Connection::open("base.db").unwrap(),
        user_input: None,
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
    //db::images::add_image("abc1.jpg", &conn);
    //db::images::add_image("abc2.jpg", &conn);
    //db::tags::add_tag_to_img("test", 1, true, &conn).unwrap();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.currently_viewing {
                CurrentlyViewing::Search { ref search_type } => {
                    match app.input_mode {
                        InputMode::View => match key.code {
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Char('i') => match app.currently_focused {
                                FocusedWidget::SearchBar => {
                                    app.currently_viewing = CurrentlyViewing::Search {
                                        search_type: ResultType::Image,
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Char('t') => match app.currently_focused {
                                FocusedWidget::SearchBar => {
                                    app.currently_viewing = CurrentlyViewing::Search {
                                        search_type: ResultType::Tag,
                                    };
                                }
                                _ => {}
                            },
                            KeyCode::Char('n') => match app.currently_focused {
                                FocusedWidget::SearchBar => {
                                    app.currently_viewing = CurrentlyViewing::Search {
                                        search_type: ResultType::Namespace,
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Char('g') => match app.currently_focused {
                                FocusedWidget::SearchBar => {
                                    app.currently_viewing = CurrentlyViewing::Search {
                                        search_type: ResultType::Group,
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Char('e') => match app.currently_focused {
                                FocusedWidget::Results => {
                                    let (table, options) = match search_type {
                                        ResultType::Tag => (
                                            "tags".to_string(),
                                            vec![EditOption::Rename, EditOption::Delete],
                                        ),
                                        ResultType::Image => (
                                            "images".to_string(),
                                            vec![
                                                EditOption::Rename,
                                                EditOption::Delete,
                                                EditOption::AddTag,
                                                EditOption::RemoveTag,
                                            ],
                                        ),
                                        ResultType::Group => (
                                            "groups".to_string(),
                                            vec![
                                                EditOption::Rename,
                                                EditOption::Delete,
                                                EditOption::AddTag,
                                                EditOption::RemoveTag,
                                            ],
                                        ),
                                        ResultType::Namespace => (
                                            "namespaces".to_string(),
                                            vec![
                                                EditOption::Rename,
                                                EditOption::Delete,
                                                EditOption::AddTag,
                                                EditOption::RemoveTag,
                                            ],
                                        ),
                                    };
                                    app.currently_viewing = CurrentlyViewing::Editing {
                                        table,
                                        entry: app.search_results.items
                                            [app.search_results.state.selected().unwrap_or(0)]
                                        .clone(),
                                        options: SelectorList {
                                            state: ListState::default(),
                                            items: options,
                                        },
                                    }
                                }
                                FocusedWidget::SearchBar => {
                                    app.input_mode = InputMode::Insert;
                                }
                                _ => {}
                            },
                            KeyCode::Tab => match app.currently_focused {
                                FocusedWidget::SearchBar => {
                                    app.currently_focused = FocusedWidget::Results
                                }
                                FocusedWidget::Results => {
                                    app.currently_focused = FocusedWidget::SearchBar
                                },
                                _ => {},
                            },
                            KeyCode::Up => match app.currently_focused {
                                FocusedWidget::Results => {
                                    app.search_results.state.select_previous()
                                }
                                _ => {}
                            },
                            KeyCode::Down => match app.currently_focused {
                                FocusedWidget::Results => app.search_results.state.select_next(),
                                _ => {}
                            },
                            _ => {}
                        },
                        InputMode::Insert => match key.code {
                            KeyCode::Enter => {
                                match search_type {
                                    ResultType::Image => {
                                        app.search_results.items = db::images::query_sql(
                                            app.search.value(),
                                            &app.connection,
                                        )
                                        .iter()
                                        .map(|image| Result {
                                            id: image.0,
                                            name: image.1.clone(),
                                            rtype: ResultType::Image,
                                        })
                                        .collect();
                                    }
                                    _ => {}
                                }
                                app.search.reset();
                            }
                            KeyCode::Tab => {
                                app.search.handle(tui_input::InputRequest::DeletePrevWord);

                                for c in app
                                    .search_results
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
                                match search_type {
                                    ResultType::Tag => {
                                        let last_word =
                                            app.search.value().split(' ').last().unwrap_or("");
                                        app.search_results.items =
                                            db::tags::get_tags_with(last_word, &app.connection)
                                                .iter()
                                                .map(|tag| Result {
                                                    name: tag.1.clone(),
                                                    id: tag.0,
                                                    rtype: search_type.clone(),
                                                })
                                                .collect();
                                    }
                                    _ => {}
                                }
                            }
                        },
                    }
                }
                CurrentlyViewing::Editing { ref entry, ref mut options, ref table } => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => {
                        options.state.select_previous();
                    },
                    KeyCode::Down => {
                        options.state.select_next();
                    },
                    KeyCode::Esc => {
                        if let None = app.user_input {
                            app.currently_viewing = CurrentlyViewing::Search {
                                search_type: entry.rtype.clone(),
                            }
                        } // if the user tries to do an operation, they need to complete it
                    },
                    KeyCode::Enter => {
                        if let Some(ref input) = app.user_input {
                            match input.wanted_by {
                                EditOption::Rename => {
                                    match &table[..] {
                                        "tags" => db::tags::rename_tag(input.influences, input.data.value(), &app.connection).unwrap(),
                                        "images" => db::images::update_path(input.influences, input.data.value(), &app.connection).unwrap(),
                                        "namespaces" => db::namespaces::rename_namespace(input.influences, input.data.value(), &app.connection).unwrap(),
                                        "groups" => todo!(),
                                        _ => {},
                                    }
                                },
                                _ => {},
                            }
                            app.user_input = None;
                        } else {
                            match options.items[options.state.selected().unwrap()] {
                                EditOption::Delete => {
                                    db::utils::remove_id(entry.id, table, &app.connection).unwrap();
                                },
                                EditOption::Rename => {
                                            app.user_input = Some(UserInput { wanted_by: EditOption::Rename, influences: entry.id, data: Input::default() });
                                },
                                EditOption::AddTag => {},
                                EditOption::RemoveTag => {},
                            }
                        }
                    }
                    _ => {
                        if let Some(ref mut input) = app.user_input {
                            input.data.handle_event(&Event::Key(key));
                        }
                    }
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
                    Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to edit the query."),
                    if let CurrentlyViewing::Search { search_type } = &app.currently_viewing {
                        Span::raw(format!(" Searching for: {:?}s", search_type))
                    } else {
                        Span::raw("")
                    },
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
                app.search_results
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
        f.render_stateful_widget(output, chunks[2], &mut app.search_results.state);
    }
    pub fn render_editing(f: &mut Frame, app: &mut App) {
        let (table, entry, ref mut options) = match &mut app.currently_viewing {
            CurrentlyViewing::Editing {
                table,
                entry,
                ref mut options,
            } => (table, entry, options),
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
            Paragraph::new(match entry.rtype {
                ResultType::Tag => {
                    format!("Name: {}\nId: {}", entry.name, entry.id)
                }
                ResultType::Image => {
                    let mut res = String::new();
                    res.push_str(&format!("Path: {}\nId: {}\nTags:\n", entry.name, entry.id));
                    for tag in db::images::get_tags_of_img(entry.id, &app.connection) {
                        res.push_str(&tag.1);
                    }
                    res
                }
                ResultType::Namespace => "".to_string(),
                ResultType::Group => "".to_string(),
            })
            .block(Block::default().borders(Borders::ALL).title("Data")),
            inner_layout[0],
        );
        f.render_stateful_widget(
            List::new(options.items.iter().map(|opt| match opt {
                EditOption::Rename => "Rename",
                EditOption::Delete => "Delete",
                EditOption::AddTag => "Add Tag",
                EditOption::RemoveTag => "Remove Tag",
            }))
            .highlight_style(Color::Yellow)
            .block(Block::default().borders(Borders::ALL).title("Options")),
            inner_layout[1],
            &mut options.state,
        );

        if let Some(input) = &app.user_input {
            let popup = Paragraph::new(input.data.value()).block(Block::default().borders(Borders::ALL).title("Input"));
            let area = centered_rect(60, 20, f.size());
            f.render_widget(Clear, area);
            f.render_widget(popup, area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ]).split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
        .split(popup_layout[1])[1]
}
