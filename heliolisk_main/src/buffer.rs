use crate::rope::HeliosRope;

#[derive(Clone)]
pub struct Snapshot {
    pub text: HeliosRope,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

/// Represents a single open document.
///
/// Consists of lines and the document's file format as a String.
#[derive(Clone, Default)]
pub struct HBuffer {
    pub text: HeliosRope,
    pub file_format: String,
    pub file_path: Option<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub undo_stack: Vec<Snapshot>,
    pub redo_stack: Vec<Snapshot>,
}

impl HBuffer {
    pub fn new() -> Self {
        Self {
            text: HeliosRope::new(),
            file_format: ".txt".to_string(),
            file_path: None,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn clamp_cursor(&mut self) {
        let lines = self.line_count();
        if lines == 0 {
            self.cursor_line = 0;
            self.cursor_col = 0;
            return;
        }

        if self.cursor_line >= lines {
            self.cursor_line = lines - 1;
        }

        let line_len = self.line_length(self.cursor_line);
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    pub fn update_viewport(&mut self, height: usize) {
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + height {
            self.scroll_offset = self.cursor_line.saturating_sub(height) + 1;
        }
    }

    pub fn line_length(&self, line_idx: usize) -> usize {
        self.text.line_len(line_idx)
    }

    pub fn line_count(&self) -> usize {
        self.text.len_lines()
    }

    pub fn char_count(&self) -> usize {
        self.text.len_chars()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        todo!("implement unsaved changes check")
    }

    pub fn insert_char(&mut self, line_idx: usize, col_idx: usize, c: char) {
        let line_start_char = self.text.line_to_char(line_idx);
        let char_idx = line_start_char + col_idx;

        self.text.insert_char(char_idx, c);
    }

    pub fn insert_line(&mut self, line_idx: usize, col_idx: usize) {
        self.insert_char(line_idx, col_idx, '\n');
    }

    pub fn delete_line(&mut self, line_index: usize) {
        if line_index >= self.line_count() {
            return;
        }

        let start_char = self.text.line_to_char(line_index);
        let end_char = self.text.line_to_char(line_index + 1);

        if start_char < end_char {
            self.text.remove(start_char..end_char);
        } else if self.text.len_chars() > 0 && start_char > 0 {
            // If deleting the trailing empty/last line, remove previous newline if applicable
            let prev_char = self.text.line_to_char(line_index.saturating_sub(1));
            let line_len = self.line_length(line_index.saturating_sub(1));
            if line_len > 0 {
                self.text.remove((prev_char + line_len - 1)..(prev_char + line_len));
            }
        }
    }

    pub fn delete_char(&mut self, line_idx: usize, col_idx: usize) {
        let line_start_char = self.text.line_to_char(line_idx);
        let char_idx = line_start_char + col_idx;

        if char_idx < self.text.len_chars() {
            self.text.remove(char_idx..char_idx + 1);
        }
    }

    pub fn quit(&self) {
        if self.has_unsaved_changes() {
            println!("Couldn't exit! File has unsaved changes!");
        }
    }

    pub fn save_snapshot(&mut self) {
        self.undo_stack.push(Snapshot {
            text: self.text.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                text: self.text.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.text = prev.text;
            self.cursor_line = prev.cursor_line;
            self.cursor_col = prev.cursor_col;
            self.clamp_cursor();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                text: self.text.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.text = next.text;
            self.cursor_line = next.cursor_line;
            self.cursor_col = next.cursor_col;
            self.clamp_cursor();
        }
    }
}
