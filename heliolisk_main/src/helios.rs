use std::io::Result;
use std::sync::mpsc::{self, Receiver, Sender};

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

/// The Global App State for Heliolisk
/// # Stores
/// - Editor State
/// - Quittable State
pub struct Helios {
    editor_state: Option<EditorState>,
    should_quit: bool,
    save_tx: Sender<std::result::Result<String, String>>,
    save_rx: Receiver<std::result::Result<String, String>>,
}

impl Helios {
    pub fn init(editor: Editor) -> Self {
        dbg!("Helios: Initialized Editor State");
        let (save_tx, save_rx) = mpsc::channel();
        Self {
            editor_state: Some(EditorState::Navigate(editor)),
            should_quit: false,
            save_tx,
            save_rx,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.should_quit {
            self.check_background_tasks();
            self.check_error_expiry();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    pub fn check_error_expiry(&mut self) {
        if let Some(state) = &mut self.editor_state {
            match state {
                EditorState::Navigate(ed) => ed.check_error_expiry(),
                EditorState::Command(ed) => ed.check_error_expiry(),
                EditorState::Edit(ed) => ed.check_error_expiry(),
                EditorState::Select(ed) => ed.check_error_expiry(),
            }
        }
    }

    pub fn check_background_tasks(&mut self) {
        while let Ok(res) = self.save_rx.try_recv() {
            let msg = match res {
                Ok(s) => s,
                Err(e) => format!("Error: {}", e),
            };
            if let Some(state) = &mut self.editor_state {
                match state {
                    EditorState::Navigate(ed) => ed.set_error_line(msg),
                    EditorState::Command(ed) => ed.set_error_line(msg),
                    EditorState::Edit(ed) => ed.set_error_line(msg),
                    EditorState::Select(ed) => ed.set_error_line(msg),
                }
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        // 1. Update Viewport (Mutation phase)
        if let Some(state) = &mut self.editor_state {
            let height = (layout[0].height as usize).saturating_sub(2);
            match state {
                EditorState::Navigate(ed) => ed.update_viewport(height),
                EditorState::Command(ed) => ed.update_viewport(height),
                EditorState::Edit(ed) => ed.update_viewport(height),
                EditorState::Select(ed) => ed.update_viewport(height),
            }
        }

        // 2. Render Content (Immutable render)
        frame.render_widget(&*self, area);

        // 3. Render Cursor and manage offsets (Immutable access)
        if let Some(state) = &self.editor_state {
            let buffers = match state {
                EditorState::Navigate(ed) => ed.get_buffers(),
                EditorState::Command(ed) => ed.get_buffers(),
                EditorState::Edit(ed) => ed.get_buffers(),
                EditorState::Select(ed) => ed.get_buffers(),
            };

            let height = (layout[0].height as usize).saturating_sub(2);
            let (cursor_col, cursor_line) = match state {
                EditorState::Navigate(ed) => ed.get_cursor_position(),
                EditorState::Command(ed) => ed.get_cursor_position(),
                EditorState::Edit(ed) => ed.get_cursor_position(),
                EditorState::Select(ed) => ed.get_cursor_position(),
            };

            let scroll_offset = match state {
                EditorState::Navigate(ed) => ed.get_scroll_offset(),
                EditorState::Command(ed) => ed.get_scroll_offset(),
                EditorState::Edit(ed) => ed.get_scroll_offset(),
                EditorState::Select(ed) => ed.get_scroll_offset(),
            };

            // Calculate visual cursor position relative to the viewport
            if cursor_line >= scroll_offset && cursor_line < scroll_offset + height {
                let line_text = buffers[0].text.line(cursor_line);
                let visual_col: usize = line_text
                    .chars()
                    .take(cursor_col)
                    .map(|c| if c == '\t' { 4 } else { 1 })
                    .sum();

                let visual_cursor_y = cursor_line - scroll_offset;
                let cursor_x = layout[0].x + visual_col as u16 + 1; // +1 for left border
                let cursor_y = layout[0].y + visual_cursor_y as u16 + 1; // +1 for top border

                if cursor_x < layout[0].x + layout[0].width - 1
                    && cursor_y < layout[0].y + layout[0].height - 1
                {
                    frame.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
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
                    EditorAction::EnterEditModeInNewLine => {
                        let mut ed = editor.enter_edit_mode();
                        ed.open_line_below();
                        EditorState::Edit(ed)
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
                        // Determine effective filename: User input > Existing Buffer Path > Default
                        let current_path = editor.get_active_buffer().file_path.clone();
                        let effective_name = file_name
                            .clone()
                            .or(current_path)
                            .unwrap_or_else(|| "helios_test.txt".to_string());

                        // Update buffer path so future saves use it
                        editor.get_active_buffer_mut().file_path = Some(effective_name.clone());

                        let buffer_clone = editor.get_active_buffer().clone();
                        let tx = self.save_tx.clone();

                        editor.set_error_line("Saving in background...".to_string());

                        std::thread::spawn(move || {
                            match file_ops::write_buffer_to_file(
                                &buffer_clone,
                                Some(effective_name.clone()),
                            ) {
                                Ok(_) => {
                                    let _ = tx.send(Ok(format!("Saved {}", effective_name)));
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(format!("Save failed: {}", e)));
                                }
                            }
                        });

                        EditorState::Command(editor)
                    }
                    EditorAction::SaveAndQuit(file_name) => {
                        // Determine effective filename: User input > Existing Buffer Path > Default
                        let current_path = editor.get_active_buffer().file_path.clone();
                        let effective_name = file_name
                            .clone()
                            .or(current_path)
                            .unwrap_or_else(|| "helios_test.txt".to_string());

                        // We use get_active_buffer() instead of direct buffers access for consistency
                        let buffer = editor.get_active_buffer();

                        match file_ops::write_buffer_to_file(buffer, Some(effective_name)) {
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
                    EditorAction::AddNewBuffer => EditorState::Command(editor),
                    _ => EditorState::Command(editor),
                },
            });
        }
    }
}

pub fn initialize_app() -> Helios {
    let args: Vec<String> = std::env::args().collect();
    let initial_buffer = if let Some(file_name) = args.get(1) {
        let path = std::path::PathBuf::from(file_name);
        match file_ops::load_file(&path) {
            Ok(buffer) => buffer,
            Err(_) => {
                // File likely doesn't exist, create new buffer with this path
                let mut buffer = HBuffer::new();
                buffer.file_path = Some(file_name.clone());
                buffer.file_format = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("txt")
                    .to_string();
                buffer
            }
        }
    } else {
        HBuffer::new()
    };

    let editor = Editor::<NavigateMode>::new(vec![initial_buffer]);

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
                .title_top(
                    buffers[0]
                        .file_path
                        .clone()
                        .unwrap_or_else(|| ".txt".to_string()),
                )
                .title_bottom(format!("{}:{}", line_pos + 1, char_pos + 1));

            let scroll_offset = match state {
                EditorState::Navigate(e) => e.get_scroll_offset(),
                EditorState::Edit(e) => e.get_scroll_offset(),
                EditorState::Select(e) => e.get_scroll_offset(),
                EditorState::Command(e) => e.get_scroll_offset(),
            };

            let viewport_height = (layout[0].height as usize).saturating_sub(2);

            let ratatui_lines: Vec<ratatui::text::Line> = (0..viewport_height)
                .map(|i| {
                    let line_idx = scroll_offset + i;
                    let line_cow = buffers[0].text.line(line_idx);
                    // Remove newline characters for rendering if necessary, though Ratatui handles them usually.
                    // Ropey lines include newlines.
                    let line_str = line_cow
                        .trim_end_matches(['\n', '\r'])
                        .replace("\t", "    ");
                    ratatui::text::Line::from(line_str)
                })
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
                        .fg(ratatui::style::Color::Black),
                )
            } else {
                Paragraph::new(command_text.clone())
            };

            status_text.block(Block::new()).render(layout[1], buf);
        }
    }
}
