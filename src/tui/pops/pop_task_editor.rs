use crate::tui::tabs::centered_rect;
use ratatui::{
    layout::{Constraint, Layout},
    text::Text,
    widgets::Clear,
    Frame,
};
use tui_textarea::TextArea;

#[derive(Debug, Default, Clone)]
pub struct PopTaskEditor<'a> {
    pub show: bool,
    pub alert_msg: String,
    pub editor: TextArea<'a>,
}

impl<'a> PopTaskEditor<'a> {
    pub fn show(&mut self) {
        self.show = !self.show
    }
}

pub fn pop_task_editor_ui(f: &mut Frame, pop_task_editor: &PopTaskEditor) {
    let editor_area = centered_rect(60, 30, f.size());
    let vertical = Layout::vertical([
        Constraint::Percentage(10),
        Constraint::Percentage(80),
        Constraint::Percentage(10),
    ]);
    let [help_area, editor, alert_area] = vertical.areas(editor_area);
    f.render_widget(Clear, editor_area);

    let help = Text::from("input name and url,pass 'Enter' key to add server").centered();
    f.render_widget(help, help_area);

    // let input_name = Paragraph::new(pop_task_editor.input_name.as_str())
    //     .style(Style::default().fg(Color::Yellow))
    //     .block(Block::bordered().title_top(Line::from("Name").right_aligned()));
    f.render_widget(pop_task_editor.editor.widget(), editor);

    // let message_block = Block::new().borders(Borders::NONE);
    let alert_msg = Text::from(pop_task_editor.alert_msg.as_str()).centered();
    f.render_widget(alert_msg, alert_area);
}
