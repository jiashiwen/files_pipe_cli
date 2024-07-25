use std::sync::Arc;

use crate::commons::struct_to_json_string_prettry;
use crate::request::{
    template_transfer_local2local, template_transfer_local2oss, template_transfer_oss2local,
    template_transfer_oss2oss, GLOBAL_RUNTIME,
};
use crate::tui::tabs::centered_rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Clear, Widget},
};
use strum::{Display, EnumCount, EnumIter, FromRepr, IntoEnumIterator};

//     let new_server_text_area = Arc::new(RwLock::new(NewServerTextArea::default()));
//     new_server_text_area
// });

#[derive(Debug, Display, Default, Copy, Clone, EnumIter, FromRepr, EnumCount, PartialEq, Eq)]
pub enum TrasnferTemplate {
    #[default]
    Oss2Oss,
    Oss2Local,
    Local2Oss,
    Local2Local,
}

impl TrasnferTemplate {
    fn prev(self) -> Self {
        let current_index = self as usize;
        let prev_index = current_index.saturating_sub(1);
        Self::from_repr(prev_index).unwrap_or(self)
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let mut next_index = current_index.saturating_add(1);
        let count = Self::COUNT;
        if next_index.eq(&count) {
            next_index = 0;
        }

        Self::from_repr(next_index).unwrap_or(self)
    }

    fn name(&self) -> String {
        match self {
            template => format!(" {template} "),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PopSelectTemplate {
    pub show: bool,
    pub selected_template: TrasnferTemplate,
    pub template: String,
}

impl PopSelectTemplate {
    pub fn show_pop(&mut self) {
        self.show = !self.show
    }

    pub fn clean(&mut self) {
        self.selected_template = TrasnferTemplate::default();
        self.template = String::default();
    }

    pub fn prev(&mut self) {
        self.selected_template = self.selected_template.prev();
    }

    pub fn next(&mut self) {
        self.selected_template = self.selected_template.next();
    }

    pub fn load_template(&mut self) {
        let template_type = self.selected_template.clone();
        let mut template_str = Arc::new("".to_string());
        let template_str_mut = Arc::get_mut(&mut template_str).unwrap();
        GLOBAL_RUNTIME.block_on(async move {
            let resp_task = match template_type {
                TrasnferTemplate::Oss2Oss => template_transfer_oss2oss().await,
                TrasnferTemplate::Oss2Local => template_transfer_oss2local().await,
                TrasnferTemplate::Local2Oss => template_transfer_local2oss().await,
                TrasnferTemplate::Local2Local => template_transfer_local2local().await,
            };
            let task = match resp_task {
                Ok(t) => t,
                Err(e) => {
                    log::error!("{:?}", e);
                    return;
                }
            };

            let task = match task.data {
                Some(t) => t,
                None => {
                    return;
                }
            };
            let task_json = match struct_to_json_string_prettry(&task) {
                Ok(j) => j,
                Err(e) => {
                    log::error!("{:?}", e);
                    return;
                }
            };
            *template_str_mut = task_json
        });

        self.template = template_str.to_string();
    }
}

impl Widget for PopSelectTemplate {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let template_area = centered_rect(30, 30, area);
        let vertical = Layout::vertical([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(template_area);

        Clear.render(template_area, buf);

        for (i, template) in TrasnferTemplate::iter().enumerate() {
            let paragraph = Paragraph::new(template.name().dark_gray())
                .centered()
                .wrap(Wrap { trim: true });
            let block = match template.eq(&self.selected_template) {
                true => Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Blue)),
                false => Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default()),
            };
            paragraph.block(block).render(vertical[i], buf)
        }
    }
}

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, FromRepr, EnumCount)]
// pub enum SelectedInput {
//     #[default]
//     Name,
//     Url,
// }
