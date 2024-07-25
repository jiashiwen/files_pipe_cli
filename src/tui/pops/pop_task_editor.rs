use crate::{
    commons::json_to_struct,
    request::{task_create, Task, GLOBAL_RUNTIME},
    tui::tabs::centered_rect,
};
use anyhow::anyhow;
use anyhow::Result;
use once_cell::sync::Lazy;
use ratatui::{
    layout::{Constraint, Layout},
    text::Text,
    widgets::{Clear, Widget},
};
use std::sync::{Arc, RwLock};
use tui_textarea::TextArea;

use super::PopSelectTemplate;

pub static GLOBAL_TASK_EDITOR: Lazy<Arc<RwLock<TextArea>>> = Lazy::new(|| {
    let task_editor = Arc::new(RwLock::new(TextArea::default()));
    task_editor
});

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PopTaskEditor {
    pub show: bool,
    pub alert_msg: String,
    pub pop_select_template: PopSelectTemplate,
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
            }
            text_area.insert_newline();
        }
    }

    pub fn create_task(&mut self) -> Result<String> {
        let mut is_error = Arc::new(false);
        let is_error_mut = Arc::get_mut(&mut is_error).unwrap();
        let mut task_id = Arc::new("".to_string());
        let task_id_mut = Arc::get_mut(&mut task_id).unwrap();

        let mut task_json = "".to_string();
        for line in GLOBAL_TASK_EDITOR.read().unwrap().lines() {
            task_json.push_str(line);
        }

        GLOBAL_RUNTIME.block_on(async move {
            let task = match json_to_struct::<Task>(&task_json) {
                Ok(t) => t,
                Err(e) => {
                    log::error!("{}", e);
                    *is_error_mut = true;
                    return;
                }
            };

            let task = match task_create(&task).await {
                Ok(t) => t,
                Err(e) => {
                    log::error!("{:?}", e);
                    *is_error_mut = true;
                    return;
                }
            };

            let task = match task.data {
                Some(t) => t,
                None => {
                    *is_error_mut = true;
                    return;
                }
            };

            *task_id_mut = task.task_id;
        });
        if *is_error {
            return Err(anyhow!("create task error"));
        }

        Ok(task_id.to_string())
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
        let help = Text::from("Pass F10 to add task").centered();
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
