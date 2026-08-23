# egui-nav-stack

App-owned back-stack screen navigation for [egui](https://github.com/emilk/egui), modeled on the philosophy of Android Navigation 3.

## What It Does

Provides `NavStack<K>` — a generic back stack where `K` is your screen/route enum. The stack is a plain list your app owns and passes in. Navigation actions (push, pop, replace) are returned from the rendering step and applied afterwards, avoiding borrow-checker conflicts.

Features:
- **Generic screen keys**: Works with any `K: Clone + PartialEq`
- **Undo/Redo history**: Built-in forward stack for back/forward navigation
- **Mouse thumb buttons**: Automatic `Extra1` (back) / `Extra2` (forward) support
- **Spring-animated transitions**: Slide, shrink, fade, and overshoot effects (optional feature)
- **Safe immediate-mode mutation**: Read-then-mutate pattern, no simultaneous borrows

## Quick Start

```rust
use egui_nav_stack::{NavAction, NavDisplay, NavStack};

#[derive(Clone, PartialEq)]
enum Screen { Home, Profile, Settings }

// In your app state:
let mut stack = NavStack::new(Screen::Home);

// In your update loop:
NavDisplay::new(&stack)
    .show(ui, |screen, ui| {
        match screen {
            Screen::Home => {
                ui.heading("Home");
                if ui.button("Go to Profile").clicked() {
                    Some(NavAction::Push(Screen::Profile))
                } else {
                    None
                }
            }
            Screen::Profile => {
                ui.heading("Profile");
                if ui.button("Back").clicked() {
                    Some(NavAction::Pop)
                } else {
                    None
                }
            }
            Screen::Settings => {
                ui.heading("Settings");
                None
            }
        }
    })
    .apply_to(&mut stack);
```

## API

### `NavStack<K>`

The back stack:

| Method | Description |
|---|---|
| `NavStack::new(root)` | Create with a root screen |
| `.push(key)` | Push a new screen (clears forward history) |
| `.pop()` → `Option<K>` | Pop the top screen (saves to forward history) |
| `.pop_to(key)` | Pop to a specific screen |
| `.pop_to_root()` | Pop all screens back to root |
| `.replace_top(key)` | Replace the top screen (clears forward history) |
| `.top()` → `&K` | Current top screen |
| `.entries()` → `&[K]` | All screens in the stack |
| `.len()` → `usize` | Number of screens |
| `.can_go_back()` → `bool` | True if stack has more than one entry |
| `.can_go_forward()` → `bool` | True if forward history is available |
| `.go_back()` | Navigate back (pops to forward history) |
| `.go_forward()` | Navigate forward (pushes from forward history) |
| `.forward_entries()` → `&[K]` | The forward (redo) history |

### `NavAction<K>`

Returned from screen rendering to request navigation:

```rust
NavAction::Push(screen)      // Push a new screen
NavAction::Pop               // Go back one screen
NavAction::PopTo(screen)     // Pop to a specific screen
NavAction::PopToRoot         // Go back to root
NavAction::ReplaceTop(screen) // Replace current screen
NavAction::Forward           // Go forward (redo)
```

### `NavDisplay`

Immediate-mode rendering widget:

```rust
NavDisplay::new(&stack)
    .mouse_nav(true)          // Enable mouse thumb button navigation (default: true)
    .show(ui, |screen, ui| {
        // Render screen content, return Option<NavAction<K>>
        None
    })
    .apply_to(&mut stack);    // Apply the returned action to the stack
```

### Animated Transitions (Optional)

Enable the `animated-transitions` feature for spring-animated screen transitions:

```toml
[dependencies]
egui-nav-stack = { path = "...", features = ["animated-transitions"] }
```

```rust
use egui_nav_stack::{NavTransition, SlideDirection};

// In your app state:
let mut transition = NavTransition::new();

// Push with animation:
stack.push_animated(Screen::Profile, &mut transition);

// Pop with animation (reverse direction):
stack.pop_animated(&mut transition);

// Render with transition effects:
NavDisplay::new(&stack)
    .transition(&mut transition)  // Attach transition state
    .show(ui, |screen, ui| { /* ... */ None })
    .apply_to_animated(&mut stack, &mut transition);
```

The push animation slides the new screen in from the right while the old screen shrinks and fades. The pop animation reverses: the top screen slides out to the right while the screen below grows back with spring overshoot (~110%).

## Running the Example

```bash
cargo run -p egui-nav-stack --example stack_navigation
```
