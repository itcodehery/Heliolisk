use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Normalized representation of a key press for mapping
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyChord {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        // Strip out irrelevant modifiers like NumLock or CapsLock
        let clean_mods = modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);
        Self { code, modifiers: clean_mods }
    }

    pub fn from_event(key: KeyEvent) -> Self {
        Self::new(key.code, key.modifiers)
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.starts_with('<') && s.ends_with('>') {
            let inner = &s[1..s.len() - 1];
            let parts: Vec<&str> = inner.split('-').collect();
            let mut mods = KeyModifiers::empty();

            for &p in &parts[..parts.len() - 1] {
                match p.to_lowercase().as_str() {
                    "c" | "ctrl" => mods |= KeyModifiers::CONTROL,
                    "a" | "alt" | "m" => mods |= KeyModifiers::ALT,
                    "s" | "shift" => mods |= KeyModifiers::SHIFT,
                    _ => {}
                }
            }

            let key_str = parts.last()?;
            let code = match key_str.to_lowercase().as_str() {
                "space" => KeyCode::Char(' '),
                "cr" | "enter" => KeyCode::Enter,
                "esc" => KeyCode::Esc,
                "bs" | "backspace" => KeyCode::Backspace,
                "tab" => KeyCode::Tab,
                "up" => KeyCode::Up,
                "down" => KeyCode::Down,
                "left" => KeyCode::Left,
                "right" => KeyCode::Right,
                single if single.chars().count() == 1 => KeyCode::Char(single.chars().next()?),
                _ => return None,
            };

            Some(Self::new(code, mods))
        } else if s.chars().count() == 1 {
            let c = s.chars().next()?;
            let mut mods = KeyModifiers::empty();
            if c.is_ascii_uppercase() {
                mods |= KeyModifiers::SHIFT;
            }
            Some(Self::new(KeyCode::Char(c), mods))
        } else {
            None
        }
    }
}

/// A node in the key mapping Trie
#[derive(Debug, Clone)]
pub struct KeyTrieNode<A: Clone> {
    pub children: HashMap<KeyChord, KeyTrieNode<A>>,
    pub action: Option<A>,
}

impl<A: Clone> Default for KeyTrieNode<A> {
    fn default() -> Self {
        Self {
            children: HashMap::new(),
            action: None,
        }
    }
}

impl<A: Clone> KeyTrieNode<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, sequence: &[KeyChord], action: A) {
        if sequence.is_empty() {
            self.action = Some(action);
            return;
        }

        let child = self.children.entry(sequence[0].clone()).or_default();
        child.insert(&sequence[1..], action);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchResult<A> {
    Complete(A),
    Prefix,
    None,
}

/// Router that processes key inputs against a Trie
#[derive(Debug, Clone)]
pub struct KeyRouter<A: Clone> {
    root: KeyTrieNode<A>,
    pending: Vec<KeyChord>,
}

impl<A: Clone> KeyRouter<A> {
    pub fn new(root: KeyTrieNode<A>) -> Self {
        Self {
            root,
            pending: Vec::new(),
        }
    }

    pub fn pending_keys(&self) -> &[KeyChord] {
        &self.pending
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub fn feed(&mut self, chord: KeyChord) -> MatchResult<A> {
        self.pending.push(chord);

        let mut current = &self.root;
        for c in &self.pending {
            if let Some(next) = current.children.get(c) {
                current = next;
            } else {
                self.pending.clear();
                return MatchResult::None;
            }
        }

        if let Some(action) = &current.action {
            let res = action.clone();
            self.pending.clear();
            MatchResult::Complete(res)
        } else if !current.children.is_empty() {
            MatchResult::Prefix
        } else {
            self.pending.clear();
            MatchResult::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keychord_parse() {
        let space = KeyChord::parse("<space>").unwrap();
        assert_eq!(space.code, KeyCode::Char(' '));

        let ctrl_s = KeyChord::parse("<c-s>").unwrap();
        assert_eq!(ctrl_s.code, KeyCode::Char('s'));
        assert!(ctrl_s.modifiers.contains(KeyModifiers::CONTROL));

        let single_char = KeyChord::parse("e").unwrap();
        assert_eq!(single_char.code, KeyCode::Char('e'));
    }

    #[test]
    fn test_key_router_multi_chord() {
        let mut root = KeyTrieNode::new();
        let space_e = vec![
            KeyChord::new(KeyCode::Char(' '), KeyModifiers::empty()),
            KeyChord::new(KeyCode::Char('e'), KeyModifiers::empty()),
        ];
        root.insert(&space_e, "toggle_explorer".to_string());

        let mut router = KeyRouter::new(root);

        // First key: space -> Prefix
        let res1 = router.feed(KeyChord::new(KeyCode::Char(' '), KeyModifiers::empty()));
        assert_eq!(res1, MatchResult::Prefix);

        // Second key: e -> Complete
        let res2 = router.feed(KeyChord::new(KeyCode::Char('e'), KeyModifiers::empty()));
        assert_eq!(res2, MatchResult::Complete("toggle_explorer".to_string()));
    }
}

