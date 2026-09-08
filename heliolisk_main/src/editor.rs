use crate::buffer::HBuffer;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use crossterm::event::KeyCode::Char;

/// Modes of the Editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Navigate,
    Edit,
    Select,
    Command,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Navigate => f.write_str("Nav"),
            Mode::Edit => f.write_str("Edi"),
            Mode::Select => f.write_str("Sel"),
            Mode::Command => f.write_str("Com"),
        }
    }
}

/// Represents an instance of the Helios Editor
pub struct Editor {
    buffers: Vec<HBuffer>,
    current_focused_index: usize,
    mode: Mode,
    command_line: String,
    error_line: String,
    error_timestamp: Option<Instant>,
    input_seq: String,
}

pub enum EditorAction {
    Quit,
    Save(Option<String>),
    SaveAndQuit(Option<String>),
    QuitAll,
    EnterCommandMode,
    EnterEditMode,
    EnterEditModeInNewLine,
    EnterSelectMode,
    EnterNavigateMode,
    ToggleFileExplorer,
    ToggleLspExplorer,
    SetTheme(String),
    TriggerHover,
    DebugPrintLinesToConsole,
    DebugPrintCurrentLineToConsole,
    AddNewBuffer,
    None,
}

impl Editor {
    pub fn new(buffers: Vec<HBuffer>) -> Self {
        Self {
            buffers,
            current_focused_index: 0,
            mode: Mode::Navigate,
            command_line: String::new(),
            error_line: String::new(),
            error_timestamp: None,
            input_seq: String::new(),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn add_new_buffer(&mut self) {
        let new_buf = HBuffer::new();

        self.buffers.push(new_buf);
        self.current_focused_index = self.buffers.len() - 1;
    }

    pub fn open_buffer(&mut self, buffer: HBuffer) {
        // Check if already open by path
        if let Some(ref path) = buffer.file_path {
            for (idx, b) in self.buffers.iter().enumerate() {
                if b.file_path.as_ref() == Some(path) {
                    self.current_focused_index = idx;
                    return;
                }
            }
        }

        // If the current buffer is the single initial empty untitled buffer, replace it
        if self.buffers.len() == 1
            && self.buffers[0].file_path.is_none()
            && self.buffers[0].text.len_chars() == 0
            && self.buffers[0].undo_stack.is_empty()
        {
            self.buffers[0] = buffer;
            self.current_focused_index = 0;
            return;
        }

        self.buffers.push(buffer);
        self.current_focused_index = self.buffers.len() - 1;
    }

    pub fn buffer_switch_forward(&mut self) {
        if self.buffers.len() >= 2 {
            if self.current_focused_index + 1 == self.buffers.len() {
                self.current_focused_index = 0;
            } else {
                self.current_focused_index += 1;
            }
        }
    }

    pub fn buffer_switch_backward(&mut self) {
        if self.buffers.len() < 2 {
        } else if self.current_focused_index == 0 {
            self.current_focused_index = self.buffers.len() - 1;
        } else {
            self.current_focused_index -= 1;
        }
    }

    pub fn update_viewport(&mut self, height: usize) {
        self.buffers[self.current_focused_index].update_viewport(height);
    }

    pub fn get_scroll_offset(&self) -> usize {
        self.buffers[self.current_focused_index].scroll_offset
    }

    pub fn move_cursor_left(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        if buffer.cursor_col > 0 {
            buffer.cursor_col -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        let line_len = buffer.line_length(buffer.cursor_line);
        if buffer.cursor_col < line_len {
            buffer.cursor_col += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        if buffer.cursor_line > 0 {
            buffer.cursor_line -= 1;
            buffer.clamp_cursor();
        }
    }

    pub fn move_cursor_start(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.cursor_col = 0;
    }

    pub fn move_cursor_down(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        if buffer.cursor_line < buffer.line_count().saturating_sub(1) {
            buffer.cursor_line += 1;
            buffer.clamp_cursor();
        }
    }

    pub fn clamp_cursor_col(&mut self) {
        self.buffers[self.current_focused_index].clamp_cursor();
    }

    pub fn get_command_line(&self) -> String {
        self.command_line.clone()
    }

    pub fn get_error_line(&self) -> String {
        self.error_line.clone()
    }

    pub fn get_buffers(&self) -> &Vec<HBuffer> {
        &self.buffers
    }

    pub fn get_focused_index(&self) -> usize {
        self.current_focused_index
    }

    pub fn get_active_buffer(&self) -> &HBuffer {
        &self.buffers[self.current_focused_index]
    }

    pub fn get_active_buffer_mut(&mut self) -> &mut HBuffer {
        &mut self.buffers[self.current_focused_index]
    }

    pub fn set_error_line(&mut self, error: String) {
        self.error_line = error;
        self.error_timestamp = Some(Instant::now());
    }

    pub fn check_error_expiry(&mut self) {
        if let Some(time) = self.error_timestamp
            && time.elapsed() >= std::time::Duration::from_secs(5)
        {
            self.error_line.clear();
            self.error_timestamp = None;
        }
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        let buffer = &self.buffers[self.current_focused_index];
        (buffer.cursor_col, buffer.cursor_line)
    }

    pub fn undo(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.undo();
        buffer.clamp_cursor();
    }

    pub fn redo(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.redo();
        buffer.clamp_cursor();
    }

    pub fn delete_to_next_whitespace(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.save_snapshot();

        let line_len = buffer.line_length(buffer.cursor_line);
        if buffer.cursor_col >= line_len {
            return;
        }

        let line_text = buffer.text.line(buffer.cursor_line);
        let chars: Vec<char> = line_text.chars().collect();
        if buffer.cursor_col >= chars.len() {
            return;
        }

        let mut delete_count = 0;
        let started_on_whitespace = chars[buffer.cursor_col].is_whitespace();

        for ch in chars.iter().skip(buffer.cursor_col) {
            let c = ch;
            if (started_on_whitespace && !c.is_whitespace()) || c.is_whitespace() {
                break;
            }
            delete_count += 1;
        }

        let line = buffer.cursor_line;
        let col = buffer.cursor_col;
        for _ in 0..delete_count {
            buffer.delete_char(line, col);
        }
        buffer.clamp_cursor();
    }

    pub fn move_word_forward(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];

        loop {
            let line_len = buffer.line_length(buffer.cursor_line);

            if buffer.cursor_col < line_len {
                let line_text = buffer.text.line(buffer.cursor_line);
                let chars: Vec<char> = line_text.chars().collect();

                let c = chars.get(buffer.cursor_col).unwrap_or(&'\n');

                if c.is_whitespace() {
                    buffer.cursor_col += 1;
                    if buffer.cursor_col < chars.len() && !chars[buffer.cursor_col].is_whitespace() {
                        break;
                    }
                } else {
                    buffer.cursor_col += 1;
                }
            } else if buffer.cursor_line < buffer.line_count().saturating_sub(1) {
                buffer.cursor_line += 1;
                buffer.cursor_col = 0;
                let line_text = buffer.text.line(buffer.cursor_line);
                if let Some(c) = line_text.chars().next()
                    && !c.is_whitespace()
                {
                    break;
                }
            } else {
                break;
            }
        }
        buffer.clamp_cursor();
    }

    pub fn move_word_backward(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];

        loop {
            if buffer.cursor_col > 0 {
                buffer.cursor_col -= 1;

                let line_text = buffer.text.line(buffer.cursor_line);
                let chars: Vec<char> = line_text.chars().collect();
                let c = chars.get(buffer.cursor_col).unwrap_or(&' ');

                if !c.is_whitespace() {
                    if buffer.cursor_col == 0 {
                        break;
                    }
                    let prev = chars.get(buffer.cursor_col - 1).unwrap_or(&' ');
                    if prev.is_whitespace() {
                        break;
                    }
                }
            } else if buffer.cursor_line > 0 {
                buffer.cursor_line -= 1;
                let line_len = buffer.line_length(buffer.cursor_line);
                buffer.cursor_col = if line_len > 0 { line_len } else { 0 };
            } else {
                break;
            }
        }
        buffer.clamp_cursor();
    }

    pub fn move_to_start_of_file(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.cursor_line = 0;
        buffer.cursor_col = 0;
    }

    pub fn move_to_end_of_file(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        let count = buffer.line_count();
        if count > 0 {
            buffer.cursor_line = count - 1;
            buffer.cursor_col = 0;
        }
    }

    pub fn move_word_end_forward(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        if buffer.cursor_col + 1 < buffer.line_length(buffer.cursor_line) {
            buffer.cursor_col += 1;
        } else if buffer.cursor_line + 1 < buffer.line_count() {
            buffer.cursor_line += 1;
            buffer.cursor_col = 0;
        } else {
            return;
        }

        loop {
            let line_text = buffer.text.line(buffer.cursor_line);
            let chars: Vec<char> = line_text.chars().collect();

            if buffer.cursor_col >= chars.len() {
                break;
            }

            let c = chars[buffer.cursor_col];

            if c.is_whitespace() {
                if buffer.cursor_col + 1 < chars.len() {
                    buffer.cursor_col += 1;
                } else if buffer.cursor_line + 1 < buffer.line_count() {
                    buffer.cursor_line += 1;
                    buffer.cursor_col = 0;
                } else {
                    break;
                }
            } else {
                let next_idx = buffer.cursor_col + 1;
                if next_idx >= chars.len() {
                    break;
                }
                let next_c = chars[next_idx];
                if next_c.is_whitespace() {
                    break;
                }
                buffer.cursor_col += 1;
            }
        }
        buffer.clamp_cursor();
    }

    pub fn move_to_line_end(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        let line_text = buffer.text.line(buffer.cursor_line);
        let trimmed = line_text.trim_end_matches(['\r', '\n']);
        let len = trimmed.chars().count();
        buffer.cursor_col = if len > 0 { len - 1 } else { 0 };
    }

    pub fn move_to_line_start_non_whitespace(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        let line_text = buffer.text.line(buffer.cursor_line);

        let mut idx = 0;
        for c in line_text.chars() {
            if !c.is_whitespace() {
                break;
            }
            idx += 1;
        }
        let len = buffer.line_length(buffer.cursor_line);

        if len == 0 || (len == 1 && idx == 1) {
            buffer.cursor_col = 0;
        } else if idx >= len {
            buffer.cursor_col = len - 1;
        } else {
            buffer.cursor_col = idx;
        }
    }

    pub fn enter_edit_mode(&mut self) {
        self.get_active_buffer_mut().save_snapshot();
        self.mode = Mode::Edit;
    }

    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
    }

    pub fn enter_select_mode(&mut self) {
        self.mode = Mode::Select;
    }

    pub fn enter_navigate_mode(&mut self) {
        self.mode = Mode::Navigate;
    }

    pub fn insert_char(&mut self, c: char) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.insert_char(buffer.cursor_line, buffer.cursor_col, c);
        buffer.cursor_col += 1;
    }

    pub fn insert_line(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.insert_line(buffer.cursor_line, buffer.cursor_col);
        buffer.cursor_line += 1;
        buffer.cursor_col = 0;
    }

    pub fn delete_char(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];

        if buffer.cursor_col == 0 {
            if buffer.cursor_line > 0 {
                let prev_line_idx = buffer.cursor_line - 1;
                let prev_line_len = buffer.line_length(prev_line_idx);

                let new_cursor_col = if prev_line_len > 0 {
                    prev_line_len - 1
                } else {
                    0
                };

                buffer.delete_char(prev_line_idx, new_cursor_col);

                buffer.cursor_line = prev_line_idx;
                buffer.cursor_col = new_cursor_col;
            }
        } else {
            buffer.delete_char(buffer.cursor_line, buffer.cursor_col - 1);
            buffer.cursor_col -= 1;
        }
    }

    pub fn delete_line(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];

        let line_to_delete = buffer.cursor_line;
        if buffer.cursor_line > 0 {
            buffer.cursor_line -= 1;
        }
        buffer.delete_line(line_to_delete);
        buffer.clamp_cursor();
    }

    pub fn open_line_below(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        let len = buffer.line_length(buffer.cursor_line);

        buffer.insert_char(buffer.cursor_line, len, '\n');
        buffer.cursor_line += 1;
        buffer.cursor_col = 0;
    }

    pub fn clear_command_line(&mut self) {
        self.command_line.clear();
    }

    pub fn execute_command(&mut self, cmd: &str) -> EditorAction {
        self.clear_command_line();
        let tokens: Vec<&str> = cmd.split_whitespace().collect();
        if tokens.is_empty() {
            return EditorAction::None;
        }

        match tokens[0] {
            "q" => EditorAction::Quit,
            "qa" => EditorAction::QuitAll,
            "w" => {
                let file_name = tokens.get(1).map(|s| s.to_string());
                EditorAction::Save(file_name)
            }
            "wq" => {
                let file_name = tokens.get(1).map(|s| s.to_string());
                EditorAction::SaveAndQuit(file_name)
            }
            "colorscheme" | "theme" => {
                let theme_name = tokens.get(1).unwrap_or(&"tokyo-night").to_string();
                EditorAction::SetTheme(theme_name)
            }
            "e" | "explorer" => EditorAction::ToggleFileExplorer,
            "wel" => EditorAction::None,
            "dla" => EditorAction::DebugPrintLinesToConsole,
            "dlc" => EditorAction::DebugPrintCurrentLineToConsole,
            _ => EditorAction::None,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> EditorAction {
        match self.mode {
            Mode::Navigate => self.handle_navigate_input(key),
            Mode::Edit => self.handle_edit_input(key),
            Mode::Select => self.handle_select_input(key),
            Mode::Command => self.handle_command_input(key),
        }
    }

    fn handle_navigate_input(&mut self, key: KeyEvent) -> EditorAction {
        let mut action = EditorAction::None;

        if self.input_seq == "d" {
            if let Char('w') = key.code {
                self.delete_to_next_whitespace();
                self.input_seq.clear();
                return EditorAction::None;
            } else {
                self.input_seq.clear();
            }
        } else if self.input_seq == "g" {
            if let Char('g') = key.code {
                self.move_to_start_of_file();
                self.input_seq.clear();
                return EditorAction::None;
            } else {
                self.input_seq.clear();
            }
        } else if self.input_seq == " " {
            self.input_seq.clear();
            if let Char('e') = key.code {
                return EditorAction::ToggleFileExplorer;
            } else if let Char('l') | Char('L') = key.code {
                return EditorAction::ToggleLspExplorer;
            }
        }

        match key.code {
            Char(' ') => {
                self.input_seq.push(' ');
            }
            Char('i') => action = EditorAction::EnterEditMode,
            Char('a') => {
                self.move_cursor_right();
                action = EditorAction::EnterEditMode;
            }
            Char('o') => {
                action = EditorAction::EnterEditModeInNewLine;
            }
            Char('d') => {
                self.input_seq.push('d');
            }
            Char('g') => {
                self.input_seq.push('g');
            }
            Char('w') => self.move_word_forward(),
            Char('e') => self.move_word_end_forward(),
            Char('b') => self.move_word_backward(),
            Char('G') => self.move_to_end_of_file(),
            Char('^') => self.move_to_line_start_non_whitespace(),
            Char('$') => self.move_to_line_end(),
            Char('K') => action = EditorAction::TriggerHover,
            Char(':') => action = EditorAction::EnterCommandMode,
            Char('v') => action = EditorAction::EnterSelectMode,
            Char('h') => self.move_cursor_left(),
            Char('l') => self.move_cursor_right(),
            Char('k') => self.move_cursor_up(),
            Char('j') => self.move_cursor_down(),
            Char('u') => self.undo(),
            Char('U') => self.redo(),
            KeyCode::Tab => self.buffer_switch_forward(),
            KeyCode::BackTab => self.buffer_switch_backward(),
            _ => {}
        }
        action
    }

    fn handle_edit_input(&mut self, key: KeyEvent) -> EditorAction {
        match key.code {
            KeyCode::Esc => EditorAction::EnterNavigateMode,
            KeyCode::CapsLock => EditorAction::EnterNavigateMode,
            KeyCode::Char(c) => {
                self.insert_char(c);
                EditorAction::None
            }
            KeyCode::Backspace => {
                self.delete_char();
                EditorAction::None
            }
            KeyCode::Delete => {
                self.delete_char();
                EditorAction::None
            }
            KeyCode::Up => {
                self.move_cursor_up();
                EditorAction::None
            }
            KeyCode::Left => {
                self.move_cursor_left();
                EditorAction::None
            }
            KeyCode::Right => {
                self.move_cursor_right();
                EditorAction::None
            }
            KeyCode::Down => {
                self.move_cursor_down();
                EditorAction::None
            }
            KeyCode::Enter => {
                self.insert_line();
                EditorAction::None
            }
            KeyCode::Tab => {
                self.insert_char('\t');
                EditorAction::None
            }
            KeyCode::Home => {
                self.move_cursor_start();
                EditorAction::None
            }
            _ => EditorAction::None,
        }
    }

    fn handle_select_input(&mut self, key: KeyEvent) -> EditorAction {
        match key.code {
            KeyCode::Esc => EditorAction::EnterNavigateMode,
            KeyCode::CapsLock => EditorAction::EnterNavigateMode,
            KeyCode::Char('h') => {
                self.move_cursor_left();
                EditorAction::None
            }
            KeyCode::Char('l') => {
                self.move_cursor_right();
                EditorAction::None
            }
            KeyCode::Char('k') => {
                self.move_cursor_up();
                EditorAction::None
            }
            KeyCode::Char('j') => {
                self.move_cursor_down();
                EditorAction::None
            }
            KeyCode::Char(c) => {
                if c == 'i' {
                    EditorAction::EnterEditMode
                } else if c == ':' {
                    EditorAction::EnterCommandMode
                } else {
                    EditorAction::None
                }
            }
            _ => EditorAction::None,
        }
    }

    fn handle_command_input(&mut self, key: KeyEvent) -> EditorAction {
        match key.code {
            KeyCode::Esc => EditorAction::EnterNavigateMode,
            KeyCode::CapsLock => EditorAction::EnterNavigateMode,
            KeyCode::Char(c) => {
                self.command_line.push(c);
                EditorAction::None
            }
            KeyCode::Backspace => {
                self.command_line.pop();
                EditorAction::None
            }
            KeyCode::Enter => {
                let cmd = self.command_line.clone();
                self.execute_command(&cmd)
            }
            _ => EditorAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_switch_backward_bounds() {
        let mut editor = Editor::new(vec![HBuffer::new(), HBuffer::new()]);
        assert_eq!(editor.current_focused_index, 0);

        editor.buffer_switch_backward();
        assert_eq!(editor.current_focused_index, 1);
        let _ = editor.get_active_buffer();

        editor.buffer_switch_backward();
        assert_eq!(editor.current_focused_index, 0);
        let _ = editor.get_active_buffer();
    }

    #[test]
    fn test_delete_line_bounds() {
        let mut editor = Editor::new(vec![HBuffer::new()]);
        editor.enter_edit_mode();
        assert_eq!(editor.get_active_buffer().cursor_line, 0);

        editor.delete_line();
        assert_eq!(editor.get_active_buffer().cursor_line, 0);
    }

    #[test]
    fn test_buffer_independent_cursor_and_scroll() {
        let mut buf1 = HBuffer::new();
        buf1.cursor_line = 5;
        buf1.cursor_col = 10;
        buf1.scroll_offset = 2;

        let mut buf2 = HBuffer::new();
        buf2.cursor_line = 1;
        buf2.cursor_col = 3;
        buf2.scroll_offset = 0;

        let mut editor = Editor::new(vec![buf1, buf2]);
        assert_eq!(editor.get_cursor_position(), (10, 5));
        assert_eq!(editor.get_scroll_offset(), 2);

        editor.buffer_switch_forward();
        assert_eq!(editor.get_cursor_position(), (3, 1));
        assert_eq!(editor.get_scroll_offset(), 0);
    }

    #[test]
    fn test_execute_command_parsing() {
        let mut editor = Editor::new(vec![HBuffer::new()]);

        // Test :w with custom name and whitespace
        match editor.execute_command("  w   test_file.rs  ") {
            EditorAction::Save(Some(name)) => assert_eq!(name, "test_file.rs"),
            _ => panic!("Expected Save with test_file.rs"),
        }

        // Test :w without filename
        match editor.execute_command("w") {
            EditorAction::Save(None) => {}
            _ => panic!("Expected Save with None"),
        }

        // Test :wq with filename
        match editor.execute_command("wq out.txt") {
            EditorAction::SaveAndQuit(Some(name)) => assert_eq!(name, "out.txt"),
            _ => panic!("Expected SaveAndQuit with out.txt"),
        }

        // Test that commands starting with 'w' but not 'w' or 'wq' are NOT mistaken for save
        match editor.execute_command("workspace") {
            EditorAction::None => {}
            _ => panic!("Expected None for unknown command workspace"),
        }
    }

    #[test]
    fn test_undo_restores_cursor_position() {
        let mut editor = Editor::new(vec![HBuffer::new()]);
        editor.enter_edit_mode();
        editor.insert_char('a');
        editor.insert_char('b');
        editor.insert_char('c');
        assert_eq!(editor.get_cursor_position(), (3, 0));

        editor.enter_navigate_mode();
        editor.undo();
        // After undo, cursor position should be restored to initial (0, 0)
        assert_eq!(editor.get_cursor_position(), (0, 0));

        editor.redo();
        // After redo, cursor position should be back at (3, 0)
        assert_eq!(editor.get_cursor_position(), (3, 0));
    }

    #[test]
    fn test_crlf_move_to_line_end() {
        let mut buffer = HBuffer::new();
        buffer.text = crate::rope::HeliosRope::from_str("hello\r\nworld\r\n");
        let mut editor = Editor::new(vec![buffer]);

        editor.move_to_line_end();
        // "hello" has 5 characters, cursor on last char 'o' should be index 4
        assert_eq!(editor.get_cursor_position(), (4, 0));
    }

    #[test]
    fn test_open_buffer_replaces_empty_and_reuses_existing() {
        let mut editor = Editor::new(vec![HBuffer::new()]);
        assert_eq!(editor.buffers.len(), 1);

        // Open first real buffer
        let mut buf1 = HBuffer::new();
        buf1.file_path = Some("file1.rs".to_string());
        editor.open_buffer(buf1);
        assert_eq!(editor.buffers.len(), 1);
        assert_eq!(editor.buffers[0].file_path.as_deref(), Some("file1.rs"));

        // Open second buffer
        let mut buf2 = HBuffer::new();
        buf2.file_path = Some("file2.rs".to_string());
        editor.open_buffer(buf2);
        assert_eq!(editor.buffers.len(), 2);
        assert_eq!(editor.current_focused_index, 1);

        // Open file1.rs again - should jump back to index 0 without duplicating
        let mut buf1_again = HBuffer::new();
        buf1_again.file_path = Some("file1.rs".to_string());
        editor.open_buffer(buf1_again);
        assert_eq!(editor.buffers.len(), 2);
        assert_eq!(editor.current_focused_index, 0);
    }
}

