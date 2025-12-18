use std::io::Result;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    EditorState,
    buffer::HBuffer,
    editor::{Editor, EditorAction, NavigateMode},
    file_ops,
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
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        frame.render_widget(self, area);

        if let Some(state) = &self.editor_state {
            let (cursor_line, cursor_col) = match state {
                EditorState::Navigate(ed) => ed.get_cursor_position(),
                EditorState::Command(ed) => ed.get_cursor_position(),
                EditorState::Edit(ed) => ed.get_cursor_position(),
                EditorState::Select(ed) => ed.get_cursor_position(),
            };

            let cursor_x = layout[0].x + cursor_line as u16 + 1; // +1 for left border
            let cursor_y = layout[0].y + cursor_col as u16 + 1; // +1 for top border

            if cursor_x < layout[0].x + layout[0].width - 1
                && cursor_y < layout[0].y + layout[0].height - 1
            {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }
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

    fn save(
        &self,
        buffers: &[HBuffer],
        file_name: Option<String>,
    ) -> std::result::Result<String, String> {
        let buffer_contents = file_ops::buffer_to_string(&buffers[0]);
        let actual_file_name = file_name.unwrap_or_else(|| String::from("helios_test.txt"));
        match file_ops::write_string_to_file(buffer_contents, Some(actual_file_name.clone())) {
            Ok(_) => Ok(format!("Saved to {}", actual_file_name)),
            Err(e) => Err(e),
        }
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
                    EditorAction::EnterEditModeInNewLine => {
                        EditorState::Edit(editor.enter_edit_mode())
                    }
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
                EditorState::Command(mut editor) => match editor.handle_input(key_event) {
                    EditorAction::Quit => {
                        self.should_quit = true;
                        EditorState::Command(editor)
                    }
                    EditorAction::EnterNavigateMode => {
                        EditorState::Navigate(editor.enter_navigate_mode())
                    }
                    EditorAction::Save(file_name) => {
                        let buffers = editor.get_buffers();
                        match self.save(buffers, file_name) {
                            Ok(s) => {
                                let mut status = String::from("Saved... ");
                                status.push_str(&s);
                                editor.set_error_line(status);
                            }
                            Err(s) => {
                                let mut status = String::from("Error Occurred... ");
                                status.push_str(&s);
                                editor.set_error_line(status);
                            }
                        }
                        EditorState::Command(editor)
                    }
                    EditorAction::SaveAndQuit(file_name) => {
                        let buffers = editor.get_buffers();
                        match self.save(buffers, file_name) {
                            Ok(_) => {
                                self.should_quit = true;
                            }
                            Err(s) => {
                                let mut status = String::from("Error Occurred... ");
                                status.push_str(&s);
                                editor.set_error_line(status);
                            }
                        }
                        EditorState::Command(editor)
                    }
                    EditorAction::QuitAll => {
                        self.should_quit = true;
                        EditorState::Command(editor)
                    }
                    _ => EditorState::Command(editor),
                },
            });
        }
    }
}

pub fn initialize_app() -> Helios {
    let alpha_buffer = HBuffer::new();
    let editor = Editor::<NavigateMode>::new(vec![alpha_buffer]);

    Helios::init(editor)
}

impl Widget for &Helios {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        if let Some(state) = &self.editor_state {
            let buffers = match state {
                EditorState::Navigate(ed) => ed.get_buffers(),
                EditorState::Command(ed) => ed.get_buffers(),
                EditorState::Edit(ed) => ed.get_buffers(),
                EditorState::Select(ed) => ed.get_buffers(),
            };

            let state_name = format!("{}", state);
            let cursor_position = match state {
                EditorState::Navigate(e) => e.get_cursor_position(),
                EditorState::Edit(e) => e.get_cursor_position(),
                EditorState::Select(e) => e.get_cursor_position(),
                EditorState::Command(e) => e.get_cursor_position(),
            };
            let (char_pos, line_pos) = cursor_position;

            let state_name = match state {
                EditorState::Navigate(_) => state_name.white(),
                EditorState::Edit(_) => state_name.green(),
                EditorState::Select(_) => state_name.yellow(),
                EditorState::Command(_) => state_name.light_red(),
            };

            let main_block = Block::bordered()
                .title_bottom(state_name)
                .title_top(".txt".to_string())
                .title_bottom(format!("{}:{}", line_pos + 1, char_pos + 1));

            let ratatui_lines: Vec<ratatui::text::Line> = buffers[0]
                .lines
                .iter()
                .map(|line| ratatui::text::Line::from(line.text.as_str()))
                .collect();

            let para = Paragraph::new(ratatui_lines);
            para.block(main_block).render(layout[0], buf);

            let command_text = match state {
                EditorState::Navigate(ed) => ed.get_command_line(),
                EditorState::Command(ed) => ed.get_command_line(),
                EditorState::Edit(ed) => ed.get_command_line(),
                EditorState::Select(ed) => ed.get_command_line(),
            };

            let error_text = match state {
                EditorState::Navigate(ed) => ed.get_error_line(),
                EditorState::Command(ed) => ed.get_error_line(),
                EditorState::Edit(ed) => ed.get_error_line(),
                EditorState::Select(ed) => ed.get_error_line(),
            };

            let status_text = if !error_text.is_empty() {
                Paragraph::new(error_text.clone()).style(
                    ratatui::style::Style::default()
                        .bg(ratatui::style::Color::Red)
                        .fg(ratatui::style::Color::White),
                )
            } else {
                Paragraph::new(command_text.clone())
            };

            status_text.block(Block::new()).render(layout[1], buf);
        }
    }
}
