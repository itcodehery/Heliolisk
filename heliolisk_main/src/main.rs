mod buffer;
mod editor;
mod helios;

use crate::{
    buffer::Buffer,
    editor::{CommandMode, EditMode, Editor, NavigateMode, SelectMode},
    helios::{Helios, initialize_app},
};
use color_eyre::Result;

pub enum EditorState {
    Navigate(Editor<NavigateMode>),
    Edit(Editor<EditMode>),
    Select(Editor<SelectMode>),
    Command(Editor<CommandMode>),
}

impl std::fmt::Display for EditorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorState::Navigate(_) => f.write_str("Navigate"),
            EditorState::Edit(_) => f.write_str("Edit"),
            EditorState::Select(_) => f.write_str("Select"),
            EditorState::Command(_) => f.write_str("Command"),
        }
    }
}

fn main() -> Result<()> {
    let mut app: Helios = initialize_app();
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
