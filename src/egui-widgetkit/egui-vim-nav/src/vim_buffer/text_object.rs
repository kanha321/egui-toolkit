//! Text object targets and boundary detection (iw, aw, i", a", i(, a(, etc.).

use std::ops::Range;
use crate::vim_buffer::motion::clamp_cursor;

/// Scope modifier for text objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextObjectScope {
    /// Inner text object excluding delimiters or surrounding whitespace (`i`).
    Inner,
    /// Around / inclusive text object including delimiters or whitespace (`a`).
    A,
}

/// Target symbol for text objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextObjectTarget {
    /// Word (`w`).
    Word,
    /// Big whitespace-delimited word (`W`).
    BigWord,
    /// Double quotes (`"`).
    DoubleQuote,
    /// Single quotes (`'`).
    SingleQuote,
    /// Backticks (`` ` ``).
    Backtick,
    /// Parentheses (`(` or `)` or `b`).
    Parentheses,
    /// Square brackets (`[` or `]`).
    Brackets,
    /// Curly braces (`{` or `}` or `B`).
    Braces,
    /// Angle brackets (`<` or `>`).
    AngleBrackets,
}

/// A combined text object specification (e.g. `iw` or `a"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextObject {
    /// Scope (Inner vs Around).
    pub scope: TextObjectScope,
    /// Target type.
    pub target: TextObjectTarget,
}

impl TextObject {
    /// Resolves the text object span covering the cursor in the given string.
    pub fn resolve_range(&self, text: &str, cursor: usize) -> Option<Range<usize>> {
        if text.is_empty() {
            return None;
        }

        let cursor = clamp_cursor(text, cursor);

        match self.target {
            TextObjectTarget::Word => resolve_word(text, cursor, self.scope, false),
            TextObjectTarget::BigWord => resolve_word(text, cursor, self.scope, true),
            TextObjectTarget::DoubleQuote => resolve_quote(text, cursor, '"', self.scope),
            TextObjectTarget::SingleQuote => resolve_quote(text, cursor, '\'', self.scope),
            TextObjectTarget::Backtick => resolve_quote(text, cursor, '`', self.scope),
            TextObjectTarget::Parentheses => resolve_pair(text, cursor, '(', ')', self.scope),
            TextObjectTarget::Brackets => resolve_pair(text, cursor, '[', ']', self.scope),
            TextObjectTarget::Braces => resolve_pair(text, cursor, '{', '}', self.scope),
            TextObjectTarget::AngleBrackets => resolve_pair(text, cursor, '<', '>', self.scope),
        }
    }
}

fn is_word_char(c: char, big: bool) -> bool {
    if big {
        !c.is_whitespace()
    } else {
        c.is_alphanumeric() || c == '_'
    }
}

fn resolve_word(
    text: &str,
    cursor: usize,
    scope: TextObjectScope,
    big: bool,
) -> Option<Range<usize>> {
    let current_char = text[cursor..].chars().next()?;
    let on_word = is_word_char(current_char, big);

    // 1. Find start of current token
    let mut start = cursor;
    for (idx, c) in text[..cursor].char_indices().rev() {
        if is_word_char(c, big) == on_word && (big || !c.is_whitespace()) {
            start = idx;
        } else {
            break;
        }
    }

    // 2. Find end of current token
    let mut end = cursor + current_char.len_utf8();
    for (idx, c) in text[end..].char_indices() {
        if is_word_char(c, big) == on_word && (big || !c.is_whitespace()) {
            end += c.len_utf8();
            let _ = idx;
        } else {
            break;
        }
    }

    if scope == TextObjectScope::Inner {
        Some(start..end)
    } else {
        // Expand to include trailing whitespace
        let mut trailing_end = end;
        for (_, c) in text[end..].char_indices() {
            if c.is_whitespace() {
                trailing_end += c.len_utf8();
            } else {
                break;
            }
        }

        if trailing_end > end {
            Some(start..trailing_end)
        } else {
            // Or leading whitespace if no trailing whitespace exists
            let mut leading_start = start;
            for (idx, c) in text[..start].char_indices().rev() {
                if c.is_whitespace() {
                    leading_start = idx;
                } else {
                    break;
                }
            }
            Some(leading_start..end)
        }
    }
}

fn resolve_quote(
    text: &str,
    cursor: usize,
    quote: char,
    scope: TextObjectScope,
) -> Option<Range<usize>> {
    let before = &text[..cursor];
    let _after = &text[cursor..];

    // Find opening quote before or at cursor
    let open_idx = if text[cursor..].starts_with(quote) {
        cursor
    } else {
        before.rfind(quote)?
    };

    // Find closing quote strictly after opening quote
    let rest = &text[open_idx + quote.len_utf8()..];
    let close_relative = rest.find(quote)?;
    let close_idx = open_idx + quote.len_utf8() + close_relative;

    if scope == TextObjectScope::Inner {
        Some((open_idx + quote.len_utf8())..close_idx)
    } else {
        Some(open_idx..(close_idx + quote.len_utf8()))
    }
}

fn resolve_pair(
    text: &str,
    cursor: usize,
    open: char,
    close: char,
    scope: TextObjectScope,
) -> Option<Range<usize>> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();

    // Scan backwards to find matching open
    let mut depth = 0;
    let mut open_idx = None;

    for &(idx, c) in chars.iter().filter(|&&(idx, _)| idx <= cursor).rev() {
        if c == close && idx < cursor {
            depth += 1;
        } else if c == open {
            if depth == 0 {
                open_idx = Some(idx);
                break;
            } else {
                depth -= 1;
            }
        }
    }

    let open_pos = open_idx?;

    // Scan forwards from open_pos to find matching close
    depth = 0;
    let mut close_idx = None;

    for &(idx, c) in chars.iter().filter(|&&(idx, _)| idx >= open_pos) {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                close_idx = Some(idx);
                break;
            }
        }
    }

    let close_pos = close_idx?;

    if scope == TextObjectScope::Inner {
        Some((open_pos + open.len_utf8())..close_pos)
    } else {
        Some(open_pos..(close_pos + close.len_utf8()))
    }
}
