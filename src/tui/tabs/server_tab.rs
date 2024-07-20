use crate::{
    request::TaskServer,
    resources::{list_servers_from_cf, remove_server_from_cf, save_task_server_to_cf},
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, StatefulWidget, Table, TableState, Widget,
    },
    Frame,
};
use std::{
    ops::Sub,
    sync::{mpsc::Sender, Arc},
};
use strum::{EnumCount, EnumIter, FromRepr};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// use crate::{RgbSwatch, THEME};

const ITEM_HEIGHT: usize = 4;
const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

pub static GLOBAL_SERVER_TABLE_DATA: Lazy<Arc<DashMap<String, TaskServer>>> = Lazy::new(|| {
    let server_table_data = Arc::new(DashMap::<String, TaskServer>::new());
    server_table_data
});

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NewServerPop {
    pub show: bool,
    input_name: String,
    input_url: String,
    /// Position of cursor in the editor area.
    name_char_index: usize,
    name_cursor_index: usize,
    url_char_index: usize,
    url_cursor_index: usize,
    alert_msg: String,
    selected_input: SelectedInput,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, FromRepr, EnumCount)]
pub enum SelectedInput {
    #[default]
    Name,
    Url,
}

impl NewServerPop {
    pub fn clear(&mut self) {
        self.input_name = "".to_string();
        self.input_url = "".to_string();
        self.name_char_index = 0;
        self.url_char_index = 0;
        self.name_cursor_index = 0;
        self.url_cursor_index = 0;
        self.alert_msg = "".to_string();
        self.selected_input = SelectedInput::default();
    }

    pub fn select_input(&mut self) {
        let current_index = self.selected_input as usize;
        let mut next_index = current_index.saturating_add(1);
        let count = SelectedInput::COUNT;
        if next_index.eq(&count) {
            next_index = 0;
        }
        self.selected_input =
            SelectedInput::from_repr(next_index).unwrap_or(SelectedInput::default());
    }
    pub fn move_cursor_left(&mut self) {
        let (move_left_idx, move_left_str) = match self.selected_input {
            SelectedInput::Name => (self.name_char_index, self.input_name.clone()),
            SelectedInput::Url => (self.url_char_index, self.input_url.clone()),
        };
        let chars_size: Vec<usize> = move_left_str
            .char_indices()
            .map(|(_, c)| {
                let char_size = match UnicodeWidthChar::width(c) {
                    Some(s) => s,
                    None => 0,
                };
                char_size
            })
            .collect::<Vec<_>>();

        if move_left_idx > 0 {
            let move_left = match chars_size.get(move_left_idx - 1) {
                Some(s) => *s,
                None => 0,
            };
            match self.selected_input {
                SelectedInput::Name => {
                    // let cursor_moved_left = self.name_index.saturating_sub(1);
                    // self.name_index = self.clamp_cursor(cursor_moved_left);
                    (self.name_char_index, self.name_cursor_index) = self.clamp_cursor(
                        self.name_char_index.saturating_sub(1),
                        self.name_cursor_index.sub(move_left),
                    );
                }
                SelectedInput::Url => {
                    // let cursor_moved_left = self.url_char_index.saturating_sub(1);
                    // self.url_char_index = self.clamp_cursor(cursor_moved_left);
                    (self.url_char_index, self.url_cursor_index) = self.clamp_cursor(
                        self.url_char_index.saturating_sub(1),
                        self.url_cursor_index.sub(move_left),
                    );
                }
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        let (move_right_idx, move_right_str) = match self.selected_input {
            SelectedInput::Name => (self.name_char_index, self.input_name.clone()),
            SelectedInput::Url => (self.url_char_index, self.input_url.clone()),
        };
        let chars_size: Vec<usize> = move_right_str
            .char_indices()
            .map(|(_, c)| {
                let char_size = match UnicodeWidthChar::width(c) {
                    Some(s) => s,
                    None => 0,
                };
                char_size
            })
            .collect::<Vec<_>>();
        let move_right = match chars_size.get(move_right_idx) {
            Some(s) => *s,
            None => 1,
        };
        match self.selected_input {
            SelectedInput::Name => {
                // let cursor_moved_right = self.name_index.saturating_add(1);
                // self.name_index = self.clamp_cursor(cursor_moved_right);
                (self.name_char_index, self.name_cursor_index) = self.clamp_cursor(
                    self.name_char_index.saturating_add(1),
                    self.name_cursor_index.saturating_add(move_right),
                );
            }
            SelectedInput::Url => {
                // let cursor_moved_right = self.url_char_index.saturating_add(1);
                // self.url_index = self.clamp_cursor(cursor_moved_right);
                // self.url_char_index = self.clamp_cursor(move_right);
                (self.url_char_index, self.url_cursor_index) = self.clamp_cursor(
                    self.url_char_index.saturating_add(1),
                    self.url_cursor_index.saturating_add(move_right),
                );
            }
        }
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        log::info!("byte_index:{}", index);
        match self.selected_input {
            SelectedInput::Name => self.input_name.insert(index, new_char),
            SelectedInput::Url => self.input_url.insert(index, new_char),
        }
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&mut self) -> usize {
        match self.selected_input {
            SelectedInput::Name => self
                .input_name
                .char_indices()
                .map(|(i, _)| i)
                .nth(self.name_char_index)
                .unwrap_or(self.input_name.len()),
            SelectedInput::Url => self
                .input_url
                .char_indices()
                .map(|(i, _)| i)
                // .nth(self.url_char_index)
                .nth(self.url_char_index)
                .unwrap_or(self.input_url.len()),
        }
    }

    pub fn delete_char(&mut self) {
        match self.selected_input {
            SelectedInput::Name => {
                let is_not_cursor_leftmost = self.name_char_index != 0;
                if is_not_cursor_leftmost {
                    // Method "remove" is not used on the saved text for deleting the selected char.
                    // Reason: Using remove on String works on bytes instead of the chars.
                    // Using remove would require special care because of char boundaries.

                    let current_index = self.name_char_index;
                    let from_left_to_current_index = current_index - 1;

                    // Getting all characters before the selected character.
                    let before_char_to_delete =
                        self.input_name.chars().take(from_left_to_current_index);
                    // Getting all characters after selected character.
                    let after_char_to_delete = self.input_name.chars().skip(current_index);

                    // Put all characters together except the selected one.
                    // By leaving the selected one out, it is forgotten and therefore deleted.
                    self.input_name = before_char_to_delete.chain(after_char_to_delete).collect();
                    self.move_cursor_left();
                }
            }
            SelectedInput::Url => {
                let is_not_cursor_leftmost = self.url_char_index != 0;
                if is_not_cursor_leftmost {
                    let current_index = self.url_char_index;
                    let from_left_to_current_index = current_index - 1;
                    let before_char_to_delete =
                        self.input_url.chars().take(from_left_to_current_index);
                    let after_char_to_delete = self.input_url.chars().skip(current_index);
                    self.input_url = before_char_to_delete.chain(after_char_to_delete).collect();
                    self.move_cursor_left();
                }
            }
        }
    }

    fn clamp_cursor(&self, index_pose: usize, new_cursor_pos: usize) -> (usize, usize) {
        let clamp_str = match self.selected_input {
            SelectedInput::Name => self.input_name.clone(),
            SelectedInput::Url => self.input_url.clone(),
        };
        let mut max = 0;
        let chars_size: Vec<usize> = clamp_str
            .char_indices()
            .map(|(_, c)| {
                let char_size = match UnicodeWidthChar::width(c) {
                    Some(s) => s,
                    None => 0,
                };
                max += char_size;
                char_size
            })
            .collect::<Vec<_>>();
        let idx_clamp = match self.selected_input {
            SelectedInput::Name => index_pose.clamp(0, self.input_name.chars().count()),
            SelectedInput::Url => index_pose.clamp(0, self.input_url.chars().count()),
        };
        (idx_clamp, new_cursor_pos.clamp(0, max))
        // match self.selected_input {
        //     SelectedInput::Name => new_cursor_pos.clamp(0, self.input_name.chars().count()),
        //     SelectedInput::Url => new_cursor_pos.clamp(0, self.input_url.chars().count()),
        // }
    }

    fn reset_cursor(&mut self) {
        match self.selected_input {
            SelectedInput::Name => self.name_char_index = 0,
            SelectedInput::Url => self.url_char_index = 0,
        }
    }

    pub fn add_server(&mut self) {
        // self.reset_cursor();
        let task_server = TaskServer {
            name: self.input_name.clone(),
            url: self.input_url.clone(),
        };
        self.clear();
        match save_task_server_to_cf(&task_server) {
            Ok(id) => self.alert_msg = format!("save task {} ok", id),
            Err(e) => {
                log::error!("{:}", e);
                self.alert_msg = "add server error".to_string()
            }
        };
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ServerTab {
    row_index: usize,
    server_ids: Vec<String>,
    colors: TableColors,
    color_index: usize,
    longest_item_lens: (u16, u16, u16),
    pub new_server: NewServerPop,
    // sender: Arc<Sender<String>>,
}

impl ServerTab {
    /// Select the previous item in the ingredients list (with wrap around)
    pub fn prev(&mut self) {
        self.refresh_data();
        match GLOBAL_SERVER_TABLE_DATA.len().eq(&0) {
            true => self.row_index = 0,
            false => {
                self.row_index = self
                    .row_index
                    .saturating_add(GLOBAL_SERVER_TABLE_DATA.len() - 1)
                    % GLOBAL_SERVER_TABLE_DATA.len()
            }
        }
    }

    /// Select the next item in the ingredients list (with wrap around)
    pub fn next(&mut self) {
        self.refresh_data();
        match GLOBAL_SERVER_TABLE_DATA.len().eq(&0) {
            true => self.row_index = 0,
            false => {
                self.row_index = self.row_index.saturating_add(1) % GLOBAL_SERVER_TABLE_DATA.len()
            }
        };
    }

    pub fn new_server(&mut self) {
        self.new_server.show = !self.new_server.show;
    }

    pub fn delete_server(&mut self) {
        match self.server_ids.get(self.row_index) {
            Some(id) => {
                let _ = remove_server_from_cf(id);
                self.refresh_data();
            }
            None => {}
        };
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    pub fn refresh_data(&mut self) {
        let vec_taskserver = match list_servers_from_cf() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                let v = Vec::<(String, TaskServer)>::new();
                v
            }
        };
        GLOBAL_SERVER_TABLE_DATA.clear();
        let mut id_len = 2;
        let mut name_len = 0;
        let mut url_len = 0;

        for (id, task_server) in vec_taskserver {
            GLOBAL_SERVER_TABLE_DATA.insert(id, task_server);
        }
        GLOBAL_SERVER_TABLE_DATA.shrink_to_fit();

        let data_iter = GLOBAL_SERVER_TABLE_DATA.iter();

        self.server_ids = data_iter
            .map(|data| {
                if UnicodeWidthStr::width(data.key().as_str()).gt(&id_len) {
                    id_len = UnicodeWidthStr::width(data.key().as_str())
                }

                if UnicodeWidthStr::width(data.value().name.as_str()).gt(&name_len) {
                    name_len = UnicodeWidthStr::width(data.value().name.as_str())
                }

                if UnicodeWidthStr::width(data.value().url.as_str()).gt(&url_len) {
                    url_len = UnicodeWidthStr::width(data.value().url.as_str())
                }
                data.key().to_string()
            })
            .collect::<Vec<String>>();

        // self.server_ids = vec_ids;
        let id_len_u16 = id_len.try_into().unwrap();
        let name_len_u16 = name_len.try_into().unwrap();
        let url_len_u16 = url_len.try_into().unwrap();
        self.longest_item_lens = (id_len_u16, name_len_u16, url_len_u16)
    }
}

impl Widget for ServerTab {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        self.set_colors();
        // RgbSwatch.render(area, buf);
        let area = area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        });
        Clear.render(area, buf);
        let scrollbar_area = Rect {
            y: area.y + 2,
            height: area.height - 3,
            ..area
        };
        render_scrollbar(self.row_index, scrollbar_area, buf);
        render_server_table(&self, area, buf);
    }
}

fn render_server_table(server_tab: &ServerTab, area: Rect, buf: &mut Buffer) {
    let mut state = TableState::default().with_selected(Some(server_tab.row_index));
    // let rows = INGREDIENTS.iter().copied();
    // let theme = THEME.recipe;

    let header_style = Style::default()
        .fg(server_tab.colors.header_fg)
        .bg(server_tab.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(server_tab.colors.selected_style_fg);

    let header = ["Id", "Name", "Url"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let data_iter = GLOBAL_SERVER_TABLE_DATA.iter();
    let rows = data_iter.enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => server_tab.colors.normal_row_color,
            _ => server_tab.colors.alt_row_color,
        };

        let item = vec![
            data.key().to_string(),
            data.value().name.clone(),
            data.value().url.clone(),
        ];
        item.into_iter()
            .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
            .collect::<Row>()
            .style(Style::new().fg(server_tab.colors.row_fg).bg(color))
            .height(4)
    });
    let bar = " █ ";

    let table = Table::new(
        rows,
        [
            // + 1 is for padding.
            Constraint::Length(server_tab.longest_item_lens.0 + 1),
            Constraint::Min(server_tab.longest_item_lens.1 + 1),
            Constraint::Min(server_tab.longest_item_lens.2),
        ],
    )
    .header(header)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]))
    .bg(server_tab.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);
    StatefulWidget::render(table, area, buf, &mut state);
}

fn render_scrollbar(position: usize, area: Rect, buf: &mut Buffer) {
    let mut state = ScrollbarState::default()
        .content_length(GLOBAL_SERVER_TABLE_DATA.len())
        .viewport_content_length(6)
        .position(position);
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▐")
        .render(area, buf, &mut state);
}

pub fn new_server_pop_ui(f: &mut Frame, pop: &NewServerPop) {
    let input_area = centered_rect(60, 30, f.size());
    let vertical = Layout::vertical([
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ]);
    let [help_area, name_area, url_area, allert_area] = vertical.areas(input_area);
    f.render_widget(Clear, input_area);

    let help = Text::from("input name and url,pass 'Enter' key to add server").centered();
    f.render_widget(help, help_area);
    let input_name = Paragraph::new(pop.input_name.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::bordered().title_top(Line::from("Name").right_aligned()));
    f.render_widget(input_name, name_area);

    let input_url = Paragraph::new(pop.input_url.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::bordered().title_top(Line::from("Url").right_aligned()));
    f.render_widget(input_url, url_area);

    match pop.selected_input {
        SelectedInput::Name => {
            f.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                // name_area.x + pop.name_char_index as u16 + 1,
                name_area.x + pop.name_cursor_index as u16 + 1,
                // Move one line down, from the border to the input line
                name_area.y + 1,
            );
        }
        SelectedInput::Url => {
            f.set_cursor(url_area.x + pop.url_cursor_index as u16 + 1, url_area.y + 1);
        }
    }

    // let message_block = Block::new().borders(Borders::NONE);
    let alert_msg = Text::from(pop.alert_msg.as_str()).centered();
    f.render_widget(alert_msg, allert_area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
