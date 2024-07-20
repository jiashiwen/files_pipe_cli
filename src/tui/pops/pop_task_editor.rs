use std::sync::{Arc, RwLock};

use crate::tui::tabs::centered_rect;
use once_cell::sync::Lazy;
use ratatui::{
    layout::{Constraint, Layout},
    text::Text,
    widgets::{Clear, Widget},
};
use tui_textarea::TextArea;

pub static GLOBAL_TASK_EDITOR: Lazy<Arc<RwLock<TextArea>>> = Lazy::new(|| {
    let task_editor = Arc::new(RwLock::new(TextArea::default()));
    task_editor
});

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PopTaskEditor {
    pub show: bool,
    pub alert_msg: String,
}

impl PopTaskEditor {
    pub fn show_editor(&mut self) {
        self.show = !self.show
    }
}

impl Widget for PopTaskEditor {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let editor_area = centered_rect(60, 30, area);
        let vertical = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ]);
        let [help_area, input_area, alert_area] = vertical.areas(editor_area);
        Clear.render(editor_area, buf);
        let help = Text::from("input task json,pass 'Enter' key to add task").centered();
        help.render(help_area, buf);
        GLOBAL_TASK_EDITOR
            .read()
            .unwrap()
            .widget()
            .render(input_area, buf);
        let alert_msg = Text::from(self.alert_msg.as_str()).centered();
        alert_msg.render(alert_area, buf);
    }
}
