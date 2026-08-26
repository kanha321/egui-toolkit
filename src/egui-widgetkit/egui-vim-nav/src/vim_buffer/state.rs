//! Central `VimBufferState` struct coordinating text buffer, cursor, history, and key dispatch.

use std::ops::Range;
use egui::{Context, Event, Key, Modifiers};

use crate::vim_buffer::history::UndoHistory;
use crate::vim_buffer::mode::{VimMode, VisualType};
use crate::vim_buffer::motion::{calculate_motion, clamp_cursor, VimMotion};
use crate::vim_buffer::operator::VimOperator;
use crate::vim_buffer::parser::{ParsedCommand, VimParser};
use crate::vim_buffer::register::VimRegisters;

/// Complete modal Vim state for a text buffer.
///
/// Encapsulates the text string, cursor byte index, visual selection anchor,
/// active modal state, sequence parser, register storage, and undo/redo stacks.
#[derive(Clone, Debug)]
pub struct VimBufferState {
    /// The live text content.
    pub text: String,
    /// Current cursor byte index (guaranteed on UTF-8 char boundary).
    pub cursor: usize,
    /// Selection anchor byte index when in Visual mode.
    pub anchor: Option<usize>,
    /// Active modal state.
    pub mode: VimMode,
    /// Grammar parser for multi-key sequences and count accumulators.
    pub parser: VimParser,
    /// Storage for unnamed, yank, and named registers.
    pub registers: VimRegisters,
    /// Multi-level undo and redo history.
    pub history: UndoHistory,
    /// Last character typed in insert mode (for `jk` chord detection).
    pub last_insert_char: Option<char>,
}

impl Default for VimBufferState {
    fn default() -> Self {
        Self::new("")
    }
}

impl VimBufferState {
    /// Creates a new Vim buffer state with the provided initial text.
    pub fn new(initial_text: impl Into<String>) -> Self {
        let text = initial_text.into();
        Self {
            text,
            cursor: 0,
            anchor: None,
            mode: VimMode::Normal,
            parser: VimParser::new(),
            registers: VimRegisters::new(),
            history: UndoHistory::new(),
            last_insert_char: None,
        }
    }

    /// Returns a reference to the buffer's string slice.
    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Replaces the buffer content, clamping the cursor appropriately.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = clamp_cursor(&self.text, self.cursor);
        self.anchor = None;
    }

    /// Current cursor byte index.
    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Active Vim mode, dynamically reflecting real-time operator-pending and replace-pending states.
    #[inline]
    pub fn mode(&self) -> VimMode {
        if self.mode.is_insert() || self.mode.is_visual() {
            return self.mode;
        }
        if self.parser.is_pending_replace() {
            return VimMode::Replace;
        }
        if let Some((operator, count)) = self.parser.pending_op() {
            return VimMode::OperatorPending { operator, count };
        }
        self.mode
    }

    /// Returns the active visual selection byte range `[start..end]`, if any.
    pub fn selection_range(&self) -> Option<Range<usize>> {
        match self.mode {
            VimMode::Visual(VisualType::Character) | VimMode::Visual(VisualType::Block) => {
                if let Some(anchor) = self.anchor {
                    let start = anchor.min(self.cursor);
                    let end = anchor.max(self.cursor);
                    // Include character under the cursor
                    let end_char_len = self.text[end..].chars().next().map_or(0, |c| c.len_utf8());
                    Some(start..(end + end_char_len))
                } else {
                    None
                }
            }
            VimMode::Visual(VisualType::Line) => {
                if let Some(anchor) = self.anchor {
                    let start_idx = anchor.min(self.cursor);
                    let end_idx = anchor.max(self.cursor);
                    // Beginning of the line containing start_idx
                    let line_start = self.text[..start_idx.min(self.text.len())].rfind('\n').map_or(0, |i| i + 1);
                    // End of the line containing end_idx (including trailing newline if present)
                    let line_end = text_line_end(&self.text, end_idx);
                    let end_with_nl = if line_end < self.text.len() { line_end + 1 } else { line_end };
                    Some(line_start..end_with_nl)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Processes incoming egui events during a frame.
    ///
    /// Returns `true` if text was modified or a key event was consumed.
    pub fn handle_input(&mut self, ctx: &Context) -> bool {
        let mut handled = false;
        // Per-event flag: when a Key event causes a mode transition into Insert
        // (or is handled in Normal/Visual mode), suppress the immediately
        // following Event::Text to prevent the trigger key from being inserted
        // as a literal character.
        let mut suppress_next_text = false;

        let events = ctx.input(|i| i.events.clone());

        for event in events {
            match event {
                Event::Key { key, pressed: true, modifiers, .. } => {
                    let was_insert = self.mode.is_insert();
                    let key_handled = self.handle_key(key, &modifiers);
                    if key_handled {
                        handled = true;
                        ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key));
                    }
                    // If this key just entered Insert mode, suppress the
                    // immediately-following Event::Text for the same key
                    if !was_insert && self.mode.is_insert() {
                        suppress_next_text = true;
                    }
                    // If we're in Normal/Visual mode and the key was handled,
                    // suppress the text event too (e.g. 'd', 'y', 'v', etc.)
                    if !self.mode.is_insert() && key_handled {
                        suppress_next_text = true;
                    }
                }
                Event::Text(text) => {
                    if suppress_next_text {
                        suppress_next_text = false;
                        continue;
                    }
                    if self.mode.is_insert() {
                        for c in text.chars() {
                            if !c.is_control() {
                                self.insert_char(c);
                                handled = true;
                            }
                        }
                    }
                    // In Normal/Visual mode, text events are always ignored
                }
                _ => {}
            }
        }

        handled
    }

    /// Handles a single key press event.
    pub fn handle_key(&mut self, key: Key, modifiers: &Modifiers) -> bool {
        // 1. Universal Escape handling: return to Normal mode
        if key == Key::Escape {
            if self.mode.is_insert() || self.mode.is_visual() || self.mode == VimMode::Replace {
                self.mode = VimMode::Normal;
                self.anchor = None;
                self.parser.reset();
                self.cursor = clamp_cursor(&self.text, self.cursor.saturating_sub(1));
                return true;
            } else if self.mode.is_operator_pending() || self.parser.pending_keys_label().len() > 0 {
                self.parser.reset();
                self.mode = VimMode::Normal;
                return true;
            }
            return false;
        }

        // 2. Insert Mode Key Handling
        if self.mode.is_insert() {
            if modifiers.ctrl {
                if key == Key::H || key == Key::Backspace {
                    self.delete_word_backward();
                    return true;
                } else if key == Key::W {
                    self.delete_word_backward();
                    return true;
                } else if key == Key::U {
                    self.delete_to_line_start();
                    return true;
                }
                return false;
            }

            match key {
                Key::Backspace => {
                    self.backspace();
                    true
                }
                Key::Delete => {
                    self.delete_forward();
                    true
                }
                Key::Enter => {
                    // In single-line modal editing, Enter is a commit action, not a newline
                    false
                }
                Key::ArrowLeft => {
                    self.cursor = calculate_motion(&self.text, self.cursor, VimMotion::Left, 1, None);
                    true
                }
                Key::ArrowRight => {
                    if self.cursor < self.text.len() {
                        self.cursor = calculate_motion(&self.text, self.cursor, VimMotion::Right, 1, None);
                    }
                    true
                }
                Key::ArrowUp => {
                    self.cursor = calculate_motion(&self.text, self.cursor, VimMotion::Up, 1, None);
                    true
                }
                Key::ArrowDown => {
                    self.cursor = calculate_motion(&self.text, self.cursor, VimMotion::Down, 1, None);
                    true
                }
                _ => false,
            }
        } else {
            // 3. Normal / Visual Mode Handling
            if modifiers.ctrl {
                if key == Key::R {
                    self.redo();
                    return true;
                } else if key == Key::V {
                    self.anchor = Some(self.cursor);
                    self.mode = VimMode::Visual(VisualType::Block);
                    return true;
                }
                return false;
            }

            // Convert key to character command
            let c = match key {
                Key::A => if modifiers.shift { 'A' } else { 'a' },
                Key::B => if modifiers.shift { 'B' } else { 'b' },
                Key::C => if modifiers.shift { 'C' } else { 'c' },
                Key::D => if modifiers.shift { 'D' } else { 'd' },
                Key::E => if modifiers.shift { 'E' } else { 'e' },
                Key::F => if modifiers.shift { 'F' } else { 'f' },
                Key::G => if modifiers.shift { 'G' } else { 'g' },
                Key::H => 'h',
                Key::I => if modifiers.shift { 'I' } else { 'i' },
                Key::J => 'j',
                Key::K => 'k',
                Key::L => 'l',
                Key::M => 'm',
                Key::N => if modifiers.shift { 'N' } else { 'n' },
                Key::O => if modifiers.shift { 'O' } else { 'o' },
                Key::P => if modifiers.shift { 'P' } else { 'p' },
                Key::Q => 'q',
                Key::R => if modifiers.shift { 'R' } else { 'r' },
                Key::S => if modifiers.shift { 'S' } else { 's' },
                Key::T => if modifiers.shift { 'T' } else { 't' },
                Key::U => if modifiers.shift { 'U' } else { 'u' },
                Key::V => if modifiers.shift { 'V' } else { 'v' },
                Key::W => if modifiers.shift { 'W' } else { 'w' },
                Key::X => if modifiers.shift { 'X' } else { 'x' },
                Key::Y => if modifiers.shift { 'Y' } else { 'y' },
                Key::Z => 'z',
                Key::Num0 => '0',
                Key::Num1 => '1',
                Key::Num2 => '2',
                Key::Num3 => '3',
                Key::Num4 => '4',
                Key::Num5 => '5',
                Key::Num6 => if modifiers.shift { '^' } else { '6' },
                Key::Num7 => '7',
                Key::Num8 => '8',
                Key::Num9 => '9',
                Key::Minus => '-',
                Key::Plus => '+',
                Key::Equals => '=',
                Key::Slash => '/',
                Key::Backslash => '\\',
                _ => return false,
            };

            let cmd = self.parser.parse_char(c, self.mode);
            self.execute_command(cmd)
        }
    }

    /// Directly passes a character into the Vim state parser.
    pub fn handle_char(&mut self, c: char) -> bool {
        if self.mode.is_insert() {
            self.insert_char(c);
            true
        } else {
            let cmd = self.parser.parse_char(c, self.mode);
            self.execute_command(cmd)
        }
    }

    fn execute_command(&mut self, cmd: ParsedCommand) -> bool {
        match cmd {
            ParsedCommand::Incomplete => {
                // Update operator-pending mode for status display
                true
            }
            ParsedCommand::Unhandled => false,

            ParsedCommand::EnterMode(mode) => {
                if mode.is_visual() {
                    self.anchor = Some(self.cursor);
                } else {
                    self.anchor = None;
                }
                self.mode = mode;
                true
            }

            ParsedCommand::AppendAfterCursor => {
                if !self.text.is_empty() {
                    let next = self.text[self.cursor..].char_indices().nth(1).map_or(self.text.len(), |(n, _)| self.cursor + n);
                    self.cursor = next;
                }
                self.mode = VimMode::Insert;
                self.anchor = None;
                true
            }

            ParsedCommand::InsertLineStart => {
                self.cursor = calculate_motion(&self.text, self.cursor, VimMotion::FirstNonBlank, 1, None);
                self.mode = VimMode::Insert;
                self.anchor = None;
                true
            }

            ParsedCommand::AppendAtLineEnd => {
                let line_end = text_line_end(&self.text, self.cursor);
                self.cursor = line_end;
                self.mode = VimMode::Insert;
                self.anchor = None;
                true
            }

            ParsedCommand::OpenLineBelow => {
                self.history.push(&self.text, self.cursor);
                let line_end = text_line_end(&self.text, self.cursor);
                self.text.insert(line_end, '\n');
                self.cursor = line_end + 1;
                self.mode = VimMode::Insert;
                self.anchor = None;
                true
            }

            ParsedCommand::OpenLineAbove => {
                self.history.push(&self.text, self.cursor);
                let line_start = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1);
                self.text.insert(line_start, '\n');
                self.cursor = line_start;
                self.mode = VimMode::Insert;
                self.anchor = None;
                true
            }

            ParsedCommand::Motion { motion, count } => {
                let target = calculate_motion(
                    &self.text,
                    self.cursor,
                    motion,
                    count,
                    self.registers.last_search,
                );
                self.cursor = target;
                match motion {
                    VimMotion::FindChar(c) => self.registers.last_search = Some((c, false, true)),
                    VimMotion::TillChar(c) => self.registers.last_search = Some((c, true, true)),
                    VimMotion::FindCharBack(c) => self.registers.last_search = Some((c, false, false)),
                    VimMotion::TillCharBack(c) => self.registers.last_search = Some((c, true, false)),
                    _ => {}
                }
                true
            }

            ParsedCommand::OperatorMotion { op, motion, count } => {
                let target = calculate_motion(
                    &self.text,
                    self.cursor,
                    motion,
                    count,
                    self.registers.last_search,
                );
                let start = self.cursor.min(target);
                let end = self.cursor.max(target);

                // Word motions like `dw` or `cw` are exclusive of destination character;
                // find inclusive motions like `df_` or line end `d$` include target character
                let end_inclusive = match motion {
                    VimMotion::FindChar(_) | VimMotion::TillChar(_) | VimMotion::LineEnd | VimMotion::MatchingPair => {
                        let c_len = self.text[end..].chars().next().map_or(0, |c| c.len_utf8());
                        end + c_len
                    }
                    _ => end,
                };

                self.apply_operator_range(op, start..end_inclusive);
                true
            }

            ParsedCommand::OperatorTextObject { op, object, count } => {
                let _ = count;
                if let Some(range) = object.resolve_range(&self.text, self.cursor) {
                    if self.mode.is_visual() {
                        self.anchor = Some(range.start);
                        self.cursor = clamp_cursor(&self.text, range.end.saturating_sub(1));
                    } else {
                        self.apply_operator_range(op, range);
                    }
                }
                true
            }

            ParsedCommand::LineOperator { op, count } => {
                let _ = count;
                let line_start = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1);
                let line_end = text_line_end(&self.text, self.cursor);
                let end_with_nl = if line_end < self.text.len() { line_end + 1 } else { line_end };

                self.apply_operator_range(op, line_start..end_with_nl);
                true
            }

            ParsedCommand::DeleteChar { forward, count } => {
                self.history.push(&self.text, self.cursor);
                for _ in 0..count {
                    if self.text.is_empty() {
                        break;
                    }
                    if forward {
                        if self.cursor < self.text.len() {
                            let c = self.text.remove(self.cursor);
                            self.registers.unnamed = c.to_string();
                        }
                    } else if self.cursor > 0 {
                        let prev_idx = self.text[..self.cursor].char_indices().next_back().map_or(0, |(n, _)| n);
                        let c = self.text.remove(prev_idx);
                        self.registers.unnamed = c.to_string();
                        self.cursor = prev_idx;
                    }
                }
                self.cursor = clamp_cursor(&self.text, self.cursor);
                true
            }

            ParsedCommand::ReplaceChar { target, count } => {
                self.history.push(&self.text, self.cursor);
                if self.mode.is_visual() {
                    // Replace every character in the visual selection with `target`
                    if let Some(range) = self.selection_range() {
                        let replaced: String = self.text[range.clone()]
                            .chars()
                            .map(|c| if c == '\n' { '\n' } else { target })
                            .collect();
                        self.text.replace_range(range.clone(), &replaced);
                        self.cursor = clamp_cursor(&self.text, range.start);
                        self.mode = VimMode::Normal;
                        self.anchor = None;
                    }
                } else {
                    for _ in 0..count {
                        if self.cursor < self.text.len() {
                            let cur_char_len = self.text[self.cursor..].chars().next().map_or(1, |c| c.len_utf8());
                            self.text.replace_range(self.cursor..(self.cursor + cur_char_len), &target.to_string());
                            self.cursor += target.len_utf8();
                        }
                    }
                    self.cursor = clamp_cursor(&self.text, self.cursor.saturating_sub(target.len_utf8()));
                }
                true
            }

            ParsedCommand::Substitute { count } => {
                self.history.push(&self.text, self.cursor);
                let mut end = self.cursor;
                for _ in 0..count {
                    if let Some((_, c)) = self.text[end..].char_indices().next() {
                        end += c.len_utf8();
                    }
                }
                let deleted = self.text.drain(self.cursor..end).collect::<String>();
                self.registers.unnamed = deleted;
                self.mode = VimMode::Insert;
                self.anchor = None;
                true
            }

            ParsedCommand::SubstituteLine => {
                self.history.push(&self.text, self.cursor);
                let line_start = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1);
                let line_end = text_line_end(&self.text, self.cursor);
                let deleted = self.text.drain(line_start..line_end).collect::<String>();
                self.registers.unnamed = deleted;
                self.cursor = line_start;
                self.mode = VimMode::Insert;
                self.anchor = None;
                true
            }

            ParsedCommand::Paste { after, count } => {
                let text_to_paste = self.registers.unnamed.clone();
                if text_to_paste.is_empty() {
                    return false;
                }

                self.history.push(&self.text, self.cursor);
                for _ in 0..count {
                    let insert_pos = if after {
                        if self.cursor < self.text.len() {
                            let next_idx = self.text[self.cursor..].char_indices().nth(1).map_or(self.text.len(), |(n, _)| self.cursor + n);
                            next_idx
                        } else {
                            self.text.len()
                        }
                    } else {
                        self.cursor
                    };

                    self.text.insert_str(insert_pos, &text_to_paste);
                    self.cursor = clamp_cursor(&self.text, insert_pos + text_to_paste.len().saturating_sub(1));
                }
                true
            }

            ParsedCommand::ToggleCase { count } => {
                self.history.push(&self.text, self.cursor);
                for _ in 0..count {
                    if self.cursor < self.text.len() {
                        let c = self.text[self.cursor..].chars().next().unwrap();
                        let toggled = if c.is_uppercase() {
                            c.to_lowercase().collect::<String>()
                        } else {
                            c.to_uppercase().collect::<String>()
                        };
                        let orig_len = c.len_utf8();
                        self.text.replace_range(self.cursor..(self.cursor + orig_len), &toggled);
                        self.cursor += toggled.len();
                    }
                }
                self.cursor = clamp_cursor(&self.text, self.cursor);
                true
            }

            ParsedCommand::Undo => {
                self.undo();
                true
            }

            ParsedCommand::Redo => {
                self.redo();
                true
            }

            ParsedCommand::VisualOperator(op) => {
                if let Some(range) = self.selection_range() {
                    self.apply_operator_range(op, range);
                }
                true
            }

            ParsedCommand::VisualPaste => {
                if let Some(range) = self.selection_range() {
                    let paste_text = self.registers.unnamed.clone();
                    self.history.push(&self.text, self.cursor);
                    // Save the deleted selection into the register before replacing
                    let deleted = self.text.drain(range.clone()).collect::<String>();
                    self.text.insert_str(range.start, &paste_text);
                    self.registers.unnamed = deleted;
                    self.cursor = clamp_cursor(&self.text, range.start + paste_text.len().saturating_sub(1));
                    self.mode = VimMode::Normal;
                    self.anchor = None;
                }
                true
            }

            ParsedCommand::VisualSwapAnchor => {
                if let Some(anchor) = self.anchor {
                    self.anchor = Some(self.cursor);
                    self.cursor = anchor;
                }
                true
            }

            ParsedCommand::VisualJoinLines => {
                if let Some(range) = self.selection_range() {
                    self.history.push(&self.text, self.cursor);
                    // Replace all newlines in range with a single space
                    let joined: String = self.text[range.clone()]
                        .lines()
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.text.replace_range(range.clone(), &joined);
                    self.cursor = clamp_cursor(&self.text, range.start);
                    self.mode = VimMode::Normal;
                    self.anchor = None;
                }
                true
            }

            ParsedCommand::VisualIndent => {
                if let Some(range) = self.selection_range() {
                    self.history.push(&self.text, self.cursor);
                    let start = range.start;
                    let end = range.end;
                    // Expand to full lines: start of first line, end of last line
                    let prefix_start = self.text[..start].rfind('\n').map_or(0, |i| i + 1);
                    let suffix_end = self.text[end..].find('\n').map_or(self.text.len(), |i| end + i);
                    let slice = &self.text[prefix_start..suffix_end];
                    let indented: String = slice
                        .lines()
                        .map(|line| format!("    {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.text.replace_range(prefix_start..suffix_end, &indented);
                    self.cursor = clamp_cursor(&self.text, prefix_start);
                    self.mode = VimMode::Normal;
                    self.anchor = None;
                }
                true
            }

            ParsedCommand::VisualOutdent => {
                if let Some(range) = self.selection_range() {
                    self.history.push(&self.text, self.cursor);
                    let start = range.start;
                    let end = range.end;
                    // Expand to full lines: start of first line, end of last line
                    let prefix_start = self.text[..start].rfind('\n').map_or(0, |i| i + 1);
                    let suffix_end = self.text[end..].find('\n').map_or(self.text.len(), |i| end + i);
                    let slice = &self.text[prefix_start..suffix_end];
                    let outdented: String = slice
                        .lines()
                        .map(|line| {
                            let spaces = line.len() - line.trim_start_matches(' ').len();
                            let remove = spaces.min(4);
                            &line[remove..]
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.text.replace_range(prefix_start..suffix_end, &outdented);
                    self.cursor = clamp_cursor(&self.text, prefix_start);
                    self.mode = VimMode::Normal;
                    self.anchor = None;
                }
                true
            }
        }
    }

    /// Applies an operator over a concrete byte span `[start..end]`.
    pub fn apply_operator_range(&mut self, op: VimOperator, range: Range<usize>) {
        let start = range.start.min(self.text.len());
        let end = range.end.min(self.text.len());

        if start >= end {
            return;
        }

        match op {
            VimOperator::Delete => {
                self.history.push(&self.text, self.cursor);
                let deleted = self.text.drain(start..end).collect::<String>();
                self.registers.unnamed = deleted;
                self.cursor = clamp_cursor(&self.text, start);
                self.mode = VimMode::Normal;
                self.anchor = None;
            }

            VimOperator::Change => {
                self.history.push(&self.text, self.cursor);
                let deleted = self.text.drain(start..end).collect::<String>();
                self.registers.unnamed = deleted;
                self.cursor = start;
                self.mode = VimMode::Insert;
                self.anchor = None;
            }

            VimOperator::Yank => {
                let yanked = self.text[start..end].to_owned();
                self.registers.store_yank(None, yanked);
                self.mode = VimMode::Normal;
                self.anchor = None;
            }

            VimOperator::ToLower => {
                self.history.push(&self.text, self.cursor);
                let lowered = self.text[start..end].to_lowercase();
                self.text.replace_range(start..end, &lowered);
                self.cursor = clamp_cursor(&self.text, start);
                self.mode = VimMode::Normal;
                self.anchor = None;
            }

            VimOperator::ToUpper => {
                self.history.push(&self.text, self.cursor);
                let uppered = self.text[start..end].to_uppercase();
                self.text.replace_range(start..end, &uppered);
                self.cursor = clamp_cursor(&self.text, start);
                self.mode = VimMode::Normal;
                self.anchor = None;
            }

            VimOperator::ToggleCase => {
                self.history.push(&self.text, self.cursor);
                let toggled: String = self.text[start..end]
                    .chars()
                    .map(|c| {
                        if c.is_uppercase() {
                            c.to_lowercase().collect::<String>()
                        } else {
                            c.to_uppercase().collect::<String>()
                        }
                    })
                    .collect();
                self.text.replace_range(start..end, &toggled);
                self.cursor = clamp_cursor(&self.text, start);
                self.mode = VimMode::Normal;
                self.anchor = None;
            }

            _ => {}
        }
    }

    /// Inserts a character at the current cursor in Insert mode.
    ///
    /// Fast exit chord: typing `jk` immediately exits Insert mode, deletes the preceding `'j'`,
    /// and returns to Normal mode.
    pub fn insert_char(&mut self, c: char) {
        if c == 'k' && self.last_insert_char == Some('j') && self.cursor > 0 {
            let prev_idx = self.text[..self.cursor].char_indices().next_back().map_or(0, |(n, _)| n);
            if self.text[prev_idx..].starts_with('j') {
                self.text.remove(prev_idx);
                self.cursor = prev_idx;
                self.mode = VimMode::Normal;
                self.anchor = None;
                self.parser.reset();
                self.last_insert_char = None;
                self.cursor = clamp_cursor(&self.text, self.cursor.saturating_sub(1));
                return;
            }
        }

        self.history.push(&self.text, self.cursor);
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.last_insert_char = Some(c);
    }

    /// Inserts a string at the current cursor in Insert mode.
    pub fn insert_str(&mut self, s: &str) {
        self.history.push(&self.text, self.cursor);
        self.text.insert_str(self.cursor, s);
        self.cursor += s.len();
        self.last_insert_char = s.chars().last();
    }

    /// Deletes character before cursor (Backspace).
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.history.push(&self.text, self.cursor);
            let prev_idx = self.text[..self.cursor].char_indices().next_back().map_or(0, |(n, _)| n);
            self.text.remove(prev_idx);
            self.cursor = prev_idx;
            self.last_insert_char = None;
        }
    }

    /// Deletes character under cursor (Delete key).
    pub fn delete_forward(&mut self) {
        if self.cursor < self.text.len() {
            self.history.push(&self.text, self.cursor);
            self.text.remove(self.cursor);
        }
    }

    /// Deletes previous word (`Ctrl+W` / `Ctrl+Backspace`).
    pub fn delete_word_backward(&mut self) {
        let prev_start = calculate_motion(&self.text, self.cursor, VimMotion::WordBackward, 1, None);
        if prev_start < self.cursor {
            self.history.push(&self.text, self.cursor);
            self.text.drain(prev_start..self.cursor);
            self.cursor = prev_start;
        }
    }

    /// Deletes to beginning of line (`Ctrl+U`).
    pub fn delete_to_line_start(&mut self) {
        let line_start = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1);
        if line_start < self.cursor {
            self.history.push(&self.text, self.cursor);
            self.text.drain(line_start..self.cursor);
            self.cursor = line_start;
        }
    }

    /// Undoes the last edit and restores cursor position.
    pub fn undo(&mut self) {
        if let Some(prev) = self.history.undo(&self.text, self.cursor) {
            self.text = prev.text;
            self.cursor = clamp_cursor(&self.text, prev.cursor);
        }
    }

    /// Redoes the undone edit and restores cursor position.
    pub fn redo(&mut self) {
        if let Some(next) = self.history.redo(&self.text, self.cursor) {
            self.text = next.text;
            self.cursor = clamp_cursor(&self.text, next.cursor);
        }
    }
}

fn text_line_end(text: &str, cursor: usize) -> usize {
    if let Some(nl) = text[cursor..].find('\n') {
        cursor + nl
    } else {
        text.len()
    }
}
