use db::wrapper::Database;
use ratatui::widgets;
#[allow(unused_imports)]
use ratatui::{
    crossterm::{
        event::{
            self,
            DisableMouseCapture,
            EnableMouseCapture,
            Event,
            KeyCode
        },
        terminal::{
            disable_raw_mode,
            enable_raw_mode,
            EnterAlternateScreen,
            LeaveAlternateScreen
        },
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
}
impl App {
    pub fn render_group(&mut self, group: &str) {
        for widget in &mut self.widgets {
            widget.set_display(widget.part_of(group))
        }
    }
    pub fn toggle_widget(&mut self, widget_index: usize) {
        if let Some(widget) = self.widgets.get_mut(widget_index) {
            widget.switch_display();
        }
    }
    pub fn add_widget(&mut self, widget: WidgetContainer) {
        self.widgets.push(widget);
    }
    pub fn remove_widget(&mut self, index: usize) -> WidgetContainer {
        self.widgets.remove(index)
    }
}
impl Default for App {
    fn default() -> Self {
        App {
            db: Database::default(),
            widgets: vec![],
            view: View {
                viewed_widgets: ViewedWidgets::Specific { toggled: vec!() },
                focused: 0,
            }
        }
    }
}
struct WidgetContainer {
    widget: Box<dyn Widget>,
    show: bool,
    position: Rect,
    group: String,
}
impl WidgetContainer {
    pub fn new(widget: Box<dyn Widget>, position: Rect, group: &str) -> Self {
        WidgetContainer {
            widget,
            show: true,
            position,
            group: group.to_owned(),
        }
    }
    pub fn switch_display(&mut self) {
        self.show = !self.show;
    }
    pub fn set_display(&mut self, state: bool) {
        self.show = state;
    }
    pub fn part_of(&self, group: &str) -> bool {
        self.group == group
    }
    pub fn get_widget(&self) -> &Box<dyn Widget> {
        &self.widget
    }
    pub fn get_widget_mut(&mut self) -> &mut Box<dyn Widget> {
        &mut self.widget
    }
    pub fn set_position(&mut self, position: Rect) {
        self.position = position;
    }
    pub fn get_position(&self) -> &Rect {
        &self.position
    }
}
struct View {
    pub viewed_widgets: ViewedWidgets,
    pub focused: usize,
}
enum ViewedWidgets {
    /// Specific inicies of widgets to be shown
    Specific {
        toggled: Vec<usize>,
    },
    /// Group of widgets to be shown
    Group {
        name: String,
    }
}


fn main() {
    let app = App {
        db: Database::open("base.db"),
        ..Default::default()
    };
    // todo!()
    // base app rendering with a few widgets using the widgetcontainer system
}
