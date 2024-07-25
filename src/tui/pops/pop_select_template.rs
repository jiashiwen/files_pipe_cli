use crate::{request::TaskServer, resources::save_task_server_to_cf, tui::tabs::centered_rect};
use color_eyre::owo_colors::colors::Black;
use color_eyre::owo_colors::OwoColorize;
use itertools::Itertools;
use ratatui::style::{Color, Style, Styled, Stylize};
use ratatui::text::Text;
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

    pub fn select_template(&mut self) {
        self.selected_template = self.selected_template.next();
    }
}

impl Widget for PopSelectTemplate {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let template_area = centered_rect(50, 60, area);
        let vertical = Layout::vertical([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(template_area);
        // let [oss2oss_area, oss2local_area, local2oss_area, local2local_area] =
        // vertical.areas(template_area);

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

        // let oss2oss = Paragraph::new("oss2oss".dark_gray()).wrap(Wrap { trim: true });

        // let mut oss2oss_block = Block::default()
        //     .borders(Borders::ALL)
        //     .style(Style::default());
        // let mut oss2local_block = Block::default()
        //     .borders(Borders::ALL)
        //     .style(Style::default());
        // let mut local2oss_block = Block::default()
        //     .borders(Borders::ALL)
        //     .style(Style::default());
        // let mut local2local_block = Block::default()
        //     .borders(Borders::ALL)
        //     .style(Style::default());

        // match self.selected_template {
        //     TrasnferTemplate::Oss2Oss => {
        //         oss2oss_block = oss2oss_block.set_style(Style::default().bg(Color::Blue))
        //     }
        //     TrasnferTemplate::Oss2Local => {
        //         oss2local_block = oss2local_block.set_style(Style::default().bg(Color::Blue))
        //     }
        //     TrasnferTemplate::Local2Oss => {
        //         local2oss_block = local2oss_block.set_style(Style::default().bg(Color::Blue))
        //     }
        //     TrasnferTemplate::Local2Local => {
        //         local2local_block = local2local_block.set_style(Style::default().bg(Color::Blue))
        //     }
        // };
        // oss2oss_block.render(oss2oss_area, buf);
        // oss2local_block.render(oss2local_area, buf);
        // local2oss_block.render(local2oss_area, buf);
        // local2local_block.render(local2local_area, buf);
    }
}

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, FromRepr, EnumCount)]
// pub enum SelectedInput {
//     #[default]
//     Name,
//     Url,
// }
