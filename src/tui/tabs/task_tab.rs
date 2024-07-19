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
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, HighlightSpacing, List, ListItem, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget,
    },
    Frame,
};
use std::{ops::Sub, sync::Arc};
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
pub struct TaskTab {
    row_index: usize,
    server_ids: Vec<String>,
    colors: TableColors,
    color_index: usize,
    longest_item_lens: (u16, u16, u16),
}

impl TaskTab {
    /// Select the previous item in the ingredients list (with wrap around)
    pub fn prev(&mut self) {
        self.flush_data();
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
        self.flush_data();
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
                self.flush_data();
            }
            None => {}
        };
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    pub fn flush_data(&mut self) {
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

impl Widget for TaskTab {
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

fn render_server_table(server_tab: &TaskTab, area: Rect, buf: &mut Buffer) {
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
