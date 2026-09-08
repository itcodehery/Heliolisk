use ratatui::style::Style;
use ratatui::text::Span;
use crate::config::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Function,
    StringLiteral,
    Number,
    Comment,
    TypeName,
    Operator,
    Punctuation,
    Plain,
}

pub struct SyntaxHighlighter;

impl SyntaxHighlighter {
    /// Tokenize and style a line of text based on simple language heuristics
    pub fn highlight_line(line: &str, ext: &str, theme: &Theme) -> Vec<Span<'static>> {
        match ext {
            "rs" | "rust" => Self::highlight_rust(line, theme),
            "toml" => Self::highlight_toml(line, theme),
            "md" | "markdown" => Self::highlight_markdown(line, theme),
            _ => Self::highlight_generic(line, theme),
        }
    }

    fn style_for_token(token: TokenType, theme: &Theme) -> Style {
        match token {
            TokenType::Keyword => Style::default().fg(theme.keyword),
            TokenType::Function => Style::default().fg(theme.function),
            TokenType::StringLiteral => Style::default().fg(theme.string),
            TokenType::Number => Style::default().fg(theme.number),
            TokenType::Comment => Style::default().fg(theme.comment),
            TokenType::TypeName => Style::default().fg(theme.type_name),
            TokenType::Operator => Style::default().fg(theme.operator),
            TokenType::Punctuation => Style::default().fg(theme.fg),
            TokenType::Plain => Style::default().fg(theme.fg),
        }
    }

    pub fn highlight_rust(line: &str, theme: &Theme) -> Vec<Span<'static>> {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            return vec![Span::styled(line.to_string(), Self::style_for_token(TokenType::Comment, theme))];
        }

        let mut spans = Vec::new();
        let mut chars = line.char_indices().peekable();
        let mut last_idx = 0;

        while let Some(&(idx, ch)) = chars.peek() {
            if ch == '"' {
                // String literal
                chars.next();
                while let Some(&(_end_idx, next_ch)) = chars.peek() {
                    chars.next();
                    if next_ch == '\\' {
                        let _ = chars.next();
                    } else if next_ch == '"' {
                        break;
                    }
                }
                let end = chars.peek().map(|(i, _)| *i).unwrap_or(line.len());
                spans.push(Span::styled(line[idx..end].to_string(), Self::style_for_token(TokenType::StringLiteral, theme)));
                last_idx = end;
            } else if ch == '/' && line[idx..].starts_with("//") {
                // Line comment
                spans.push(Span::styled(line[idx..].to_string(), Self::style_for_token(TokenType::Comment, theme)));
                last_idx = line.len();
                break;
            } else if ch.is_ascii_digit() {
                // Number
                chars.next();
                while let Some(&(_, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_digit() || next_ch == '.' || next_ch == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                let end = chars.peek().map(|(i, _)| *i).unwrap_or(line.len());
                spans.push(Span::styled(line[idx..end].to_string(), Self::style_for_token(TokenType::Number, theme)));
                last_idx = end;
            } else if ch.is_alphabetic() || ch == '_' {
                // Word (keyword, type, function, or ident)
                chars.next();
                while let Some(&(_, next_ch)) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                let end = chars.peek().map(|(i, _)| *i).unwrap_or(line.len());
                let word = &line[idx..end];

                let token_type = match word {
                    "fn" | "let" | "mut" | "pub" | "use" | "mod" | "struct" | "enum"
                    | "impl" | "match" | "if" | "else" | "while" | "loop" | "for" | "in"
                    | "return" | "break" | "continue" | "as" | "where" | "trait" | "type"
                    | "const" | "static" | "ref" | "self" | "Self" | "true" | "false" => TokenType::Keyword,
                    "Option" | "Result" | "Some" | "None" | "Ok" | "Err" | "String" | "Vec"
                    | "usize" | "u8" | "u16" | "u32" | "u64" | "i32" | "i64" | "bool" | "char" => TokenType::TypeName,
                    _ => {
                        if line[end..].trim_start().starts_with('(') {
                            TokenType::Function
                        } else {
                            TokenType::Plain
                        }
                    }
                };

                spans.push(Span::styled(word.to_string(), Self::style_for_token(token_type, theme)));
                last_idx = end;
            } else {
                chars.next();
                let end = chars.peek().map(|(i, _)| *i).unwrap_or(line.len());
                let op = &line[idx..end];
                let token_type = if "+-*/%=&|!<>:;,".contains(ch) {
                    TokenType::Operator
                } else {
                    TokenType::Plain
                };
                spans.push(Span::styled(op.to_string(), Self::style_for_token(token_type, theme)));
                last_idx = end;
            }
        }

        if last_idx < line.len() {
            spans.push(Span::styled(line[last_idx..].to_string(), Self::style_for_token(TokenType::Plain, theme)));
        }

        spans
    }

    pub fn highlight_toml(line: &str, theme: &Theme) -> Vec<Span<'static>> {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            vec![Span::styled(line.to_string(), Self::style_for_token(TokenType::Comment, theme))]
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            vec![Span::styled(line.to_string(), Self::style_for_token(TokenType::Keyword, theme))]
        } else if let Some((k, v)) = line.split_once('=') {
            vec![
                Span::styled(k.to_string(), Self::style_for_token(TokenType::TypeName, theme)),
                Span::styled("=".to_string(), Self::style_for_token(TokenType::Operator, theme)),
                Span::styled(v.to_string(), Self::style_for_token(TokenType::StringLiteral, theme)),
            ]
        } else {
            vec![Span::styled(line.to_string(), Self::style_for_token(TokenType::Plain, theme))]
        }
    }

    pub fn highlight_markdown(line: &str, theme: &Theme) -> Vec<Span<'static>> {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            vec![Span::styled(line.to_string(), Self::style_for_token(TokenType::Keyword, theme))]
        } else if trimmed.starts_with('`') || trimmed.starts_with("```") {
            vec![Span::styled(line.to_string(), Self::style_for_token(TokenType::StringLiteral, theme))]
        } else if trimmed.starts_with('-') || trimmed.starts_with('*') {
            vec![Span::styled(line.to_string(), Self::style_for_token(TokenType::Operator, theme))]
        } else {
            vec![Span::styled(line.to_string(), Self::style_for_token(TokenType::Plain, theme))]
        }
    }

    pub fn highlight_generic(line: &str, theme: &Theme) -> Vec<Span<'static>> {
        vec![Span::styled(line.to_string(), Style::default().fg(theme.fg))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_rust_keywords_and_comments() {
        let theme = Theme::tokyo_night();

        // Line comment
        let comment_spans = SyntaxHighlighter::highlight_line("// This is a comment", "rs", &theme);
        assert_eq!(comment_spans.len(), 1);
        assert_eq!(comment_spans[0].style.fg, Some(theme.comment));

        // Keyword line
        let code_spans = SyntaxHighlighter::highlight_line("pub fn main() {}", "rs", &theme);
        assert!(!code_spans.is_empty());
        assert_eq!(code_spans[0].content, "pub");
        assert_eq!(code_spans[0].style.fg, Some(theme.keyword));
    }

    #[test]
    fn test_syntax_toml_key_value() {
        let theme = Theme::tokyo_night();
        let spans = SyntaxHighlighter::highlight_line("name = \"heliolisk\"", "toml", &theme);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].content, "name ");
        assert_eq!(spans[1].content, "=");
        assert_eq!(spans[2].content, " \"heliolisk\"");
    }
}

