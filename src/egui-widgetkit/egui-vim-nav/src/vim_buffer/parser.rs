//! Keystroke sequence, count accumulator, and chord parser.

use crate::vim_buffer::mode::{VimMode, VisualType};
use crate::vim_buffer::motion::VimMotion;
use crate::vim_buffer::operator::VimOperator;
use crate::vim_buffer::text_object::{TextObject, TextObjectScope, TextObjectTarget};

/// Fully parsed Vim action to execute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedCommand {
    /// Incomplete sequence waiting for more keys (e.g. `d` or `3`).
    Incomplete,
    /// Unrecognized key combination.
    Unhandled,
    /// Enter a new mode (e.g. `i` for Insert, `v` for Visual, `Esc` for Normal).
    EnterMode(VimMode),
    /// Execute a pure motion.
    Motion {
        motion: VimMotion,
        count: usize,
    },
    /// Execute an operator with a motion target (e.g. `dw`, `3cw`, `d$`).
    OperatorMotion {
        op: VimOperator,
        motion: VimMotion,
        count: usize,
    },
    /// Execute an operator with a text object target (e.g. `diw`, `ca"`).
    OperatorTextObject {
        op: VimOperator,
        object: TextObject,
        count: usize,
    },
    /// Execute a whole-line operator (e.g. `dd`, `cc`, `yy`).
    LineOperator {
        op: VimOperator,
        count: usize,
    },
    /// Single character deletion (`x` or `X`).
    DeleteChar {
        forward: bool,
        count: usize,
    },
    /// Replace single character (`r<char>`).
    ReplaceChar {
        target: char,
        count: usize,
    },
    /// Substitute character with insert (`s`).
    Substitute {
        count: usize,
    },
    /// Substitute line (`S`).
    SubstituteLine,
    /// Paste register text (`p` or `P`).
    Paste {
        after: bool,
        count: usize,
    },
    /// Toggle character case (`~`).
    ToggleCase {
        count: usize,
    },
    /// Undo last edit (`u`).
    Undo,
    /// Redo undone edit (`Ctrl+r`).
    Redo,
    /// Insert at line start (`I`).
    InsertLineStart,
    /// Append at line end (`A`).
    AppendAtLineEnd,
    /// Append after cursor (`a`).
    AppendAfterCursor,
    /// Open new line below (`o`).
    OpenLineBelow,
    /// Open new line above (`O`).
    OpenLineAbove,
    /// Apply an operator over the active visual selection range.
    VisualOperator(VimOperator),
    /// Paste register content, replacing the visual selection.
    VisualPaste,
    /// Swap cursor and anchor endpoints in Visual mode (`o`).
    VisualSwapAnchor,
    /// Join all lines in the visual selection into one (`J`).
    VisualJoinLines,
    /// Indent the visual selection (`>`).
    VisualIndent,
    /// Outdent the visual selection (`<`).
    VisualOutdent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingCharMode {
    Find(bool, bool), // (is_till, is_forward)
    Replace,
    Register,
}

/// Grammar parser holding multi-key sequence buffers.
#[derive(Clone, Debug, Default)]
pub struct VimParser {
    count_acc: Option<usize>,
    pending_op: Option<(VimOperator, usize)>,
    pending_g: bool,
    pending_text_scope: Option<TextObjectScope>,
    pending_char: Option<PendingCharMode>,
    selected_register: Option<char>,
}

impl VimParser {
    /// Creates a fresh parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets any pending partial sequence (e.g. when Escape is pressed).
    pub fn reset(&mut self) {
        self.count_acc = None;
        self.pending_op = None;
        self.pending_g = false;
        self.pending_text_scope = None;
        self.pending_char = None;
        self.selected_register = None;
    }

    /// Returns a string representation of any pending keys for UI display (e.g. `"3d..."`).
    pub fn pending_keys_label(&self) -> String {
        let mut s = String::new();
        if let Some(c) = self.count_acc {
            s.push_str(&c.to_string());
        }
        if let Some((op, _)) = self.pending_op {
            match op {
                VimOperator::Delete => s.push('d'),
                VimOperator::Change => s.push('c'),
                VimOperator::Yank => s.push('y'),
                VimOperator::ToLower => s.push_str("gu"),
                VimOperator::ToUpper => s.push_str("gU"),
                _ => {}
            }
        }
        if self.pending_g {
            s.push('g');
        }
        if let Some(scope) = self.pending_text_scope {
            match scope {
                TextObjectScope::Inner => s.push('i'),
                TextObjectScope::A => s.push('a'),
            }
        }
        if let Some(pc) = self.pending_char {
            match pc {
                PendingCharMode::Find(false, true) => s.push('f'),
                PendingCharMode::Find(true, true) => s.push('t'),
                PendingCharMode::Find(false, false) => s.push('F'),
                PendingCharMode::Find(true, false) => s.push('T'),
                PendingCharMode::Replace => s.push('r'),
                PendingCharMode::Register => s.push('"'),
            }
        }
        s
    }

    /// Returns the active pending operator and count prefix, if any (e.g. `(Delete, 1)` after typing `d`).
    #[inline]
    pub fn pending_op(&self) -> Option<(VimOperator, usize)> {
        self.pending_op
    }

    /// Returns `true` if waiting for a replacement character after typing `r`.
    #[inline]
    pub fn is_pending_replace(&self) -> bool {
        matches!(self.pending_char, Some(PendingCharMode::Replace))
    }

    /// Returns `true` if waiting for a target character after typing `f`, `t`, `F`, or `T`.
    #[inline]
    pub fn is_pending_find(&self) -> bool {
        matches!(self.pending_char, Some(PendingCharMode::Find(..)))
    }

    /// Returns `true` if waiting for a register character after typing `"`.
    #[inline]
    pub fn is_pending_register(&self) -> bool {
        matches!(self.pending_char, Some(PendingCharMode::Register))
    }

    /// Parses a single character or key event in Normal or Visual mode.
    pub fn parse_char(&mut self, c: char, current_mode: VimMode) -> ParsedCommand {
        // 1. Handle pending character arguments (f<char>, t<char>, r<char>, "<reg>)
        if let Some(mode) = self.pending_char.take() {
            let count = self.take_count();
            match mode {
                PendingCharMode::Find(is_till, forward) => {
                    let motion = if forward {
                        if is_till { VimMotion::TillChar(c) } else { VimMotion::FindChar(c) }
                    } else {
                        if is_till { VimMotion::TillCharBack(c) } else { VimMotion::FindCharBack(c) }
                    };
                    return self.finalize_motion(motion, count);
                }
                PendingCharMode::Replace => {
                    self.reset();
                    return ParsedCommand::ReplaceChar { target: c, count };
                }
                PendingCharMode::Register => {
                    self.selected_register = Some(c);
                    return ParsedCommand::Incomplete;
                }
            }
        }

        // 2. Handle text object targets after 'i' or 'a'
        if let Some(scope) = self.pending_text_scope.take() {
            let target = match c {
                'w' => Some(TextObjectTarget::Word),
                'W' => Some(TextObjectTarget::BigWord),
                '"' => Some(TextObjectTarget::DoubleQuote),
                '\'' => Some(TextObjectTarget::SingleQuote),
                '`' => Some(TextObjectTarget::Backtick),
                '(' | ')' | 'b' => Some(TextObjectTarget::Parentheses),
                '[' | ']' => Some(TextObjectTarget::Brackets),
                '{' | '}' | 'B' => Some(TextObjectTarget::Braces),
                '<' | '>' => Some(TextObjectTarget::AngleBrackets),
                _ => None,
            };

            if let Some(t) = target {
                let object = TextObject { scope, target: t };
                let count = self.take_count();
                if let Some((op, op_count)) = self.pending_op.take() {
                    self.reset();
                    return ParsedCommand::OperatorTextObject {
                        op,
                        object,
                        count: count * op_count,
                    };
                } else if current_mode.is_visual() {
                    self.reset();
                    return ParsedCommand::OperatorTextObject {
                        op: VimOperator::Yank, // Visual object selection
                        object,
                        count,
                    };
                }
            }
            self.reset();
            return ParsedCommand::Unhandled;
        }

        // 3. Handle 'g' prefix commands (gg, gu, gU, ge, gE)
        if self.pending_g {
            self.pending_g = false;
            let count = self.take_count();
            return match c {
                'g' => self.finalize_motion(VimMotion::BufferStart, count),
                'e' => self.finalize_motion(VimMotion::WordEndBackward, count),
                'E' => self.finalize_motion(VimMotion::WordEndBackward, count),
                'u' => {
                    self.pending_op = Some((VimOperator::ToLower, count));
                    ParsedCommand::Incomplete
                }
                'U' => {
                    self.pending_op = Some((VimOperator::ToUpper, count));
                    ParsedCommand::Incomplete
                }
                _ => {
                    self.reset();
                    ParsedCommand::Unhandled
                }
            };
        }

        // 4. Count accumulator (1-9 to start, 0-9 subsequently)
        if c.is_ascii_digit() && (c != '0' || self.count_acc.is_some()) {
            let digit = c.to_digit(10).unwrap() as usize;
            self.count_acc = Some(self.count_acc.unwrap_or(0) * 10 + digit);
            return ParsedCommand::Incomplete;
        }

        // 5. Operator-pending repetitions (e.g. dd, cc, yy)
        if let Some((op, op_count)) = self.pending_op {
            let is_line_op = match (op, c) {
                (VimOperator::Delete, 'd') => true,
                (VimOperator::Change, 'c') => true,
                (VimOperator::Yank, 'y') => true,
                _ => false,
            };
            if is_line_op {
                self.reset();
                let count = self.take_count() * op_count;
                return ParsedCommand::LineOperator { op, count };
            }

            // Text object scope entry ('i' or 'a')
            if c == 'i' {
                self.pending_text_scope = Some(TextObjectScope::Inner);
                return ParsedCommand::Incomplete;
            }
            if c == 'a' {
                self.pending_text_scope = Some(TextObjectScope::A);
                return ParsedCommand::Incomplete;
            }
        }

        // 6. Visual mode operator shortcuts — act on selection immediately
        if current_mode.is_visual() {
            match c {
                'd' | 'x' => {
                    self.reset();
                    return ParsedCommand::VisualOperator(VimOperator::Delete);
                }
                'c' | 's' | 'S' | 'C' => {
                    self.reset();
                    return ParsedCommand::VisualOperator(VimOperator::Change);
                }
                'y' | 'Y' => {
                    self.reset();
                    return ParsedCommand::VisualOperator(VimOperator::Yank);
                }
                'p' | 'P' => {
                    self.reset();
                    return ParsedCommand::VisualPaste;
                }
                '~' => {
                    self.reset();
                    return ParsedCommand::VisualOperator(VimOperator::ToggleCase);
                }
                'r' => {
                    // Visual replace needs a target char — let pending_char handle it
                    self.pending_char = Some(PendingCharMode::Replace);
                    return ParsedCommand::Incomplete;
                }
                'o' => {
                    self.reset();
                    return ParsedCommand::VisualSwapAnchor;
                }
                'u' => {
                    self.reset();
                    return ParsedCommand::VisualOperator(VimOperator::ToLower);
                }
                'U' => {
                    self.reset();
                    return ParsedCommand::VisualOperator(VimOperator::ToUpper);
                }
                'J' => {
                    self.reset();
                    return ParsedCommand::VisualJoinLines;
                }
                'D' | 'X' => {
                    self.reset();
                    return ParsedCommand::VisualOperator(VimOperator::Delete);
                }
                '>' => {
                    self.reset();
                    return ParsedCommand::VisualIndent;
                }
                '<' => {
                    self.reset();
                    return ParsedCommand::VisualOutdent;
                }
                // Text object scope entry in visual mode
                'i' => {
                    self.pending_text_scope = Some(TextObjectScope::Inner);
                    return ParsedCommand::Incomplete;
                }
                'a' => {
                    self.pending_text_scope = Some(TextObjectScope::A);
                    return ParsedCommand::Incomplete;
                }
                // Mode toggling while already in visual mode
                'v' => {
                    self.reset();
                    return if current_mode == VimMode::Visual(VisualType::Character) {
                        ParsedCommand::EnterMode(VimMode::Normal)
                    } else {
                        ParsedCommand::EnterMode(VimMode::Visual(VisualType::Character))
                    };
                }
                'V' => {
                    self.reset();
                    return if current_mode == VimMode::Visual(VisualType::Line) {
                        ParsedCommand::EnterMode(VimMode::Normal)
                    } else {
                        ParsedCommand::EnterMode(VimMode::Visual(VisualType::Line))
                    };
                }
                // Motions (h/j/k/l/w/b/e/$/0/^/G/gg/f/t/;/,/%) fall through
                // to the normal motion parsing below, which correctly extends
                // the selection via cursor movement.
                _ => {}
            }
        }

        // 7. Direct command mapping
        match c {
            // Mode changes
            'i' => {
                self.reset();
                ParsedCommand::EnterMode(VimMode::Insert)
            }
            'a' => {
                self.reset();
                ParsedCommand::AppendAfterCursor
            }
            'I' => {
                self.reset();
                ParsedCommand::InsertLineStart
            }
            'A' => {
                self.reset();
                ParsedCommand::AppendAtLineEnd
            }
            'o' => {
                self.reset();
                ParsedCommand::OpenLineBelow
            }
            'O' => {
                self.reset();
                ParsedCommand::OpenLineAbove
            }
            'v' => {
                self.reset();
                ParsedCommand::EnterMode(VimMode::Visual(VisualType::Character))
            }
            'V' => {
                self.reset();
                ParsedCommand::EnterMode(VimMode::Visual(VisualType::Line))
            }
            'R' => {
                self.reset();
                ParsedCommand::EnterMode(VimMode::Replace)
            }

            // Operators
            'd' => {
                let count = self.take_count();
                self.pending_op = Some((VimOperator::Delete, count));
                ParsedCommand::Incomplete
            }
            'c' => {
                let count = self.take_count();
                self.pending_op = Some((VimOperator::Change, count));
                ParsedCommand::Incomplete
            }
            'y' => {
                let count = self.take_count();
                self.pending_op = Some((VimOperator::Yank, count));
                ParsedCommand::Incomplete
            }
            'r' => {
                self.pending_char = Some(PendingCharMode::Replace);
                ParsedCommand::Incomplete
            }
            '"' => {
                self.pending_char = Some(PendingCharMode::Register);
                ParsedCommand::Incomplete
            }
            'g' => {
                self.pending_g = true;
                ParsedCommand::Incomplete
            }

            // Instant mutations
            'x' => {
                let count = self.take_count();
                ParsedCommand::DeleteChar { forward: true, count }
            }
            'X' => {
                let count = self.take_count();
                ParsedCommand::DeleteChar { forward: false, count }
            }
            's' => {
                let count = self.take_count();
                ParsedCommand::Substitute { count }
            }
            'S' => {
                self.reset();
                ParsedCommand::SubstituteLine
            }
            'C' => {
                self.reset();
                ParsedCommand::OperatorMotion {
                    op: VimOperator::Change,
                    motion: VimMotion::LineEnd,
                    count: 1,
                }
            }
            'D' => {
                self.reset();
                ParsedCommand::OperatorMotion {
                    op: VimOperator::Delete,
                    motion: VimMotion::LineEnd,
                    count: 1,
                }
            }
            'Y' => {
                self.reset();
                ParsedCommand::LineOperator { op: VimOperator::Yank, count: 1 }
            }
            'p' => {
                let count = self.take_count();
                ParsedCommand::Paste { after: true, count }
            }
            'P' => {
                let count = self.take_count();
                ParsedCommand::Paste { after: false, count }
            }
            '~' => {
                let count = self.take_count();
                ParsedCommand::ToggleCase { count }
            }
            'u' => {
                self.reset();
                ParsedCommand::Undo
            }

            // Single key inline search triggers
            'f' => {
                self.pending_char = Some(PendingCharMode::Find(false, true));
                ParsedCommand::Incomplete
            }
            't' => {
                self.pending_char = Some(PendingCharMode::Find(true, true));
                ParsedCommand::Incomplete
            }
            'F' => {
                self.pending_char = Some(PendingCharMode::Find(false, false));
                ParsedCommand::Incomplete
            }
            'T' => {
                self.pending_char = Some(PendingCharMode::Find(true, false));
                ParsedCommand::Incomplete
            }
            ';' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::RepeatFind, count)
            }
            ',' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::RepeatFindRev, count)
            }

            // Standard motions
            'h' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::Left, count)
            }
            'j' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::Down, count)
            }
            'k' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::Up, count)
            }
            'l' | ' ' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::Right, count)
            }
            'w' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::WordForward, count)
            }
            'W' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::BigWordForward, count)
            }
            'b' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::WordBackward, count)
            }
            'B' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::BigWordBackward, count)
            }
            'e' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::WordEnd, count)
            }
            'E' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::BigWordEnd, count)
            }
            '0' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::LineStart, count)
            }
            '^' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::FirstNonBlank, count)
            }
            '$' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::LineEnd, count)
            }
            'G' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::BufferEnd, count)
            }
            '%' => {
                let count = self.take_count();
                self.finalize_motion(VimMotion::MatchingPair, count)
            }

            _ => ParsedCommand::Unhandled,
        }
    }

    fn take_count(&mut self) -> usize {
        self.count_acc.take().unwrap_or(1)
    }

    fn finalize_motion(&mut self, motion: VimMotion, count: usize) -> ParsedCommand {
        if let Some((op, op_count)) = self.pending_op.take() {
            self.reset();
            ParsedCommand::OperatorMotion {
                op,
                motion,
                count: count * op_count,
            }
        } else {
            self.reset();
            ParsedCommand::Motion { motion, count }
        }
    }
}
