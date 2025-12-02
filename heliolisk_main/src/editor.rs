use crate::Buffer;
use std::marker::PhantomData;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crossterm::event::{Event::Key, KeyCode::Char, read};

// States of the Document
pub struct NavigateMode;
pub struct EditMode;
pub struct SelectMode;
pub struct CommandMode;

#[allow(dead_code)]
pub struct Editor<State = NavigateMode> {
    buffers: Vec<Buffer>,
    current_focused_index: i32,
    is_quittable: bool,
    state: PhantomData<State>,
}

#[allow(dead_code)]
impl Editor {
    pub fn new(buffers: Vec<Buffer>) -> Self {
        Self {
            buffers,
            current_focused_index: 0,
            is_quittable: true,
            state: PhantomData::<NavigateMode>,
        }
    }

    pub fn buffer_switch_forward() {
        todo!("Implement switching between buffers in forwards order");
    }

    pub fn buffer_switch_backward() {
        todo!("Implement switching between buffers in backwards order");
    }

    pub fn render(&self) {
        enable_raw_mode().unwrap();
        loop {
            match read() {
                Ok(Key(event)) => {
                    println!("{:?}\r", event);
                    match event.code {
                        Char(c) => {
                            if c == 'q' {
                                break;
                            } else if c == ':' {
                            }
                        }
                        _ => (),
                    }
                }
                Err(err) => println!("Error: {}", err),
                _ => (),
            }
        }
        disable_raw_mode().unwrap();
    }
}

impl<S> Editor<S> {
    fn transition<NewState>(self) -> Editor<NewState> {
        Editor {
            buffers: self.buffers,
            current_focused_index: self.current_focused_index,
            is_quittable: self.is_quittable,
            state: PhantomData,
        }
    }
}

impl Editor<NavigateMode> {
    pub fn enter_edit_mode(self) -> Editor<EditMode> {
        self.transition()
    }

    pub fn enter_command_mode(self) -> Editor<CommandMode> {
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
}

impl Editor<SelectMode> {
    pub fn enter_navigate_mode(self) -> Editor<NavigateMode> {
        self.transition()
    }

    pub fn enter_command_mode(self) -> Editor<CommandMode> {
        self.transition()
    }
}

impl Editor<CommandMode> {
    pub fn enter_navigate_mode(self) -> Editor<NavigateMode> {
        self.transition()
    }
}
