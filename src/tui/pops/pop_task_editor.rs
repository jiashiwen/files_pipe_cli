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

// impl Widget for PopTaskEditor<'a> {
//     fn render(mut self, area: Rect, buf: &mut Buffer) {
//         self.set_colors();
//         // RgbSwatch.render(area, buf);
//         let area = area.inner(Margin {
//             vertical: 1,
//             horizontal: 2,
//         });
//         Clear.render(area, buf);
//         let scrollbar_area = Rect {
//             y: area.y + 2,
//             height: area.height - 3,
//             ..area
//         };
//         render_scrollbar(self.row_index, scrollbar_area, buf);
//         render_task_table(&self, area, buf);
//     }
// }
// fn validate(textarea: &mut TextArea) -> bool {
//     if let Err(err) = textarea.lines()[0].parse::<f64>() {
//         textarea.set_style(Style::default().fg(Color::LightRed));
//         textarea.set_block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .title(format!("ERROR: {}", err)),
//         );
//         false
//     } else {
//         textarea.set_style(Style::default().fg(Color::LightGreen));
//         textarea.set_block(Block::default().borders(Borders::ALL).title("OK"));
//         true
//     }
// }

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

// pub fn pop_task_editor_ui(f: &mut Frame, pop_task_editor: &PopTaskEditor) {
//     let editor_area = centered_rect(60, 30, f.size());
//     let vertical = Layout::vertical([
//         Constraint::Percentage(10),
//         Constraint::Percentage(80),
//         Constraint::Percentage(10),
//     ]);
//     let [help_area, editor, alert_area] = vertical.areas(editor_area);
//     f.render_widget(Clear, editor_area);
//     let help = Text::from("input name and url,pass 'Enter' key to add server").centered();
//     f.render_widget(help, help_area);
//     f.render_widget(pop_task_editor.editor.widget(), editor);

//     // let message_block = Block::new().borders(Borders::NONE);
//     let alert_msg = Text::from(pop_task_editor.alert_msg.as_str()).centered();
//     f.render_widget(alert_msg, alert_area);
// }
