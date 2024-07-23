use crate::{
    commons::struct_to_json_string_prettry,
    request::{list_all_tasks, task_remove, task_show, task_status, Task, TaskId, GLOBAL_RUNTIME},
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Cell, Clear, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState, Widget,
    },
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct TaskRow {
    id: String,
    name: String,
    task_type: String,
    status: String,
}

pub static GLOBAL_TASKS_LIST: Lazy<Arc<DashMap<String, TaskRow>>> = Lazy::new(|| {
    let server_table_data = Arc::new(DashMap::<String, TaskRow>::new());
    server_table_data
});

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TaskTab {
    row_index: usize,
    task_ids: Vec<String>,
    colors: TableColors,
    color_index: usize,
    longest_item_lens: (u16, u16, u16, u16),
}

impl TaskTab {
    /// Select the previous item in the ingredients list (with wrap around)
    pub fn prev(&mut self) {
        match GLOBAL_TASKS_LIST.len().eq(&0) {
            true => self.row_index = 0,
            false => {
                self.row_index = self.row_index.saturating_add(GLOBAL_TASKS_LIST.len() - 1)
                    % GLOBAL_TASKS_LIST.len()
            }
        }
    }

    /// Select the next item in the ingredients list (with wrap around)
    pub fn next(&mut self) {
        match GLOBAL_TASKS_LIST.len().eq(&0) {
            true => self.row_index = 0,
            false => self.row_index = self.row_index.saturating_add(1) % GLOBAL_TASKS_LIST.len(),
        };
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    // Todo 设置事件，进行测试
    pub fn get_task(&mut self) -> String {
        let task_id = self.task_ids.get(self.row_index).unwrap().to_string();
        let mut task_json = Arc::new("".to_string());
        let t_j = Arc::get_mut(&mut task_json).unwrap();
        GLOBAL_RUNTIME.block_on(async move {
            let id = TaskId { task_id };
            let resp_task = match task_show(&id).await {
                Ok(t) => t,
                Err(e) => {
                    log::error!("{:?}", e);
                    return;
                }
            };

            let t = match resp_task.data {
                Some(t) => t,
                None => {
                    return;
                }
            };

            let task_json = match struct_to_json_string_prettry(&t) {
                Ok(j) => j,
                Err(e) => {
                    log::error!("{:?}", e);
                    return;
                }
            };

            t_j.push_str(&task_json);
        });
        task_json.to_string()
    }

    pub fn delete_task(&mut self) {
        match self.task_ids.get(self.row_index) {
            Some(t) => {
                GLOBAL_RUNTIME.block_on(async move {
                    let _ = match task_remove(&TaskId { task_id: t.clone() }).await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{:?}", e);
                            return;
                        }
                    };
                });
            }
            None => {}
        };
        self.refresh_data();
    }

    pub fn refresh_data(&mut self) {
        GLOBAL_TASKS_LIST.clear();
        GLOBAL_RUNTIME.block_on(async move {
            let reps = match list_all_tasks().await {
                Ok(r) => r,
                Err(e) => {
                    log::error!("{:?}", e);
                    return;
                }
            };
            let tasks = match reps.data {
                Some(v) => v,
                None => return,
            };

            // let mut builder = Builder::default();
            for resp_task in tasks {
                let status = match task_status(&TaskId {
                    task_id: resp_task.task.task_id(),
                })
                .await
                {
                    Ok(t_s) => match t_s.data {
                        Some(s) => s.status.to_string(),
                        None => "stopped".to_string(),
                    },
                    Err(e) => {
                        log::error!("{:?}", e);
                        "stopped".to_string()
                    }
                };
                let task_row = TaskRow {
                    id: resp_task.task.task_id(),
                    name: resp_task.task.task_name(),
                    task_type: resp_task.task.task_type().to_string(),
                    status,
                };
                GLOBAL_TASKS_LIST.insert(resp_task.task.task_id(), task_row);
            }
            GLOBAL_TASKS_LIST.shrink_to_fit();
        });

        let mut id_len = 2;
        let mut name_len = 0;
        let mut type_len = 0;
        let mut status_len = 0;

        let data_iter = GLOBAL_TASKS_LIST.iter();

        self.task_ids = data_iter
            .map(|data| {
                if UnicodeWidthStr::width(data.key().as_str()).gt(&id_len) {
                    id_len = UnicodeWidthStr::width(data.key().as_str())
                }

                if UnicodeWidthStr::width(data.value().name.as_str()).gt(&name_len) {
                    name_len = UnicodeWidthStr::width(data.value().name.as_str())
                }

                if UnicodeWidthStr::width(data.value().task_type.as_str()).gt(&type_len) {
                    type_len = UnicodeWidthStr::width(data.value().task_type.as_str())
                }

                if UnicodeWidthStr::width(data.value().status.as_str()).gt(&type_len) {
                    status_len = UnicodeWidthStr::width(data.value().status.as_str())
                }
                data.key().to_string()
            })
            .collect::<Vec<String>>();

        let id_len_u16 = id_len.try_into().unwrap();
        let name_len_u16 = name_len.try_into().unwrap();
        let type_len_u16 = type_len.try_into().unwrap();
        let status_len = status_len.try_into().unwrap();
        self.longest_item_lens = (id_len_u16, name_len_u16, type_len_u16, status_len)
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
        render_task_table(&self, area, buf);
    }
}

fn render_task_table(server_tab: &TaskTab, area: Rect, buf: &mut Buffer) {
    let mut state = TableState::default().with_selected(Some(server_tab.row_index));
    // let rows = INGREDIENTS.iter().copied();
    // let theme = THEME.recipe;

    let header_style = Style::default()
        .fg(server_tab.colors.header_fg)
        .bg(server_tab.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(server_tab.colors.selected_style_fg);

    let header = ["Id", "Name", "Type", "Status"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let data_iter = GLOBAL_TASKS_LIST.iter();
    let rows = data_iter.enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => server_tab.colors.normal_row_color,
            _ => server_tab.colors.alt_row_color,
        };

        let item = vec![
            data.key().to_string(),
            data.value().name.clone(),
            data.value().task_type.clone(),
            data.value().status.clone(),
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
            Constraint::Min(server_tab.longest_item_lens.3),
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
        .content_length(GLOBAL_TASKS_LIST.len())
        .viewport_content_length(6)
        .position(position);
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▐")
        .render(area, buf, &mut state);
}
