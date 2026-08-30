# egui-nav-stack

## What It Is

`egui-nav-stack` is an **app-owned back-stack screen navigation system** for `egui`, inspired by Android Navigation 3 and browser undo/redo history. The stack is a plain list the consuming app owns — not state hidden inside a controller.

**Crate path:** `src/egui-widgetkit/egui-nav-stack/`

---

## What It Does

- **Back-stack navigation**: Push screens onto a stack, pop to go back, replace the top screen.
- **Undo/Redo history**: Popping a screen saves it to forward history — `go_forward()` restores it, just like browser back/forward.
- **Safe immediate-mode rendering**: Reads the stack immutably to render, returns a `NavResponse` with an action to apply *after* rendering — no borrow checker conflicts.
- **Mouse button navigation**: Thumb buttons (Extra1/Extra2) automatically trigger back/forward.
- **Spring-animated transitions** (opt-in via `animated-transitions` feature): Physics-driven slide/shrink/fade/overshoot screen transitions.
- **Zero global state**: Stack and transition are plain structs owned by your app.

---

## How To Use It

### Add the Dependency

```toml
[dependencies]
egui-nav-stack = { path = "../egui-nav-stack" }

# For animated transitions:
egui-nav-stack = { path = "../egui-nav-stack", features = ["animated-transitions"] }
```

---

### Basic Navigation

```rust
use egui_nav_stack::{NavAction, NavDisplay, NavStack};

#[derive(Clone, PartialEq)]
enum Screen { Home, Profile, Settings }

// Store in your app state
let mut stack = NavStack::new(Screen::Home);

// In your update loop — render the active screen
NavDisplay::new(&stack)
    .show(ui, |screen, ui| {
        match screen {
            Screen::Home => {
                ui.heading("Home");
                if ui.button("Go to Profile").clicked() {
                    return Some(NavAction::Push(Screen::Profile));
                }
                None
            }
            Screen::Profile => {
                ui.heading("Profile");
                if ui.button("Settings").clicked() {
                    return Some(NavAction::Push(Screen::Settings));
                }
                if ui.button("Back").clicked() {
                    return Some(NavAction::Pop);
                }
                None
            }
            Screen::Settings => {
                ui.heading("Settings");
                if ui.button("Back").clicked() {
                    return Some(NavAction::Pop);
                }
                None
            }
        }
    })
    .apply_to(&mut stack);
```

---

### Navigation Actions

```rust
pub enum NavAction<K> {
    Push(K),         // Push new screen
    Pop,             // Go back (saves to forward history)
    Forward,         // Redo / go forward
    PopToRoot,       // Pop all screens back to root
    ReplaceTop(K),   // Replace current screen
}
```

---

### Stack Operations

```rust
let mut stack = NavStack::new(Screen::Home);

stack.push(Screen::Profile);                    // Push
stack.push(Screen::Settings);
assert_eq!(stack.len(), 3);

let popped = stack.go_back();                   // Pop (undo)
assert_eq!(popped, Some(Screen::Settings));
assert!(stack.can_go_forward());

let restored = stack.go_forward();              // Redo
assert_eq!(restored, Some(Screen::Settings));

stack.pop_to_root();                            // Pop to root
assert_eq!(stack.top(), Some(&Screen::Home));

stack.replace_top(Screen::Profile);             // Replace top

stack.clear();                                  // Clear everything
```

**Query methods:**

```rust
stack.top()              // Option<&K> — current active screen
stack.len()              // number of screens on stack
stack.is_empty()         // true if empty
stack.can_pop()          // true if len > 1
stack.can_go_back()      // alias for can_pop()
stack.can_go_forward()   // true if forward history exists
stack.entries()          // &[K] — all active entries
stack.forward_entries()  // &[K] — forward history
```

---

### Animated Transitions

Enable the `animated-transitions` feature for spring-physics-driven slide transitions.

```rust
use egui_nav_stack::{
    NavAction, NavDisplay, NavStack, NavTransition,
    SlideDirection, MotionPhysics,
};

// Store both in your app state
let mut stack = NavStack::new(Screen::Home);
let mut transition = NavTransition::new()
    .with_direction(SlideDirection::FromRight)
    .with_physics(MotionPhysics::Snappy)
    .with_shrink(0.10)       // background shrinks to 90%
    .with_overshoot(0.40);   // pop overexpansion 40%

// Render with transitions
NavDisplay::new(&stack)
    .transition(&mut transition)
    .show(ui, |screen, ui| {
        // ... render screen ...
        None
    })
    .apply_to_animated(&mut stack, &mut transition);
```

**Slide directions:**

```rust
pub enum SlideDirection {
    FromRight,    // default — iOS style
    FromLeft,
    FromTop,
    FromBottom,
}
```

**How transitions work:**

- **Push**: Incoming screen slides in from the edge. Background screen shrinks and fades out.
- **Pop**: Outgoing screen slides out toward the edge. Background screen expands back with spring overshoot.
- Spring physics naturally produces overshoot oscillation for a bouncy, physical feel.
- Transitions are settled when `transition.is_settled()` returns true.

---

### NavDisplay Builder

```rust
NavDisplay::new(&stack)
    .mouse_nav(true)                    // thumb button back/forward (default: true)
    .empty_fallback(|ui| {              // shown when stack is empty
        ui.label("No screens");
    })
    .transition(&mut transition)        // attach animated transitions
    .show(ui, |screen, ui| { ... })
    .apply_to(&mut stack);              // or .apply_to_animated(&mut stack, &mut transition)
```

---

### API Reference

#### `NavStack<K>`

| Method | Description |
| :--- | :--- |
| `new(root)` | Create with root screen |
| `empty()` | Create empty stack |
| `push(key)` | Push screen (clears forward history) |
| `pop() -> Option<K>` | Pop to forward history |
| `go_back() -> Option<K>` | Alias for pop |
| `go_forward() -> Option<K>` | Restore from forward history |
| `pop_to_root() -> Vec<K>` | Pop all to root |
| `replace_top(key) -> Option<K>` | Replace top screen |
| `apply(action)` | Dispatch NavAction |
| `top() -> Option<&K>` | Current active screen |
| `len()` / `is_empty()` | Stack size queries |
| `can_pop()` / `can_go_forward()` | Navigation availability |

#### `NavTransition<K>` (feature: `animated-transitions`)

| Method | Description |
| :--- | :--- |
| `new()` | Create with defaults |
| `.with_direction(SlideDirection)` | Set slide direction |
| `.with_physics(MotionPhysics)` | Set spring physics |
| `.with_shrink(f32)` | Background shrink factor (0.0–0.5) |
| `.with_overshoot(f32)` | Pop overexpansion factor (0.0–1.0) |
| `trigger_push(out, in)` | Manually trigger push transition |
| `trigger_pop(out, in)` | Manually trigger pop transition |
| `snap()` | Instantly stop animation |
| `is_settled() -> bool` | True when idle |
