use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{
    Clear, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Wrap,
};
use ratatui::{
    style::Style,
    symbols,
    widgets::{Block, Paragraph, Widget},
};

use crate::tui::tabs::centered_rect;

const HELP: &'static [&'static str] = &[
    "TAB: select tab",
    "Q/Esc: quit",
    "F1:help",
    "",
    "Server",
    "K/↑: Up",
    "J/↓: Down",
    "A: Add server",
    "D: Delete server",
    "Enter: Set server to current",
    "",
    "Task",
    "K/↑: Up",
    "J/↓: Down",
    "F: Refresh task table",
    "C: Create task",
    "E: Edit task",
    "D: Delete task",
    "R: Run task",
    "I: Interupt task",
    "S: Show task status",
];

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct PopHelp {
    pub show: bool,
    row_index: usize,
}

impl PopHelp {
    pub fn show_switch(&mut self) {
        self.show = !self.show
    }

    pub fn prev_line(&mut self) {
        match HELP.len().eq(&0) {
            true => self.row_index = 0,
            false => self.row_index = self.row_index.saturating_add(HELP.len() - 1) % HELP.len(),
        }
    }

    pub fn next_line(&mut self) {
        match HELP.len().eq(&0) {
            true => self.row_index = 0,
            false => self.row_index = self.row_index.saturating_add(1) % HELP.len(),
        };
    }
}

impl Widget for PopHelp {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.show {
            let help_area = centered_rect(30, 30, area);
            Clear.render(help_area, buf);

            render_pop_help(&self, help_area, buf);
            render_scrollbar(self.row_index, help_area, buf);
        }
    }
}

fn render_pop_help(pop_help: &PopHelp, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .border_set(symbols::border::ROUNDED)
        .style(Style::new().gray());

    let mut lines = vec![];
    for item in HELP.iter() {
        let line = Line::from(item.to_string());
        lines.push(line);
    }
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(block)
        .scroll((pop_help.row_index as u16, 0));
    paragraph.render(area, buf)
}

fn render_scrollbar(position: usize, area: Rect, buf: &mut Buffer) {
    let mut state = ScrollbarState::default()
        .content_length(HELP.len())
        .viewport_content_length(6)
        .position(position);
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▐")
        .render(area, buf, &mut state);
}
