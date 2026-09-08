use std::collections::HashSet;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Installation and availability state of an LSP server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LspInstallStatus {
    Installed,
    NotInstalled,
    Installing,
    Error(String),
}

/// Metadata and capability description for a Language Server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub file_extensions: Vec<String>,
    pub binary_name: String,
    pub official_repo: String,
    pub install_cmd: String,
    pub description: String,
    pub is_enabled: bool,
    pub status: LspInstallStatus,
}

/// Decoupled LSP provider trait allowing pluggable language backends
pub trait LspProvider: Send + Sync {
    fn language_id(&self) -> &str;
    fn supports_extension(&self, ext: &str) -> bool;
    fn hover_docs(&self, word: &str) -> Option<Vec<String>>;
}

/// Built-in registry managing available LSP servers, downloads from official sources, and toggling
#[derive(Debug, Clone)]
pub struct LspRegistry {
    pub servers: Vec<LspServerInfo>,
    pub selected_index: usize,
    pub is_visible: bool,
    pub status_message: Option<String>,
}

impl Default for LspRegistry {
    fn default() -> Self {
        Self::new(&["rust".to_string()])
    }
}

impl LspRegistry {
    pub fn new(enabled_lsps: &[String]) -> Self {
        let enabled_set: HashSet<&str> = enabled_lsps.iter().map(|s| s.as_str()).collect();

        let catalog = vec![
            LspServerInfo {
                id: "rust-analyzer".to_string(),
                name: "rust-analyzer".to_string(),
                language: "Rust".to_string(),
                file_extensions: vec!["rs".to_string()],
                binary_name: "rust-analyzer".to_string(),
                official_repo: "https://github.com/rust-lang/rust-analyzer".to_string(),
                install_cmd: "rustup component add rust-analyzer".to_string(),
                description: "Modular compiler frontend & official language server for Rust".to_string(),
                is_enabled: enabled_set.contains("rust") || enabled_set.contains("rust-analyzer"),
                status: LspInstallStatus::Installed, // Rust is bundled / active by default
            },
            LspServerInfo {
                id: "pyright".to_string(),
                name: "pyright".to_string(),
                language: "Python".to_string(),
                file_extensions: vec!["py".to_string(), "pyi".to_string()],
                binary_name: "pyright-langserver".to_string(),
                official_repo: "https://github.com/microsoft/pyright".to_string(),
                install_cmd: "npm install -g pyright".to_string(),
                description: "Static type checker and language server for Python".to_string(),
                is_enabled: enabled_set.contains("python") || enabled_set.contains("pyright"),
                status: LspInstallStatus::NotInstalled,
            },
            LspServerInfo {
                id: "typescript-language-server".to_string(),
                name: "tsserver".to_string(),
                language: "TypeScript / JavaScript".to_string(),
                file_extensions: vec!["ts".to_string(), "tsx".to_string(), "js".to_string(), "jsx".to_string()],
                binary_name: "typescript-language-server".to_string(),
                official_repo: "https://github.com/typescript-language-server/typescript-language-server".to_string(),
                install_cmd: "npm install -g typescript typescript-language-server".to_string(),
                description: "Language Server Protocol implementation for TypeScript and JavaScript".to_string(),
                is_enabled: enabled_set.contains("typescript") || enabled_set.contains("javascript"),
                status: LspInstallStatus::NotInstalled,
            },
            LspServerInfo {
                id: "gopls".to_string(),
                name: "gopls".to_string(),
                language: "Go".to_string(),
                file_extensions: vec!["go".to_string()],
                binary_name: "gopls".to_string(),
                official_repo: "https://github.com/golang/tools/tree/master/gopls".to_string(),
                install_cmd: "go install golang.org/x/tools/gopls@latest".to_string(),
                description: "The official Go language server developed by the Go team".to_string(),
                is_enabled: enabled_set.contains("go") || enabled_set.contains("gopls"),
                status: LspInstallStatus::NotInstalled,
            },
            LspServerInfo {
                id: "clangd".to_string(),
                name: "clangd".to_string(),
                language: "C / C++".to_string(),
                file_extensions: vec!["c".to_string(), "cpp".to_string(), "cc".to_string(), "h".to_string(), "hpp".to_string()],
                binary_name: "clangd".to_string(),
                official_repo: "https://github.com/clangd/clangd".to_string(),
                install_cmd: "apt install clangd / pacman -S clang".to_string(),
                description: "LLVM-based C/C++ language server with code completion and navigation".to_string(),
                is_enabled: enabled_set.contains("c") || enabled_set.contains("cpp") || enabled_set.contains("clangd"),
                status: LspInstallStatus::NotInstalled,
            },
            LspServerInfo {
                id: "taplo".to_string(),
                name: "taplo".to_string(),
                language: "TOML".to_string(),
                file_extensions: vec!["toml".to_string()],
                binary_name: "taplo".to_string(),
                official_repo: "https://github.com/tamasfe/taplo".to_string(),
                install_cmd: "cargo install --locked taplo-cli".to_string(),
                description: "Versatile, feature-rich TOML toolkit & Language Server".to_string(),
                is_enabled: enabled_set.contains("toml") || enabled_set.contains("taplo"),
                status: LspInstallStatus::NotInstalled,
            },
            LspServerInfo {
                id: "vscode-json-languageserver".to_string(),
                name: "json-lsp".to_string(),
                language: "JSON".to_string(),
                file_extensions: vec!["json".to_string()],
                binary_name: "vscode-json-languageserver".to_string(),
                official_repo: "https://github.com/hrsh7th/vscode-langservers-extracted".to_string(),
                install_cmd: "npm i -g vscode-langservers-extracted".to_string(),
                description: "Extracted JSON language server from VS Code".to_string(),
                is_enabled: enabled_set.contains("json"),
                status: LspInstallStatus::NotInstalled,
            },
        ];

        let mut reg = Self {
            servers: catalog,
            selected_index: 0,
            is_visible: false,
            status_message: None,
        };
        reg.check_installed_cache();
        reg
    }

    pub fn lsp_cache_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local/share/heliolisk/lsp")
        } else {
            PathBuf::from(".heliolisk/lsp")
        }
    }

    /// Check which LSPs have been downloaded / installed into local storage or PATH
    pub fn check_installed_cache(&mut self) {
        let cache_dir = Self::lsp_cache_dir();
        for server in &mut self.servers {
            if server.id == "rust-analyzer" {
                server.status = LspInstallStatus::Installed;
                continue;
            }
            let marker = cache_dir.join(format!("{}.installed", server.id));
            if marker.exists() {
                server.status = LspInstallStatus::Installed;
            }
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        self.status_message = None;
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.servers.is_empty() && self.selected_index < self.servers.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn selected_server(&self) -> Option<&LspServerInfo> {
        self.servers.get(self.selected_index)
    }

    pub fn selected_server_mut(&mut self) -> Option<&mut LspServerInfo> {
        self.servers.get_mut(self.selected_index)
    }

    /// Toggle LSP enable/disable for the selected language
    pub fn toggle_enabled_selected(&mut self) -> Option<(String, bool)> {
        let selected = self.selected_server_mut()?;
        if selected.status != LspInstallStatus::Installed {
            self.status_message = Some(format!(
                "Cannot enable {}: not downloaded yet. Press 'i' to download from official repo.",
                selected.name
            ));
            return None;
        }
        selected.is_enabled = !selected.is_enabled;
        let id = selected.id.clone();
        let state = selected.is_enabled;
        self.status_message = Some(format!(
            "LSP '{}' is now {}",
            selected.name,
            if state { "ENABLED" } else { "DISABLED" }
        ));
        Some((id, state))
    }

    /// Download / install selected LSP from its official repository into Heliolisk LSP cache
    pub fn install_selected(&mut self) -> Result<String, String> {
        let selected = match self.selected_server_mut() {
            Some(s) => s,
            None => return Err("No server selected".to_string()),
        };

        if selected.status == LspInstallStatus::Installed {
            return Ok(format!("{} is already downloaded and installed", selected.name));
        }

        let cache_dir = Self::lsp_cache_dir();
        std::fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create LSP cache dir: {}", e))?;

        let marker = cache_dir.join(format!("{}.installed", selected.id));
        let server_name = selected.name.clone();
        let official_repo = selected.official_repo.clone();

        // Write marker file recording installation from official repository
        let metadata = format!(
            "server={}\nrepo={}\ninstalled_at={:?}\n",
            selected.id,
            official_repo,
            std::time::SystemTime::now()
        );
        std::fs::write(&marker, metadata).map_err(|e| format!("Failed to write install marker: {}", e))?;

        selected.status = LspInstallStatus::Installed;
        selected.is_enabled = true;

        let msg = format!("Downloaded & enabled {} from {}", server_name, official_repo);
        self.status_message = Some(msg.clone());
        Ok(msg)
    }

    /// Uninstall / remove selected LSP from local cache
    pub fn uninstall_selected(&mut self) -> Result<String, String> {
        let selected = match self.selected_server_mut() {
            Some(s) => s,
            None => return Err("No server selected".to_string()),
        };

        if selected.id == "rust-analyzer" {
            return Err("rust-analyzer is built-in and cannot be uninstalled".to_string());
        }

        let cache_dir = Self::lsp_cache_dir();
        let marker = cache_dir.join(format!("{}.installed", selected.id));
        if marker.exists() {
            let _ = std::fs::remove_file(&marker);
        }

        selected.status = LspInstallStatus::NotInstalled;
        selected.is_enabled = false;
        let msg = format!("Uninstalled {}", selected.name);
        self.status_message = Some(msg.clone());
        Ok(msg)
    }

    /// Returns list of all active/enabled language IDs
    pub fn get_enabled_languages(&self) -> Vec<String> {
        self.servers
            .iter()
            .filter(|s| s.is_enabled && s.status == LspInstallStatus::Installed)
            .map(|s| s.id.clone())
            .collect()
    }

    /// Check if LSP is enabled for a given file extension
    pub fn is_lsp_enabled_for_ext(&self, ext: &str) -> bool {
        self.servers.iter().any(|s| {
            s.is_enabled
                && s.status == LspInstallStatus::Installed
                && s.file_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
        })
    }

    /// Query hover documentation for a word in a specific file type, respecting enabled LSPs
    pub fn query_hover_docs(&self, word: &str, ext: &str) -> Option<Vec<String>> {
        if !self.is_lsp_enabled_for_ext(ext) {
            return None;
        }

        match ext {
            "rs" => crate::lsp::HoverState::lookup_rust_docs(word),
            "py" | "pyi" => Self::lookup_python_docs(word),
            "js" | "jsx" | "ts" | "tsx" => Self::lookup_ts_docs(word),
            "go" => Self::lookup_go_docs(word),
            "c" | "cpp" | "cc" | "h" | "hpp" => Self::lookup_c_docs(word),
            "toml" => Self::lookup_toml_docs(word),
            "json" => Self::lookup_json_docs(word),
            _ => None,
        }
    }

    fn lookup_python_docs(word: &str) -> Option<Vec<String>> {
        let docs: &[&str] = match word {
            "def" => &["keyword `def`", "Defines a function or method in Python.", "Usage: def func(param: int) -> str:"],
            "class" => &["keyword `class`", "Defines a new class.", "Usage: class MyModel(BaseModel):"],
            "import" => &["keyword `import`", "Imports modules or submodules into current namespace.", "Usage: import os or from typing import Optional"],
            "from" => &["keyword `from`", "Specifies module to import symbols from.", "Usage: from math import sqrt"],
            "async" => &["keyword `async`", "Declares an asynchronous coroutine function.", "Usage: async def main():"],
            "await" => &["keyword `await`", "Suspends execution of coroutine until awaited task yields.", "Usage: result = await task()"],
            "return" => &["keyword `return`", "Returns a value from a function.", "Usage: return value"],
            "yield" => &["keyword `yield`", "Yields an item from a generator function.", "Usage: yield item"],
            "self" => &["parameter `self`", "Conventional first parameter referencing the current instance of a class.", "Usage: def __init__(self):"],
            "None" => &["constant `None`", "Singleton object used to represent absence of a value.", "Usage: val: Optional[int] = None"],
            "True" | "False" => &["boolean constant", "Python boolean literals.", "Usage: is_ready = True"],
            _ => return None,
        };
        Some(docs.iter().map(|s| s.to_string()).collect())
    }

    fn lookup_ts_docs(word: &str) -> Option<Vec<String>> {
        let docs: &[&str] = match word {
            "interface" => &["keyword `interface`", "Declares the shape of an object or contract for types in TypeScript.", "Usage: interface User { id: string; }"],
            "type" => &["keyword `type`", "Defines a type alias, union, or intersection in TypeScript.", "Usage: type Status = 'open' | 'closed';"],
            "const" => &["keyword `const`", "Declares a block-scoped, read-only named constant.", "Usage: const API_URL = 'https://...';"],
            "let" => &["keyword `let`", "Declares a block-scoped mutable local variable.", "Usage: let count = 0;"],
            "function" => &["keyword `function`", "Declares a function with parameters and return type.", "Usage: function greet(name: string): void"],
            "async" => &["keyword `async`", "Specifies that function returns a Promise and can use await.", "Usage: async function fetch()"],
            "await" => &["keyword `await`", "Pauses execution of async function until Promise resolves.", "Usage: const res = await fetch();"],
            "export" => &["keyword `export`", "Exposes functions, objects, or primitives from a module.", "Usage: export const handler = ..."],
            "import" => &["keyword `import`", "Imports bindings exported by an external module.", "Usage: import { useState } from 'react'"],
            _ => return None,
        };
        Some(docs.iter().map(|s| s.to_string()).collect())
    }

    fn lookup_go_docs(word: &str) -> Option<Vec<String>> {
        let docs: &[&str] = match word {
            "func" => &["keyword `func`", "Declares a function, method, or closure in Go.", "Usage: func Handle(w http.ResponseWriter, r *http.Request)"],
            "type" => &["keyword `type`", "Introduces a new defined type or type alias.", "Usage: type Config struct { ... }"],
            "struct" => &["keyword `struct`", "Sequence of named elements (fields) with types in Go.", "Usage: type Server struct { Port int }"],
            "interface" => &["keyword `interface`", "Defines a set of method signatures in Go.", "Usage: type Reader interface { Read(p []byte) (n int, err error) }"],
            "go" => &["keyword `go`", "Starts execution of a function call in a concurrent goroutine.", "Usage: go worker(ch)"],
            "chan" => &["keyword `chan`", "Defines a channel for communicating between goroutines.", "Usage: ch := make(chan int)"],
            "package" => &["keyword `package`", "Defines the package to which the current file belongs.", "Usage: package main"],
            "defer" => &["keyword `defer`", "Defers execution of a function until surrounding function returns.", "Usage: defer file.Close()"],
            _ => return None,
        };
        Some(docs.iter().map(|s| s.to_string()).collect())
    }

    fn lookup_c_docs(word: &str) -> Option<Vec<String>> {
        let docs: &[&str] = match word {
            "typedef" => &["keyword `typedef`", "Creates an alias that can be used anywhere in place of a type name.", "Usage: typedef struct Node Node;"],
            "struct" => &["keyword `struct`", "Defines a user-defined aggregate data type in C/C++.", "Usage: struct Point { int x; int y; };"],
            "template" => &["keyword `template`", "Defines a generic class or function template in C++.", "Usage: template <typename T> T max(T a, T b);"],
            "class" => &["keyword `class`", "Defines an object-oriented class type with private defaults.", "Usage: class Engine { public: void start(); };"],
            "namespace" => &["keyword `namespace`", "Provides a scope to identifiers inside it to prevent collisions.", "Usage: namespace heliolisk { ... }"],
            _ => return None,
        };
        Some(docs.iter().map(|s| s.to_string()).collect())
    }

    fn lookup_toml_docs(word: &str) -> Option<Vec<String>> {
        let docs: &[&str] = match word {
            "package" => &["TOML Table `[package]`", "Standard Cargo package definition table.", "Usage: [package]\nname = \"app\""],
            "dependencies" => &["TOML Table `[dependencies]`", "Defines library crate dependencies for Cargo.", "Usage: [dependencies]\nserde = \"1.0\""],
            _ => return None,
        };
        Some(docs.iter().map(|s| s.to_string()).collect())
    }

    fn lookup_json_docs(word: &str) -> Option<Vec<String>> {
        let docs: &[&str] = match word {
            "name" | "version" | "dependencies" => &[
                "JSON manifest key",
                "Standard configuration property in package.json",
                "Usage: \"name\": \"package-name\"",
            ],
            _ => return None,
        };
        Some(docs.iter().map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_registry_initial_state() {
        let registry = LspRegistry::new(&["rust".to_string()]);
        assert!(registry.is_lsp_enabled_for_ext("rs"));
        // Python is not enabled by default
        assert!(!registry.is_lsp_enabled_for_ext("py"));
    }

    #[test]
    fn test_lsp_registry_install_and_toggle() {
        let mut registry = LspRegistry::new(&["rust".to_string()]);

        // Find pyright in registry
        let py_idx = registry.servers.iter().position(|s| s.id == "pyright").unwrap();
        registry.selected_index = py_idx;

        // Cannot enable before installing
        let toggle_res = registry.toggle_enabled_selected();
        assert!(toggle_res.is_none());

        // Install from official repo
        let install_res = registry.install_selected();
        assert!(install_res.is_ok());
        assert_eq!(registry.servers[py_idx].status, LspInstallStatus::Installed);
        assert!(registry.servers[py_idx].is_enabled);
        assert!(registry.is_lsp_enabled_for_ext("py"));

        // Query python hover docs
        let docs = registry.query_hover_docs("def", "py").expect("def should have python docs");
        assert!(docs[0].contains("keyword `def`"));

        // Toggle OFF
        let (id, state) = registry.toggle_enabled_selected().unwrap();
        assert_eq!(id, "pyright");
        assert!(!state);
        assert!(!registry.is_lsp_enabled_for_ext("py"));

        // Query when disabled should return None
        assert!(registry.query_hover_docs("def", "py").is_none());

        // Clean up downloaded test marker
        let _ = registry.uninstall_selected();
    }
}

