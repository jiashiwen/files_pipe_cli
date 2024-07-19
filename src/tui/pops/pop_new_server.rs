use std::ops::Sub;

use strum::{EnumCount, EnumIter, FromRepr};
use tui_textarea::TextArea;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Default, Clone)]
pub struct PopNewServer<'a> {
    pub show: bool,
    input_name: String,
    input_url: String,
    textarea_name: TextArea<'a>,
    textarea_url: TextArea<'a>,
    /// Position of cursor in the editor area.
    name_char_index: usize,
    name_cursor_index: usize,
    url_char_index: usize,
    url_cursor_index: usize,
    alert_msg: String,
    selected_input: SelectedInput,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, FromRepr, EnumCount)]
pub enum SelectedInput {
    #[default]
    Name,
    Url,
}

impl<'a> PopNewServer<'a> {
    pub fn clear(&mut self) {
        self.input_name = "".to_string();
        self.input_url = "".to_string();
        self.name_char_index = 0;
        self.url_char_index = 0;
        self.name_cursor_index = 0;
        self.url_cursor_index = 0;
        self.alert_msg = "".to_string();
        self.selected_input = SelectedInput::default();
    }

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
    pub fn move_cursor_left(&mut self) {
        let (move_left_idx, move_left_str) = match self.selected_input {
            SelectedInput::Name => (self.name_char_index, self.input_name.clone()),
            SelectedInput::Url => (self.url_char_index, self.input_url.clone()),
        };
        let chars_size: Vec<usize> = move_left_str
            .char_indices()
            .map(|(_, c)| {
                let char_size = match UnicodeWidthChar::width(c) {
                    Some(s) => s,
                    None => 0,
                };
                char_size
            })
            .collect::<Vec<_>>();

        if move_left_idx > 0 {
            let move_left = match chars_size.get(move_left_idx - 1) {
                Some(s) => *s,
                None => 0,
            };
            match self.selected_input {
                SelectedInput::Name => {
                    // let cursor_moved_left = self.name_index.saturating_sub(1);
                    // self.name_index = self.clamp_cursor(cursor_moved_left);
                    (self.name_char_index, self.name_cursor_index) = self.clamp_cursor(
                        self.name_char_index.saturating_sub(1),
                        self.name_cursor_index.sub(move_left),
                    );
                }
                SelectedInput::Url => {
                    // let cursor_moved_left = self.url_char_index.saturating_sub(1);
                    // self.url_char_index = self.clamp_cursor(cursor_moved_left);
                    (self.url_char_index, self.url_cursor_index) = self.clamp_cursor(
                        self.url_char_index.saturating_sub(1),
                        self.url_cursor_index.sub(move_left),
                    );
                }
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        let (move_right_idx, move_right_str) = match self.selected_input {
            SelectedInput::Name => (self.name_char_index, self.input_name.clone()),
            SelectedInput::Url => (self.url_char_index, self.input_url.clone()),
        };
        let chars_size: Vec<usize> = move_right_str
            .char_indices()
            .map(|(_, c)| {
                let char_size = match UnicodeWidthChar::width(c) {
                    Some(s) => s,
                    None => 0,
                };
                char_size
            })
            .collect::<Vec<_>>();
        let move_right = match chars_size.get(move_right_idx) {
            Some(s) => *s,
            None => 1,
        };
        match self.selected_input {
            SelectedInput::Name => {
                // let cursor_moved_right = self.name_index.saturating_add(1);
                // self.name_index = self.clamp_cursor(cursor_moved_right);
                (self.name_char_index, self.name_cursor_index) = self.clamp_cursor(
                    self.name_char_index.saturating_add(1),
                    self.name_cursor_index.saturating_add(move_right),
                );
            }
            SelectedInput::Url => {
                // let cursor_moved_right = self.url_char_index.saturating_add(1);
                // self.url_index = self.clamp_cursor(cursor_moved_right);
                // self.url_char_index = self.clamp_cursor(move_right);
                (self.url_char_index, self.url_cursor_index) = self.clamp_cursor(
                    self.url_char_index.saturating_add(1),
                    self.url_cursor_index.saturating_add(move_right),
                );
            }
        }
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        log::info!("byte_index:{}", index);
        match self.selected_input {
            SelectedInput::Name => self.input_name.insert(index, new_char),
            SelectedInput::Url => self.input_url.insert(index, new_char),
        }
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&mut self) -> usize {
        match self.selected_input {
            SelectedInput::Name => self
                .input_name
                .char_indices()
                .map(|(i, _)| i)
                .nth(self.name_char_index)
                .unwrap_or(self.input_name.len()),
            SelectedInput::Url => self
                .input_url
                .char_indices()
                .map(|(i, _)| i)
                // .nth(self.url_char_index)
                .nth(self.url_char_index)
                .unwrap_or(self.input_url.len()),
        }
    }

    pub fn delete_char(&mut self) {
        match self.selected_input {
            SelectedInput::Name => {
                let is_not_cursor_leftmost = self.name_char_index != 0;
                if is_not_cursor_leftmost {
                    // Method "remove" is not used on the saved text for deleting the selected char.
                    // Reason: Using remove on String works on bytes instead of the chars.
                    // Using remove would require special care because of char boundaries.

                    let current_index = self.name_char_index;
                    let from_left_to_current_index = current_index - 1;

                    // Getting all characters before the selected character.
                    let before_char_to_delete =
                        self.input_name.chars().take(from_left_to_current_index);
                    // Getting all characters after selected character.
                    let after_char_to_delete = self.input_name.chars().skip(current_index);

                    // Put all characters together except the selected one.
                    // By leaving the selected one out, it is forgotten and therefore deleted.
                    self.input_name = before_char_to_delete.chain(after_char_to_delete).collect();
                    self.move_cursor_left();
                }
            }
            SelectedInput::Url => {
                let is_not_cursor_leftmost = self.url_char_index != 0;
                if is_not_cursor_leftmost {
                    let current_index = self.url_char_index;
                    let from_left_to_current_index = current_index - 1;
                    let before_char_to_delete =
                        self.input_url.chars().take(from_left_to_current_index);
                    let after_char_to_delete = self.input_url.chars().skip(current_index);
                    self.input_url = before_char_to_delete.chain(after_char_to_delete).collect();
                    self.move_cursor_left();
                }
            }
        }
    }

    fn clamp_cursor(&self, index_pose: usize, new_cursor_pos: usize) -> (usize, usize) {
        let clamp_str = match self.selected_input {
            SelectedInput::Name => self.input_name.clone(),
            SelectedInput::Url => self.input_url.clone(),
        };
        let mut max = 0;
        let chars_size: Vec<usize> = clamp_str
            .char_indices()
            .map(|(_, c)| {
                let char_size = match UnicodeWidthChar::width(c) {
                    Some(s) => s,
                    None => 0,
                };
                max += char_size;
                char_size
            })
            .collect::<Vec<_>>();
        let idx_clamp = match self.selected_input {
            SelectedInput::Name => index_pose.clamp(0, self.input_name.chars().count()),
            SelectedInput::Url => index_pose.clamp(0, self.input_url.chars().count()),
        };
        (idx_clamp, new_cursor_pos.clamp(0, max))
        // match self.selected_input {
        //     SelectedInput::Name => new_cursor_pos.clamp(0, self.input_name.chars().count()),
        //     SelectedInput::Url => new_cursor_pos.clamp(0, self.input_url.chars().count()),
        // }
    }

    fn reset_cursor(&mut self) {
        match self.selected_input {
            SelectedInput::Name => self.name_char_index = 0,
            SelectedInput::Url => self.url_char_index = 0,
        }
    }

    // pub fn add_server(&mut self) {
    //     // self.reset_cursor();
    //     let task_server = TaskServer {
    //         name: self.input_name.clone(),
    //         url: self.input_url.clone(),
    //     };
    //     self.clear();
    //     match save_task_server_to_cf(&task_server) {
    //         Ok(id) => self.alert_msg = format!("save task {} ok", id),
    //         Err(e) => {
    //             log::error!("{:}", e);
    //             self.alert_msg = "add server error".to_string()
    //         }
    //     };
    // }
}
