use std::sync::{Arc, RwLock};

use crate::{commons::json_to_struct, request::Task, tui::tabs::centered_rect};
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

    pub fn show_editer_with_text(&mut self, text: Vec<String>) {
        self.show = !self.show;
        let mut text_area = GLOBAL_TASK_EDITOR.write().unwrap();
        for str in text {
            for char in str.chars() {
                text_area.insert_char(char);
                text_area.insert_newline();
            }
        }
    }

    pub fn create_task(&mut self, task_id: &str) {
        let mut task_json = "".to_string();
        for line in GLOBAL_TASK_EDITOR.read().unwrap().lines() {
            task_json.push_str(line);
        }
        let task = match json_to_struct::<Task>(&task_json) {
            Ok(t) => t,
            Err(e) => {
                log::error!("{:?}", e);
                self.alert_msg = "task create err".to_string();
                return;
            }
        };

        log::info!("{:?}", task);
        // GLOBAL_RUNTIME.block_on(async move {
        //     let task = match json_to_struct::<Task>(&task_json) {
        //         Ok(t) => t,
        //         Err(e) => {
        //             log::error!("{}", e);
        //             return;
        //         }
        //     };

        //     let task = match task_create(&task).await {
        //         Ok(t) => t,
        //         Err(e) => {
        //             log::error!("{:?}", e);
        //             return;
        //         }
        //     };

        //     let task = match task.data {
        //         Some(t) => t,
        //         None => {
        //             return;
        //         }
        //     };
        //     // println!("task {} created", task.task_id.as_str());
        // });
    }
}

impl Widget for PopTaskEditor {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let editor_area = centered_rect(60, 70, area);
        let vertical = Layout::vertical([
            // Constraint::Percentage(10),
            Constraint::Length(2),
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
