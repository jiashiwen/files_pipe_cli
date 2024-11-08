use super::{
    pops::{PopHelp, PopNewServer, PopTaskEditor, GLOBAL_TASK_EDITOR},
    tabs::{AboutTab, ServerTab, TaskTab},
    term, THEME,
};
use color_eyre::{eyre::Context, Result};
use itertools::Itertools;
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Color,
    terminal::Terminal,
    text::{Line, Span},
    widgets::{Block, Tabs, Widget},
};
use std::time::Duration;
use strum::{Display, EnumCount, EnumIter, FromRepr, IntoEnumIterator};
use tui_textarea::TextArea;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct App {
    mode: Mode,
    tab: Tab,
    about_tab: AboutTab,
    server_tab: ServerTab,
    task_tab: TaskTab,
    pop_help: PopHelp,
    pop_task_editor: PopTaskEditor,
    pop_new_server: PopNewServer,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Running,
    Destroy,
    Quit,
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, EnumCount, PartialEq, Eq)]
enum Tab {
    #[default]
    About,
    Server,
    Task, // Recipe,
          // Email,
          // Traceroute,
          // Weather,
}

pub fn run(terminal: &mut Terminal<impl Backend>) -> Result<()> {
    App::default().run(terminal)
}

impl App {
    /// Run the app until the user quits.
    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        self.server_tab.refresh_data();
        self.task_tab.refresh_data();
        while self.is_running() {
            self.draw(terminal)?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    /// Draw a single frame of the app.
    fn draw(&self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        terminal
            .draw(|frame| {
                frame.render_widget(self, frame.size());
            })
            .wrap_err("terminal.draw")?;
        Ok(())
    }

    /// Handle events from the terminal.
    ///
    /// This function is called once per frame, The events are polled from the stdin with timeout of
    /// 1/50th of a second. This was chosen to try to match the default frame rate of a GIF in VHS.
    fn handle_events(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        match term::next_event(timeout)? {
            Some(Event::Key(key)) if key.kind == KeyEventKind::Press => self.handle_key_press(key),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        if self.pop_help.show {
            match key.code {
                KeyCode::Esc => {
                    self.pop_help.show = false;
                }
                KeyCode::Char('k') | KeyCode::Up => self.pop_help.prev_line(),
                KeyCode::Char('j') | KeyCode::Down => self.pop_help.next_line(),
                _ => {}
            }
            return;
        }
        match self.tab {
            Tab::About => match key.code {
                KeyCode::Char('k') | KeyCode::Up => self.prev(),
                KeyCode::Char('j') | KeyCode::Down => self.next(),
                KeyCode::Char('d') | KeyCode::Delete => self.destroy(),
                _ => {}
            },
            Tab::Server => {
                self.server_tab.refresh_data();
                if self.pop_new_server.show {
                    match key.code {
                        KeyCode::Esc => {
                            self.pop_new_server.show = false;
                        }
                        KeyCode::Enter => {
                            self.pop_new_server.add_server();
                            self.server_tab.refresh_data();
                        }
                        KeyCode::Tab => self.pop_new_server.select_input(),
                        _ => self.pop_new_server.input(key),
                    }
                    return;
                }

                match key.code {
                    KeyCode::Char('k') | KeyCode::Up => self.server_tab.prev(),
                    KeyCode::Char('j') | KeyCode::Down => self.server_tab.next(),
                    KeyCode::Char('a') => self.pop_new_server.show_pop(),
                    KeyCode::Enter => self.server_tab.set_current_server(),
                    KeyCode::Char('d') => self.server_tab.delete_server(),
                    _ => {}
                }
            }

            Tab::Task => {
                if self.task_tab.pop_alert.show {
                    match key.code {
                        KeyCode::Esc => {
                            self.task_tab.pop_alert.show = false;
                            return;
                        }
                        _ => {
                            return;
                        }
                    }
                }

                if self.pop_task_editor.show {
                    if self.pop_task_editor.pop_select_template.show {
                        match key.code {
                            KeyCode::Esc => {
                                self.pop_task_editor.pop_select_template.show = false;
                                self.pop_task_editor.pop_select_template.clean();
                                return;
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                self.pop_task_editor.pop_select_template.prev();
                                return;
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                self.pop_task_editor.pop_select_template.next();
                                return;
                            }
                            KeyCode::Enter => {
                                self.pop_task_editor.pop_select_template.load_template();
                                self.pop_task_editor.show_editer_with_text(vec![self
                                    .pop_task_editor
                                    .pop_select_template
                                    .template
                                    .clone()]);
                                self.pop_task_editor.pop_select_template.clean();
                                self.pop_task_editor.pop_select_template.show = false;

                                return;
                            }
                            _ => {
                                return;
                            }
                        }
                    }
                    match key.code {
                        KeyCode::Esc => {
                            self.pop_task_editor.show = false;
                            let mut editor = GLOBAL_TASK_EDITOR.write().unwrap();
                            *editor = TextArea::default();
                            return;
                        }

                        KeyCode::F(9) => {
                            self.pop_task_editor.pop_select_template.show_pop();
                            return;
                        }
                        KeyCode::F(10) => {
                            match self.pop_task_editor.create_task() {
                                Ok(id) => {
                                    self.pop_task_editor.alert_msg = format!("task {} created", id)
                                }
                                Err(_) => {
                                    self.pop_task_editor.alert_msg = "create task error".to_string()
                                }
                            };
                            let mut editor = GLOBAL_TASK_EDITOR.write().unwrap();
                            *editor = TextArea::default();
                            self.task_tab.refresh_data();
                            return;
                        }
                        _ => {
                            GLOBAL_TASK_EDITOR.write().unwrap().input(key);
                            return;
                        }
                    }
                }
                match key.code {
                    KeyCode::Char('k') | KeyCode::Up => self.task_tab.prev(),
                    KeyCode::Char('j') | KeyCode::Down => self.task_tab.next(),
                    KeyCode::Char('f') => self.task_tab.refresh_data(),
                    KeyCode::Char('c') => self.pop_task_editor.show_editor(),
                    KeyCode::Char('e') => {
                        let task_str = self.task_tab.get_task();
                        self.pop_task_editor.show_editer_with_text(vec![task_str]);
                    }
                    KeyCode::Char('d') => {
                        self.task_tab.delete_task();
                    }
                    KeyCode::Char('r') => {
                        self.task_tab.run_task();
                    }
                    KeyCode::Char('i') => {
                        self.task_tab.stop_task();
                    }
                    _ => {}
                }
            }
        };

        match key.code {
            KeyCode::Tab => self.next_tab(),
            KeyCode::F(1) => self.pop_help.show_switch(),
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            _ => {}
        };
    }

    fn prev(&mut self) {
        match self.tab {
            Tab::About => self.about_tab.prev_row(),
            _ => {}
        }
    }

    fn next(&mut self) {
        match self.tab {
            Tab::About => self.about_tab.next_row(),
            _ => {}
        }
    }

    fn prev_tab(&mut self) {
        self.tab = self.tab.prev();
    }

    fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }

    fn destroy(&mut self) {
        self.mode = Mode::Destroy;
    }
}

/// Implement Widget for &App rather than for App as we would otherwise have to clone or copy the
/// entire app state on every frame. For this example, the app state is small enough that it doesn't
/// matter, but for larger apps this can be a significant performance improvement.
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]);
        let [title_bar, tab, bottom_bar] = vertical.areas(area);

        Block::new().style(THEME.root).render(area, buf);
        self.render_title_bar(title_bar, buf);
        self.clone().render_selected_tab(tab, buf);

        let keys = match self.tab {
            Tab::About => vec![
                ("TAB".to_string(), "SelectTab".to_string()),
                ("H/←".to_string(), "Left".to_string()),
                ("L/→".to_string(), "Right".to_string()),
                ("K/↑".to_string(), "Up".to_string()),
                ("J/↓".to_string(), "Down".to_string()),
                ("D/Del".to_string(), "Destroy".to_string()),
                ("Q/Esc".to_string(), "Quit".to_string()),
            ],
            Tab::Server => {
                vec![
                    ("TAB".to_string(), "SelectTab".to_string()),
                    ("K/↑".to_string(), "Up".to_string()),
                    ("J/↓".to_string(), "Down".to_string()),
                    ("A/Add".to_string(), "Add".to_string()),
                    ("D/Del".to_string(), "Del".to_string()),
                    ("Enter".to_string(), "Set Server".to_string()),
                    ("Q/Esc".to_string(), "Quit".to_string()),
                ]
            }
            Tab::Task => vec![
                ("TAB".to_string(), "SelectTab".to_string()),
                ("K/↑".to_string(), "Up".to_string()),
                ("J/↓".to_string(), "Down".to_string()),
                ("C".to_string(), "Create".to_string()),
                ("E".to_string(), "Edit".to_string()),
                ("Enter".to_string(), "Show Task".to_string()),
                ("R".to_string(), "Run Task".to_string()),
                ("I".to_string(), "Stop Task".to_string()),
                ("S".to_string(), "Task Status".to_string()),
                ("D/Del".to_string(), "Del Task".to_string()),
                ("Q/Esc".to_string(), "Quit".to_string()),
            ],
        };

        App::render_bottom_bar(keys, bottom_bar, buf);

        self.pop_help.render(area, buf);

        if self.pop_task_editor.show {
            self.pop_task_editor.clone().render(area, buf);
        }

        if self.pop_task_editor.pop_select_template.show {
            self.pop_task_editor
                .pop_select_template
                .clone()
                .render(area, buf)
        }

        if self.pop_new_server.show {
            self.pop_new_server.clone().render(area, buf);
        }
    }
}

impl App {
    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([Constraint::Min(0), Constraint::Length(43)]);
        let [title, tabs] = layout.areas(area);

        Span::styled("Mario UI", THEME.app_title).render(title, buf);
        let titles = Tab::iter().map(Tab::title);
        Tabs::new(titles)
            .style(THEME.tabs)
            .highlight_style(THEME.tabs_selected)
            .select(self.tab as usize)
            .divider("")
            .padding("", "")
            .render(tabs, buf);
    }

    fn render_selected_tab(self, area: Rect, buf: &mut Buffer) {
        match self.tab {
            Tab::About => self.about_tab.render(area, buf),
            Tab::Server => {
                let tab = self.server_tab.clone();
                tab.render(area, buf);
            }
            Tab::Task => {
                let tab = self.task_tab.clone();
                tab.render(area, buf)
            }
        };
    }

    fn render_bottom_bar(cmd_list: Vec<(String, String)>, area: Rect, buf: &mut Buffer) {
        let spans = cmd_list
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
                let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
                [key, desc]
            })
            .collect_vec();
        Line::from(spans)
            .centered()
            .style((Color::Indexed(236), Color::Indexed(232)))
            .render(area, buf);
    }
}

impl Tab {
    fn next(self) -> Self {
        let current_index = self as usize;
        let mut next_index = current_index.saturating_add(1);
        let count = Self::COUNT;
        if next_index.eq(&count) {
            next_index = 0;
        }

        Self::from_repr(next_index).unwrap_or(self)
    }

    fn prev(self) -> Self {
        let current_index = self as usize;
        let prev_index = current_index.saturating_sub(1);
        Self::from_repr(prev_index).unwrap_or(self)
    }

    fn title(self) -> String {
        match self {
            Self::About => String::new(),
            tab => format!(" {tab} "),
        }
    }
}
