use ratatui::prelude::*;
use ratatui::{
    style::Style,
    symbols,
    widgets::{Block, Paragraph, Widget},
};

use crate::tui::tabs::centered_rect;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PopAlert {
    pub show: bool,
    alert_msg: String,
}

impl PopAlert {
    pub fn alert_switch(&mut self) {
        self.show = !self.show
    }

    pub fn set_alert_msg(&mut self, msg: &str) {
        self.alert_msg = msg.to_string()
    }
}

impl Widget for PopAlert {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.show {
            render_pop_alert(&self, area, buf)
        }
    }
}

fn render_pop_alert(pop_alert: &PopAlert, area: Rect, buf: &mut Buffer) {
    let template_area = centered_rect(30, 30, area);
    let block = Block::bordered()
        .border_set(symbols::border::DOUBLE)
        .style(Style::new().green());
    let paragraph = Paragraph::new(pop_alert.alert_msg.to_string())
        .centered()
        .block(block);
    paragraph.render(template_area, buf)
}
