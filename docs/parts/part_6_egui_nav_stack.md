# Part 6 — `egui-nav-stack` (App-Owned Screen Back-Stack & Undo/Redo Navigation)

## Overview
`egui-nav-stack` is an app-owned navigation stack and destination system for `egui`, combining the architectural philosophy of Android Navigation 3 with **browser-style undo/redo history** and hardware mouse thumb buttons navigation.

Rather than hiding navigation state inside a monolithic controller or router object, **the back stack and forward redo stack are plain lists the application owns (`NavStack<K>`)**. The display widget (`NavDisplay<K>`) reads the stack immutably each frame, automatically listens for hardware mouse thumb button clicks (`PointerButton::Extra1` for Back, `PointerButton::Extra2` for Forward), and returns navigation action requests (`NavAction<K>`), completely eliminating borrow-checker conflicts in immediate mode.

---

## What It Does

### 1. App-Owned Back Stack with Undo/Redo History (`NavStack<K>`)
- **State Ownership (`CODING_RULES §2`)**: The consuming application stores `NavStack<K>` as a plain struct field (e.g., inside `TestAppState`) and passes it by `&` / `&mut` reference.
- **Type Parameter `K`**: Application-defined destination enum holding screen identities and typed payload arguments.
- **Undo / Redo History**:
  - `push(key: K)` / `push_animated(key, &mut transition)` — Pushes a new destination and clears forward redo history.
  - `pop()` / `go_back()` / `pop_animated(&mut transition)` — Pops the active top destination and moves it onto the forward history stack (Undo navigation).
  - `go_forward()` / `go_forward_animated(&mut transition)` — Restores the next screen from the forward history stack (Redo navigation).
  - `can_go_back() -> bool` (`len() > 1`) & `can_go_forward() -> bool` (`!forward_entries.is_empty()`).
  - `forward_entries() -> &[K]` & `forward_len() -> usize`.
  - `pop_to(predicate)` / `pop_to_root()` — Pops destinations while preserving them into forward history in order.
  - `replace_top(key: K)` & `set_stack(entries)` — Replaces destinations and clears forward history.
  - `apply(action)` / `apply_animated(action, &mut transition)` — Applies navigation actions cleanly.
- **Inspection & Query**:
  - `top() -> Option<&K>` / `top_mut() -> Option<&mut K>`
  - `len()`, `is_empty()`, `can_pop()` (`len() > 1`)
  - `iter() -> impl Iterator<Item = &K>` and `entries() -> &[K]`

---

### 2. Immediate-Mode Display & Mouse Thumb Buttons (`NavDisplay<'a, K>`)
- **Hardware Mouse Thumb Buttons**:
  - Automatically listens for **Mouse Button 4 (`PointerButton::Extra1` / Lower Thumb)** to navigate **Back**.
  - Automatically listens for **Mouse Button 5 (`PointerButton::Extra2` / Upper Thumb)** to navigate **Forward**.
  - Enabled by default via `.mouse_nav(true)` (can be disabled via `.mouse_nav(false)`).
- **Safe Mutation Pattern (`CODING_RULES §3`)**:
  - `.show()` reads `&NavStack<K>` immutably during screen drawing.
  - Returns `NavResponse<K>` holding an optional `NavAction<K>` request (from UI button clicks or mouse thumb buttons).
  - Mutations are applied after `.show()` returns (via `.apply_to(&mut stack)` or `.apply_to_animated(&mut stack, &mut transition)`), preventing borrow conflicts.
- **Empty Fallback**: Renders nothing by default when empty (`ui.allocate_response(Vec2::ZERO, Sense::hover())`), or renders custom `.empty_fallback(...)` if provided.

---

### 3. Data Passing Patterns

1. **Forward Data Passing (Args in the Destination Key)**:
   ```rust
   #[derive(Clone, Debug, PartialEq)]
   enum Screen {
       Dashboard,
       ItemDetail { id: usize, title: String, stars: u32 },
   }
   ```
   Arguments are carried directly in the screen enum variant and unpacked inside the render closure.

2. **Backward Data Passing (Hoisted App State with `.take()`)**:
   - Secondary screen (e.g. `Screen::Settings`) writes choices to hoisted state (e.g. `state.pending_accent = Some(color)`).
   - Primary screen (e.g. `Screen::Dashboard`) reads and applies it via `state.pending_accent.take()` when focus returns.

---

### 4. Spring-Animated Transitions (`NavTransition<K>`)
Available when the `animated-transitions` feature is enabled (depends on `egui-spring` / `spring-core`).

- **Direct Action Notification**: Transitions are triggered explicitly at the moment `push_animated` / `pop_animated` / `go_forward_animated` is invoked.
- **Dual-Layer Rendering**:
  - **On Push / Forward**:
    - **Background Screen (Below)**: Simultaneously shrinks ($1.0 \to 0.90$) towards the center **AND** fades its opacity ($1.0 \to 0.0$) with inputs disabled (`ui.add_enabled_ui(false, ...)`).
    - **Incoming Screen (Foreground)**: Simultaneously slides in from the user-selected direction directly over the fading background.
  - **On Pop / Back (Opposite Animation)**:
    - **Outgoing Screen (Foreground)**: Slides to exit towards the **same side it entered from** while fading out.
    - **Incoming Screen (Background)**: Expands from behind with motion physics, smoothly **overexpanding up to $\approx 110\%$** on the peak spring overshoot before settling cleanly at $100\%$, while fading in.
- **Directional Sliding (`SlideDirection`)**:
  - `SlideDirection::FromRight` (default)
  - `SlideDirection::FromLeft`
  - `SlideDirection::FromTop`
  - `SlideDirection::FromBottom`
- **Motion Physics Modes (`MotionPhysics`)**:
  - `Custom(SpringParams::new(26.0, 0.58))` (signature preset by default)
  - `Snappy` ($\omega_0 = 32.0, \zeta = 0.85$)
  - `Bouncy` ($\omega_0 = 20.0, \zeta = 0.50$)
  - `OpenRGB` ($\omega_0 = 22.0, \zeta = 0.65$)
  - `Gentle` ($\omega_0 = 18.0, \zeta = 0.90$)
  - `Off` (instant screen switch with zero animation delay)

---

## Code Examples

### Example: Navigation Stack with Undo/Redo & Mouse Thumb Buttons
```rust
use egui_nav_stack::{MotionPhysics, NavAction, NavDisplay, NavStack, NavTransition, SlideDirection};

#[derive(Clone, Debug, PartialEq)]
enum AppScreen {
    Home,
    Catalog,
    Detail(u32),
}

// In app state:
let mut stack = NavStack::new(AppScreen::Home);
let mut transition = NavTransition::new()
    .with_direction(SlideDirection::FromRight)
    .with_physics(MotionPhysics::Default);

// In update loop:
NavDisplay::new(&stack)
    .transition(&mut transition)
    .mouse_nav(true) // Automatically listens for Mouse Button 4 (Back) & 5 (Forward)
    .show(ui, |screen, ui| {
        match screen {
            AppScreen::Home => {
                ui.heading("Home");
                if ui.button("Catalog ➔").clicked() {
                    Some(NavAction::Push(AppScreen::Catalog))
                } else {
                    None
                }
            }
            AppScreen::Catalog => {
                ui.heading("Catalog");
                if ui.button("Item #42").clicked() {
                    Some(NavAction::Push(AppScreen::Detail(42)))
                } else if ui.button("⬅ Back").clicked() {
                    Some(NavAction::Pop)
                } else {
                    None
                }
            }
            AppScreen::Detail(id) => {
                ui.heading(format!("Item Details: #{}", id));
                if ui.button("⬅ Back").clicked() {
                    Some(NavAction::Pop)
                } else {
                    None
                }
            }
        }
    })
    .apply_to_animated(&mut stack, &mut transition);
```

---

## Settings Page Customization Options

| Setting Key | UI Control | Range / Options | Description |
|---|---|---|---|
| `nav.mouse_nav` | Checkbox / Toggle | `true` / `false` | Enables hardware mouse thumb buttons 4 (Back) and 5 (Forward) navigation |
| `nav.physics` | ComboBox | `Signature Custom`, `Snappy`, `Bouncy`, `OpenRGB`, `Gentle`, `Off` | Motion physics preset controlling transition duration, stiffness, and overshoot |
| `nav.slide_direction` | ComboBox | `From Right`, `From Left`, `From Top`, `From Bottom` | Directional entry and exit axis for sliding screens |
| `nav.shrink_factor` | Slider | `0.0 ..= 0.30` | Shrink intensity factor applied to background screens during transitions |
| `nav.overshoot_factor` | Slider | `0.0 ..= 1.00` | Pop overexpansion factor scaling spring overshoot up to ~110% |
| `nav.show_breadcrumbs` | Checkbox / Toggle | `true` / `false` | Displays clickable breadcrumb ancestor trails above the main screen viewport |
