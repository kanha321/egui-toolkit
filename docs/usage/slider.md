# Slider

## What It Is

`Slider` is a **spring-animated numeric slider** with a scaling knob thumb, active track highlight, prominent value badge, and inline modal Vim direct editing. Supports any numeric type via the `Numeric` trait.

**Module:** `egui-widgets` → `slider`

---

## What It Does

- **Spring knob**: The thumb scales up on hover/drag and bounces on click.
- **Active track**: The filled portion of the track glides smoothly to match the value.
- **Value badge**: Current value displayed as a badge near the knob with smooth width animation.
- **Inline editing**: Press Enter/F on a focused slider to type a value directly, using the Vim buffer for text editing.
- **Layouts**: Inline (label + slider on one line) or Stacked (label above slider).
- **Step snapping**: Optional step quantization.
- **Prefix/suffix**: Display units like `%`, `Hz`, `dB` alongside the value.

---

## How To Use It

### Basic Usage

```rust
use egui_widgets::Slider;

let mut volume = 75.0f32;

Slider::new(&mut volume, 0.0..=100.0)
    .label("Volume")
    .show(ui);
```

### With Units

```rust
Slider::new(&mut frequency, 20.0..=20000.0)
    .label("Frequency")
    .suffix(" Hz")
    .show(ui);

Slider::new(&mut opacity, 0.0..=1.0)
    .label("Opacity")
    .prefix("×")
    .show_value(true)
    .show(ui);
```

### Step Snapping

```rust
Slider::new(&mut count, 0..=100)
    .label("Items")
    .step(5.0)   // snap to multiples of 5
    .show(ui);
```

### Layouts

```rust
// Inline: [Label] [----●----] [value]  (default)
Slider::new(&mut val, 0.0..=1.0).inline().show(ui);

// Stacked: Label above slider
Slider::new(&mut val, 0.0..=1.0).stacked().show(ui);
```

### Custom Styling

```rust
Slider::new(&mut val, 0.0..=100.0)
    .track_height(4.0)
    .knob_radius(8.0)
    .track_active(Color32::from_rgb(137, 180, 250))
    .track_inactive(Color32::from_rgb(60, 63, 80))
    .knob_fill(Color32::WHITE)
    .knob_stroke(Stroke::new(1.0, Color32::GRAY))
    .show(ui);
```

### Keyboard Navigation

```rust
Slider::new(&mut val, 0.0..=100.0)
    .label("Brightness")
    .focused(is_focused)   // highlight when focused
    .show(ui);

// When focused and Enter/F is pressed, the slider enters inline
// text editing mode using the Vim buffer system
```

### Physics Tuning

```rust
Slider::new(&mut val, 0.0..=1.0)
    .spring_params(SpringParams::snappy())  // knob/track spring character
    .click_momentum(150.0)                  // velocity boost on click
    .knob_scale_mult(1.3)                   // hover scale multiplier
    .motion(true)                           // enable/disable animation
    .show(ui);
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `Slider::new(&mut T, range)` | Create slider |
| `.label(text)` | Label text |
| `.show_value(bool)` | Show value badge |
| `.prefix(str)` / `.suffix(str)` | Value prefix/suffix |
| `.step(f64)` | Step size for snapping |
| `.inline()` / `.stacked()` / `.layout(SliderLayout)` | Layout mode |
| `.track_height(f32)` / `.knob_radius(f32)` | Geometry |
| `.track_active(c)` / `.track_inactive(c)` | Track colors |
| `.knob_fill(c)` / `.knob_stroke(s)` | Knob styling |
| `.spring_params(p)` / `.click_momentum(f32)` / `.knob_scale_mult(f32)` | Physics |
| `.motion(bool)` | Enable/disable animation |
| `.focused(bool)` | Keyboard focus state |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut s)` | Explicit `SliderState` |
| `.id_source(id)` | Custom ID |
| `.show(ui) -> Response` | Render |
