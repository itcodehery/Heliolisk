use crate::buffer::HBuffer;
use std::marker::PhantomData;

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
    is_quittable: bool,
    command_line: String,
    error_line: String,
    state: PhantomData<State>,
}

pub enum EditorAction {
    Quit,
    Save,
    SaveAndQuit,
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
            command_line: String::new(),
            error_line: String::new(),
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
            command_line: self.command_line,
            error_line: self.error_line,
            state: PhantomData,
        }
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

    pub fn get_buffers(&self) -> Vec<HBuffer> {
        let buffers = &self.buffers;
        buffers.to_vec()
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_line)
    }
}

impl Editor<NavigateMode> {
    pub fn handle_input(&mut self, key: KeyEvent) -> EditorAction {
        let mut action = EditorAction::None;
        match key.code {
            Char('i') => action = EditorAction::EnterEditMode,
            Char('a') => action = EditorAction::EnterEditMode,
            Char('o') => {
                action = EditorAction::EnterEditModeInNewLine;
            }
            Char(':') => action = EditorAction::EnterCommandMode,
            Char('v') => action = EditorAction::EnterSelectMode,
            Char('h') => self.move_cursor_left(),
            Char('l') => self.move_cursor_right(),
            Char('k') => self.move_cursor_up(),
            Char('j') => self.move_cursor_down(),
            KeyCode::Tab => self.buffer_switch_forward(),
            KeyCode::BackTab => self.buffer_switch_backward(),
            _ => {}
        }
        action
    }

    pub fn enter_edit_mode(self) -> Editor<EditMode> {
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
        self.buffers[self.current_focused_index].insert_line();
        self.cursor_line += 1;
        self.move_cursor_start();
    }

    pub fn delete_char(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];
        // If the cursor is at the start of the character of the line, delete the line
        if self.cursor_col == 0 && self.cursor_line == 0 {
        } else if self.cursor_col == 0 && buffer.lines[self.cursor_line].text.is_empty() {
            buffer.delete_line(self.cursor_line);
            self.move_cursor_up();
        } else {
            // minus 1 here cuz the deletion must happen before the cursor
            if self.cursor_col != 0 {
                buffer.delete_char(self.cursor_line, self.cursor_col - 1);
            }
            self.move_cursor_left();
        }
    }

    pub fn delete_to_next_whitespace(&mut self) {
        todo!(
            "Implement dw to delete word, but how it actually works is that it deletes the characters till a whitespace occurs."
        )
    }

    pub fn delete_line(&mut self) {
        let buffer = &mut self.buffers[self.current_focused_index];

        let line_to_delete = self.cursor_line;
        self.cursor_line -= 1;
        buffer.delete_line(line_to_delete);
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
            "w" => EditorAction::Save,
            "wq" => EditorAction::SaveAndQuit,
            "qa" => EditorAction::QuitAll,
            "wel" => EditorAction::None,
            "dla" => EditorAction::DebugPrintLinesToConsole, // DebugPrint Line All
            "dlc" => EditorAction::DebugPrintCurrentLineToConsole, // DebugPrint Line Current
            _ => EditorAction::None,
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
