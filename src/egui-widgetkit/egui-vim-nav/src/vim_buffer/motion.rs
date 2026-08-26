//! Motion targets and cursor navigation calculation.

/// Fundamental Vim motions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VimMotion {
    /// Move cursor left one character (`h` / Backspace).
    Left,
    /// Move cursor right one character (`l` / Space).
    Right,
    /// Move cursor up one line (`k`).
    Up,
    /// Move cursor down one line (`j`).
    Down,

    /// Next word start (`w`).
    WordForward,
    /// Word end (`e`).
    WordEnd,
    /// Previous word start (`b`).
    WordBackward,
    /// Previous word end (`ge`).
    WordEndBackward,

    /// Next whitespace-delimited word start (`W`).
    BigWordForward,
    /// Whitespace-delimited word end (`E`).
    BigWordEnd,
    /// Previous whitespace-delimited word start (`B`).
    BigWordBackward,

    /// Start of line (`0`).
    LineStart,
    /// First non-blank character of line (`^`).
    FirstNonBlank,
    /// End of line (`$`).
    LineEnd,

    /// Forward character find inclusive (`f<char>`).
    FindChar(char),
    /// Forward character find exclusive (`t<char>`).
    TillChar(char),
    /// Backward character find inclusive (`F<char>`).
    FindCharBack(char),
    /// Backward character find exclusive (`T<char>`).
    TillCharBack(char),

    /// Repeat last inline find in same direction (`;`).
    RepeatFind,
    /// Repeat last inline find in opposite direction (`,`).
    RepeatFindRev,

    /// Start of buffer (`gg`).
    BufferStart,
    /// End of buffer (`G`).
    BufferEnd,

    /// Matching structural delimiter pair (`%`).
    MatchingPair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharClass {
    Word,
    Punct,
    Space,
}

#[inline]
fn classify(c: char) -> CharClass {
    if c.is_whitespace() {
        CharClass::Space
    } else if c.is_alphanumeric() || c == '_' {
        CharClass::Word
    } else {
        CharClass::Punct
    }
}

/// Resolves a motion from a current cursor position on a text slice.
///
/// Returns the resulting target byte offset, guaranteed to lie on a valid UTF-8 character boundary.
pub fn calculate_motion(
    text: &str,
    cursor: usize,
    motion: VimMotion,
    count: usize,
    last_find: Option<(char, bool, bool)>, // (char, is_till, forward)
) -> usize {
    if text.is_empty() {
        return 0;
    }

    let count = count.max(1);
    let mut current = clamp_cursor(text, cursor);

    match motion {
        VimMotion::Left => {
            for _ in 0..count {
                if let Some((prev_idx, _)) = text[..current].char_indices().next_back() {
                    current = prev_idx;
                } else {
                    break;
                }
            }
            current
        }

        VimMotion::Right => {
            for _ in 0..count {
                if let Some((next_idx, c)) = text[current..].char_indices().nth(1) {
                    current += next_idx;
                    let _ = c;
                } else {
                    break;
                }
            }
            current
        }

        VimMotion::Up => {
            // Find current column offset from line start
            let line_start = text[..current].rfind('\n').map_or(0, |i| i + 1);
            let col = current - line_start;

            if line_start == 0 {
                return 0; // Already on first line
            }

            let prev_line_end = line_start.saturating_sub(1);
            let prev_line_start = text[..prev_line_end].rfind('\n').map_or(0, |i| i + 1);
            let prev_line_len = prev_line_end - prev_line_start;

            clamp_cursor(text, prev_line_start + col.min(prev_line_len))
        }

        VimMotion::Down => {
            let line_start = text[..current].rfind('\n').map_or(0, |i| i + 1);
            let col = current - line_start;

            if let Some(next_newline) = text[current..].find('\n') {
                let next_line_start = current + next_newline + 1;
                let next_line_end = text[next_line_start..].find('\n').map_or(text.len(), |i| next_line_start + i);
                let next_line_len = next_line_end - next_line_start;
                clamp_cursor(text, next_line_start + col.min(next_line_len))
            } else {
                current
            }
        }

        VimMotion::WordForward => {
            for _ in 0..count {
                current = next_word_start(text, current);
            }
            current
        }

        VimMotion::WordEnd => {
            for _ in 0..count {
                current = next_word_end(text, current);
            }
            current
        }

        VimMotion::WordBackward => {
            for _ in 0..count {
                current = prev_word_start(text, current);
            }
            current
        }

        VimMotion::WordEndBackward => {
            for _ in 0..count {
                current = prev_word_end(text, current);
            }
            current
        }

        VimMotion::BigWordForward => {
            for _ in 0..count {
                current = next_big_word_start(text, current);
            }
            current
        }

        VimMotion::BigWordEnd => {
            for _ in 0..count {
                current = next_big_word_end(text, current);
            }
            current
        }

        VimMotion::BigWordBackward => {
            for _ in 0..count {
                current = prev_big_word_start(text, current);
            }
            current
        }

        VimMotion::LineStart => {
            text[..current].rfind('\n').map_or(0, |i| i + 1)
        }

        VimMotion::FirstNonBlank => {
            let line_start = text[..current].rfind('\n').map_or(0, |i| i + 1);
            let line_rest = &text[line_start..];
            if let Some((idx, _)) = line_rest.char_indices().find(|(_, c)| !c.is_whitespace()) {
                line_start + idx
            } else {
                line_start
            }
        }

        VimMotion::LineEnd => {
            if let Some(nl) = text[current..].find('\n') {
                let target = current + nl;
                if target > 0 {
                    clamp_cursor(text, target.saturating_sub(1))
                } else {
                    0
                }
            } else {
                clamp_cursor(text, text.len().saturating_sub(1))
            }
        }

        VimMotion::FindChar(target) => find_inline_char(text, current, target, false, true, count),
        VimMotion::TillChar(target) => find_inline_char(text, current, target, true, true, count),
        VimMotion::FindCharBack(target) => find_inline_char(text, current, target, false, false, count),
        VimMotion::TillCharBack(target) => find_inline_char(text, current, target, true, false, count),

        VimMotion::RepeatFind => {
            if let Some((target, is_till, forward)) = last_find {
                find_inline_char(text, current, target, is_till, forward, count)
            } else {
                current
            }
        }

        VimMotion::RepeatFindRev => {
            if let Some((target, is_till, forward)) = last_find {
                find_inline_char(text, current, target, is_till, !forward, count)
            } else {
                current
            }
        }

        VimMotion::BufferStart => 0,

        VimMotion::BufferEnd => clamp_cursor(text, text.len().saturating_sub(1)),

        VimMotion::MatchingPair => find_matching_pair(text, current),
    }
}

/// Clamps cursor to a valid UTF-8 character boundary within text bounds.
pub fn clamp_cursor(text: &str, cursor: usize) -> usize {
    if text.is_empty() {
        return 0;
    }
    let mut c = cursor.min(text.len().saturating_sub(1));
    while !text.is_char_boundary(c) && c > 0 {
        c -= 1;
    }
    c
}

fn next_word_start(text: &str, cursor: usize) -> usize {
    let chars: Vec<(usize, char)> = text[cursor..].char_indices().collect();
    if chars.is_empty() {
        return cursor;
    }

    let initial_class = classify(chars[0].1);
    let mut i = 0;

    // 1. Skip current word/punct group if we didn't start on space
    if initial_class != CharClass::Space {
        while i < chars.len() && classify(chars[i].1) == initial_class {
            i += 1;
        }
    }

    // 2. Skip any intermediate whitespace
    while i < chars.len() && classify(chars[i].1) == CharClass::Space {
        i += 1;
    }

    if i < chars.len() {
        cursor + chars[i].0
    } else {
        clamp_cursor(text, text.len().saturating_sub(1))
    }
}

fn next_word_end(text: &str, cursor: usize) -> usize {
    let chars: Vec<(usize, char)> = text[cursor..].char_indices().collect();
    if chars.len() <= 1 {
        return clamp_cursor(text, text.len().saturating_sub(1));
    }

    let mut i = 1;

    // Skip whitespace first
    while i < chars.len() && classify(chars[i].1) == CharClass::Space {
        i += 1;
    }

    if i >= chars.len() {
        return clamp_cursor(text, text.len().saturating_sub(1));
    }

    let target_class = classify(chars[i].1);
    while i + 1 < chars.len() && classify(chars[i + 1].1) == target_class {
        i += 1;
    }

    cursor + chars[i].0
}

fn prev_word_start(text: &str, cursor: usize) -> usize {
    let chars: Vec<(usize, char)> = text[..cursor].char_indices().collect();
    if chars.is_empty() {
        return 0;
    }

    let mut i = chars.len().saturating_sub(1);

    // Skip whitespace
    while i > 0 && classify(chars[i].1) == CharClass::Space {
        i -= 1;
    }

    let target_class = classify(chars[i].1);
    while i > 0 && classify(chars[i - 1].1) == target_class {
        i -= 1;
    }

    chars[i].0
}

fn prev_word_end(text: &str, cursor: usize) -> usize {
    let chars: Vec<(usize, char)> = text[..cursor].char_indices().collect();
    if chars.len() <= 1 {
        return 0;
    }

    let mut i = chars.len().saturating_sub(1);
    let initial_class = classify(chars[i].1);

    // Skip current word class
    if initial_class != CharClass::Space {
        while i > 0 && classify(chars[i].1) == initial_class {
            i -= 1;
        }
    }

    // Skip whitespace
    while i > 0 && classify(chars[i].1) == CharClass::Space {
        i -= 1;
    }

    chars[i].0
}

fn next_big_word_start(text: &str, cursor: usize) -> usize {
    let chars: Vec<(usize, char)> = text[cursor..].char_indices().collect();
    if chars.is_empty() {
        return cursor;
    }

    let mut i = 0;
    // Skip non-space
    while i < chars.len() && !chars[i].1.is_whitespace() {
        i += 1;
    }
    // Skip spaces
    while i < chars.len() && chars[i].1.is_whitespace() {
        i += 1;
    }

    if i < chars.len() {
        cursor + chars[i].0
    } else {
        clamp_cursor(text, text.len().saturating_sub(1))
    }
}

fn next_big_word_end(text: &str, cursor: usize) -> usize {
    let chars: Vec<(usize, char)> = text[cursor..].char_indices().collect();
    if chars.len() <= 1 {
        return clamp_cursor(text, text.len().saturating_sub(1));
    }

    let mut i = 1;
    while i < chars.len() && chars[i].1.is_whitespace() {
        i += 1;
    }
    while i + 1 < chars.len() && !chars[i + 1].1.is_whitespace() {
        i += 1;
    }

    if i < chars.len() {
        cursor + chars[i].0
    } else {
        clamp_cursor(text, text.len().saturating_sub(1))
    }
}

fn prev_big_word_start(text: &str, cursor: usize) -> usize {
    let chars: Vec<(usize, char)> = text[..cursor].char_indices().collect();
    if chars.is_empty() {
        return 0;
    }

    let mut i = chars.len().saturating_sub(1);
    while i > 0 && chars[i].1.is_whitespace() {
        i -= 1;
    }
    while i > 0 && !chars[i - 1].1.is_whitespace() {
        i -= 1;
    }

    chars[i].0
}

fn find_inline_char(
    text: &str,
    cursor: usize,
    target: char,
    is_till: bool,
    forward: bool,
    count: usize,
) -> usize {
    if forward {
        let after = &text[cursor..];
        let mut matches = 0;
        let mut chars = after.char_indices();
        chars.next(); // Skip current char

        for (idx, c) in chars {
            if c == target {
                matches += 1;
                if matches == count {
                    let target_pos = cursor + idx;
                    return if is_till {
                        clamp_cursor(text, target_pos.saturating_sub(1))
                    } else {
                        target_pos
                    };
                }
            } else if c == '\n' {
                break;
            }
        }
    } else {
        let before = &text[..cursor];
        let mut matches = 0;
        let chars: Vec<(usize, char)> = before.char_indices().collect();

        for (idx, c) in chars.into_iter().rev() {
            if c == target {
                matches += 1;
                if matches == count {
                    return if is_till {
                        let next_idx = text[idx..].char_indices().nth(1).map_or(cursor, |(n, _)| idx + n);
                        next_idx
                    } else {
                        idx
                    };
                }
            } else if c == '\n' {
                break;
            }
        }
    }

    cursor
}

fn find_matching_pair(text: &str, cursor: usize) -> usize {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let current_char = text[cursor..].chars().next().unwrap_or('\0');

    let (target_char, forward) = match current_char {
        '(' => (')', true),
        '[' => (']', true),
        '{' => ('}', true),
        '<' => ('>', true),
        ')' => ('(', false),
        ']' => ('[', false),
        '}' => ('{', false),
        '>' => ('<', false),
        _ => return cursor,
    };

    let mut depth = 0;
    if forward {
        for &(idx, c) in chars.iter().filter(|&&(idx, _)| idx >= cursor) {
            if c == current_char {
                depth += 1;
            } else if c == target_char {
                depth -= 1;
                if depth == 0 {
                    return idx;
                }
            }
        }
    } else {
        for &(idx, c) in chars.iter().filter(|&&(idx, _)| idx <= cursor).rev() {
            if c == current_char {
                depth += 1;
            } else if c == target_char {
                depth -= 1;
                if depth == 0 {
                    return idx;
                }
            }
        }
    }

    cursor
}
