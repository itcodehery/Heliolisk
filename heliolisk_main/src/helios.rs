use std::io::Result;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    EditorState,
    editor::{CommandMode, EditMode, Editor, EditorAction, NavigateMode, SelectMode},
};

/// The Global App State for Helios
/// # Stores
/// - Editor State
/// - Quittable State
pub struct Helios {
    editor_state: Option<EditorState>,
    should_quit: bool,
}

impl Helios {
    pub fn init(editor: Editor) -> Self {
        dbg!("Helios: Initialized Editor State");
        Self {
            editor_state: Some(EditorState::Navigate(editor)),
            should_quit: false,
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

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let Some(state) = self.editor_state.take() {
            self.editor_state = Some(match state {
                EditorState::Navigate(mut editor) => match editor.handle_input(key_event) {
                    EditorAction::Quit => {
                        self.should_quit = true;
                        EditorState::Navigate(editor)
                    }
                    EditorAction::EnterEditMode => EditorState::Edit(editor.enter_edit_mode()),
                    EditorAction::EnterCommandMode => {
                        EditorState::Command(editor.enter_command_mode())
                    }
                    EditorAction::EnterSelectMode => {
                        EditorState::Select(editor.enter_select_mode())
                    }
                    _ => EditorState::Navigate(editor),
                },
                EditorState::Edit(mut editor) => match editor.handle_input(key_event) {
                    EditorAction::EnterNavigateMode => {
                        EditorState::Navigate(editor.enter_navigate_mode())
                    }
                    EditorAction::EnterSelectMode => {
                        EditorState::Select(editor.enter_select_mode())
                    }
                    _ => EditorState::Edit(editor),
                },
                EditorState::Select(mut editor) => match editor.handle_input(key_event) {
                    EditorAction::EnterNavigateMode => {
                        EditorState::Navigate(editor.enter_navigate_mode())
                    }
                    EditorAction::EnterCommandMode => {
                        EditorState::Command(editor.enter_command_mode())
                    }
                    EditorAction::EnterEditMode => EditorState::Edit(editor.enter_edit_mode()),
                    _ => EditorState::Select(editor),
                },
                EditorState::Command(mut editor) => {
                    match editor.handle_input(key_event) {
                        EditorAction::Quit => {
                            self.should_quit = true;
                            EditorState::Command(editor)
                        }
                        EditorAction::EnterNavigateMode => {
                            EditorState::Navigate(editor.enter_navigate_mode())
                        }
                        EditorAction::Save => {
                            // TODO: Implement Save
                            EditorState::Command(editor)
                        }
                        EditorAction::SaveAndQuit => {
                            // TODO: Implement Save
                            self.should_quit = true;
                            EditorState::Command(editor)
                        }
                        EditorAction::QuitAll => {
                            self.should_quit = true;
                            EditorState::Command(editor)
                        }
                        _ => EditorState::Command(editor),
                    }
                }
            });
        }
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
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(95), Constraint::Percentage(5)])
            .split(area);
        if let Some(state) = &self.editor_state {
            let _ = Block::bordered()
                .border_set(border::PLAIN)
                .title_bottom(format!("{}", state))
                .render(layout[0], buf);

            let command_text = match state {
                EditorState::Navigate(ed) => ed.get_command_line(),
                EditorState::Command(ed) => ed.get_command_line(),
                EditorState::Edit(ed) => ed.get_command_line(),
                EditorState::Select(ed) => ed.get_command_line(),
            };
            Paragraph::new(command_text)
                .block(Block::bordered().border_set(border::PLAIN))
                .render(layout[1], buf);
        }
    }
}
