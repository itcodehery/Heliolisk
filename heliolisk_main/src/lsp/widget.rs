use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};

use crate::config::Theme;
use crate::lsp::registry::{LspInstallStatus, LspRegistry};

pub struct LspExplorerWidget<'a> {
    pub registry: &'a LspRegistry,
    pub theme: &'a Theme,
}

impl<'a> Widget for LspExplorerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.registry.is_visible {
            return;
        }

        // Center modal layout
        let popup_width = (area.width * 4 / 5).clamp(60, 95);
        let popup_height = (area.height * 4 / 5).clamp(18, 28);

        let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        let popup_rect = Rect::new(popup_x, popup_y, popup_width, popup_height);

        Clear.render(popup_rect, buf);

        let block = Block::bordered()
            .title(" \u{f085} Language Server (LSP) Manager ")
            .border_style(Style::default().fg(self.theme.border_focused))
            .style(Style::default().bg(self.theme.bg));

        let inner = block.inner(popup_rect);
        block.render(popup_rect, buf);

        // Split into: [Server List (left)] and [Server Details (right)]
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(inner);

        let list_area = chunks[0];
        let detail_area = chunks[1];

        // 1. Render Server List
        let mut list_lines = Vec::new();
        list_lines.push(Line::from(vec![
            Span::styled("  STATUS  ", Style::default().fg(self.theme.comment)),
            Span::styled("LANGUAGE / SERVER", Style::default().fg(self.theme.comment).add_modifier(Modifier::BOLD)),
        ]));
        list_lines.push(Line::from(""));

        for (idx, server) in self.registry.servers.iter().enumerate() {
            let is_selected = idx == self.registry.selected_index;

            let (status_icon, status_color) = match &server.status {
                LspInstallStatus::Installed => {
                    if server.is_enabled {
                        ("\u{f00c} ON ", Color::Green) //  ON
                    } else {
                        ("\u{f00d} OFF", Color::DarkGray) //  OFF
                    }
                }
                LspInstallStatus::NotInstalled => ("\u{f019} GET", Color::Yellow), //  GET
                LspInstallStatus::Installing => ("\u{f110} ...", Color::Cyan),     //  ...
                LspInstallStatus::Error(_) => ("\u{f071} ERR", Color::Red),        //  ERR
            };

            let prefix = if is_selected { "> " } else { "  " };

            let mut style = Style::default().fg(self.theme.fg);
            if is_selected {
                style = style
                    .bg(self.theme.explorer_selected_bg)
                    .fg(self.theme.explorer_selected_fg)
                    .add_modifier(Modifier::BOLD);
            }

            list_lines.push(Line::from(vec![
                Span::styled(format!("{}{:<8} ", prefix, status_icon), Style::default().fg(status_color)),
                Span::styled(format!("{:<12} ({})", server.name, server.language), style),
            ]));
        }

        Paragraph::new(list_lines).render(list_area, buf);

        // 2. Render Detail Pane
        let mut detail_lines = Vec::new();
        if let Some(selected) = self.registry.selected_server() {
            detail_lines.push(Line::from(vec![
                Span::styled("Server: ", Style::default().fg(self.theme.keyword).add_modifier(Modifier::BOLD)),
                Span::styled(&selected.name, Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD)),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Language: ", Style::default().fg(self.theme.keyword)),
                Span::styled(&selected.language, Style::default().fg(self.theme.function)),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Extensions: ", Style::default().fg(self.theme.keyword)),
                Span::styled(selected.file_extensions.join(", "), Style::default().fg(self.theme.type_name)),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Repository: ", Style::default().fg(self.theme.keyword)),
                Span::styled(&selected.official_repo, Style::default().fg(self.theme.string)),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Install Command: ", Style::default().fg(self.theme.keyword)),
                Span::styled(&selected.install_cmd, Style::default().fg(self.theme.number)),
            ]));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled("Description: ", Style::default().fg(self.theme.comment)),
            ]));
            detail_lines.push(Line::from(selected.description.clone()));

            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(self.theme.keyword)),
                match &selected.status {
                    LspInstallStatus::Installed => {
                        if selected.is_enabled {
                            Span::styled("\u{f00c} Installed & Active", Style::default().fg(Color::Green))
                        } else {
                            Span::styled("\u{f00d} Installed (Disabled)", Style::default().fg(Color::DarkGray))
                        }
                    }
                    LspInstallStatus::NotInstalled => Span::styled("\u{f019} Not Installed", Style::default().fg(Color::Yellow)),
                    LspInstallStatus::Installing => Span::styled("Installing from repo...", Style::default().fg(Color::Cyan)),
                    LspInstallStatus::Error(e) => Span::styled(format!("Error: {}", e), Style::default().fg(Color::Red)),
                },
            ]));
        }

        // Instructions footer
        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(vec![
            Span::styled("Keybinds:", Style::default().fg(self.theme.comment).add_modifier(Modifier::BOLD)),
        ]));
        detail_lines.push(Line::from("  [Enter/Space] Toggle ON/OFF  |  [i] Download/Install"));
        detail_lines.push(Line::from("  [u] Uninstall/Remove         |  [Esc/q] Close Explorer"));

        if let Some(msg) = &self.registry.status_message {
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled(format!(" \u{f05a} {}", msg), Style::default().fg(self.theme.function).add_modifier(Modifier::BOLD)),
            ]));
        }

        let detail_block = Block::bordered()
            .title(" Details & Actions ")
            .border_style(Style::default().fg(self.theme.border));
        let detail_inner = detail_block.inner(detail_area);
        detail_block.render(detail_area, buf);
        Paragraph::new(detail_lines).render(detail_inner, buf);
    }
}
