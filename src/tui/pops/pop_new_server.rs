use crate::tui::tabs::centered_rect;
use once_cell::sync::Lazy;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Widget},
};
use std::sync::{Arc, RwLock};
use strum::{EnumCount, EnumIter, FromRepr};
use tui_textarea::{Input, TextArea};

pub static GLOBAL_NEWSERVER_TEXT_AREA: Lazy<Arc<RwLock<NewServerTextArea>>> = Lazy::new(|| {
    let mut name = TextArea::default();
    let mut url = TextArea::default();
    name.set_block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default()),
    );
    url.set_block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray)),
    );

    let text_area = NewServerTextArea {
        name_text_area: name,
        url_text_area: url,
    };

    let new_server_text_area = Arc::new(RwLock::new(text_area));
    new_server_text_area
});

#[derive(Debug, Default, Clone)]
pub struct NewServerTextArea<'a> {
    name_text_area: TextArea<'a>,
    url_text_area: TextArea<'a>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PopNewServer {
    pub show: bool,
    alert_msg: String,
    selected_input: SelectedInput,
}

impl PopNewServer {
    pub fn show_pop(&mut self) {
        self.show = !self.show
    }

    pub fn input(&self, input: impl Into<Input>) {
        match self.selected_input {
            SelectedInput::Name => {
                GLOBAL_NEWSERVER_TEXT_AREA
                    .write()
                    .unwrap()
                    .name_text_area
                    .input(input);
            }
            SelectedInput::Url => {
                GLOBAL_NEWSERVER_TEXT_AREA
                    .write()
                    .unwrap()
                    .url_text_area
                    .input(input);
            }
        }
    }
}

impl Widget for PopNewServer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let editor_area = centered_rect(60, 30, area);
        let vertical = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
        ]);
        let [help_area, name_area, url_area, alert_area] = vertical.areas(editor_area);
        Clear.render(editor_area, buf);
        let help = Text::from("add new server,pass 'Enter' key to add server").centered();
        help.render(help_area, buf);
        GLOBAL_NEWSERVER_TEXT_AREA
            .write()
            .unwrap()
            .name_text_area
            .widget()
            .render(name_area, buf);
        GLOBAL_NEWSERVER_TEXT_AREA
            .write()
            .unwrap()
            .url_text_area
            .widget()
            .render(url_area, buf);
        let alert_msg = Text::from(self.alert_msg.as_str()).centered();
        alert_msg.render(alert_area, buf);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, FromRepr, EnumCount)]
pub enum SelectedInput {
    #[default]
    Name,
    Url,
}

impl PopNewServer {
    pub fn clear(&mut self) {}
    pub fn select_input(&mut self) {
        let current_index = self.selected_input as usize;
        let mut next_index = current_index.saturating_add(1);
        let count = SelectedInput::COUNT;
        if next_index.eq(&count) {
            next_index = 0;
        }
        self.selected_input =
            SelectedInput::from_repr(next_index).unwrap_or(SelectedInput::default());
    }
}
