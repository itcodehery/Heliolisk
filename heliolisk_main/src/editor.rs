use crate::Buffer;
use std::marker::PhantomData;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crossterm::event::{Event::Key, KeyCode::Char, read};

// States of the Document
#[allow(dead_code)]
pub struct NavigateMode;
#[allow(dead_code)]
pub struct EditMode;
#[allow(dead_code)]
pub struct SelectMode;

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

    pub fn quit() {
        todo!("Implement Changes check");
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
