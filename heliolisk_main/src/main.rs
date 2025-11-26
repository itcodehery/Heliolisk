mod buffer;
mod editor;

use crossterm::terminal;

use crate::buffer::Buffer;
use crate::editor::Editor;

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let new_buffer = Buffer::new();
    let mut editor = Editor::new(vec![new_buffer]);

    terminal::disable_raw_mode()?;
    Ok(())
}
