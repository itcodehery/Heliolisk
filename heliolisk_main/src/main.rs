mod buffer;
mod editor;

use crate::{buffer::Buffer, editor::Editor};

fn main() {
    let editor = Editor::new(vec![]);
    editor.render();
}
