use std::{io::stdout, rc::Rc};

use db::wrapper::Database;
#[allow(unused_imports)]
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};
#[allow(unused_imports)]
use tui_input::{backend::crossterm::EventHandler, Input};

struct App {
    db: Database,
    widgets: Vec<WidgetContainer>,
    view: View,
    area: Rect,
}
impl App {
    pub fn add_widget(&mut self, widget: WidgetContainer) -> usize {
        self.widgets.push(widget);
        self.widgets.len() - 1
    }
    pub fn focus_next(&mut self) {
        if let Some(widget) = self.widgets.get(self.view.focused) {
            self.view.focused = widget.next;
        }
    }
    pub fn gen_layout(area: Rect, page_type: PageType) -> Rc<[Rect]> {
        match page_type {
            PageType::Result => {
                Layout::new(
                    Direction::Vertical, 
                    [
                        Constraint::Length(1),
                        Constraint::Min(3),
                    ].as_ref()
                ).split(area)
            },
            PageType::Edit => {
                let outer = Layout::new(
                    Direction::Vertical,
                    [
                        Constraint::Length(1),
                        Constraint::Min(3)
                    ].as_ref()
                    ).split(area);
                let inner = Layout::new(
                    Direction::Horizontal,
                    [
                        Constraint::Percentage(50),
                                               Constraint::Percentage(50)
                    ].as_ref()
                    ).split(outer[1]);
                Rc::new(
                    [outer[0], inner[0], inner[1]]
                )
            },
        }
    }
}
impl Default for App {
    fn default() -> Self {
        App {
            db: Database::default(),
            widgets: vec![],
            view: View {
                viewed_widgets: ViewedWidgets::Specific { toggled: vec![] },
                focused: 0,
            },
            area: Rect::default(),
        }
    }
}
enum PageType {
    Result,
    Edit,
}

struct Page {
    widgets: Vec<WidgetContainer>,
    page_type: PageType,
    // todo!()
    // implement pages to use instead of widgetcontainers in the app 
    // yes yes
}
struct WidgetContainer {
    widget_type: WidgetType,
    area: Rect,
    styling: Style,
    borders: Borders,
    group: String,
    next: usize,
}
impl Default for WidgetContainer {
    fn default() -> Self {
        WidgetContainer {
            widget_type: WidgetType::Paragraph {
                text: String::new(),
            },
            area: Rect::default(),
            styling: Style::default(),
            borders: Borders::NONE,
            group: "default".to_string(),
            next: 0,
        }
    }
}
impl WidgetContainer {
    pub fn render(&mut self, frame: &mut Frame) {
        match self.widget_type {
            WidgetType::Paragraph { ref text } => frame.render_widget(
                Paragraph::new(Line::raw(text))
                    .block(Block::default().borders(self.borders))
                    .style(self.styling),
                self.area,
            ),
            WidgetType::List { selector, ref list } => {
                if selector {
                    frame.render_stateful_widget(List::new(list.iter().map(|line| &line[..])).block(Block::default().borders(self.borders)).style(self.styling), self.area, &mut ListState::default());
                } else {
                    frame.render_widget(
                        List::new(list.iter().map(|line| &line[..]))
                            .block(Block::default().borders(self.borders))
                            .style(self.styling),
                        self.area,
                    )
                }
            }
            WidgetType::Input { ref input } => {
                frame.render_widget(
                    Paragraph::new(input.value())
                        .block(Block::default().borders(self.borders))
                        .style(self.styling),
                    self.area,
                );
            }
        };
    }
    pub fn part_of_group(&self, group: &str) -> bool {
        self.group == group
    }
}
enum WidgetType {
    Input { input: Input },
    Paragraph { text: String },
    List { selector: bool, list: Vec<String> },
}

struct View {
    pub viewed_widgets: ViewedWidgets,
    pub focused: usize,
}
enum ViewedWidgets {
    /// Specific indices of widgets to be shown
    Specific { toggled: Vec<usize> },
    /// Group of widgets to be shown
    Group { name: String },
}

fn main() {
    // setup
    enable_raw_mode().unwrap();
    stdout().execute(EnterAlternateScreen).unwrap();
    stdout().execute(EnableMouseCapture).unwrap();

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();

    let app = App {
        db: Database::open("base.db"),
        ..Default::default()
    };

    let res = run_app(&mut terminal, app);

    disable_raw_mode().unwrap();
    stdout().execute(LeaveAlternateScreen).unwrap();
    stdout().execute(DisableMouseCapture).unwrap();
    terminal.show_cursor().unwrap();

    match res {
        Ok(_) => {}
        Err(err) => {
            eprint!("App failed with:\n{}", err);
        }
    }

    // todo!()
    // base app rendering with a few widgets using the widgetcontainer system
}
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> std::io::Result<()> {
    terminal.draw(|f| ui(f, &mut app)).unwrap();
        let layout = App::gen_layout(app.area, PageType::Result);
        app.add_widget(WidgetContainer {
            widget_type: WidgetType::Input { input: Input::default() },
            area: layout[0],
            styling: Style::default(),
            borders: Borders::ALL,
            group: "input_1".to_string(),
            next: 1,
        });
        app.add_widget(WidgetContainer {
            widget_type: WidgetType::List { selector: true, list: vec![] },
            area: layout[1],
            styling: Style::default(),
            borders: Borders::ALL,
            group: "input_1".to_string(),
            next: 0,
        });
        let layout = App::gen_layout(app.area, PageType::Edit);
        app.add_widget(
            WidgetContainer {
                widget_type: WidgetType::Paragraph { text: "Edting".to_string() },
                area: layout[0],
                styling: Style::default(),
                borders: Borders::NONE,
                group: "edit_1".to_string(),
                next: 2,
            }
        );
        app.add_widget(WidgetContainer {
            widget_type: WidgetType::List { selector: false, list: vec![] },
            area: layout[1],
            styling: Style::default(),
            borders: Borders::ALL,
            group: "edit_1".to_string(),
            next: 3,
        });
        app.add_widget(WidgetContainer {
            widget_type: WidgetType::List { selector: true, list: vec![] },
            area: layout[2],
            styling: Style::default(),
            borders: Borders::ALL,
            group: "edit_1".to_string(),
            next: 4,
        });
        drop(layout);
    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;
        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('a') => {
                    app.add_widget(WidgetContainer {
                        widget_type: WidgetType::Paragraph {
                            text: "Hello World!".to_string(),
                        },
                        area: app.area,
                        styling: Style::default().bold(),
                        borders: Borders::ALL,
                        ..Default::default()
                    });
                    app.view.viewed_widgets = ViewedWidgets::Specific { toggled: vec![0] };
                },
                KeyCode::Char('b') => {
                    app.add_widget(WidgetContainer {
                        widget_type: WidgetType::Paragraph {
                            text: "Hello Second!".to_string(),
                        },
                        area: app.area,
                        styling: Style::default(),
                        borders: Borders::BOTTOM,
                        group: "group_b".to_string(),
                        ..Default::default()
                    });
                    app.view.viewed_widgets = ViewedWidgets::Group { name: "group_b".to_string() };
                },
                KeyCode::Char('e') => {
                    app.view.viewed_widgets = ViewedWidgets::Group { name: "edit_1".to_string() };
                },
                KeyCode::Char('r') => {
                    app.view.viewed_widgets = ViewedWidgets::Group { name: "input_1".to_string() };
                },
                _ => {}
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    app.area = frame.area();
    match &app.view.viewed_widgets {
        ViewedWidgets::Specific { ref toggled } => {
            for index in toggled {
                if let Some(widget) = app.widgets.get_mut(*index) {
                    widget.render(frame);
                }
            }
        },
        ViewedWidgets::Group { ref name } => {
            for widget in &mut app.widgets {
                if widget.part_of_group(name) {
                    widget.render(frame);
                }
            }
        },
    }
}
