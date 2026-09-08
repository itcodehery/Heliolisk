pub mod registry;
pub mod widget;

#[allow(unused_imports)]
pub use registry::{LspInstallStatus, LspProvider, LspRegistry, LspServerInfo};
pub use widget::LspExplorerWidget;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};

use crate::config::Theme;

/// State for the floating hover documentation popup
#[derive(Debug, Clone, Default)]
pub struct HoverState {
    pub is_visible: bool,
    pub title: String,
    pub content: Vec<String>,
}

impl HoverState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, title: String, content: Vec<String>) {
        self.title = title;
        self.content = content;
        self.is_visible = true;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.title.clear();
        self.content.clear();
    }

    /// Retrieve detailed documentation and usage for Rust keywords, standard types, and symbols
    pub fn lookup_rust_docs(word: &str) -> Option<Vec<String>> {
        let docs: &[&str] = match word {
            // Keywords & Declarations
            "fn" => &[
                "keyword `fn`",
                "Defines a function, method, or closure parameter list.",
                "Usage: fn function_name(param: Type) -> ReturnType { ... }",
            ],
            "let" => &[
                "keyword `let`",
                "Binds a value to a local variable pattern.",
                "Usage: let [mut] name: Type = value;",
            ],
            "mut" => &[
                "keyword `mut`",
                "Specifies that a variable binding, reference, or pointer is mutable.",
                "Usage: let mut x = 5; or fn foo(&mut self)",
            ],
            "pub" => &[
                "keyword `pub`",
                "Specifies public visibility for modules, functions, structs, or fields.",
                "Usage: pub fn api() {} or pub(crate) struct Internal;",
            ],
            "struct" => &[
                "keyword `struct`",
                "Defines a structured record, tuple-struct, or unit-struct.",
                "Usage: struct Point { x: f64, y: f64 }",
            ],
            "enum" => &[
                "keyword `enum`",
                "Defines an algebraic data type (enumeration) with multiple variants.",
                "Usage: enum Message { Quit, Move { x: i32, y: i32 } }",
            ],
            "impl" => &[
                "keyword `impl`",
                "Implements inherent methods or trait definitions for a given type.",
                "Usage: impl MyStruct { ... } or impl Trait for MyStruct { ... }",
            ],
            "trait" => &[
                "keyword `trait`",
                "Defines an interface that types can implement to share behavior.",
                "Usage: trait Printable { fn print(&self); }",
            ],
            "type" => &[
                "keyword `type`",
                "Defines a type alias or associated type inside a trait.",
                "Usage: type Kilometers = i32;",
            ],
            "use" => &[
                "keyword `use`",
                "Brings external or module items into the current local scope.",
                "Usage: use std::collections::HashMap;",
            ],
            "mod" => &[
                "keyword `mod`",
                "Declares or defines a module.",
                "Usage: mod sub_module; or mod tests { ... }",
            ],
            "const" => &[
                "keyword `const`",
                "Defines compile-time constant values or items evaluated at compile-time.",
                "Usage: const MAX_POINTS: u32 = 100_000;",
            ],
            "static" => &[
                "keyword `static`",
                "Defines global variable with `'static` lifetime across the entire program.",
                "Usage: static BANNER: &str = \"WELCOME\";",
            ],
            "ref" => &[
                "keyword `ref`",
                "Binds by reference during pattern matching.",
                "Usage: if let Some(ref val) = opt { ... }",
            ],
            "self" => &[
                "keyword `self`",
                "The current instance receiver of a method, or current module scope.",
                "Usage: fn method(&self) or use self::module;",
            ],
            "Self" => &[
                "type `Self`",
                "The implementing type within a trait or impl block.",
                "Usage: fn new() -> Self { Self { ... } }",
            ],
            "as" => &[
                "keyword `as`",
                "Performs primitive casting, disambiguates traits, or renames imports.",
                "Usage: let x = 42 as f64; or use std::io::Result as IoResult;",
            ],
            "where" => &[
                "keyword `where`",
                "Specifies generic type bounds and lifetime constraints cleanly.",
                "Usage: fn foo<T>(x: T) where T: Clone + Send { ... }",
            ],

            // Control flow
            "if" => &[
                "keyword `if`",
                "Conditional branching expression.",
                "Usage: if condition { ... } else { ... }",
            ],
            "else" => &[
                "keyword `else`",
                "Alternative branch for an `if` expression.",
                "Usage: if cond { a } else { b }",
            ],
            "match" => &[
                "keyword `match`",
                "Pattern-matching control flow expression.",
                "Usage: match value { Pattern1 => expr, Pattern2 => expr }",
            ],
            "while" => &[
                "keyword `while`",
                "Loop that executes continuously while a condition remains true.",
                "Usage: while condition { ... } or while let Some(x) = opt { ... }",
            ],
            "loop" => &[
                "keyword `loop`",
                "Infinite loop expression (exit via `break`). Can return a value.",
                "Usage: loop { if done { break result; } }",
            ],
            "for" => &[
                "keyword `for`",
                "Iterates over elements produced by an IntoIterator type.",
                "Usage: for item in collection { ... }",
            ],
            "in" => &[
                "keyword `in`",
                "Used in `for` loops to iterate over an iterator or range.",
                "Usage: for i in 0..10 { ... }",
            ],
            "return" => &[
                "keyword `return`",
                "Returns early from the enclosing function with an optional value.",
                "Usage: return Ok(result);",
            ],
            "break" => &[
                "keyword `break`",
                "Exits a loop immediately, optionally returning a value from `loop`.",
                "Usage: break 'label val;",
            ],
            "continue" => &[
                "keyword `continue`",
                "Skips the rest of current loop iteration and advances to next.",
                "Usage: continue 'label;",
            ],
            "async" => &[
                "keyword `async`",
                "Transforms a block or function into a Future.",
                "Usage: async fn fetch() -> Result<Data> { ... }",
            ],
            "await" => &[
                "keyword `await`",
                "Suspends execution until a Future completes.",
                "Usage: let res = future.await;",
            ],
            "unsafe" => &[
                "keyword `unsafe`",
                "Indicates code or blocks containing operations bypassing safety checks.",
                "Usage: unsafe { raw_ptr.read() }",
            ],

            // Standard Library core types
            "Option" => &[
                "enum `Option<T>`",
                "Type representing an optional value: either Some(T) or None.",
                "Usage: let opt: Option<i32> = Some(10);",
            ],
            "Some" => &[
                "enum variant `Option::Some(T)`",
                "Contains the value in an Option.",
                "Usage: Some(value)",
            ],
            "None" => &[
                "enum variant `Option::None`",
                "Signifies absence of a value in an Option.",
                "Usage: None",
            ],
            "Result" => &[
                "enum `Result<T, E>`",
                "Type for returning and propagating errors: Ok(T) or Err(E).",
                "Usage: fn read() -> Result<String, io::Error>",
            ],
            "Ok" => &[
                "enum variant `Result::Ok(T)`",
                "Contains the success value in a Result.",
                "Usage: Ok(result)",
            ],
            "Err" => &[
                "enum variant `Result::Err(E)`",
                "Contains the error value in a Result.",
                "Usage: Err(error)",
            ],
            "Vec" => &[
                "struct `Vec<T>`",
                "A contiguous, growable array type stored on the heap.",
                "Usage: let mut v = Vec::new(); v.push(1);",
            ],
            "String" => &[
                "struct `String`",
                "An owned, growable, UTF-8 encoded string buffer.",
                "Usage: let s = String::from(\"hello\");",
            ],
            "str" => &[
                "primitive type `str`",
                "String slice: an immutable, dynamically-sized UTF-8 view.",
                "Usage: let s: &str = \"borrowed\";",
            ],
            "bool" => &[
                "primitive type `bool`",
                "Boolean type representing either `true` or `false`.",
                "Usage: let is_valid: bool = true;",
            ],
            "true" | "false" => &[
                "literal boolean",
                "Boolean truth value literal.",
                "Usage: true or false",
            ],
            "usize" | "isize" => &[
                "primitive integer",
                "Pointer-sized integer type matching target architecture width.",
                "Usage: let idx: usize = 0;",
            ],
            "u8" | "u16" | "u32" | "u64" | "u128" => &[
                "primitive integer",
                "Unsigned fixed-width integer type.",
                "Usage: let val: u32 = 42;",
            ],
            "i8" | "i16" | "i32" | "i64" | "i128" => &[
                "primitive integer",
                "Signed fixed-width integer type.",
                "Usage: let val: i32 = -42;",
            ],
            "f32" | "f64" => &[
                "primitive float",
                "IEEE 754 floating point number type.",
                "Usage: let pi: f64 = 3.14159265;",
            ],
            "char" => &[
                "primitive type `char`",
                "A 4-byte unicode scalar value.",
                "Usage: let ch: char = '🦀';",
            ],
            _ => return None,
        };

        Some(docs.iter().map(|s| s.to_string()).collect())
    }
}

pub struct HoverPopupWidget<'a> {
    pub hover: &'a HoverState,
    pub theme: &'a Theme,
    pub cursor_pos: (u16, u16),
}

impl<'a> Widget for HoverPopupWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.hover.is_visible || self.hover.content.is_empty() {
            return;
        }

        // Calculate size and position near cursor
        let max_content_width = self
            .hover
            .content
            .iter()
            .map(|l| l.len())
            .max()
            .unwrap_or(20)
            .max(self.hover.title.len());

        let width = (max_content_width as u16 + 4).min(area.width.saturating_sub(4)).max(25);
        let height = ((self.hover.content.len() as u16) + 2)
            .min(area.height.saturating_sub(4))
            .max(3);

        let (cx, cy) = self.cursor_pos;
        let x = if cx + width < area.width {
            cx + 1
        } else {
            cx.saturating_sub(width)
        };

        let y = if cy + height < area.height {
            cy + 1
        } else {
            cy.saturating_sub(height)
        };

        let popup_area = Rect::new(x, y, width, height);

        // Clear underlying buffer so text behind the popup doesn't bleed through
        Clear.render(popup_area, buf);

        let block = Block::bordered()
            .title(format!(" \u{f02d} {} ", self.hover.title))
            .border_style(Style::default().fg(self.theme.border_focused))
            .style(Style::default().bg(self.theme.status_bg));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let lines: Vec<Line> = self
            .hover
            .content
            .iter()
            .map(|text| {
                Line::from(vec![Span::styled(
                    text.clone(),
                    Style::default()
                        .fg(self.theme.fg)
                        .add_modifier(Modifier::BOLD),
                )])
            })
            .collect();

        Paragraph::new(lines).render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_rust_docs_keywords() {
        let self_docs = HoverState::lookup_rust_docs("Self").expect("Self should have docs");
        assert!(self_docs[0].contains("type `Self`"));

        let fn_docs = HoverState::lookup_rust_docs("fn").expect("fn should have docs");
        assert!(fn_docs[0].contains("keyword `fn`"));

        let match_docs = HoverState::lookup_rust_docs("match").expect("match should have docs");
        assert!(match_docs[0].contains("keyword `match`"));
    }

    #[test]
    fn test_lookup_rust_docs_types() {
        let opt_docs = HoverState::lookup_rust_docs("Option").expect("Option should have docs");
        assert!(opt_docs[0].contains("enum `Option<T>`"));

        let vec_docs = HoverState::lookup_rust_docs("Vec").expect("Vec should have docs");
        assert!(vec_docs[0].contains("struct `Vec<T>`"));
    }
}

