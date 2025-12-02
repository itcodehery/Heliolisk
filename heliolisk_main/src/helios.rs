use std::io::Result;

use ratatui::{
    DefaultTerminal, Frame,
    layout::Rect,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    EditorState,
    editor::{Editor, NavigateMode},
};

/// The Global App State for Helios
/// # Stores
/// - Editor State
/// - Quittable State
pub struct Helios {
    editor_state: EditorState,
    should_quit: bool,
}

impl Helios {
    pub fn init(editor: Editor) -> Self {
        dbg!("Helios: Initialized Editor State");
        Self {
            editor_state: EditorState::Navigate(editor),
            should_quit: true,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_events(&mut self) -> Result<()> {
        todo!()
    }
}

pub fn initialize_app() -> Helios {
    let alpha_buffer = crate::Buffer::new();
    let editor = Editor::<NavigateMode>::new(vec![alpha_buffer]);
    let helios = Helios::init(editor);

    return helios;
}

impl Widget for &Helios {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let title = Line::from(" Counter App Tutorial ");
        let instructions = Line::from("Hello World".to_string());
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(format!("{}", self.editor_state));

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
