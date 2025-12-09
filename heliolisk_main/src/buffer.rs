#[derive(Clone, Default)]
pub struct HLine {
    pub text: String,
}

impl HLine {
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }
}

/// Represents a single open document.
///
/// Consists of lines and the document's file format as a String.
#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct HBuffer {
    pub lines: Vec<HLine>,
    pub file_format: String,
}

impl HBuffer {
    pub fn new() -> Self {
        dbg!("Helios: New Buffer Created!");
        Self {
            lines: vec![HLine::new()],
            file_format: ".txt".to_string(),
        }
    }

    pub fn line_length(&self, line_idx: usize) -> usize {
        self.lines.get(line_idx).map(|l| l.text.len()).unwrap_or(0)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn char_count(&self) -> usize {
        self.lines.iter().map(|l| l.text.len()).sum()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        todo!("implement unsaved changes check")
    }

    pub fn insert_char(&mut self, line_idx: usize, col_idx: usize, c: char) {
        if line_idx < self.lines.len() {
            self.lines[line_idx].text.insert(col_idx, c);
        }
    }

    pub fn insert_line(&mut self) {
        self.lines.push(HLine::new());
    }

    pub fn delete_line(&mut self, line_index: usize) {
        // Optimization needed: Complexity -> O(n)
        if line_index < self.lines.len() && self.lines.len() != 1 {
            self.lines.remove(line_index);
        }
    }

    pub fn delete_char(&mut self, line_idx: usize, col_idx: usize) {
        if line_idx < self.lines.len() {
            let line = &mut self.lines[line_idx].text;

            if let Some((byte_idx, _)) = line.char_indices().nth(col_idx) {
                line.remove(byte_idx);
            }
        }
    }

    pub fn quit(&self) {
        if self.has_unsaved_changes() {
            println!("Couldn't exit! File has unsaved changes!");
        }
    }
}
