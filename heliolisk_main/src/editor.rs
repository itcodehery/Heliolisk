use crate::buffer::HBuffer;
use std::marker::PhantomData;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};

use crossterm::event::KeyCode::Char;

// States of the Document

/// Navigate Mode
/// # Allows:
/// - Cursor Movement
/// - Movement into any other Mode
/// - Switching Buffers
pub struct NavigateMode;
/// Edit Mode:
/// # Allows:
/// - Editing Text of the Buffer
pub struct EditMode;
/// Select Mode:
/// # Allows:
/// - Selecting tokens in the buffers
/// - Tokens include words, sentences and lines
pub struct SelectMode;
/// Command Mode:
/// # Allows:
/// - Execution of Editor Level and Buffer Level Commands
pub struct CommandMode;

/// Represents an instance of the Helios Editor
///
/// # Handles:
/// - Multiple Buffers and their Focus
/// - The Cursor
/// - The Command Line
/// - State of the Editor Independent from their Buffers
pub struct Editor<State = NavigateMode> {
    buffers: Vec<HBuffer>,
    current_focused_index: usize,
    cursor_col: usize,
    cursor_line: usize,
    scroll_offset: usize,
    is_quittable: bool,
    command_line: String,
    error_line: String,
    error_timestamp: Option<Instant>,
    input_seq: String,
    state: PhantomData<State>,
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
    DebugPrintLinesToConsole,
    DebugPrintCurrentLineToConsole,
    None,
}

impl Editor {
    pub fn new(buffers: Vec<HBuffer>) -> Self {
        dbg!("Helios: New Editor Created with Buffer!");
        Self {
            buffers,
            current_focused_index: 0,
            is_quittable: true,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            command_line: String::new(),
            error_line: String::new(),
            error_timestamp: None,
            input_seq: String::new(),
            state: PhantomData::<NavigateMode>,
        }
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
            self.current_focused_index = self.buffers.len();
        } else {
            self.current_focused_index -= 1;
        }
    }
}

impl<S> Editor<S> {
    fn transition<NewState>(self) -> Editor<NewState> {
        Editor {
            buffers: self.buffers,
            current_focused_index: self.current_focused_index,
            is_quittable: self.is_quittable,
            cursor_col: self.cursor_col,
            cursor_line: self.cursor_line,
            scroll_offset: self.scroll_offset,
            command_line: self.command_line,
            error_line: self.error_line,
            error_timestamp: self.error_timestamp,
            input_seq: self.input_seq,
            state: PhantomData,
        }
    }

    pub fn update_viewport(&mut self, height: usize) {
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + height {
            self.scroll_offset = self.cursor_line - height + 1;
        }
    }

    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let buffer = &self.buffers[self.current_focused_index];
        let line_len = buffer.line_length(self.cursor_line);
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        }
    }

    fn move_cursor_up(&mut self) {
        // todo!("Panics for some reason. Fix this!");
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_cursor_col();
        }
    }

    fn move_cursor_start(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col = 0;
        }
    }

    fn move_cursor_down(&mut self) {
        // todo!("Panics for some reason. Fix this!");
        let buffer = &self.buffers[self.current_focused_index];
        if self.cursor_line < buffer.line_count() - 1 {
            self.cursor_line += 1;
            self.clamp_cursor_col();
        }
    }

    fn clamp_cursor_col(&mut self) {
        let buffer = &self.buffers[self.current_focused_index];
        let line_len = buffer.line_length(self.cursor_line);
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
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
            && time.elapsed() >= std::time::Duration::from_secs(10)
        {
            self.error_line.clear();
            self.error_timestamp = None;
        }
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_line)
    }

    pub fn undo(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.undo();
    }

    pub fn redo(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.redo();
    }

    pub fn delete_to_next_whitespace(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];

        // Snapshot before modification
        buffer.save_snapshot();

        let line_len = buffer.line_length(self.cursor_line);
        if self.cursor_col >= line_len {
            return;
        }

        let line_text = buffer.text.line(self.cursor_line);
        // line_text includes chars
        // We need to look ahead from cursor_col

        // Since line_text is a Cow<str>, and we want to iterate chars.
        // It's easier to just work with char indices if possible or converting to string/vec.
        // Accessing via chars iterator is O(N).

        let chars: Vec<char> = line_text.chars().collect();
        if self.cursor_col >= chars.len() {
            return; // Should be covered by line_len check but just in case
        }

        let mut delete_count = 0;
        let started_on_whitespace = chars[self.cursor_col].is_whitespace();

        for ch in chars.iter().skip(self.cursor_col) {
            let c = ch;

            if (started_on_whitespace && !c.is_whitespace()) || c.is_whitespace() {
                break;
            }
            delete_count += 1;
        }

        // Perform deletion
        for _ in 0..delete_count {
            // We always delete at current cursor_col, shrinking the line
            buffer.delete_char(self.cursor_line, self.cursor_col);
        }
    }

    pub fn move_word_forward(&mut self) {
        let buffer = &self.buffers[self.current_focused_index];

        loop {
            let line_len = buffer.line_length(self.cursor_line);

            // If strictly inside the line
            if self.cursor_col < line_len {
                let line_text = buffer.text.line(self.cursor_line);
                let chars: Vec<char> = line_text.chars().collect();

                // Check if current is whitespace (or special) to know if we crossed a boundary
                // Simple logic for now: skip current word (non-whitespace), then skip whitespace.

                // Better logic:
                // 1. If on word char, skip until whitespace.
                // 2. Skip whitespace until word char.
                // But vim 'w' is "start of next word".

                // Let's implement simple step-by-step advance.
                // Note: This is an O(N) naive implementation in a loop.

                let c = chars.get(self.cursor_col).unwrap_or(&'\n');

                if c.is_whitespace() {
                    // If we are on whitespace, we are looking for non-whitespace
                    self.cursor_col += 1;
                    if self.cursor_col < chars.len() && !chars[self.cursor_col].is_whitespace() {
                        // Found start of next word
                        break;
                    }
                } else {
                    // We are on a word, move until whitespace or end
                    self.cursor_col += 1;
                    // But we might hit whitespace immediately.
                    // If we hit whitespace, we continue loop to next iteration which handles whitespace.
                }
            } else {
                // End of line, move to next line
                if self.cursor_line < buffer.line_count() - 1 {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                    // Check if 0 is a word char
                    let line_text = buffer.text.line(self.cursor_line);
                    if let Some(c) = line_text.chars().next()
                        && !c.is_whitespace()
                    {
                        break;
                    }
                } else {
                    break; // End of file
                }
            }
        }
        self.clamp_cursor_col();
    }

    pub fn move_word_backward(&mut self) {
        // Simplified 'b' implementation
        let buffer = &self.buffers[self.current_focused_index];

        loop {
            if self.cursor_col > 0 {
                self.cursor_col -= 1;

                let line_text = buffer.text.line(self.cursor_line);
                let chars: Vec<char> = line_text.chars().collect();
                let c = chars.get(self.cursor_col).unwrap_or(&' ');

                // If we moved onto a word char, check if it's the start
                if !c.is_whitespace() {
                    // Check if prev is whitespace or start of line
                    if self.cursor_col == 0 {
                        break;
                    }
                    let prev = chars.get(self.cursor_col - 1).unwrap_or(&' ');
                    if prev.is_whitespace() {
                        break;
                    }
                }
                // If we are on whitespace, keep going back (loop continues)
            } else {
                // Start of line, go to prev line end
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    let line_len = buffer.line_length(self.cursor_line);
                    // Set to end of line, but we need strictly inside?
                    // Vim 'b' from start of line goes to end of prev line's last word.
                    self.cursor_col = if line_len > 0 { line_len } else { 0 };
                } else {
                    break; // Start of file
                }
            }
        }
    }

    pub fn move_to_start_of_file(&mut self) {
        self.cursor_line = 0;
        self.cursor_col = 0;
    }

    pub fn move_to_end_of_file(&mut self) {
        let buffer = &self.buffers[self.current_focused_index];
        let count = buffer.line_count();
        if count > 0 {
            self.cursor_line = count - 1;
            self.cursor_col = 0; // Ideally end of line? standard G goes to start of last line usually?
            // User request just said "G to move to the end of the file".
        }
    }

    pub fn move_word_end_forward(&mut self) {
        let buffer = &self.buffers[self.current_focused_index];
        // 1. Advance once
        if self.cursor_col + 1 < buffer.line_length(self.cursor_line) {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < buffer.line_count() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            return;
        }

        loop {
            let buffer = &self.buffers[self.current_focused_index];
            let line_text = buffer.text.line(self.cursor_line);
            let chars: Vec<char> = line_text.chars().collect();

            if self.cursor_col >= chars.len() {
                break;
            }

            let c = chars[self.cursor_col];

            if c.is_whitespace() {
                // Skip whitespace
                if self.cursor_col + 1 < chars.len() {
                    self.cursor_col += 1;
                } else if self.cursor_line + 1 < buffer.line_count() {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                } else {
                    break;
                }
            } else {
                // Check next char
                let next_idx = self.cursor_col + 1;
                if next_idx >= chars.len() {
                    break;
                }
                let next_c = chars[next_idx];
                if next_c.is_whitespace() {
                    break;
                }
                self.cursor_col += 1;
            }
        }
    }

    pub fn move_to_line_end(&mut self) {
        let buffer = &self.buffers[self.current_focused_index];
        let line_text = buffer.text.line(self.cursor_line);
        let len = line_text.chars().count(); // Chars count

        // Exclude newline if present
        let chars: Vec<char> = line_text.chars().collect();
        if let Some(last) = chars.last()
            && (*last == '\n' || *last == '\r')
        {
            self.cursor_col = if len > 1 { len - 2 } else { 0 };
            return;
        }
        self.cursor_col = if len > 0 { len - 1 } else { 0 };
    }

    pub fn move_to_line_start_non_whitespace(&mut self) {
        let buffer = &self.buffers[self.current_focused_index];
        let line_text = buffer.text.line(self.cursor_line);

        let mut idx = 0;
        for c in line_text.chars() {
            if !c.is_whitespace() {
                break;
            }
            idx += 1;
        }
        // If line is all whitespace, maybe go to end?
        // But let's clamp to line length - 1 (before newline) if possible.
        let len = buffer.line_length(self.cursor_line);

        // Handle empty lines or just newline
        if len == 0 || (len == 1 && idx == 1) {
            // mostly newline
            self.cursor_col = 0;
        } else if idx >= len {
            self.cursor_col = len - 1;
        } else {
            self.cursor_col = idx;
        }
    }
}

impl Editor<NavigateMode> {
    pub fn handle_input(&mut self, key: KeyEvent) -> EditorAction {
        let mut action = EditorAction::None;

        // Handle pending sequences (like 'd' waiting for 'w')
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
        }

        match key.code {
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

    pub fn enter_edit_mode(mut self) -> Editor<EditMode> {
        self.get_active_buffer_mut().save_snapshot();
        self.transition()
    }

    pub fn enter_command_mode(self) -> Editor<CommandMode> {
        self.transition()
    }

    pub fn enter_select_mode(self) -> Editor<SelectMode> {
        self.transition()
    }
}

impl Editor<EditMode> {
    pub fn enter_navigate_mode(self) -> Editor<NavigateMode> {
        self.transition()
    }

    pub fn enter_select_mode(self) -> Editor<SelectMode> {
        self.transition()
    }

    pub fn insert_char(&mut self, c: char) {
        let buffer = &mut self.buffers[self.current_focused_index];
        buffer.insert_char(self.cursor_line, self.cursor_col, c);
        self.cursor_col += 1;
    }

    pub fn insert_line(&mut self) {
        self.buffers[self.current_focused_index].insert_line(self.cursor_line, self.cursor_col);
        self.cursor_line += 1;
        self.move_cursor_start();
    }

    pub fn delete_char(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];

        if self.cursor_col == 0 {
            if self.cursor_line > 0 {
                let prev_line_idx = self.cursor_line - 1;
                let prev_line_len = buffer.line_length(prev_line_idx);

                // The newline character is at len - 1
                let new_cursor_col = if prev_line_len > 0 {
                    prev_line_len - 1
                } else {
                    0
                };

                buffer.delete_char(prev_line_idx, new_cursor_col);

                self.cursor_line = prev_line_idx;
                self.cursor_col = new_cursor_col;
            }
        } else {
            buffer.delete_char(self.cursor_line, self.cursor_col - 1);
            self.cursor_col -= 1;
        }
    }

    pub fn delete_line(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];

        let line_to_delete = self.cursor_line;
        self.cursor_line -= 1;
        buffer.delete_line(line_to_delete);
    }

    pub fn open_line_below(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        let len = buffer.line_length(self.cursor_line);

        buffer.insert_char(self.cursor_line, len, '\n');
        self.cursor_line += 1;
        self.cursor_col = 0;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> EditorAction {
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
}

impl Editor<SelectMode> {
    pub fn enter_navigate_mode(self) -> Editor<NavigateMode> {
        self.transition()
    }

    pub fn enter_command_mode(self) -> Editor<CommandMode> {
        self.transition()
    }

    pub fn enter_edit_mode(self) -> Editor<EditMode> {
        self.transition()
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> EditorAction {
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
                    return EditorAction::EnterEditMode;
                } else if c == ':' {
                    return EditorAction::EnterCommandMode;
                }
                EditorAction::None
            }
            _ => EditorAction::None,
        }
    }
}

impl Editor<CommandMode> {
    pub fn enter_navigate_mode(self) -> Editor<NavigateMode> {
        self.transition()
    }

    pub fn clear_command_line(&mut self) {
        self.command_line.clear();
    }

    pub fn execute_command(&mut self, cmd: &str) -> EditorAction {
        self.clear_command_line();
        match cmd {
            "q" => EditorAction::Quit,
            // "w" => EditorAction::Save,
            // "wq" => EditorAction::SaveAndQuit,
            "qa" => EditorAction::QuitAll,
            "wel" => EditorAction::None,
            "dla" => EditorAction::DebugPrintLinesToConsole, // DebugPrint Line All
            "dlc" => EditorAction::DebugPrintCurrentLineToConsole, // DebugPrint Line Current
            _ => {
                // Spaghetti code btw
                if cmd.starts_with("w") || cmd.starts_with("wq") {
                    let splits = cmd.split(" ");

                    if cmd.starts_with("wq") {
                        if splits.clone().count() == 2 {
                            return EditorAction::SaveAndQuit(Some(
                                splits.last().unwrap().to_string().clone(),
                            ));
                        } else {
                            return EditorAction::SaveAndQuit(None);
                        }
                    }
                    if splits.clone().count() == 2 {
                        EditorAction::Save(Some(splits.last().unwrap().to_string().clone()))
                    } else {
                        EditorAction::Save(None)
                    }
                } else {
                    EditorAction::None
                }
            }
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> EditorAction {
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
