#[derive(Clone)]
#[allow(dead_code)]
pub struct Line {
    text: String,
}

#[allow(dead_code)]
impl Line {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }
}

#[allow(dead_code)]
pub struct Buffer {
    lines: Vec<Line>,
    file_format: String,
}

#[allow(dead_code)]
impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            file_format: ".txt".to_string(),
        }
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

    pub fn quit(&self) {
        if self.has_unsaved_changes() {
            println!("Couldn't exit! File has unsaved changes!");
        }
    }
}
