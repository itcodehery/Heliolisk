use std::io::Result;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    buffer::HBuffer,
    config::{HelioliskConfig, Theme, ThemePreset},
    editor::{Editor, EditorAction, Mode},
    explorer::{FileExplorer, widget::FileExplorerWidget},
    file_ops,
    lsp::{HoverPopupWidget, HoverState, LspExplorerWidget, LspRegistry},
    syntax::SyntaxHighlighter,
};

pub struct SaveResult {
    pub buffer_idx: usize,
    pub version: u64,
    pub result: std::result::Result<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    Editor,
    Explorer,
    LspManager,
}

/// The Global App State for Heliolisk
pub struct Helios {
    pub editor: Editor,
    pub config: HelioliskConfig,
    pub theme: Theme,
    pub explorer: FileExplorer,
    pub hover: HoverState,
    pub lsp_registry: LspRegistry,
    pub focused_pane: FocusedPane,
    pub should_quit: bool,
    pub buffer_save_versions: std::collections::HashMap<usize, u64>,
    pub save_tx: Sender<SaveResult>,
    pub save_rx: Receiver<SaveResult>,
}

impl Helios {
    pub fn init(editor: Editor) -> Self {
        let (save_tx, save_rx) = mpsc::channel();
        let config = HelioliskConfig::load();
        let theme_preset = ThemePreset::from_name(&config.theme).unwrap_or(ThemePreset::TokyoNight);
        let theme = Theme::preset(theme_preset);
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let explorer = FileExplorer::new(&current_dir, config.sidebar_width);
        let lsp_registry = LspRegistry::new(&config.enabled_lsps);

        Self {
            editor,
            config,
            theme,
            explorer,
            hover: HoverState::new(),
            lsp_registry,
            focused_pane: FocusedPane::Editor,
            should_quit: false,
            buffer_save_versions: std::collections::HashMap::new(),
            save_tx,
            save_rx,
        }
    }

    pub fn set_theme(&mut self, name: &str) {
        if let Some(preset) = ThemePreset::from_name(name) {
            self.theme = Theme::preset(preset);
            self.editor.set_error_line(format!("Switched theme to {}", self.theme.name));
        } else {
            self.editor.set_error_line(format!(
                "Unknown theme '{}'. Options: tokyo-night, catppuccin-mocha, gruvbox-dark, nord, solarized-dark",
                name
            ));
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
        while let Ok(save_res) = self.save_rx.try_recv() {
            let latest_version = self
                .buffer_save_versions
                .get(&save_res.buffer_idx)
                .copied()
                .unwrap_or(0);
            if save_res.version >= latest_version {
                let msg = match save_res.result {
                    Ok(s) => s,
                    Err(e) => format!("Error: {}", e),
                };
                self.editor.set_error_line(msg);
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let main_area = vertical_layout[0];
        let status_area = vertical_layout[1];

        // Horizontal split if explorer is visible
        let (explorer_area, editor_area) = if self.explorer.is_visible {
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Length(self.explorer.width),
                    Constraint::Min(1),
                ])
                .split(main_area);
            (Some(horizontal_layout[0]), horizontal_layout[1])
        } else {
            (None, main_area)
        };

        // 1. Update Viewports
        let editor_height = (editor_area.height as usize).saturating_sub(2);
        self.editor.update_viewport(editor_height);
        if let Some(exp_area) = explorer_area {
            let exp_height = (exp_area.height as usize).saturating_sub(2);
            self.explorer.update_viewport(exp_height);
        }

        // 2. Render Explorer if visible
        if let Some(exp_area) = explorer_area {
            let is_focused = self.focused_pane == FocusedPane::Explorer;
            frame.render_widget(
                FileExplorerWidget {
                    explorer: &self.explorer,
                    theme: &self.theme,
                    is_focused,
                },
                exp_area,
            );
        }

        // 3. Render Editor
        self.render_editor_pane(frame, editor_area);

        // 4. Render Status Line
        self.render_status_line(frame, status_area);

        // 5. Render Cursor & Hover Popup
        let active_buffer = self.editor.get_active_buffer();
        let (cursor_col, cursor_line) = self.editor.get_cursor_position();
        let scroll_offset = self.editor.get_scroll_offset();

        let mut screen_cursor = None;
        if cursor_line >= scroll_offset && cursor_line < scroll_offset + editor_height {
            let line_text = active_buffer.text.line(cursor_line);
            use unicode_width::UnicodeWidthChar;
            let visual_col: usize = line_text
                .chars()
                .take(cursor_col)
                .map(|c| if c == '\t' { 4 } else { c.width().unwrap_or(0) })
                .sum();

            let visual_cursor_y = cursor_line - scroll_offset;
            let cursor_x = editor_area.x + visual_col as u16 + 1;
            let cursor_y = editor_area.y + visual_cursor_y as u16 + 1;

            if cursor_x < editor_area.x + editor_area.width - 1
                && cursor_y < editor_area.y + editor_area.height - 1
            {
                screen_cursor = Some((cursor_x, cursor_y));
                if self.focused_pane == FocusedPane::Editor {
                    frame.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }

        // 6. Render Floating Hover Popup if active
        if self.hover.is_visible {
            let popup_pos = screen_cursor.unwrap_or((editor_area.x + 2, editor_area.y + 2));
            frame.render_widget(
                HoverPopupWidget {
                    hover: &self.hover,
                    theme: &self.theme,
                    cursor_pos: popup_pos,
                },
                area,
            );
        }

        // 7. Render LSP Explorer Modal if open
        if self.lsp_registry.is_visible {
            frame.render_widget(
                LspExplorerWidget {
                    registry: &self.lsp_registry,
                    theme: &self.theme,
                },
                area,
            );
        }
    }

    fn render_editor_pane(&self, frame: &mut Frame, area: Rect) {
        let active_buffer = self.editor.get_active_buffer();
        let mode = self.editor.mode();
        let state_name = format!("{}", mode);
        let (char_pos, line_pos) = self.editor.get_cursor_position();

        let mode_color = match mode {
            Mode::Navigate => self.theme.nav_mode,
            Mode::Edit => self.theme.edit_mode,
            Mode::Select => self.theme.select_mode,
            Mode::Command => self.theme.command_mode,
        };

        let border_color = if self.focused_pane == FocusedPane::Editor {
            self.theme.border_focused
        } else {
            self.theme.border
        };

        let file_name = active_buffer.file_path.clone().unwrap_or_else(|| "unnamed.txt".to_string());
        let ext = active_buffer
            .file_path
            .as_ref()
            .and_then(|p| p.rsplit('.').next())
            .unwrap_or(&active_buffer.file_format);

        let file_icon = match ext {
            "rs" => "\u{e7a8}",
            "toml" => "\u{e6b2}",
            "md" => "\u{e609}",
            "json" => "\u{e60b}",
            _ => "\u{f15b}",
        };

        let main_block = Block::bordered()
            .title_bottom(Span::styled(format!(" {} ", state_name), Style::default().fg(mode_color)))
            .title_top(Span::styled(
                format!(" {} {} ({}) ", file_icon, file_name, self.theme.name),
                Style::default().fg(self.theme.fg),
            ))
            .title_bottom(Span::styled(
                format!(" {}:{} ", line_pos + 1, char_pos + 1),
                Style::default().fg(self.theme.status_fg),
            ))
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(self.theme.bg));

        let scroll_offset = self.editor.get_scroll_offset();
        let viewport_height = (area.height as usize).saturating_sub(2);

        let ratatui_lines: Vec<Line> = (0..viewport_height)
            .map(|i| {
                let line_idx = scroll_offset + i;
                let line_cow = active_buffer.text.line(line_idx);
                let line_str = line_cow
                    .trim_end_matches(['\n', '\r'])
                    .replace('\t', "    ");

                let spans = SyntaxHighlighter::highlight_line(&line_str, ext, &self.theme);
                Line::from(spans)
            })
            .collect();

        Paragraph::new(ratatui_lines)
            .block(main_block)
            .render(area, frame.buffer_mut());
    }

    fn render_status_line(&self, frame: &mut Frame, area: Rect) {
        let command_text = self.editor.get_command_line();
        let error_text = self.editor.get_error_line();

        let status_widget = if !error_text.is_empty() {
            Paragraph::new(format!(" \u{f071} {}", error_text)).style(
                Style::default()
                    .bg(self.theme.error_bg)
                    .fg(self.theme.error_fg),
            )
        } else if !command_text.is_empty() {
            Paragraph::new(format!(" :{}", command_text)).style(
                Style::default()
                    .bg(self.theme.status_bg)
                    .fg(self.theme.status_fg),
            )
        } else {
            let info = format!(" \u{f013} <Space>e: Explorer | <Space>l: LSP | K: Hover | :theme <name> | Preset: {}", self.theme.name);
            Paragraph::new(info).style(
                Style::default()
                    .bg(self.theme.status_bg)
                    .fg(self.theme.status_fg),
            )
        };

        status_widget.render(area, frame.buffer_mut());
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
        // Dismiss hover window on any navigation or escape
        if self.hover.is_visible && (key_event.code == KeyCode::Esc || self.focused_pane == FocusedPane::Editor) {
            self.hover.hide();
        }

        // Delegate to LspManager if LSP manager has focus
        if self.focused_pane == FocusedPane::LspManager {
            match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => self.lsp_registry.move_down(),
                KeyCode::Char('k') | KeyCode::Up => self.lsp_registry.move_up(),
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some((_id, _state)) = self.lsp_registry.toggle_enabled_selected() {
                        self.config.enabled_lsps = self.lsp_registry.get_enabled_languages();
                        let _ = self.config.save();
                    }
                }
                KeyCode::Char('i') => {
                    if let Ok(msg) = self.lsp_registry.install_selected() {
                        self.config.enabled_lsps = self.lsp_registry.get_enabled_languages();
                        let _ = self.config.save();
                        self.editor.set_error_line(msg);
                    }
                }
                KeyCode::Char('u') => {
                    if let Ok(msg) = self.lsp_registry.uninstall_selected() {
                        self.config.enabled_lsps = self.lsp_registry.get_enabled_languages();
                        let _ = self.config.save();
                        self.editor.set_error_line(msg);
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.lsp_registry.is_visible = false;
                    self.focused_pane = FocusedPane::Editor;
                }
                _ => {}
            }
            return;
        }

        // Delegate to FileExplorer if explorer has focus
        if self.focused_pane == FocusedPane::Explorer {
            match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => self.explorer.move_down(),
                KeyCode::Char('k') | KeyCode::Up => self.explorer.move_up(),
                KeyCode::Char('h') => self.explorer.collapse_selected(),
                KeyCode::Char('r') => self.explorer.refresh(),
                KeyCode::Enter | KeyCode::Char('l') => {
                    if let Some(file_path) = self.explorer.toggle_or_open_selected() {
                        if file_path.is_file() {
                            if let Ok(buf) = file_ops::load_file(&file_path) {
                                self.editor.open_buffer(buf);
                                self.focused_pane = FocusedPane::Editor;
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.focused_pane = FocusedPane::Editor;
                }
                KeyCode::Char(' ') => {
                    // Space+e in explorer closes it
                    self.explorer.toggle_visibility();
                    self.focused_pane = FocusedPane::Editor;
                }
                _ => {}
            }
            return;
        }

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
            EditorAction::ToggleFileExplorer => {
                self.explorer.toggle_visibility();
                if self.explorer.is_visible {
                    self.focused_pane = FocusedPane::Explorer;
                } else {
                    self.focused_pane = FocusedPane::Editor;
                }
            }
            EditorAction::ToggleLspExplorer => {
                self.lsp_registry.toggle_visibility();
                if self.lsp_registry.is_visible {
                    self.focused_pane = FocusedPane::LspManager;
                } else {
                    self.focused_pane = FocusedPane::Editor;
                }
            }
            EditorAction::SetTheme(name) => {
                self.set_theme(&name);
            }
            EditorAction::TriggerHover => {
                self.trigger_hover();
            }
            EditorAction::Save(file_name) => {
                let buffer_idx = self.editor.get_focused_index();
                let current_path = self.editor.get_active_buffer().file_path.clone();
                let effective_name = file_name
                    .or(current_path)
                    .unwrap_or_else(|| "helios_test.txt".to_string());

                self.editor.get_active_buffer_mut().file_path = Some(effective_name.clone());

                let buffer_clone = self.editor.get_active_buffer().clone();
                let tx = self.save_tx.clone();

                let version_entry = self.buffer_save_versions.entry(buffer_idx).or_insert(0);
                *version_entry += 1;
                let version = *version_entry;

                self.editor.set_error_line("Saving in background...".to_string());

                std::thread::spawn(move || {
                    let result = match file_ops::write_buffer_to_file(
                        &buffer_clone,
                        Some(effective_name.clone()),
                    ) {
                        Ok(_) => Ok(format!("Saved {}", effective_name)),
                        Err(e) => Err(format!("Save failed: {}", e)),
                    };
                    let _ = tx.send(SaveResult {
                        buffer_idx,
                        version,
                        result,
                    });
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

    fn trigger_hover(&mut self) {
        let (col, line) = self.editor.get_cursor_position();
        let buf = self.editor.get_active_buffer();
        let line_text = buf.text.line(line);

        let ext = buf
            .file_path
            .as_ref()
            .and_then(|p| p.rsplit('.').next())
            .unwrap_or(&buf.file_format);

        // Extract token under cursor
        let chars: Vec<char> = line_text.chars().collect();
        if col < chars.len() {
            let mut start = col;
            while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                start -= 1;
            }
            let mut end = col;
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                end += 1;
            }

            let word: String = chars[start..end].iter().collect();
            if !word.is_empty() {
                let docs = if let Some(keyword_docs) = self.lsp_registry.query_hover_docs(&word, ext) {
                    keyword_docs
                } else if !self.lsp_registry.is_lsp_enabled_for_ext(ext) {
                    vec![
                        format!("Symbol: {}", word),
                        format!("LSP for '.{}' is not enabled or downloaded.", ext),
                        "Press <Space>l to open LSP Manager and install/enable support.".to_string(),
                    ]
                } else {
                    vec![
                        format!("Symbol: {}", word),
                        format!("Line {}, Column {}", line + 1, col + 1),
                        "Documentation preview (LSP ready)".into(),
                    ]
                };

                self.hover.show(format!("Doc: {}", word), docs);
                return;
            }
        }

        self.hover.show(
            "Hover".to_string(),
            vec![format!("Line: {}, Column: {}", line + 1, col + 1)],
        );
    }
}

pub fn initialize_app() -> Helios {
    let args: Vec<String> = std::env::args().collect();
    let initial_buffer = if let Some(file_name) = args.get(1) {
        let path = PathBuf::from(file_name);
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


