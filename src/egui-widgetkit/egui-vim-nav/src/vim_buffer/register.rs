//! Vim registers and clipboard store.

use std::collections::HashMap;

/// Storage for Vim registers (unnamed `""`, yank `"0`, named `"a`..`"z`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VimRegisters {
    /// Unnamed default register (`""`).
    pub unnamed: String,
    /// Yank register (`"0`).
    pub yank: String,
    /// Named registers (`"a` .. `"z`).
    pub named: HashMap<char, String>,
    /// Last inline search query (character, is_till, is_forward).
    pub last_search: Option<(char, bool, bool)>,
}

impl VimRegisters {
    /// Creates an empty register store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores text into a register.
    pub fn store(&mut self, reg: Option<char>, text: String) {
        if let Some(r) = reg {
            let r_lower = r.to_ascii_lowercase();
            if r.is_ascii_uppercase() {
                // Uppercase register appends
                self.named.entry(r_lower).or_default().push_str(&text);
            } else {
                self.named.insert(r_lower, text.clone());
            }
        }
        self.unnamed = text;
    }

    /// Stores yanked text into `"0` and `""`.
    pub fn store_yank(&mut self, reg: Option<char>, text: String) {
        self.yank = text.clone();
        self.store(reg, text);
    }

    /// Retrieves text from a register (or unnamed if `None`).
    pub fn get(&self, reg: Option<char>) -> &str {
        if let Some(r) = reg {
            let r_lower = r.to_ascii_lowercase();
            if r == '0' {
                return &self.yank;
            }
            if let Some(s) = self.named.get(&r_lower) {
                return s.as_str();
            }
        }
        &self.unnamed
    }
}
