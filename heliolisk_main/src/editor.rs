use crate::buffer::*;

#[allow(dead_code)]
pub struct Editor {
    buffers: Vec<Buffer>,
    current_focused_index: i32,
    is_quittable: bool,
}

#[allow(dead_code)]
impl Editor {
    pub fn new(buffers: Vec<Buffer>) -> Self {
        Self {
            buffers,
            current_focused_index: 0,
            is_quittable: true,
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

    pub fn render() {
        todo!("Implement editor rendering.");
    }
}
