use crate::{
    request::TaskServer,
    resources::{list_servers_from_cf, remove_server_from_cf},
    tui::THEME,
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Cell, Clear, HighlightSpacing, List, ListItem, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget,
    },
    Frame,
};
use std::sync::Arc;
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NewServerPop {
    pub show: bool,
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    // input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

impl NewServerPop {
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }
}

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ServerTab {
    row_index: usize,
    server_ids: Vec<String>,
    colors: TableColors,
    color_index: usize,
    longest_item_lens: (u16, u16, u16),
    pub new_server: NewServerPop,
}

impl ServerTab {
    /// Select the previous item in the ingredients list (with wrap around)
    pub fn prev(&mut self) {
        self.flush_data();
        // self.row_index = self.row_index.saturating_add(INGREDIENTS.len() - 1) % INGREDIENTS.len();
        let table_len = match GLOBAL_SERVER_TABLE_DATA.len().eq(&0) {
            true => 1,
            false => GLOBAL_SERVER_TABLE_DATA.len(),
        };
        self.row_index = self.row_index.saturating_add(table_len - 1) % table_len;
    }

    /// Select the next item in the ingredients list (with wrap around)
    pub fn next(&mut self) {
        self.flush_data();
        // self.row_index = self.row_index.saturating_add(1) % INGREDIENTS.len();
        let table_len = match GLOBAL_SERVER_TABLE_DATA.len().eq(&0) {
            true => 1,
            false => GLOBAL_SERVER_TABLE_DATA.len(),
        };
        self.row_index = self.row_index.saturating_add(1) % table_len;
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
        let mut vec_ids = vec![];
        for (id, task_server) in vec_taskserver {
            if UnicodeWidthStr::width(id.as_str()).gt(&id_len) {
                id_len = UnicodeWidthStr::width(id.as_str())
            }

            if UnicodeWidthStr::width(task_server.name.as_str()).gt(&name_len) {
                name_len = UnicodeWidthStr::width(task_server.name.as_str())
            }

            if UnicodeWidthStr::width(task_server.url.as_str()).gt(&url_len) {
                url_len = UnicodeWidthStr::width(task_server.url.as_str())
            }

            vec_ids.push(id.clone());
            GLOBAL_SERVER_TABLE_DATA.insert(id, task_server);
        }
        GLOBAL_SERVER_TABLE_DATA.shrink_to_fit();
        self.server_ids = vec_ids;
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
        // if self.new_server.show {
        //     render_new_server_pop(&self.new_server, area, buf);
        // }
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

// fn render_new_server_pop(new_server: &NewServerPop, area: Rect, buf: &mut Buffer) {
//     // let block = Block::bordered().title("New Server");
//     let input_area = centered_rect(60, 20, area);
//     let input = Paragraph::new(new_server.input.as_str())
//         .style(Style::default().fg(Color::Yellow))
//         .block(Block::bordered().title("New Server"));
//     // f.render_widget(input, input_area);
//     Widget::render(Clear, input_area, buf); //this clears out the background
//     Widget::render(input, input_area, buf);

//     // Widget::set_cursor(
//     //     // Draw the cursor at the current position in the input field.
//     //     // This position is can be controlled via the left and right arrow key
//     //     input_area.x + new_server.character_index as u16 + 1,
//     //     // Move one line down, from the border to the input line
//     //     input_area.y + 1,
//     // );
// }

pub fn new_server_pop_ui(f: &mut Frame, pop: &NewServerPop) {
    // let vertical = Layout::vertical([
    //     Constraint::Length(1),
    //     Constraint::Length(3),
    //     Constraint::Min(1),
    // ]);
    // let [help_area, input_area, messages_area] = vertical.areas(f.size());

    // let (msg, style) = match pop.input_mode {
    //     InputMode::Normal => (
    //         vec![
    //             "Press ".into(),
    //             "q".bold(),
    //             " to exit, ".into(),
    //             "e".bold(),
    //             " to start editing.".bold(),
    //         ],
    //         Style::default().add_modifier(Modifier::RAPID_BLINK),
    //     ),
    //     InputMode::Editing => (
    //         vec![
    //             "Press ".into(),
    //             "Esc".bold(),
    //             " to stop editing, ".into(),
    //             "Enter".bold(),
    //             " to record the message".into(),
    //         ],
    //         Style::default(),
    //     ),
    // };
    // let text = Text::from(Line::from(msg)).patch_style(style);
    // let help_message = Paragraph::new(text);
    // f.render_widget(help_message, help_area);

    let input_area = centered_rect(60, 20, f.size());
    // let input = Paragraph::new(new_server.input.as_str())
    //     .style(Style::default().fg(Color::Yellow))
    //     .block(Block::bordered().title("New Server"));
    // f.render_widget(input, input_area);
    // Widget::render(Clear, input_area, buf); //this clears out the background
    // Widget::render(input, input_area, buf);

    let input = Paragraph::new(pop.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::bordered().title("Input"));
    f.render_widget(Clear, input_area);
    f.render_widget(input, input_area);
    f.set_cursor(
        // Draw the cursor at the current position in the input field.
        // This position is can be controlled via the left and right arrow key
        input_area.x + pop.character_index as u16 + 1,
        // Move one line down, from the border to the input line
        input_area.y + 1,
    );
    // match pop.input_mode {
    //     InputMode::Normal =>
    //         // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
    //         {}

    //     InputMode::Editing => {
    //         // Make the cursor visible and ask ratatui to put it at the specified coordinates after
    //         // rendering
    //         #[allow(clippy::cast_possible_truncation)]
    //         f.set_cursor(
    //             // Draw the cursor at the current position in the input field.
    //             // This position is can be controlled via the left and right arrow key
    //             input_area.x + pop.character_index as u16 + 1,
    //             // Move one line down, from the border to the input line
    //             input_area.y + 1,
    //         );
    //     }
    // }

    let messages: Vec<ListItem> = pop
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = Line::from(Span::raw(format!("{i}: {m}")));
            ListItem::new(content)
        })
        .collect();
    let messages = List::new(messages).block(Block::bordered().title("Messages"));
    // f.render_widget(messages, messages_area);
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
