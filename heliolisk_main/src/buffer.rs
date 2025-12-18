use crate::rope::HeliosRope;

/// Represents a single open document.
///
/// Consists of lines and the document's file format as a String.
#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct HBuffer {
    pub text: HeliosRope,
    pub file_format: String,
    pub file_path: Option<String>,
    pub undo_stack: Vec<HeliosRope>,
    pub redo_stack: Vec<HeliosRope>,
}

impl HBuffer {
    pub fn new() -> Self {
        dbg!("Helios: New Buffer Created!");
        Self {
            text: HeliosRope::new(),
            file_format: ".txt".to_string(),
            file_path: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
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
        // We need to find the char index from line/col
        let line_start_char = self.text.line_to_char(line_idx);
        let char_idx = line_start_char + col_idx;

        // Safety: Ensure we don't insert past the line end (careful with newlines)
        // For now, simple insertion. Ropes handle newlines as characters.
        self.text.insert_char(char_idx, c);
    }

    pub fn insert_line(&mut self, line_idx: usize, col_idx: usize) {
        // Inserting a line is just inserting a newline char
        self.insert_char(line_idx, col_idx, '\n');
    }

    pub fn delete_line(&mut self, line_index: usize) {
        // Delete a range of characters corresponding to the line
        let start_char = self.text.line_to_char(line_index);
        let end_char = self.text.line_to_char(line_index + 1);

        self.text.remove(start_char..end_char);
    }

    pub fn delete_char(&mut self, line_idx: usize, col_idx: usize) {
        let line_start_char = self.text.line_to_char(line_idx);
        let char_idx = line_start_char + col_idx;

        // Ensure we are deleting a valid char
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
        self.undo_stack.push(self.text.clone());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev_text) = self.undo_stack.pop() {
            self.redo_stack.push(self.text.clone());
            self.text = prev_text;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next_text) = self.redo_stack.pop() {
            self.undo_stack.push(self.text.clone());
            self.text = next_text;
        }
    }
}
