# egui-widgets

App-agnostic, token-aware, spring-animated UI component library for [egui](https://github.com/emilk/egui).

## Overview

`egui-widgets` acts as a polished, cohesive design-system layer between raw egui primitives and consuming applications. Every component:
- **Flows from `ThemePalette`**: Colors, surfaces, typography, and status roles automatically adapt to the active theme.
- **Harnesses Spring Physics**: Buttons, switches, tabs, sliders, progress bars, and checkboxes use `spring-core` analytical ODE physics for natural micro-interactions.
- **Is Fully Customizable**: Every fill, stroke, radius, padding, spring preset, and icon can be overridden via ergonomic builder methods.
- **Maintains Zero Global State**: Stateless immediate mode with ID memory or explicit caller-owned state structs.

## Components

- **`Button` & `IconButton`**: Primary, Secondary, Ghost, Danger, Outline variants with spring press bounce and hover luminance glide.
- **`Switch`**: Fluid spring-driven toggle switch with elastic arrival and track morphing.
- **`SegmentedTabs`**: Sliding pill tab switcher powered by spring physics.
- **`ProgressBar`**: Smooth spring-catching progress indicator with status variants.
- **`Slider`**: Interactive slider with spring-scaling knob thumb and track highlight.
- **`Checkbox` & `RadioButton`**: Spring checkmark draw-in and radio dot bounce.
- **`Card`**: Styled surface container with optional spring hover lift.
- **`Badge` & `StatusChip`**: Semantic status tags with optional pulsing dots.
- **`TextInput`**: Styled text field with spring focus ring expansion.

## Quick Start

```rust
use egui_widgets::{Button, Switch, ProgressBar, Badge, Card};
use egui_themes::ThemePalette;

// Switch
let mut is_active = true;
Switch::new(&mut is_active)
    .label("High Performance Mode")
    .show(ui);

// Spring-animated Button
if Button::new("Deploy Changes")
    .primary()
    .icon("🚀")
    .show(ui)
    .clicked()
{
    println!("Deployed!");
}
```
