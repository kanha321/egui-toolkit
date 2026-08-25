use eframe::egui;
use egui::{Color32, Rect, Rounding, Stroke, Ui, Vec2};
use egui_spring::{HighlightConfig, HighlightGroup, MotionPhysics};
use egui_vim_nav::{FocusGraph, FocusRegion, Navigator, VimAction, VimKeyHandler};
use egui_themes::ThemePalette;
use spring_core::SpringParams;

// ─── Highlight Layers (2-Tier: Outer Container + Inner Active Element) ────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HighlightLayer {
    /// Outer container highlight (glides around Section 1, 2, 3 container frames)
    Section,
    /// Single inner element highlight (glides seamlessly across active keys and actions)
    Item,
}

// ─── Section Definition ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SectionId {
    Keyboard,
    QuickActions,
    EventLog,
}

// ─── Staggered Keyboard Definition ──────────────────────────────────────────

pub struct KeySpec {
    pub id: &'static str,
    pub label: &'static str,
    pub width: f32,
}

impl KeySpec {
    const fn new(id: &'static str, label: &'static str, width: f32) -> Self {
        Self { id, label, width }
    }
}

const KB_ROW_0: &[KeySpec] = &[
    KeySpec::new("1", "1", 38.0),
    KeySpec::new("2", "2", 38.0),
    KeySpec::new("3", "3", 38.0),
    KeySpec::new("4", "4", 38.0),
    KeySpec::new("5", "5", 38.0),
    KeySpec::new("6", "6", 38.0),
    KeySpec::new("7", "7", 38.0),
    KeySpec::new("8", "8", 38.0),
    KeySpec::new("9", "9", 38.0),
    KeySpec::new("0", "0", 38.0),
    KeySpec::new("-", "-", 38.0),
    KeySpec::new("=", "=", 38.0),
    KeySpec::new("Bksp", "Bksp", 62.0),
];

const KB_ROW_1: &[KeySpec] = &[
    KeySpec::new("Tab", "Tab", 54.0),
    KeySpec::new("Q", "Q", 38.0),
    KeySpec::new("W", "W", 38.0),
    KeySpec::new("E", "E", 38.0),
    KeySpec::new("R", "R", 38.0),
    KeySpec::new("T", "T", 38.0),
    KeySpec::new("Y", "Y", 38.0),
    KeySpec::new("U", "U", 38.0),
    KeySpec::new("I", "I", 38.0),
    KeySpec::new("O", "O", 38.0),
    KeySpec::new("P", "P", 38.0),
    KeySpec::new("[", "[", 38.0),
    KeySpec::new("]", "]", 38.0),
    KeySpec::new("\\", "\\", 40.0),
];

const KB_ROW_2: &[KeySpec] = &[
    KeySpec::new("Caps", "Caps", 64.0),
    KeySpec::new("A", "A", 38.0),
    KeySpec::new("S", "S", 38.0),
    KeySpec::new("D", "D", 38.0),
    KeySpec::new("F", "F", 38.0),
    KeySpec::new("G", "G", 38.0),
    KeySpec::new("H", "H", 38.0),
    KeySpec::new("J", "J", 38.0),
    KeySpec::new("K", "K", 38.0),
    KeySpec::new("L", "L", 38.0),
    KeySpec::new(";", ";", 38.0),
    KeySpec::new("'", "'", 38.0),
    KeySpec::new("Enter", "Enter", 68.0),
];

const KB_ROW_3: &[KeySpec] = &[
    KeySpec::new("LShift", "Shift", 78.0),
    KeySpec::new("Z", "Z", 38.0),
    KeySpec::new("X", "X", 38.0),
    KeySpec::new("C", "C", 38.0),
    KeySpec::new("V", "V", 38.0),
    KeySpec::new("B", "B", 38.0),
    KeySpec::new("N", "N", 38.0),
    KeySpec::new("M", "M", 38.0),
    KeySpec::new(",", ",", 38.0),
    KeySpec::new(".", ".", 38.0),
    KeySpec::new("/", "/", 38.0),
    KeySpec::new("RShift", "Shift", 50.0),
    KeySpec::new("Up", "Up", 38.0),
];

const KB_ROW_4: &[KeySpec] = &[
    KeySpec::new("Ctrl", "Ctrl", 46.0),
    KeySpec::new("Win", "Win", 40.0),
    KeySpec::new("Alt", "Alt", 42.0),
    KeySpec::new("Space", "Space", 200.0),
    KeySpec::new("RAlt", "Alt", 42.0),
    KeySpec::new("RCtrl", "Ctrl", 46.0),
    KeySpec::new("Left", "Left", 38.0),
    KeySpec::new("Down", "Down", 38.0),
    KeySpec::new("Right", "Right", 38.0),
];

const QUICK_ACTIONS: &[&'static str] = &[
    "Phrase: Hello",
    "Phrase: Rust",
    "Phrase: Vim-Nav",
    "Action: Clear",
    "Action: Space",
    "Action: Newline",
];

// ─── Key Action Handlers ─────────────────────────────────────────────────────

fn apply_key_primary(key_id: &str, display: &mut String, redo_buffer: &mut Vec<char>) -> String {
    redo_buffer.clear();
    match key_id {
        "Space" => {
            display.push(' ');
            "Typed Space".to_string()
        }
        "Bksp" => {
            if let Some(c) = display.pop() {
                redo_buffer.push(c);
            }
            "Backspace (deleted 1 char)".to_string()
        }
        "Tab" => {
            display.push_str("    ");
            "Typed Tab (4 spaces)".to_string()
        }
        "Enter" => {
            display.push('\n');
            "Typed Newline".to_string()
        }
        "Caps" | "LShift" | "RShift" | "Ctrl" | "RCtrl" | "Alt" | "RAlt" | "Win"
        | "Up" | "Down" | "Left" | "Right" => {
            format!("Key '{}' activated", key_id)
        }
        s => {
            let text = if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() {
                s.to_lowercase()
            } else {
                s.to_string()
            };
            display.push_str(&text);
            format!("Typed '{}'", text)
        }
    }
}

fn apply_key_secondary(key_id: &str, display: &mut String, redo_buffer: &mut Vec<char>) -> String {
    redo_buffer.clear();
    match key_id {
        "1" => { display.push('!'); "Shift-typed '!'".to_string() }
        "2" => { display.push('@'); "Shift-typed '@'".to_string() }
        "3" => { display.push('#'); "Shift-typed '#'".to_string() }
        "4" => { display.push('$'); "Shift-typed '$'".to_string() }
        "5" => { display.push('%'); "Shift-typed '%'".to_string() }
        "6" => { display.push('^'); "Shift-typed '^'".to_string() }
        "7" => { display.push('&'); "Shift-typed '&'".to_string() }
        "8" => { display.push('*'); "Shift-typed '*'".to_string() }
        "9" => { display.push('('); "Shift-typed '('".to_string() }
        "0" => { display.push(')'); "Shift-typed ')'".to_string() }
        "-" => { display.push('_'); "Shift-typed '_'".to_string() }
        "=" => { display.push('+'); "Shift-typed '+'".to_string() }
        "[" => { display.push('{'); "Shift-typed '{'".to_string() }
        "]" => { display.push('}'); "Shift-typed '}'".to_string() }
        "\\" => { display.push('|'); "Shift-typed '|'".to_string() }
        ";" => { display.push(':'); "Shift-typed ':'".to_string() }
        "'" => { display.push('"'); "Shift-typed '\"'".to_string() }
        "," => { display.push('<'); "Shift-typed '<'".to_string() }
        "." => { display.push('>'); "Shift-typed '>'".to_string() }
        "/" => { display.push('?'); "Shift-typed '?'".to_string() }
        "Space" => { display.push('_'); "Alternate Space (typed '_')".to_string() }
        "Bksp" => { display.clear(); "Cleared entire text buffer".to_string() }
        "Enter" => { display.push_str("\n───\n"); "Inserted divider line".to_string() }
        s if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() => {
            let upper = s.to_uppercase();
            display.push_str(&upper);
            format!("Shift-typed UPPERCASE '{}'", upper)
        }
        other => format!("Secondary action on '{}'", other),
    }
}

// ─── Scene State ────────────────────────────────────────────────────────────

/// Scene-specific state for the Vim Nav demo.
///
/// Owned by `TestAppState` per CODING_RULES §2 — no global/static state.
pub struct VimNavDemoState {
    pub section_graph: FocusGraph<SectionId>,
    pub section_nav: Navigator<SectionId>,

    pub kb_graph: FocusGraph<&'static str>,
    pub kb_nav: Navigator<&'static str>,

    pub action_graph: FocusGraph<&'static str>,
    pub action_nav: Navigator<&'static str>,

    /// Multi-layer highlight manager holding an arbitrary number of independent spring highlights
    pub highlights: HighlightGroup<HighlightLayer>,

    pub key_handler: VimKeyHandler,
    pub text_input: String,
    pub typed_display: String,
    pub redo_buffer: Vec<char>,
    pub event_log: Vec<String>,
}

impl VimNavDemoState {
    pub fn new(palette: &ThemePalette) -> Self {
        // 1. Inter-Section Focus Graph (Ctrl + HJKL)
        let mut section_graph = FocusGraph::new();
        section_graph.connect_horizontal(SectionId::Keyboard, SectionId::QuickActions);
        section_graph.connect_vertical(SectionId::Keyboard, SectionId::EventLog);
        section_graph.connect_vertical(SectionId::QuickActions, SectionId::EventLog);

        // 2. Keyboard Intra-Section Graph
        let mut kb_graph = FocusGraph::new();
        let all_rows: &[&[KeySpec]] = &[KB_ROW_0, KB_ROW_1, KB_ROW_2, KB_ROW_3, KB_ROW_4];
        for row in all_rows {
            for i in 0..row.len().saturating_sub(1) {
                kb_graph.connect_horizontal(row[i].id, row[i + 1].id);
            }
        }

        // Staggered vertical connections
        kb_graph.connect_vertical("1", "Tab");
        kb_graph.connect_vertical("2", "Q");
        kb_graph.connect_vertical("3", "W");
        kb_graph.connect_vertical("4", "E");
        kb_graph.connect_vertical("5", "R");
        kb_graph.connect_vertical("6", "T");
        kb_graph.connect_vertical("7", "Y");
        kb_graph.connect_vertical("8", "U");
        kb_graph.connect_vertical("9", "I");
        kb_graph.connect_vertical("0", "O");
        kb_graph.connect_vertical("-", "P");
        kb_graph.connect_vertical("=", "[");
        kb_graph.connect_vertical("Bksp", "]");

        kb_graph.connect_vertical("Tab", "Caps");
        kb_graph.connect_vertical("Q", "A");
        kb_graph.connect_vertical("W", "S");
        kb_graph.connect_vertical("E", "D");
        kb_graph.connect_vertical("R", "F");
        kb_graph.connect_vertical("T", "G");
        kb_graph.connect_vertical("Y", "H");
        kb_graph.connect_vertical("U", "J");
        kb_graph.connect_vertical("I", "K");
        kb_graph.connect_vertical("O", "L");
        kb_graph.connect_vertical("P", ";");
        kb_graph.connect_vertical("[", "'");
        kb_graph.connect_vertical("]", "Enter");
        kb_graph.connect_vertical("\\", "Enter");

        kb_graph.connect_vertical("Caps", "LShift");
        kb_graph.connect_vertical("A", "Z");
        kb_graph.connect_vertical("S", "X");
        kb_graph.connect_vertical("D", "C");
        kb_graph.connect_vertical("F", "V");
        kb_graph.connect_vertical("G", "B");
        kb_graph.connect_vertical("H", "N");
        kb_graph.connect_vertical("J", "M");
        kb_graph.connect_vertical("K", ",");
        kb_graph.connect_vertical("L", ".");
        kb_graph.connect_vertical(";", "/");
        kb_graph.connect_vertical("'", "RShift");
        kb_graph.connect_vertical("Enter", "Up");

        kb_graph.connect_vertical("LShift", "Ctrl");
        kb_graph.connect_vertical("Z", "Win");
        kb_graph.connect_vertical("X", "Alt");
        kb_graph.connect_vertical("C", "Space");
        kb_graph.connect_vertical("V", "Space");
        kb_graph.connect_vertical("B", "Space");
        kb_graph.connect_vertical("N", "Space");
        kb_graph.connect_vertical("M", "Space");
        kb_graph.connect_vertical(",", "RAlt");
        kb_graph.connect_vertical(".", "RCtrl");
        kb_graph.connect_vertical("/", "Left");
        kb_graph.connect_vertical("RShift", "Down");
        kb_graph.connect_vertical("Up", "Down");
        kb_graph.connect_horizontal("Left", "Down");
        kb_graph.connect_horizontal("Down", "Right");

        // 3. Quick Actions Vertical List Graph
        let mut action_graph = FocusGraph::new();
        for i in 0..QUICK_ACTIONS.len().saturating_sub(1) {
            action_graph.connect_vertical(QUICK_ACTIONS[i], QUICK_ACTIONS[i + 1]);
        }

        // 4. Initialize Multi-Layer Highlight Group with 3 completely independent highlights
        let mut highlights = HighlightGroup::new();

        // 1. Outer Section Highlight (Gentle preset: smooth, cushioned container glide)
        highlights.add(
            HighlightLayer::Section,
            HighlightConfig::new()
                .with_motion(MotionPhysics::Gentle)
                .with_fill(Color32::from_rgba_unmultiplied(palette.accent.r(), palette.accent.g(), palette.accent.b(), 12))
                .with_stroke(Stroke::new(2.0, palette.accent))
                .with_rounding(6.0)
                .with_padding(3.0),
        );

        // 2. Single Inner Active Element Highlight (Custom preset: ω0 = 26.0, ζ = 0.58)
        // Seamlessly glides and morphs across focused keyboard keys and quick action items
        highlights.add(
            HighlightLayer::Item,
            HighlightConfig::new()
                .with_motion(MotionPhysics::Custom(SpringParams::new(26.0, 0.58)))
                .with_fill(Color32::from_rgba_unmultiplied(palette.info.r(), palette.info.g(), palette.info.b(), 36))
                .with_stroke(Stroke::new(2.0, palette.info))
                .with_rounding(5.0)
                .with_padding(2.0),
        );

        Self {
            section_graph,
            section_nav: Navigator::new().with_initial_focus(SectionId::Keyboard),

            kb_graph,
            kb_nav: Navigator::new().with_initial_focus("J"),

            action_graph,
            action_nav: Navigator::new().with_initial_focus(QUICK_ACTIONS[0]),

            highlights,

            key_handler: VimKeyHandler::new().with_tab(false),
            text_input: String::new(),
            typed_display: String::from("VIM-NAV "),
            redo_buffer: Vec::new(),
            event_log: Vec::new(),
        }
    }
}

impl Default for VimNavDemoState {
    fn default() -> Self {
        Self::new(&ThemePalette::default())
    }
}

/// Renders the multi-section Vim Nav demo scene with primary/secondary clicks,
/// 2-tier spring motion physics (outer section + inner items), and mouse/keyboard navigation.
pub fn show(ui: &mut Ui, state: &mut VimNavDemoState, palette: &ThemePalette) {
    let ctx = ui.ctx();
    let pointer_moved = ui.input(|i| i.pointer.delta() != Vec2::ZERO);

    // Suppress egui's default Tab focus cycling (unconditionally)
    ctx.input_mut(|i| {
        i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
        i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
    });

    // 1. Inter-Section Navigation with Ctrl + H/J/K/L
    if let Some(event) = state.key_handler.handle_section_nav(
        ctx,
        &mut state.section_nav,
        &state.section_graph,
    ) {
        let msg = format!(
            "Section Nav {:?}: {:?} ➔ {:?}",
            event.direction,
            event.previous.unwrap_or(SectionId::Keyboard),
            event.current,
        );
        state.event_log.push(msg);
        if state.event_log.len() > 25 {
            state.event_log.remove(0);
        }
    }

    let active_section = state.section_nav.focused().copied().unwrap_or(SectionId::Keyboard);

    // 2. Intra-Section Navigation with plain H/J/K/L within the currently active section
    match active_section {
        SectionId::Keyboard => {
            if let Some(event) = state.key_handler.handle_input(
                ctx,
                &mut state.kb_nav,
                &state.kb_graph,
            ) {
                let msg = format!(
                    "Key Nav {:?}: {} ➔ {}",
                    event.direction,
                    event.previous.unwrap_or("None"),
                    event.current,
                );
                state.event_log.push(msg);
                if state.event_log.len() > 25 {
                    state.event_log.remove(0);
                }
            }
        }
        SectionId::QuickActions => {
            if let Some(event) = state.key_handler.handle_input(
                ctx,
                &mut state.action_nav,
                &state.action_graph,
            ) {
                let msg = format!(
                    "Action Nav {:?}: {} ➔ {}",
                    event.direction,
                    event.previous.unwrap_or("None"),
                    event.current,
                );
                state.event_log.push(msg);
                if state.event_log.len() > 25 {
                    state.event_log.remove(0);
                }
            }
        }
        SectionId::EventLog => {}
    }

    // Direct detection of thumb buttons:
    // Extra1 = Lower Thumb (Button 4 / Back)
    // Extra2 = Upper Thumb (Button 5 / Forward)
    let lower_thumb_clicked = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra1));
    let upper_thumb_clicked = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra2));

    if lower_thumb_clicked {
        if let Some(c) = state.typed_display.pop() {
            state.redo_buffer.push(c);
            state.event_log.push(format!("Mouse Lower Thumb (Button 4 / Back): Deleted '{}'", c));
        } else {
            state.event_log.push("Mouse Lower Thumb (Button 4 / Back): Buffer already empty".to_string());
        }
        if state.event_log.len() > 25 {
            state.event_log.remove(0);
        }
    }

    if upper_thumb_clicked {
        if let Some(c) = state.redo_buffer.pop() {
            state.typed_display.push(c);
            state.event_log.push(format!("Mouse Upper Thumb (Button 5 / Forward): Redid/Restored '{}'", c));
        } else {
            state.typed_display.push(' ');
            state.event_log.push("Mouse Upper Thumb (Button 5 / Forward): Advanced (typed Space)".to_string());
        }
        if state.event_log.len() > 25 {
            state.event_log.remove(0);
        }
    }

    // 3. Action keys (F/Enter = Primary Click, D = Secondary Click, Q/Esc = Back)
    if let Some(action) = state.key_handler.handle_action(ctx) {
        match action {
            VimAction::PrimaryClick | VimAction::Enter => match active_section {
                SectionId::Keyboard => {
                    if let Some(focused) = state.kb_nav.focused() {
                        let desc = apply_key_primary(focused, &mut state.typed_display, &mut state.redo_buffer);
                        let msg = format!("Primary Click (F / Enter): {}", desc);
                        state.event_log.push(msg);
                    }
                }
                SectionId::QuickActions => {
                    state.redo_buffer.clear();
                    if let Some(focused) = state.action_nav.focused() {
                        match *focused {
                            "Phrase: Hello" => state.typed_display.push_str("Hello "),
                            "Phrase: Rust" => state.typed_display.push_str("Rust "),
                            "Phrase: Vim-Nav" => state.typed_display.push_str("Vim-Nav "),
                            "Action: Clear" => state.typed_display.clear(),
                            "Action: Space" => state.typed_display.push(' '),
                            "Action: Newline" => state.typed_display.push('\n'),
                            _ => {}
                        }
                        let msg = format!("Primary Click (F / Enter): Executed '{}'", focused);
                        state.event_log.push(msg);
                    }
                }
                SectionId::EventLog => {
                    state.event_log.clear();
                    state.event_log.push("Primary Click (F / Enter): Cleared event log".to_string());
                }
            },
            VimAction::SecondaryClick => match active_section {
                SectionId::Keyboard => {
                    if let Some(focused) = state.kb_nav.focused() {
                        let desc = apply_key_secondary(focused, &mut state.typed_display, &mut state.redo_buffer);
                        let msg = format!("Secondary Click (D / Right-Click): {}", desc);
                        state.event_log.push(msg);
                    }
                }
                SectionId::QuickActions => {
                    state.redo_buffer.clear();
                    if let Some(focused) = state.action_nav.focused() {
                        let wrapped = format!("[{}] ", focused);
                        state.typed_display.push_str(&wrapped);
                        let msg = format!("Secondary Click (D / Right-Click): Inserted wrapped '{}'", focused);
                        state.event_log.push(msg);
                    }
                }
                SectionId::EventLog => {
                    let ts = format!("Timestamp: {} entries", state.event_log.len());
                    state.event_log.push(format!("Secondary Click (D / Right-Click): {}", ts));
                }
            },
            VimAction::Back => {
                if !lower_thumb_clicked {
                    if let Some(c) = state.typed_display.pop() {
                        state.redo_buffer.push(c);
                        state.event_log.push(format!("Back Action (Q / Esc): Deleted '{}'", c));
                    }
                }
            }
            VimAction::Forward => {
                if !upper_thumb_clicked {
                    if let Some(c) = state.redo_buffer.pop() {
                        state.typed_display.push(c);
                        state.event_log.push(format!("Forward Action: Redid/Restored '{}'", c));
                    }
                }
            }
        }
        if state.event_log.len() > 25 {
            state.event_log.remove(0);
        }
    }

    // Header & Info
    ui.heading("⌨️ egui-vim-nav + egui-spring — 2-Tier Spring Highlights (Outer + Inner)");
    ui.label("Outer Spring = Section Focus (Ctrl+HJKL)  •  Inner Spring = Element Focus (HJKL/Mouse)  •  F/D/Q/Mouse Controls");
    ui.add_space(6.0);

    // Live Typed Output Bar
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Typed Output:").strong());
            ui.label(
                egui::RichText::new(&state.typed_display)
                    .monospace()
                    .color(palette.success)
                    .size(15.0),
            );
        });
    });

    ui.add_space(4.0);

    // Current Active Section & Guard Test
    ui.horizontal(|ui| {
        ui.label("Active Section:");
        let sec_text = match active_section {
            SectionId::Keyboard => "【 Section 1: Keyboard (Left) 】",
            SectionId::QuickActions => "【 Section 2: Quick Actions (Right) 】",
            SectionId::EventLog => "【 Section 3: Event Log (Bottom) 】",
        };
        ui.colored_label(palette.info, egui::RichText::new(sec_text).strong());

        ui.separator();
        ui.label("Text input guard test:");
        ui.add(egui::TextEdit::singleline(&mut state.text_input).hint_text("Type HJKL/F/D/Q/Ctrl here..."));
    });

    ui.add_space(4.0);

    // Motion Physics Selectors (Outer Section + Inner Element)
    ui.group(|ui| {
        ui.vertical(|ui| {
            // Layer 1: Outer Container Section
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Outer Section Physics:").strong());
                let current_motion = state.highlights.get(&HighlightLayer::Section).map(|h| h.motion).unwrap_or_default();
                for &(opt, label) in &[
                    (MotionPhysics::Gentle, "Gentle (18/0.90)"),
                    (MotionPhysics::Default, "Default"),
                    (MotionPhysics::Snappy, "Snappy"),
                    (MotionPhysics::OpenRGB, "OpenRGB"),
                    (MotionPhysics::Off, "Off (Instant)"),
                ] {
                    if ui.selectable_label(current_motion == opt, label).clicked() {
                        if let Some(layer) = state.highlights.get_mut(&HighlightLayer::Section) {
                            layer.set_motion(opt);
                        }
                    }
                }
            });

            // Layer 2: Single Inner Active Element (Keyboard Key or Quick Action)
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Inner Element Physics:").strong());
                let current_motion = state.highlights.get(&HighlightLayer::Item).map(|h| h.motion).unwrap_or_default();
                let is_custom = matches!(current_motion, MotionPhysics::Custom(_));
                if ui.selectable_label(is_custom, "Custom (26/0.58)").clicked() {
                    let p = SpringParams::new(26.0, 0.58);
                    if let Some(layer) = state.highlights.get_mut(&HighlightLayer::Item) {
                        layer.set_params(p);
                    }
                }
                for &(opt, label) in &[
                    (MotionPhysics::Snappy, "Snappy"),
                    (MotionPhysics::Gentle, "Gentle"),
                    (MotionPhysics::Bouncy, "Bouncy"),
                    (MotionPhysics::Default, "Default"),
                    (MotionPhysics::Off, "Off (Instant)"),
                ] {
                    if ui.selectable_label(current_motion == opt, label).clicked() {
                        if let Some(layer) = state.highlights.get_mut(&HighlightLayer::Item) {
                            layer.set_motion(opt);
                        }
                    }
                }
            });
        });
    });

    ui.add_space(6.0);

    // Track targets for both Outer Section and Single Inner Active Element
    let mut outer_target_rect: Option<Rect> = None;
    let mut outer_target_rounding: Rounding = Rounding::same(6.0);

    let mut inner_target_rect: Option<Rect> = None;
    let mut inner_target_rounding: Rounding = Rounding::same(4.0);

    // Main 2-column layout: Left = Keyboard, Right = Quick Actions
    ui.horizontal(|ui| {
        // ─── Section 1: Keyboard ─────────────────────────────────────────
        let kb_is_active_section = active_section == SectionId::Keyboard;

        let kb_frame_resp = egui::Frame::group(ui.style())
            .stroke(Stroke::new(1.0, palette.surface0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Section 1 • Staggered Keyboard").strong());
                        if kb_is_active_section {
                            ui.colored_label(palette.info, "(ACTIVE)");
                        }
                    });
                    ui.add_space(4.0);

                    let all_rows: &[&[KeySpec]] = &[KB_ROW_0, KB_ROW_1, KB_ROW_2, KB_ROW_3, KB_ROW_4];
                    let key_h = 34.0;

                    for row in all_rows {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
                            for spec in *row {
                                let key_resp = FocusRegion::show(ui, &mut state.kb_nav, &spec.id, |ui, _focused| {
                                    let size = egui::vec2(spec.width, key_h);
                                    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

                                    let is_mod = spec.id.len() > 1 && spec.id != "10";
                                    let bg = if is_mod {
                                        palette.crust
                                    } else {
                                        palette.surface0
                                    };

                                    let stroke = Stroke::new(1.0, palette.surface1);
                                    ui.painter().rect(rect, 4.0, bg, stroke);

                                    let text_color = if is_mod {
                                        palette.subtext0
                                    } else {
                                        palette.text
                                    };

                                    let font_size = if spec.label.len() > 3 { 10.5 } else { 12.5 };
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        spec.label,
                                        egui::FontId::proportional(font_size),
                                        text_color,
                                    );
                                    rect
                                });

                                let rect = key_resp.inner;

                                // Mouse hover sets active section to Keyboard
                                if key_resp.is_hovered && pointer_moved {
                                    state.section_nav.set_focus(Some(SectionId::Keyboard));
                                    state.kb_nav.set_focus(Some(spec.id));
                                }

                                // Primary Click (Left Click)
                                if key_resp.is_clicked {
                                    state.section_nav.set_focus(Some(SectionId::Keyboard));
                                    state.kb_nav.set_focus(Some(spec.id));
                                    let desc = apply_key_primary(spec.id, &mut state.typed_display, &mut state.redo_buffer);
                                    let msg = format!("Mouse Left-Click (Primary): {}", desc);
                                    state.event_log.push(msg);
                                    if state.event_log.len() > 25 {
                                        state.event_log.remove(0);
                                    }
                                }

                                // Secondary Click (Right Click)
                                if key_resp.is_secondary_clicked {
                                    state.section_nav.set_focus(Some(SectionId::Keyboard));
                                    state.kb_nav.set_focus(Some(spec.id));
                                    let desc = apply_key_secondary(spec.id, &mut state.typed_display, &mut state.redo_buffer);
                                    let msg = format!("Mouse Right-Click (Secondary): {}", desc);
                                    state.event_log.push(msg);
                                    if state.event_log.len() > 25 {
                                        state.event_log.remove(0);
                                    }
                                }

                                // If this key is focused and Keyboard is active, track for single Item highlight
                                if key_resp.is_focused && kb_is_active_section {
                                    inner_target_rect = Some(rect);
                                    inner_target_rounding = Rounding::same(5.0);
                                }
                            }
                        });
                    }
                });
            });

        if kb_is_active_section {
            outer_target_rect = Some(kb_frame_resp.response.rect);
            outer_target_rounding = Rounding::same(6.0);
        }

        ui.add_space(8.0);

        // ─── Section 2: Quick Actions Deck ───────────────────────────────
        let act_is_active_section = active_section == SectionId::QuickActions;

        let act_frame_resp = egui::Frame::group(ui.style())
            .stroke(Stroke::new(1.0, palette.surface1)) // user instruction maps Quick action card stroke to surface1. Wait, outer section is surface1? Oh, the Quick action SECTION frame isn't explicitly mentioned, let's look at Event Log frame stroke... Event log frame stroke is surface0. I should use surface0 for quick actions frame too to match, or wait. Quick action card stroke is from_gray(60) -> surface1. Quick action section frame is from_gray(45) -> surface0.
            .stroke(Stroke::new(1.0, palette.surface0)) // fixed
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Section 2 • Quick Actions").strong());
                        if act_is_active_section {
                            ui.colored_label(palette.info, "(ACTIVE)");
                        }
                    });
                    ui.add_space(4.0);

                    for action in QUICK_ACTIONS {
                        let act_resp = FocusRegion::show(ui, &mut state.action_nav, action, |ui, _focused| {
                            let size = egui::vec2(190.0, 28.0);
                            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

                            let bg = palette.base;
                            let stroke = Stroke::new(1.0, palette.surface1);
                            ui.painter().rect(rect, 4.0, bg, stroke);

                            ui.painter().text(
                                rect.left_center() + egui::vec2(10.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                *action,
                                egui::FontId::proportional(12.0),
                                palette.text,
                            );
                            rect
                        });

                        let rect = act_resp.inner;

                        // Mouse hover sets active section to QuickActions
                        if act_resp.is_hovered && pointer_moved {
                            state.section_nav.set_focus(Some(SectionId::QuickActions));
                            state.action_nav.set_focus(Some(action));
                        }

                        // Primary Click (Left Click)
                        if act_resp.is_clicked {
                            state.redo_buffer.clear();
                            state.section_nav.set_focus(Some(SectionId::QuickActions));
                            state.action_nav.set_focus(Some(action));
                            match *action {
                                "Phrase: Hello" => state.typed_display.push_str("Hello "),
                                "Phrase: Rust" => state.typed_display.push_str("Rust "),
                                "Phrase: Vim-Nav" => state.typed_display.push_str("Vim-Nav "),
                                "Action: Clear" => state.typed_display.clear(),
                                "Action: Space" => state.typed_display.push(' '),
                                "Action: Newline" => state.typed_display.push('\n'),
                                _ => {}
                            }
                            let msg = format!("Mouse Left-Click (Primary): Executed '{}'", action);
                            state.event_log.push(msg);
                            if state.event_log.len() > 25 {
                                state.event_log.remove(0);
                            }
                        }

                        // Secondary Click (Right Click)
                        if act_resp.is_secondary_clicked {
                            state.redo_buffer.clear();
                            state.section_nav.set_focus(Some(SectionId::QuickActions));
                            state.action_nav.set_focus(Some(action));
                            let wrapped = format!("[{}] ", action);
                            state.typed_display.push_str(&wrapped);
                            let msg = format!("Mouse Right-Click (Secondary): Inserted wrapped '{}'", action);
                            state.event_log.push(msg);
                            if state.event_log.len() > 25 {
                                state.event_log.remove(0);
                            }
                        }

                        // If this action is focused and QuickActions is active, track for single Item highlight
                        if act_resp.is_focused && act_is_active_section {
                            inner_target_rect = Some(rect);
                            inner_target_rounding = Rounding::same(4.0);
                        }

                        ui.add_space(2.0);
                    }
                });
            });

        if act_is_active_section {
            outer_target_rect = Some(act_frame_resp.response.rect);
            outer_target_rounding = Rounding::same(6.0);
        }
    });

    ui.add_space(8.0);

    // ─── Section 3: Event Log & Inspector ────────────────────────────────────
    let log_is_active_section = active_section == SectionId::EventLog;

    let log_frame_resp = egui::Frame::group(ui.style())
        .stroke(Stroke::new(1.0, palette.surface0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Section 3 • Navigation & Action Event Log").strong());
                    if log_is_active_section {
                        ui.colored_label(palette.info, "(ACTIVE)");
                    }
                });
                egui::ScrollArea::vertical()
                    .max_height(75.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in &state.event_log {
                            ui.monospace(entry);
                        }
                    });
            });
        });

    let log_interact = ui.interact(
        log_frame_resp.response.rect,
        log_frame_resp.response.id.with("log_interact"),
        egui::Sense::click(),
    );

    if log_interact.hovered() && pointer_moved {
        state.section_nav.set_focus(Some(SectionId::EventLog));
    }

    if log_interact.clicked() {
        state.section_nav.set_focus(Some(SectionId::EventLog));
        state.event_log.push("Mouse Left-Click (Primary): Focused Event Log".to_string());
        if state.event_log.len() > 25 {
            state.event_log.remove(0);
        }
    }

    if log_interact.secondary_clicked() {
        state.section_nav.set_focus(Some(SectionId::EventLog));
        state.event_log.clear();
        state.event_log.push("Mouse Right-Click (Secondary): Cleared Event Log".to_string());
    }

    if log_is_active_section {
        outer_target_rect = Some(log_frame_resp.response.rect);
        outer_target_rounding = Rounding::same(6.0);
    }

    // ─── 2-Tier Spring Physics (Outer Container + Single Active Element) ────
    if let Some(target_rect) = outer_target_rect {
        state.highlights.set_target_with_corner_rounding(&HighlightLayer::Section, target_rect, outer_target_rounding);
    }

    if let Some(target_rect) = inner_target_rect {
        state.highlights.set_target_with_corner_rounding(&HighlightLayer::Item, target_rect, inner_target_rounding);
    }

    let dt = ui.input(|i| i.stable_dt).min(0.05);
    state.highlights.update(dt);

    if !state.highlights.is_settled() {
        ui.ctx().request_repaint();
    }

    // 1. Paint Outer Container Highlight (glides between section panels)
    state.highlights.paint_layer(&HighlightLayer::Section, ui.painter());

    // 2. Paint Single Inner Active Element Highlight (glides seamlessly between keys and actions)
    if active_section != SectionId::EventLog {
        state.highlights.paint_layer(&HighlightLayer::Item, ui.painter());
    }

    ui.add_space(4.0);
    ui.label("Controls: Ctrl+HJKL / Mouse = Move Section  •  HJKL / Mouse = Move Item  •  F = Primary  •  D = Secondary  •  Q/Back = Undo");
}
