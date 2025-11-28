#[derive(Clone)]
#[allow(dead_code)]
pub struct Line {
    chars: Vec<char>,
    char_count: i32,
}

#[allow(dead_code)]
impl Line {
    pub fn new() -> Self {
        Self {
            chars: vec![],
            char_count: 0,
        }
    }
}

#[allow(dead_code)]
pub struct Buffer {
    lines: Vec<Line>,
    line_count: i32,
    file_format: String,
}

#[allow(dead_code)]
impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            line_count: 0,
            file_format: ".txt".to_string(),
        }
    }
}
