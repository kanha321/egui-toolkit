//! Showcase scene exercising `egui-nav-stack`.
//!
//! Demonstrates:
//! - App-owned back-stack and forward undo/redo history (`NavStack<Screen>`).
//! - Hardware mouse thumb buttons navigation (`PointerButton::Extra1` for Back, `PointerButton::Extra2` for Forward).
//! - Spring-animated screen transitions with directional slide, shrink, fade, and pop overshoot.
//! - Forward data passing via destination key parameters (`Screen::ItemDetail { id, title, .. }`).
//! - Backward data passing via hoisted application state (`pending_accent`).
//! - Breadcrumb trail navigation with clickable ancestor jumps (`stack.pop_to(...)`).
//! - Live visual stack inspector panel & transition physics controls.

use egui::{Color32, ComboBox, RichText, Rounding, Stroke, Ui, Vec2};
use egui_layout::{Section, Split};
use egui_nav_stack::{MotionPhysics, NavAction, NavDisplay, NavStack, NavTransition, SlideDirection};
use egui_themes::ThemePalette;
use spring_core::SpringParams;

/// App destinations representing screens and their forward arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Catalog,
    ItemDetail {
        id: usize,
        title: String,
        author: String,
        stars: u32,
    },
    Settings,
}

impl Screen {
    pub fn display_name(&self) -> String {
        match self {
            Screen::Dashboard => "🏠 Dashboard".to_string(),
            Screen::Catalog => "📦 Catalog".to_string(),
            Screen::ItemDetail { id, .. } => format!("🔍 Item #{}", id),
            Screen::Settings => "⚙️ Settings".to_string(),
        }
    }
}

/// App-owned state for the NavStack demo scene.
pub struct NavStackDemoState {
    pub stack: NavStack<Screen>,
    pub transition: NavTransition<Screen>,
    /// Hoisted state for backward data passing (e.g. settings $\to$ dashboard)
    pub accent_color: Option<Color32>,
    pub pending_accent: Option<Color32>,
}

impl Default for NavStackDemoState {
    fn default() -> Self {
        Self {
            stack: NavStack::new(Screen::Dashboard),
            transition: NavTransition::new()
                .with_direction(SlideDirection::FromRight)
                .with_physics(MotionPhysics::Custom(SpringParams::new(26.0, 0.58)))
                .with_shrink(0.10)
                .with_overshoot(0.40),
            accent_color: None,
            pending_accent: None,
        }
    }
}

pub fn show(ui: &mut Ui, state: &mut NavStackDemoState, palette: &ThemePalette) {
    // Backward data passing: apply pending state when returning to Dashboard
    if state.stack.top() == Some(&Screen::Dashboard) {
        if let Some(new_color) = state.pending_accent.take() {
            state.accent_color = Some(new_color);
        }
    }

    let accent = state.accent_color.unwrap_or(palette.info);
    let mut pending = state.pending_accent;
    let mut main_action = None;
    let mut inspector_action = None;
    let mut bar_action = None;
    let mut pop_to_target = None;

    let mut current_physics = state.transition.physics();
    let mut current_dir = state.transition.slide_direction();

    Split::horizontal()
        .spacing(8.0)
        // Main Screen Viewport (Left / Center)
        .add_section(
            Section::fraction(0.68)
                .min_size_2d(300.0, 200.0)
                .card()
                .title("🧭 Screen Viewport")
                .subtitle("Undo/Redo navigation stack (Mouse Button 4 Back / Button 5 Forward)")
                .padding(10.0)
                .content(|ui| {
                    if let Some((target, action)) = render_breadcrumb_bar(ui, &state.stack, palette) {
                        if let Some(t) = target {
                            pop_to_target = Some(t);
                        }
                        if let Some(a) = action {
                            bar_action = Some(a);
                        }
                    }
                    ui.separator();
                    ui.add_space(4.0);

                    // Render active screen with spring transitions and automatic mouse thumb button listening
                    let response = NavDisplay::new(&state.stack)
                        .transition(&mut state.transition)
                        .mouse_nav(true)
                        .show(ui, |screen, ui| {
                            match screen {
                                Screen::Dashboard => render_dashboard(ui, accent),
                                Screen::Catalog => render_catalog(ui),
                                Screen::ItemDetail { id, title, author, stars } => {
                                    render_item_detail(ui, *id, title, author, *stars)
                                }
                                Screen::Settings => render_settings(ui, &mut pending, palette),
                            }
                        });

                    main_action = response.action;
                }),
        )
        // Stack Inspector & Physics Controls (Right)
        .add_section(
            Section::remainder()
                .min_size_2d(200.0, 200.0)
                .card()
                .title("📊 Stack & Transitions")
                .subtitle("Live undo/redo history & motion physics")
                .padding(10.0)
                .content(|ui| {
                    inspector_action = render_inspector(
                        ui,
                        &state.stack,
                        &mut current_physics,
                        &mut current_dir,
                        palette,
                    );
                }),
        )
        .show(ui);

    state.pending_accent = pending;
    if current_physics != state.transition.physics() {
        state.transition.set_physics(current_physics);
    }
    if current_dir != state.transition.slide_direction() {
        state.transition.set_direction(current_dir);
    }

    // Apply all requested actions safely after rendering with direct transition triggering
    if let Some(target) = pop_to_target {
        state.stack.pop_to_animated(|s| s == &target, &mut state.transition);
    }
    if let Some(action) = main_action.or(inspector_action).or(bar_action) {
        state.stack.apply_animated(action, &mut state.transition);
    }
}

fn render_breadcrumb_bar(
    ui: &mut Ui,
    stack: &NavStack<Screen>,
    palette: &ThemePalette,
) -> Option<(Option<Screen>, Option<NavAction<Screen>>)> {
    let mut target_to_pop = None;
    let mut nav_action = None;

    ui.horizontal_wrapped(|ui| {
        // Undo / Redo Toolbar Buttons
        if stack.can_go_back() {
            if ui.button("⬅ Back (Mouse 4)").clicked() {
                nav_action = Some(NavAction::Pop);
            }
        } else {
            ui.add_enabled_ui(false, |ui| {
                let _ = ui.button("⬅ Back (Mouse 4)");
            });
        }

        if stack.can_go_forward() {
            if ui.button("➔ Forward (Mouse 5)").clicked() {
                nav_action = Some(NavAction::Forward);
            }
        } else {
            ui.add_enabled_ui(false, |ui| {
                let _ = ui.button("➔ Forward (Mouse 5)");
            });
        }

        ui.separator();
        ui.label(RichText::new("Trail:").strong());

        let total = stack.len();
        for (i, screen) in stack.iter().enumerate() {
            let is_active = i == total - 1;
            let name = screen.display_name();

            if is_active {
                ui.label(RichText::new(name).strong().color(palette.success));
            } else {
                if ui.link(name).clicked() {
                    target_to_pop = Some(screen.clone());
                }
                ui.label("➔");
            }
        }
    });

    if target_to_pop.is_some() || nav_action.is_some() {
        Some((target_to_pop, nav_action))
    } else {
        None
    }
}

fn render_dashboard(ui: &mut Ui, accent_color: Color32) -> Option<NavAction<Screen>> {
    ui.colored_label(
        accent_color,
        RichText::new("Welcome to the Dashboard").size(18.0).strong(),
    );
    ui.add_space(6.0);
    ui.label("Demonstrating undo/redo navigation with mouse thumb buttons 4 & 5.");
    ui.label(format!("Active accent color: RGB({}, {}, {})", accent_color.r(), accent_color.g(), accent_color.b()));
    ui.add_space(14.0);

    let mut action = None;
    ui.horizontal(|ui| {
        if ui.button("📦 Browse Catalog ➔").clicked() {
            action = Some(NavAction::Push(Screen::Catalog));
        }
        if ui.button("⚙️ App Settings ➔").clicked() {
            action = Some(NavAction::Push(Screen::Settings));
        }
    });

    action
}

fn render_catalog(ui: &mut Ui) -> Option<NavAction<Screen>> {
    ui.heading("📦 Product Catalog");
    ui.label("Select an item to navigate forward with typed payload arguments:");
    ui.add_space(8.0);

    let items = [
        (1, "Egui Layout Engine", "WidgetKit Team", 128),
        (2, "Analytical Spring Solver", "Physics Lab", 256),
        (3, "Vim Navigation Graph", "Input Guild", 64),
        (4, "Navigation 3 Undo/Redo Stack", "Architecture Group", 512),
    ];

    let mut action = None;
    for (id, title, author, stars) in items {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("#{}: {}", id, title)).strong());
                ui.label(format!("by {}", author));
                ui.label(format!("⭐ {}", stars));
                if ui.button("View Details ➔").clicked() {
                    action = Some(NavAction::Push(Screen::ItemDetail {
                        id,
                        title: title.to_string(),
                        author: author.to_string(),
                        stars,
                    }));
                }
            });
        });
        ui.add_space(2.0);
    }

    ui.add_space(10.0);
    if ui.button("⬅ Back to Dashboard").clicked() {
        action = Some(NavAction::Pop);
    }

    action
}

fn render_item_detail(
    ui: &mut Ui,
    id: usize,
    title: &str,
    author: &str,
    stars: u32,
) -> Option<NavAction<Screen>> {
    ui.heading(format!("🔍 Item Details: {}", title));
    ui.add_space(6.0);

    ui.group(|ui| {
        ui.label(format!("Item ID: {}", id));
        ui.label(format!("Author: {}", author));
        ui.label(format!("Popularity: ⭐ {} stars", stars));
        ui.label("Forward data passing: Arguments are carried directly in the Screen key enum.");
    });

    ui.add_space(14.0);
    let mut action = None;
    ui.horizontal(|ui| {
        if ui.button("⬅ Back to Catalog").clicked() {
            action = Some(NavAction::Pop);
        }
        if ui.button("🏠 Pop to Root (Dashboard)").clicked() {
            action = Some(NavAction::PopToRoot);
        }
    });

    action
}

fn render_settings(ui: &mut Ui, pending_accent: &mut Option<Color32>, palette: &ThemePalette) -> Option<NavAction<Screen>> {
    ui.heading("⚙️ Settings (Backward Data Flow Demo)");
    ui.label("Pick a new theme accent color. The choice is saved into hoisted app state and applied when returning to Dashboard:");
    ui.add_space(10.0);

    let colors = [
        ("Catppuccin Blue", palette.swatches.get(0).copied().unwrap_or(palette.info)),
        ("Tokyo Night Cyan", palette.swatches.get(1).copied().unwrap_or(palette.sys_controls)),
        ("Gruvbox Green", palette.swatches.get(2).copied().unwrap_or(palette.success)),
        ("Nord Frost", palette.swatches.get(3).copied().unwrap_or(palette.info_alt)),
        ("Flamingo Pink", palette.swatches.get(4).copied().unwrap_or(palette.accent)),
    ];

    for (name, color) in colors {
        ui.horizontal(|ui| {
            let (rect, _resp) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect(rect, Rounding::same(4.0), color, Stroke::NONE);
            if ui.button(name).clicked() {
                *pending_accent = Some(color);
            }
        });
    }

    if let Some(pending) = *pending_accent {
        ui.add_space(6.0);
        ui.colored_label(
            palette.success,
            format!("Selected pending color: RGB({}, {}, {})", pending.r(), pending.g(), pending.b()),
        );
    }

    ui.add_space(14.0);
    let mut action = None;
    if ui.button("⬅ Return & Apply").clicked() {
        action = Some(NavAction::Pop);
    }

    action
}

fn render_inspector(
    ui: &mut Ui,
    stack: &NavStack<Screen>,
    physics: &mut MotionPhysics,
    direction: &mut SlideDirection,
    palette: &ThemePalette,
) -> Option<NavAction<Screen>> {
    ui.label(format!("Back Depth: {} screen(s)", stack.len()));
    ui.label(format!("Forward Depth: {} screen(s)", stack.forward_len()));
    ui.separator();

    // 1. Back Stack
    ui.label(RichText::new("Active Back Stack:").size(11.0).strong());
    ui.add_space(2.0);

    for (i, screen) in stack.iter().enumerate() {
        let is_top = i == stack.len() - 1;
        let prefix = if is_top { "▶ [ACTIVE] " } else { "  " };
        let text = format!("{}{}: {}", prefix, i, screen.display_name());

        if is_top {
            ui.colored_label(palette.warning, text);
        } else {
            ui.label(text);
        }
    }

    // 2. Forward Redo Stack
    if stack.can_go_forward() {
        ui.add_space(4.0);
        ui.label(RichText::new("Forward Redo Stack:").size(11.0).strong().color(palette.info));
        for (i, screen) in stack.forward_entries().iter().enumerate() {
            ui.label(format!("  ⏩ {}: {}", i, screen.display_name()));
        }
    }

    ui.add_space(8.0);
    ui.separator();
    ui.label(RichText::new("Transition Controls:").strong());

    // Motion Physics selector
    let mut selected_idx = match physics {
        MotionPhysics::Custom(_) => 0,
        MotionPhysics::Snappy => 1,
        MotionPhysics::Bouncy => 2,
        MotionPhysics::OpenRGB => 3,
        MotionPhysics::Gentle => 4,
        MotionPhysics::Off => 5,
        _ => 0,
    };

    let physics_labels = [
        "Signature Custom (26 / 0.58)",
        "Snappy (32 / 0.85)",
        "Bouncy (20 / 0.50)",
        "OpenRGB (22 / 0.65)",
        "Gentle (18 / 0.90)",
        "Off (Instant)",
    ];
    ComboBox::from_label("Physics")
        .selected_text(physics_labels[selected_idx])
        .show_ui(ui, |ui| {
            for (idx, label) in physics_labels.iter().enumerate() {
                if ui.selectable_value(&mut selected_idx, idx, *label).clicked() {
                    *physics = match idx {
                        0 => MotionPhysics::Custom(SpringParams::new(26.0, 0.58)),
                        1 => MotionPhysics::Snappy,
                        2 => MotionPhysics::Bouncy,
                        3 => MotionPhysics::OpenRGB,
                        4 => MotionPhysics::Gentle,
                        _ => MotionPhysics::Off,
                    };
                }
            }
        });

    // Slide Direction selector
    let mut dir_idx = match direction {
        SlideDirection::FromRight => 0,
        SlideDirection::FromLeft => 1,
        SlideDirection::FromTop => 2,
        SlideDirection::FromBottom => 3,
    };

    let dir_labels = ["From Right (➔)", "From Left (⬅)", "From Top (⬆)", "From Bottom (⬇)"];
    ComboBox::from_label("Slide Dir")
        .selected_text(dir_labels[dir_idx])
        .show_ui(ui, |ui| {
            for (idx, label) in dir_labels.iter().enumerate() {
                if ui.selectable_value(&mut dir_idx, idx, *label).clicked() {
                    *direction = match idx {
                        0 => SlideDirection::FromRight,
                        1 => SlideDirection::FromLeft,
                        2 => SlideDirection::FromTop,
                        _ => SlideDirection::FromBottom,
                    };
                }
            }
        });

    ui.add_space(8.0);
    ui.separator();
    ui.label(RichText::new("Quick Actions:").strong());

    let mut action = None;
    ui.horizontal(|ui| {
        if stack.can_go_back() && ui.button("⬅ Back (Mouse 4)").clicked() {
            action = Some(NavAction::Pop);
        }
        if stack.can_go_forward() && ui.button("➔ Forward (Mouse 5)").clicked() {
            action = Some(NavAction::Forward);
        }
    });

    if stack.len() > 1 && ui.button("🏠 Pop to Root").clicked() {
        action = Some(NavAction::PopToRoot);
    }

    ui.add_space(4.0);
    ui.label(RichText::new("💡 Tip: Click mouse thumb buttons (4 & 5) anywhere!").size(10.0).italics());

    action
}
