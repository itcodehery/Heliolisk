use std::marker::PhantomData;

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

// States of the Document
pub struct NavigateMode;
pub struct EditMode;
pub struct SelectMode;

#[allow(dead_code)]
pub struct Buffer<State = NavigateMode> {
    lines: Vec<Line>,
    line_count: i32,
    file_format: String,
    state: PhantomData<State>,
}

#[allow(dead_code)]
impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            line_count: 0,
            file_format: ".txt".to_string(),
            state: PhantomData::<NavigateMode>,
        }
    }

    fn write_mode(&self) -> Buffer<EditMode> {
        Buffer {
            lines: self.lines.clone(),
            line_count: self.line_count,
            file_format: self.file_format.clone(),
            state: PhantomData::<EditMode>,
        }
    }

    fn navigate_mode(&self) -> Buffer<NavigateMode> {
        Buffer {
            lines: self.lines.clone(),
            line_count: self.line_count,
            file_format: self.file_format.clone(),
            state: PhantomData::<NavigateMode>,
        }
    }

    fn select_mode(&self) -> Buffer<SelectMode> {
        Buffer {
            lines: self.lines.clone(),
            line_count: self.line_count,
            file_format: self.file_format.clone(),
            state: PhantomData::<SelectMode>,
        }
    }
}
