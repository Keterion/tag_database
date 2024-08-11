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
    pages: Vec<Page>,
    view: View,
    area: Rect,
}
impl App {
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
    }
    pub fn focus_next(&mut self) {
        if let Some(page) = self.pages.get_mut(self.view.focused) {
            page.focus_next();
        }
    }
}
impl Default for App {
    fn default() -> Self {
        App {
            db: Database::default(),
            pages: vec![Page::default()],
            view: View { focused: 0 },
            area: Rect::default(),
        }
    }
}
enum PageType {
    Result,
    Edit,
    Custom,
}

struct Page {
    widgets: Vec<WidgetContainer>,
    page_type: PageType,
    focused_widget: usize,
    // todo!()
    // implement pages to use instead of widgetcontainers in the app
    // yes yes
}
impl Default for Page {
    fn default() -> Self {
        Page {
            widgets: vec![WidgetContainer::default()],
            page_type: PageType::Custom,
            focused_widget: 0,
        }
    }
}
impl Page {
    pub fn focus_next(&mut self) {
        if let Some(page) = self.widgets.get(self.focused_widget) {
            self.focused_widget = page.next;
        }
    }
    pub fn generate_page(page_type: PageType, area: Rect) -> Self {
        match page_type {
            PageType::Result => {
                let layout = Layout::new(
                    Direction::Vertical,
                    [Constraint::Length(3), Constraint::Min(3)].as_ref(),
                )
                .split(area);
                let mut widgets: Vec<WidgetContainer> = Vec::new();
                widgets.push(WidgetContainer {
                    widget_type: WidgetType::Input {
                        input: Input::default(),
                    },
                    area: layout[0],
                    styling: Style::default(),
                    borders: Borders::ALL,
                    group: "input_1".to_string(),
                    next: 1,
                });
                widgets.push(WidgetContainer {
                    widget_type: WidgetType::List {
                        selector: true,
                        list: vec![],
                    },
                    area: layout[1],
                    styling: Style::default(),
                    borders: Borders::ALL,
                    group: "input_1".to_string(),
                    next: 0,
                });

                Page {
                    widgets,
                    page_type,
                    focused_widget: 0,
                }
            }
            PageType::Edit => {
                let outer = Layout::new(
                    Direction::Vertical,
                    [Constraint::Length(1), Constraint::Min(3)].as_ref(),
                )
                .split(area);
                let inner = Layout::new(
                    Direction::Horizontal,
                    [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                )
                .split(outer[1]);

                let mut widgets: Vec<WidgetContainer> = Vec::new();
                widgets.push(WidgetContainer {
                    widget_type: WidgetType::Paragraph {
                        text: "Edting".to_string(),
                    },
                    area: outer[0],
                    styling: Style::default(),
                    borders: Borders::NONE,
                    group: "edit_1".to_string(),
                    next: 2,
                });
                widgets.push(WidgetContainer {
                    widget_type: WidgetType::List {
                        selector: false,
                        list: vec![],
                    },
                    area: inner[0],
                    styling: Style::default(),
                    borders: Borders::ALL,
                    group: "edit_1".to_string(),
                    next: 2,
                });
                widgets.push(WidgetContainer {
                    widget_type: WidgetType::List {
                        selector: true,
                        list: vec![],
                    },
                    area: inner[1],
                    styling: Style::default(),
                    borders: Borders::ALL,
                    group: "edit_1".to_string(),
                    next: 2,
                });
                Page {
                    widgets,
                    page_type,
                    focused_widget: 2,
                }
            }
            PageType::Custom => Page::default(),
        }
    }
    pub fn add_widget(&mut self, widget: WidgetContainer) {
        self.widgets.push(widget);
    }
    pub fn render(&mut self, frame: &mut Frame) {
        for widget in &mut self.widgets {
            widget.render(frame);
        }
    }
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
                    frame.render_stateful_widget(
                        List::new(list.iter().map(|line| &line[..]))
                            .block(Block::default().borders(self.borders))
                            .style(self.styling),
                        self.area,
                        &mut ListState::default(),
                    );
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
    pub focused: usize,
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
}
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> std::io::Result<()> {
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.add_page(Page::generate_page(PageType::Result, app.area));
    app.add_page(Page::generate_page(PageType::Edit, app.area));
    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;
        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('e') => {
                    app.view.focused = 0;
                }
                KeyCode::Char('r') => {
                    app.view.focused = 1;
                }
                _ => {}
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    app.area = frame.area();
    if let Some(page) = app.pages.get_mut(app.view.focused) {
        page.render(frame);
    }
}
