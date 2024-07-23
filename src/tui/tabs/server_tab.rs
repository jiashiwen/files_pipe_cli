use crate::{
    request::{set_current_server, TaskServer, GLOBAL_CURRENT_SERVER, GLOBAL_RUNTIME},
    resources::{list_servers_from_cf, remove_server_from_cf},
    tui::THEME,
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, Cell, Clear, HighlightSpacing, Padding, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, StatefulWidget, Table, TableState, Widget,
    },
};
use std::{borrow::BorrowMut, sync::Arc};
use unicode_width::UnicodeWidthStr;

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerTab {
    row_index: usize,
    server_ids: Vec<String>,
    colors: TableColors,
    color_index: usize,
    longest_item_lens: (u16, u16, u16),
    current_server: TaskServer,
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

    pub fn delete_server(&mut self) {
        match self.server_ids.get(self.row_index) {
            Some(id) => {
                let _ = remove_server_from_cf(id);
                self.refresh_data();
            }
            None => {}
        };
    }

    pub fn set_current_server(&mut self) {
        match self.server_ids.get(self.row_index) {
            Some(id) => {
                let _ = set_current_server(&id);
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

        let id_len_u16 = id_len.try_into().unwrap();
        let name_len_u16 = name_len.try_into().unwrap();
        let url_len_u16 = url_len.try_into().unwrap();
        self.longest_item_lens = (id_len_u16, name_len_u16, url_len_u16);

        let global_current_server = GLOBAL_CURRENT_SERVER.read().unwrap();
        self.current_server = global_current_server.clone();
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

        let current_server = format!("Current Server: {}", self.current_server.url);

        Block::new()
            .title(current_server.bold().white())
            .title_alignment(Alignment::Center)
            .style(THEME.content)
            .padding(Padding::new(1, 1, 2, 1))
            .render(area, buf);

        let scrollbar_area = Rect {
            y: area.y + 2,
            height: area.height - 2,
            ..area
        };

        let table_area = Rect {
            y: area.y + 2,
            height: area.height - 2,
            ..area
        };
        render_scrollbar(self.row_index, scrollbar_area, buf);
        render_server_table(&self, table_area, buf);
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
            .height(3)
        // .height(4)
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
        // bar.into(),
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
