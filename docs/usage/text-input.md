# TextInput

## What It Is

`TextInput` is a **spring-animated text input field** with a fluid 4-corner cursor simulation, typing swipe reveals, deletion wipe dissolutions, selection drag-and-drop with 2D spring flight docking, and integrated Vim modal text editing.

**Module:** `egui-widgets` → `input`

---

## What It Does

- **Spring cursor (`SpringCursor`)**: A fluid 4-corner animated cursor that morphs from a parent highlight into a thin caret when entering the field, and fades out on leaving.
- **Typing swipe animation**: New characters slide in from the right with a spring-driven swipe.
- **Deletion wipe**: Deleted characters fade and dissolve with a spring-driven wipe effect.
- **Drag-and-drop selection**: Selected text can be dragged out and dropped at a new position with 2D spring flight animation.
- **Vim buffer integration**: Full modal Vim editing (Normal/Insert/Visual/Replace modes) via `VimBufferState`.
- **Mode indicator**: Optional colored stripe showing the current Vim mode (Normal=blue, Insert=green, Visual=purple).
- **Glow ring**: Animated focus glow ring around the field border.
- **Icon and clear button**: Optional leading icon and trailing clear (×) button.
- **Password mode**: Mask characters with bullets.
- **Text alignment**: Left, Center, or Right.

---

## How To Use It

### Basic Usage

```rust
use egui_widgets::TextInput;

let mut query = String::new();

TextInput::new(&mut query)
    .placeholder("Search...")
    .show(ui);
```

### With Icon and Clear Button

```rust
TextInput::new(&mut query)
    .placeholder("Search packages...")
    .icon("🔍")
    .clear_button(true)
    .show(ui);
```

### Password Field

```rust
let mut password = String::new();

TextInput::new(&mut password)
    .placeholder("Enter password")
    .password(true)
    .show(ui);
```

### Text Alignment

```rust
TextInput::new(&mut text)
    .align_left()       // default
    .show(ui);

TextInput::new(&mut text)
    .align_center()
    .show(ui);

TextInput::new(&mut text)
    .align_right()
    .show(ui);
```

### With Vim Buffer Integration

```rust
use egui_vim_nav::VimBufferState;

// Store in app state
let mut vim_buffer = VimBufferState::new("");

TextInput::new(&mut text)
    .vim_buffer(&mut vim_buffer)
    .mode_indicator(true)        // show vim mode stripe
    .editing(is_in_text_mode)    // true when modal editing is active
    .show(ui);
```

### Keyboard Navigation and Focus Morphing

```rust
TextInput::new(&mut text)
    .focused(is_focused)
    .editing(is_editing)
    // Cursor morphs from the parent highlight rect into the text caret
    .spawn_origin(parent_highlight_rect, 6.0)
    .show(ui);
```

### Custom Styling

```rust
TextInput::new(&mut text)
    .fill(Color32::from_rgb(25, 28, 38))
    .stroke(Stroke::new(1.0, Color32::from_rgb(50, 55, 70)))
    .focus_stroke(Stroke::new(2.0, accent_color))
    .text_color(Color32::WHITE)
    .rounding(8.0)
    .padding(Vec2::new(12.0, 8.0))
    .width(300.0)
    .height(36.0)
    .glow_ring(true)
    .show(ui);
```

### With Theme Palette

```rust
TextInput::new(&mut text)
    .palette(&palette)
    .placeholder("Type here...")
    .show(ui);
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `TextInput::new(&mut String)` | Create text input |
| `.placeholder(str)` | Placeholder text |
| `.icon(text)` | Leading icon |
| `.clear_button(bool)` | Show clear (×) button |
| `.password(bool)` | Mask characters |
| `.align_left()` / `.align_center()` / `.align_right()` / `.align(TextAlign)` | Text alignment |
| `.vim_buffer(&mut VimBufferState)` | Attach Vim buffer |
| `.mode_indicator(bool)` | Show Vim mode stripe |
| `.focused(bool)` / `.editing(bool)` | Focus and editing state |
| `.spawn_origin(rect, rounding)` | Cursor morph origin |
| `.fill(c)` / `.stroke(s)` / `.focus_stroke(s)` | Colors |
| `.text_color(c)` | Text color |
| `.rounding(r)` / `.padding(v)` | Geometry |
| `.width(f32)` / `.min_width(f32)` / `.desired_width(f32)` / `.height(f32)` | Sizing |
| `.glow_ring(bool)` | Focus glow effect |
| `.spring_params(p)` / `.motion(bool)` | Physics control |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut InputState)` | Explicit state |
| `.id_source(id)` | Custom ID |
| `.show(ui) -> Response` | Render |
