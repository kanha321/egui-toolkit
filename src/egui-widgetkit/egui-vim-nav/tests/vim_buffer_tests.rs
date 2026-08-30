use egui_vim_nav::{
    calculate_motion, TextObject, TextObjectScope, TextObjectTarget, UndoHistory, VimBufferState,
    VimMode, VimMotion,
};

#[test]
fn test_vim_motions_word_and_bounds() {
    let text = "hello world_123 (foo.bar) end";

    // Test 'w'
    let mut cursor = 0;
    cursor = calculate_motion(text, cursor, VimMotion::WordForward, 1, None);
    assert_eq!(cursor, 6); // 'world_123'
    cursor = calculate_motion(text, cursor, VimMotion::WordForward, 1, None);
    assert_eq!(cursor, 16); // '('

    // Test 'e'
    let mut end_cursor = 0;
    end_cursor = calculate_motion(text, end_cursor, VimMotion::WordEnd, 1, None);
    assert_eq!(end_cursor, 4); // 'o' in 'hello'

    // Test 'b'
    let back_cursor = calculate_motion(text, 16, VimMotion::WordBackward, 1, None);
    assert_eq!(back_cursor, 6); // 'world_123'

    // Test '0' and '$'
    assert_eq!(calculate_motion(text, 10, VimMotion::LineStart, 1, None), 0);
    assert_eq!(calculate_motion(text, 10, VimMotion::LineEnd, 1, None), 28);
}

#[test]
fn test_vim_inline_searches() {
    let text = "alpha_beta_gamma_delta";
    // Indices of '_': 5, 10, 16

    // 'f_'
    let target = calculate_motion(text, 0, VimMotion::FindChar('_'), 1, None);
    assert_eq!(target, 5);

    // '2f_'
    let target2 = calculate_motion(text, 0, VimMotion::FindChar('_'), 2, None);
    assert_eq!(target2, 10);

    // 't_'
    let target_till = calculate_motion(text, 0, VimMotion::TillChar('_'), 1, None);
    assert_eq!(target_till, 4); // 'a' before '_'

    // 'F_' backwards from 12
    let target_back = calculate_motion(text, 12, VimMotion::FindCharBack('_'), 1, None);
    assert_eq!(target_back, 10);
}

#[test]
fn test_vim_matching_pair() {
    let text = "fn calculate(a: [i32; 4]) -> { let x = 1; }";

    // Matching '(' at index 12 -> ')' at index 24
    let match_paren = calculate_motion(text, 12, VimMotion::MatchingPair, 1, None);
    assert_eq!(match_paren, 24);

    // Matching '[' at index 16 -> ']' at index 23
    let match_bracket = calculate_motion(text, 16, VimMotion::MatchingPair, 1, None);
    assert_eq!(match_bracket, 23);

    // Matching '{' at index 29 -> '}' at index 42
    let match_brace = calculate_motion(text, 29, VimMotion::MatchingPair, 1, None);
    assert_eq!(match_brace, 42);
}

#[test]
fn test_vim_text_objects() {
    let text = "let name = \"alice_smith\";";

    // Text object: iw on alice_smith (index 14)
    let obj_iw = TextObject {
        scope: TextObjectScope::Inner,
        target: TextObjectTarget::Word,
    };
    let range_iw = obj_iw.resolve_range(text, 14).expect("resolved iw");
    assert_eq!(&text[range_iw], "alice_smith");

    // Text object: i" inside quotes
    let obj_iq = TextObject {
        scope: TextObjectScope::Inner,
        target: TextObjectTarget::DoubleQuote,
    };
    let range_iq = obj_iq.resolve_range(text, 14).expect("resolved i\"");
    assert_eq!(&text[range_iq], "alice_smith");

    // Text object: a" around quotes
    let obj_aq = TextObject {
        scope: TextObjectScope::A,
        target: TextObjectTarget::DoubleQuote,
    };
    let range_aq = obj_aq.resolve_range(text, 14).expect("resolved a\"");
    assert_eq!(&text[range_aq], "\"alice_smith\"");
}

#[test]
fn test_vim_buffer_state_operations() {
    let mut state = VimBufferState::new("hello world foo bar");

    // 1. Delete word: 'dw'
    state.handle_char('d');
    state.handle_char('w');
    assert_eq!(state.text(), "world foo bar");

    // 2. Undo: 'u'
    state.handle_char('u');
    assert_eq!(state.text(), "hello world foo bar");

    // 3. Redo
    state.redo();
    assert_eq!(state.text(), "world foo bar");

    // 4. Replace char: 'rX'
    state.handle_char('r');
    state.handle_char('X');
    assert_eq!(state.text(), "Xorld foo bar");

    // 5. Change word: 'cw' -> replaces word and enters Insert mode
    state.handle_char('c');
    state.handle_char('w');
    assert_eq!(state.mode(), VimMode::Insert);
    assert_eq!(state.text(), "foo bar");

    // 6. Type inside Insert mode
    state.insert_str("new_word ");
    assert_eq!(state.text(), "new_word foo bar");
}

#[test]
fn test_vim_registers_and_paste() {
    let mut state = VimBufferState::new("first second third");

    // Delete first word into unnamed register
    state.handle_char('d');
    state.handle_char('w');
    assert_eq!(state.text(), "second third");
    assert_eq!(state.registers.unnamed, "first ");

    // Move to end of line '$'
    state.handle_char('$');

    // Paste 'p' after cursor
    state.handle_char('p');
    assert_eq!(state.text(), "second thirdfirst ");
}

#[test]
fn test_vim_undo_history_stack() {
    let mut hist = UndoHistory::new();
    // Initial state is "version 1"
    hist.push("version 1", 0);
    // User modifies to "version 2"
    hist.push("version 2", 4);
    // Current live buffer is "version 3"

    let undo1 = hist.undo("version 3", 8).unwrap();
    assert_eq!(undo1.text, "version 2");
    assert_eq!(undo1.cursor, 4);

    let undo2 = hist.undo("version 2", 4).unwrap();
    assert_eq!(undo2.text, "version 1");
    assert_eq!(undo2.cursor, 0);

    let redo1 = hist.redo("version 1", 0).unwrap();
    assert_eq!(redo1.text, "version 2");

    let redo2 = hist.redo("version 2", 4).unwrap();
    assert_eq!(redo2.text, "version 3");
}

#[test]
fn test_vim_jk_fast_exit() {
    let mut state = VimBufferState::new("hello ");

    // Enter insert mode
    state.handle_char('A');
    assert_eq!(state.mode(), VimMode::Insert);

    // Type "world"
    for c in "world".chars() {
        state.insert_char(c);
    }
    assert_eq!(state.text(), "hello world");

    // Press 'j' -> inserted
    state.insert_char('j');
    assert_eq!(state.text(), "hello worldj");
    assert_eq!(state.mode(), VimMode::Insert);

    // Press 'k' -> exits insert mode, removes 'j'
    state.insert_char('k');
    assert_eq!(state.text(), "hello world");
    assert_eq!(state.mode(), VimMode::Normal);
}

#[test]
fn test_vim_visual_delete_selection() {
    let mut state = VimBufferState::new("hello world");
    // Enter visual mode
    state.handle_char('v');
    assert!(state.mode().is_visual());
    // Select 'h', 'e', 'l', 'l', 'o' (move right 4 times to select 5 chars)
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    // Delete selection
    state.handle_char('d');
    assert_eq!(state.text(), " world");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_change_selection() {
    let mut state = VimBufferState::new("hello world");
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    // Change selection (delete + enter Insert)
    state.handle_char('c');
    assert_eq!(state.text(), " world");
    assert_eq!(state.mode(), VimMode::Insert);
}

#[test]
fn test_vim_visual_yank_selection() {
    let mut state = VimBufferState::new("hello world");
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    // Yank selection
    state.handle_char('y');
    // Text should be unchanged
    assert_eq!(state.text(), "hello world");
    assert!(state.mode().is_normal());
    // Both unnamed and yank registers should contain the yanked text
    assert_eq!(state.registers.unnamed, "hello");
    assert_eq!(state.registers.yank, "hello");
}

#[test]
fn test_vim_visual_paste_replaces() {
    let mut state = VimBufferState::new("hello world");
    // First, yank "hello" by deleting and undoing (to fill the register)
    state.handle_char('d');
    state.handle_char('w');
    assert_eq!(state.text(), "world");
    assert_eq!(state.registers.unnamed, "hello ");

    // Now select "wor" with visual mode
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    // Paste over the selection
    state.handle_char('p');
    assert_eq!(state.text(), "hello ld");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_toggle_case() {
    let mut state = VimBufferState::new("hello WORLD");
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('~');
    assert_eq!(state.text(), "HELLO WORLD");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_replace_char() {
    let mut state = VimBufferState::new("hello world");
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    // Replace all selected chars with '*'
    state.handle_char('r');
    state.handle_char('*');
    assert_eq!(state.text(), "***** world");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_swap_anchor() {
    let mut state = VimBufferState::new("hello world");
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    assert_eq!(state.cursor(), 2);
    // Swap cursor and anchor
    state.handle_char('o');
    assert_eq!(state.cursor(), 0);
    assert!(state.mode().is_visual());
}

#[test]
fn test_vim_visual_join_lines() {
    let mut state = VimBufferState::new("hello\nworld\nfoo");
    // Select everything
    state.handle_char('v');
    // Move to end
    state.handle_char('$');
    state.handle_char('j');
    state.handle_char('j');
    // Join lines
    state.handle_char('J');
    assert_eq!(state.text(), "hello world foo");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_indent() {
    let mut state = VimBufferState::new("line1\nline2\nline3");
    state.handle_char('v');
    state.handle_char('j');
    state.handle_char('j');
    state.handle_char('>');
    assert!(state.text().contains("    line1"));
    assert!(state.text().contains("    line2"));
    assert!(state.text().contains("    line3"));
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_outdent() {
    let mut state = VimBufferState::new("    line1\n    line2\n    line3");
    state.handle_char('v');
    state.handle_char('j');
    state.handle_char('j');
    state.handle_char('<');
    assert_eq!(state.text(), "line1\nline2\nline3");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_x_deletes_selection() {
    let mut state = VimBufferState::new("abcdef");
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    // x in visual mode should delete the selection, not just one char
    state.handle_char('x');
    assert_eq!(state.text(), "def");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_s_changes_selection() {
    let mut state = VimBufferState::new("abcdef");
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    // s in visual mode should delete selection and enter Insert
    state.handle_char('s');
    assert_eq!(state.text(), "def");
    assert_eq!(state.mode(), VimMode::Insert);
}

#[test]
fn test_vim_visual_lowercase_uppercase() {
    let mut state = VimBufferState::new("Hello World");
    // Select "Hello"
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    // Lowercase with 'u'
    state.handle_char('u');
    assert_eq!(state.text(), "hello World");
    assert!(state.mode().is_normal());

    // Now uppercase "hello"
    state.handle_char('0'); // go to start
    state.handle_char('v');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('l');
    state.handle_char('U');
    assert_eq!(state.text(), "HELLO World");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_handle_input_no_phantom_char() {
    // Verify that transitioning Normal → Insert via 'i' key
    // does NOT insert a literal 'i' into the buffer.
    let mut state = VimBufferState::new("hello");
    assert_eq!(state.mode(), VimMode::Normal);

    // Simulate what handle_input would see: Key::I followed by Text("i")
    // Since we can't easily create egui Context in tests, we use handle_key + insert_char
    // to verify the mode transition logic:
    let handled = state.handle_key(egui::Key::I, &egui::Modifiers::NONE);
    assert!(handled);
    assert_eq!(state.mode(), VimMode::Insert);
    // Text should remain unchanged — the 'i' was consumed as a mode switch
    assert_eq!(state.text(), "hello");
    // Cursor should be at position 0 (where 'i' enters insert)
    assert_eq!(state.cursor(), 0);
}

#[test]
fn test_vim_visual_line_selects_whole_line() {
    let mut state = VimBufferState::new("first line\nsecond line\nthird line");
    // Move to middle of second line
    state.handle_char('j');
    state.handle_char('w');
    assert_eq!(state.cursor(), 18); // on "line" in "second line"

    // Press 'V' to enter Visual Line mode
    state.handle_char('V');
    assert!(state.mode().is_visual());

    // Selection range must cover the entire second line including newline
    let range = state.selection_range().expect("has selection range");
    assert_eq!(&state.text()[range], "second line\n");

    // Delete the visual line
    state.handle_char('d');
    assert_eq!(state.text(), "first line\nthird line");
    assert!(state.mode().is_normal());
}

#[test]
fn test_vim_visual_line_multi_line_expansion() {
    let mut state = VimBufferState::new("line1\nline2\nline3\nline4");
    // Start at line 2
    state.handle_char('j');
    state.handle_char('V');

    // Move down to line 3
    state.handle_char('j');

    // Selection must cover line2 and line3
    let range = state.selection_range().expect("has selection range");
    assert_eq!(&state.text()[range], "line2\nline3\n");

    // Change lines (delete and enter insert)
    state.handle_char('c');
    assert_eq!(state.text(), "line1\nline4");
    assert_eq!(state.mode(), VimMode::Insert);
}

#[test]
fn test_vim_dynamic_operator_pending_and_replace_mode() {
    use egui_vim_nav::VimOperator;

    let mut state = VimBufferState::new("hello world");

    // 1. Operator Pending for Delete ('d')
    state.handle_char('d');
    assert_eq!(
        state.mode(),
        VimMode::OperatorPending {
            operator: VimOperator::Delete,
            count: 1
        }
    );
    // Complete with 'w'
    state.handle_char('w');
    assert_eq!(state.text(), "world");
    assert_eq!(state.mode(), VimMode::Normal);

    // 2. Operator Pending for Change ('c')
    state.handle_char('c');
    assert_eq!(
        state.mode(),
        VimMode::OperatorPending {
            operator: VimOperator::Change,
            count: 1
        }
    );
    // Complete with 'w' -> enters Insert mode
    state.handle_char('w');
    assert_eq!(state.mode(), VimMode::Insert);

    // 3. Single Char Replace ('r')
    state.set_text("test");
    state.mode = VimMode::Normal;
    state.handle_char('r');
    assert_eq!(state.mode(), VimMode::Replace);
    state.handle_char('b');
    assert_eq!(state.text(), "best");
    assert_eq!(state.mode(), VimMode::Normal);
}

#[test]
fn test_vim_ctrl_c_returns_to_normal_mode() {
    use egui::{Key, Modifiers};

    let mut state = VimBufferState::new("hello world");

    // 1. From Insert mode -> Normal mode on Ctrl+C
    state.handle_char('i');
    assert_eq!(state.mode(), VimMode::Insert);
    let handled = state.handle_key(Key::C, &Modifiers::CTRL);
    assert!(handled);
    assert_eq!(state.mode(), VimMode::Normal);

    // 2. From Visual mode -> Normal mode on Ctrl+C (clearing selection)
    state.handle_char('v');
    state.handle_char('e');
    assert!(state.mode().is_visual());
    assert!(state.selection_range().is_some());
    let handled = state.handle_key(Key::C, &Modifiers::CTRL);
    assert!(handled);
    assert_eq!(state.mode(), VimMode::Normal);
    assert!(state.selection_range().is_none());

    // 3. From Replace mode -> Normal mode on Ctrl+C
    state.handle_char('R');
    assert_eq!(state.mode(), VimMode::Replace);
    let handled = state.handle_key(Key::C, &Modifiers::CTRL);
    assert!(handled);
    assert_eq!(state.mode(), VimMode::Normal);
}
