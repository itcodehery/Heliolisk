mod buffer;
mod config;
mod editor;
mod explorer;
mod file_ops;
mod helios;
mod lsp;
mod rope;
mod syntax;

use crate::helios::{Helios, initialize_app};

use color_eyre::Result;

fn main() -> Result<()> {
    let mut app: Helios = initialize_app();
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
