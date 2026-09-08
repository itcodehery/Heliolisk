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
    buffer::HBuffer,
    editor::{Editor, EditorAction, Mode},
    file_ops,
};

/// The Global App State for Heliolisk
pub struct Helios {
    editor: Editor,
    should_quit: bool,
    save_tx: Sender<std::result::Result<String, String>>,
    save_rx: Receiver<std::result::Result<String, String>>,
}

impl Helios {
    pub fn init(editor: Editor) -> Self {
        dbg!("Helios: Initialized Editor State");
        let (save_tx, save_rx) = mpsc::channel();
        Self {
            editor,
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
        self.editor.check_error_expiry();
    }

    pub fn check_background_tasks(&mut self) {
        while let Ok(res) = self.save_rx.try_recv() {
            let msg = match res {
                Ok(s) => s,
                Err(e) => format!("Error: {}", e),
            };
            self.editor.set_error_line(msg);
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        // 1. Update Viewport (Mutation phase)
        let height = (layout[0].height as usize).saturating_sub(2);
        self.editor.update_viewport(height);

        // 2. Render Content (Immutable render)
        frame.render_widget(&*self, area);

        // 3. Render Cursor and manage offsets (Immutable access)
        let active_buffer = self.editor.get_active_buffer();
        let (cursor_col, cursor_line) = self.editor.get_cursor_position();
        let scroll_offset = self.editor.get_scroll_offset();

        // Calculate visual cursor position relative to the viewport
        if cursor_line >= scroll_offset && cursor_line < scroll_offset + height {
            let line_text = active_buffer.text.line(cursor_line);
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

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let action = self.editor.handle_input(key_event);
        match action {
            EditorAction::Quit | EditorAction::QuitAll => {
                self.should_quit = true;
            }
            EditorAction::EnterNavigateMode => {
                self.editor.set_mode(Mode::Navigate);
            }
            EditorAction::EnterEditMode => {
                self.editor.enter_edit_mode();
            }
            EditorAction::EnterEditModeInNewLine => {
                self.editor.enter_edit_mode();
                self.editor.open_line_below();
            }
            EditorAction::EnterCommandMode => {
                self.editor.enter_command_mode();
            }
            EditorAction::EnterSelectMode => {
                self.editor.enter_select_mode();
            }
            EditorAction::Save(file_name) => {
                let current_path = self.editor.get_active_buffer().file_path.clone();
                let effective_name = file_name
                    .or(current_path)
                    .unwrap_or_else(|| "helios_test.txt".to_string());

                self.editor.get_active_buffer_mut().file_path = Some(effective_name.clone());

                let buffer_clone = self.editor.get_active_buffer().clone();
                let tx = self.save_tx.clone();

                self.editor.set_error_line("Saving in background...".to_string());

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
            }
            EditorAction::SaveAndQuit(file_name) => {
                let current_path = self.editor.get_active_buffer().file_path.clone();
                let effective_name = file_name
                    .or(current_path)
                    .unwrap_or_else(|| "helios_test.txt".to_string());

                let buffer = self.editor.get_active_buffer();

                match file_ops::write_buffer_to_file(buffer, Some(effective_name)) {
                    Ok(_) => {
                        self.should_quit = true;
                    }
                    Err(s) => {
                        let mut status = String::from("Error Occurred... ");
                        status.push_str(&s);
                        self.editor.set_error_line(status);
                    }
                }
            }
            EditorAction::AddNewBuffer => {
                self.editor.add_new_buffer();
            }
            EditorAction::DebugPrintLinesToConsole => {}
            EditorAction::DebugPrintCurrentLineToConsole => {}
            EditorAction::None => {}
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

    let editor = Editor::new(vec![initial_buffer]);

    Helios::init(editor)
}

impl Widget for &Helios {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let active_buffer = self.editor.get_active_buffer();
        let mode = self.editor.mode();
        let state_name = format!("{}", mode);
        let (char_pos, line_pos) = self.editor.get_cursor_position();

        let state_name = match mode {
            Mode::Navigate => state_name.white(),
            Mode::Edit => state_name.green(),
            Mode::Select => state_name.yellow(),
            Mode::Command => state_name.light_red(),
        };

        let main_block = Block::bordered()
            .title_bottom(state_name)
            .title_top(
                active_buffer
                    .file_path
                    .clone()
                    .unwrap_or_else(|| ".txt".to_string()),
            )
            .title_bottom(format!("{}:{}", line_pos + 1, char_pos + 1));

        let scroll_offset = self.editor.get_scroll_offset();
        let viewport_height = (layout[0].height as usize).saturating_sub(2);

        let ratatui_lines: Vec<ratatui::text::Line> = (0..viewport_height)
            .map(|i| {
                let line_idx = scroll_offset + i;
                let line_cow = active_buffer.text.line(line_idx);
                let line_str = line_cow
                    .trim_end_matches(['\n', '\r'])
                    .replace('\t', "    ");
                ratatui::text::Line::from(line_str)
            })
            .collect();

        let para = Paragraph::new(ratatui_lines);
        para.block(main_block).render(layout[0], buf);

        let command_text = self.editor.get_command_line();
        let error_text = self.editor.get_error_line();

        let status_text = if !error_text.is_empty() {
            Paragraph::new(error_text).style(
                ratatui::style::Style::default()
                    .bg(ratatui::style::Color::Red)
                    .fg(ratatui::style::Color::Black),
            )
        } else {
            Paragraph::new(command_text)
        };

        status_text.block(Block::new()).render(layout[1], buf);
    }
}

