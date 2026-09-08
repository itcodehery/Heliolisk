use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::config::Theme;
use crate::explorer::{FileExplorer, FileNodeType};

pub struct FileExplorerWidget<'a> {
    pub explorer: &'a FileExplorer,
    pub theme: &'a Theme,
    pub is_focused: bool,
}

impl<'a> Widget for FileExplorerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.is_focused {
            self.theme.border_focused
        } else {
            self.theme.border
        };

        let block = Block::bordered()
            .title(" \u{f07c} Explorer ")
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(self.theme.bg));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let height = inner_area.height as usize;
        let visible_nodes = self.explorer.flatten_visible();
        let scroll = self.explorer.scroll_offset;

        let lines: Vec<Line> = (0..height)
            .filter_map(|i| {
                let idx = scroll + i;
                visible_nodes.get(idx).map(|(depth, node)| {
                    let is_selected = idx == self.explorer.selected_index;
                    let indent = "  ".repeat(*depth);

                    let (icon, color) = match node.node_type {
                        FileNodeType::Directory => {
                            let arrow = if node.is_expanded { "\u{f078} \u{f07c} " } else { "\u{f054} \u{f07b} " }; //   vs  
                            (arrow, self.theme.explorer_dir)
                        }
                        FileNodeType::File => {
                            let ext = node.name.rsplit('.').next().unwrap_or("");
                            let file_icon = match ext {
                                "rs" => "\u{e7a8} ",      //  Rust
                                "toml" => "\u{e6b2} ",    //  Config / TOML
                                "md" => "\u{e609} ",      //  Markdown
                                "json" => "\u{e60b} ",    //  JSON
                                "lock" => "\u{f023} ",    //  Lock
                                "gitignore" => "\u{e702} ", //  Git
                                "sh" | "bash" | "nu" => "\u{f489} ", //  Terminal
                                _ => "\u{f15b} ",         //  Generic file
                            };
                            (file_icon, self.theme.explorer_file)
                        }
                    };

                    let content = format!("{}{}{}", indent, icon, node.name);
                    let mut style = Style::default().fg(color);

                    if is_selected {
                        style = style
                            .bg(self.theme.explorer_selected_bg)
                            .fg(self.theme.explorer_selected_fg)
                            .add_modifier(Modifier::BOLD);
                    }

                    Line::from(vec![Span::styled(content, style)])
                })
            })
            .collect();

        Paragraph::new(lines).render(inner_area, buf);
    }
}
