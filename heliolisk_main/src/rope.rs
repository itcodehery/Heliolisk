use ropey::Rope;
use std::borrow::Cow;

#[derive(Clone, Default)]
pub struct HeliosRope {
    pub inner: Rope,
}

impl HeliosRope {
    pub fn new() -> Self {
        Self {
            inner: Rope::new(),
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            inner: Rope::from_str(text),
        }
    }

    pub fn len_lines(&self) -> usize {
        self.inner.len_lines()
    }

    pub fn len_chars(&self) -> usize {
        self.inner.len_chars()
    }

    pub fn line(&self, line_idx: usize) -> Cow<str> {
        if line_idx >= self.len_lines() {
            return Cow::Borrowed("");
        }
        self.inner.line(line_idx).into()
    }

    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        if char_idx <= self.len_chars() {
            self.inner.insert_char(char_idx, ch);
        }
    }

    pub fn remove(&mut self, char_range: std::ops::Range<usize>) {
        if char_range.end <= self.len_chars() {
            self.inner.remove(char_range);
        }
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        if line_idx >= self.len_lines() {
            return self.len_chars();
        }
        self.inner.line_to_char(line_idx)
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.inner.char_to_line(char_idx)
    }

    /// Returns the length of a specific line in characters, including newline.
    pub fn line_len(&self, line_idx: usize) -> usize {
        if line_idx >= self.len_lines() {
            return 0;
        }
        self.inner.line(line_idx).len_chars()
    }
}

impl std::fmt::Display for HeliosRope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
