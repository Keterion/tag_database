use std::io::stdout;

mod page;
use page::*;

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
    #[allow(dead_code)]
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
        if let Some(page) = self.get_viewed_page() {
            page.focus_next();
        }
    }
    pub fn get_viewed_page(&mut self) -> Option<&mut Page> {
        match self.view.viewed_page {
            SelectionType::Index { index } => self.pages.get_mut(index),
            SelectionType::Uid { ref uid } => {
                for page in self.pages.iter_mut() {
                    if page.uid == *uid {
                        return Some(page);
                    }
                }
                println!("No viewed page found for uid {}", uid);
                None
            }
        }
    }
}
impl Default for App {
    fn default() -> Self {
        App {
            db: Database::default(),
            pages: vec![Page::default()],
            view: View {
                viewed_page: SelectionType::Uid {
                    uid: "0".to_string(),
                },
            },
            area: Rect::default(),
        }
    }
}

fn main() {
    // setup
    enable_raw_mode().unwrap();
    stdout().execute(EnterAlternateScreen).unwrap();
    stdout().execute(EnableMouseCapture).unwrap();

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();

    let app = App {
        db: Database::open("base.db"),
        pages: vec![],
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
    // keypress handling for each widget (based on focus)
    //
}
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> std::io::Result<()> {
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.add_page(Page::generate_page(PageType::Result, app.area, "result1"));
    app.add_page(Page::generate_page(PageType::Edit, app.area, "edit1"));
    app.view.viewed_page = SelectionType::Index { index: 0 };
    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;
        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('e') => {
                    app.view.viewed_page = SelectionType::Uid {
                        uid: "edit1".to_string(),
                    };
                }
                KeyCode::Char('r') => {
                    app.view.viewed_page = SelectionType::Uid {
                        uid: "result1".to_string(),
                    };
                }
                KeyCode::Tab => {
                    app.focus_next();
                }
                _ => {}
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    app.area = frame.area();
    if let Some(page) = app.get_viewed_page() {
        page.render(frame);
    } else {
        println!("No page viewed");
    }
}
