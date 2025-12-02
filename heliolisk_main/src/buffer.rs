#[derive(Clone)]
pub struct Line {
    text: String,
}

impl Line {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }
}

pub struct Buffer {
    lines: Vec<Line>,
    file_format: String,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![],
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

    pub fn delete_char(&mut self, line_idx: usize, col_idx: usize) {
        if line_idx < self.lines.len() {
            self.lines[line_idx].text.remove(col_idx);
        }
    }

    pub fn quit(&self) {
        if self.has_unsaved_changes() {
            println!("Couldn't exit! File has unsaved changes!");
        }
    }
}
