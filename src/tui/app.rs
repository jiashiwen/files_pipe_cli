use super::{
    pops::{PopTaskEditor, GLOBAL_TASK_EDITOR},
    tabs::{
        new_server_pop_ui, AboutTab, EmailTab, RecipeTab, ServerTab, TaskTab, TracerouteTab,
        WeatherTab,
    },
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
    Frame,
};
use std::time::Duration;
use strum::{Display, EnumCount, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct App {
    mode: Mode,
    tab: Tab,
    about_tab: AboutTab,
    server_tab: ServerTab,
    task_tab: TaskTab,
    recipe_tab: RecipeTab,
    email_tab: EmailTab,
    traceroute_tab: TracerouteTab,
    weather_tab: WeatherTab,
    pop_task_editor: PopTaskEditor,
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
    ServerTab,
    TaskTab, // Recipe,
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
                // app_ui(frame, self);
                if self.server_tab.new_server.show {
                    new_server_pop_ui(frame, &self.server_tab.new_server)
                }
                // if GLOBAL_TASK_EDITOR.read().unwrap().show {
                //     let editor = GLOBAL_TASK_EDITOR.read().unwrap();
                //     pop_task_editor_ui(frame, &editor);
                // }
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
        match self.tab {
            Tab::About => match key.code {
                KeyCode::Char('k') | KeyCode::Up => self.prev(),
                KeyCode::Char('j') | KeyCode::Down => self.next(),
                KeyCode::Char('d') | KeyCode::Delete => self.destroy(),
                _ => {}
            },
            Tab::ServerTab => {
                self.server_tab.refresh_data();
                if self.server_tab.new_server.show {
                    match key.code {
                        KeyCode::Char('n') => self.server_tab.new_server(),
                        KeyCode::Esc => {
                            self.server_tab.new_server.show = false;
                            self.server_tab.new_server.clear();
                        }
                        KeyCode::Tab => self.server_tab.new_server.select_input(),
                        KeyCode::Enter => self.server_tab.new_server.add_server(),
                        KeyCode::Char(to_insert) => {
                            self.server_tab.new_server.enter_char(to_insert);
                        }
                        KeyCode::Backspace => {
                            self.server_tab.new_server.delete_char();
                        }
                        KeyCode::Left => {
                            self.server_tab.new_server.move_cursor_left();
                        }
                        KeyCode::Right => {
                            self.server_tab.new_server.move_cursor_right();
                        }

                        _ => {}
                    }
                    return;
                }
                match key.code {
                    KeyCode::Char('k') | KeyCode::Up => self.server_tab.prev(),
                    KeyCode::Char('j') | KeyCode::Down => self.server_tab.next(),
                    KeyCode::Char('n') => self.server_tab.new_server(),
                    KeyCode::Char('d') => self.server_tab.delete_server(),
                    _ => {}
                }
            }

            Tab::TaskTab => {
                if self.pop_task_editor.show {
                    match key.code {
                        KeyCode::Esc => {
                            self.pop_task_editor.show = false;
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
                    KeyCode::Char('e') => self.pop_task_editor.show_editor(),
                    _ => {}
                }
            }
            _ => {}
        };

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Tab => self.next_tab(),
            _ => {}
        };
    }

    fn prev(&mut self) {
        match self.tab {
            Tab::About => self.about_tab.prev_row(),
            // Tab::Recipe => self.recipe_tab.prev(),
            // Tab::Email => self.email_tab.prev(),
            // Tab::Traceroute => self.traceroute_tab.prev_row(),
            // Tab::Weather => self.weather_tab.prev(),
            _ => {}
        }
    }

    fn next(&mut self) {
        match self.tab {
            Tab::About => self.about_tab.next_row(),
            // Tab::Recipe => self.recipe_tab.next(),
            // Tab::Email => self.email_tab.next(),
            // Tab::Traceroute => self.traceroute_tab.next_row(),
            // Tab::Weather => self.weather_tab.next(),
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

    // pub fn show_popup(&self) -> bool {
    //     self.pop.show_popup.clone()
    // }
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
        App::render_bottom_bar(bottom_bar, buf);
        if self.pop_task_editor.show {
            self.pop_task_editor.clone().render(area, buf);
        }
    }
}

impl App {
    // fn render(self, area: Rect, buf: &mut Buffer) {
    //     let vertical = Layout::vertical([
    //         Constraint::Length(1),
    //         Constraint::Min(0),
    //         Constraint::Length(1),
    //     ]);
    //     let [title_bar, tab, bottom_bar] = vertical.areas(area);

    //     Block::new().style(THEME.root).render(area, buf);
    //     self.render_title_bar(title_bar, buf);
    //     self.clone().render_selected_tab(tab, buf);
    // GLOBAL_TASK_EDITOR
    //     //     .read()
    //     //     .unwrap()
    //     //     .editor
    //     //     .widget()
    //     //     .render(area, buf);
    //     App::render_bottom_bar(bottom_bar, buf);
    // }
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

    fn render_title_bar_ui(&self, f: &mut Frame, area: Rect) {
        let layout = Layout::horizontal([Constraint::Min(0), Constraint::Length(43)]);
        let [title, tabs_area] = layout.areas(area);

        // Span::styled("Mario UI", THEME.app_title).render(title, buf);
        f.render_widget(Span::styled("Mario UI", THEME.app_title), title);
        let titles = Tab::iter().map(Tab::title);
        let tabs = Tabs::new(titles)
            .style(THEME.tabs)
            .highlight_style(THEME.tabs_selected)
            .select(self.tab as usize)
            .divider("")
            .padding("", "");
        // .render(tabs, buf);
        f.render_widget(tabs, tabs_area)
    }

    fn render_selected_tab(self, area: Rect, buf: &mut Buffer) {
        match self.tab {
            Tab::About => self.about_tab.render(area, buf),
            Tab::ServerTab => {
                let tab = self.server_tab.clone();
                tab.render(area, buf);
            }
            Tab::TaskTab => {
                let tab = self.task_tab.clone();

                tab.render(area, buf)
            } // Tab::Recipe => self.recipe_tab.render(area, buf),
              // Tab::Email => self.email_tab.render(area, buf),
              // Tab::Traceroute => self.traceroute_tab.render(area, buf),
              // Tab::Weather => self.weather_tab.render(area, buf),
        };
    }

    fn render_bottom_bar(area: Rect, buf: &mut Buffer) {
        let keys = [
            ("H/←", "Left"),
            ("L/→", "Right"),
            ("K/↑", "Up"),
            ("J/↓", "Down"),
            ("D/Del", "Destroy"),
            ("Q/Esc", "Quit"),
        ];
        let spans = keys
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

pub fn bottom_ui(f: &mut Frame, area: Rect) {
    let keys = [
        ("H/←", "Left"),
        ("L/→", "Right"),
        ("K/↑", "Up"),
        ("J/↓", "Down"),
        ("D/Del", "Destroy"),
        ("Q/Esc", "Quit"),
    ];
    let spans = keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
            let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec();
    let line = Line::from(spans)
        .centered()
        .style((Color::Indexed(236), Color::Indexed(232)));
    f.render_widget(line, area);
}

pub fn app_ui(f: &mut Frame, app: &App) {
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]);
    let [title_bar, tab, bottom_bar] = vertical.areas(f.size());

    app.render_title_bar_ui(f, title_bar);
    match app.tab {
        Tab::About => f.render_widget(app.about_tab, tab),
        Tab::ServerTab => {
            let mut server_tab = app.server_tab.clone();
            // server_tab.refresh_data();
            f.render_widget(server_tab, tab)
        }
        Tab::TaskTab => {
            let mut task_tab = app.task_tab.clone();
            // task_tab.refresh_data();
            f.render_widget(task_tab, tab)
        }
    };

    // f.render_widget(bottom_bar_wiget, bottom_bar);
    bottom_ui(f, bottom_bar)
    // GLOBAL_TASK_EDITOR
    //     .read()
    //     .unwrap()
    //     .editor
    //     .widget()
    //     .render(area, buf);
    // App::render_bottom_bar(bottom_bar, buf);
}
