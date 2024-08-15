use ratatui::{prelude::*, widgets::*};
use tui_input::Input;

pub enum PageType {
    Result,
    Edit,
    Custom,
}

pub struct Page {
    widgets: Vec<WidgetContainer>,
    page_type: PageType,
    focused_widget: usize,
    uid: String,
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
                    next: 0,
                });
                widgets.push(WidgetContainer {
                    widget_type: WidgetType::List {
                        selector: false,
                        list: vec![],
                    },
                    area: inner[0],
                    styling: Style::default(),
                    borders: Borders::ALL,
                    next: 1,
                });
                widgets.push(WidgetContainer {
                    widget_type: WidgetType::List {
                        selector: true,
                        list: vec![],
                    },
                    area: inner[1],
                    styling: Style::default(),
                    borders: Borders::ALL,
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
pub struct WidgetContainer {
    widget_type: WidgetType,
    area: Rect,
    styling: Style,
    borders: Borders,
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
}
pub enum WidgetType {
    Input { input: Input },
    Paragraph { text: String },
    List { selector: bool, list: Vec<String> },
}

pub struct View {
    pub viewed_page: SelectionType,
}
pub enum SelectionType {
    Index {
        index: usize
    },
    UID {
        uid: String
    }
}
