//! Widgets showcase scene displaying all `egui-widgets` components.

use egui::{Rect, Rounding, Stroke, Ui, Vec2};
use egui_spring::{HighlightConfig, HighlightGroup, MotionPhysics, sync_highlight_stroke_color, set_highlight_fill, update_and_paint};
use egui_themes::ThemePalette;
use egui_vim_nav::{
    Direction, FocusGraph, Navigator, Scrolloff, VimAction, VimActionState, VimBufferState, VimKeyHandler, VimMode,
    FocusLevel, ModalTransition, check_modal_transition, hierarchical_move,
};
use egui_widgets::{
    Badge, Button, Card, Checkbox, Dropdown, DropdownOption, DropdownState, InputState,
    ProgressBar, ProgressVariant, RadioButton, SegmentedTabs, Slider, SliderState, Switch,
    TextInput,
};
use spring_core::SpringParams;

/// Cluster deployment region demo enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DemoRegion {
    #[default]
    UsEast,
    UsWest,
    EuCentral,
    EuWest,
    ApSouth,
    ApNortheast,
    SaEast,
    AfSouth,
}

/// Logging level demo enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DemoLogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

/// Widget category tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WidgetsCategory {
    ButtonsAndBadges,
    SwitchesAndToggles,
    SlidersAndInputs,
    SurfacesAndProgress,
    #[default]
    CompositeDashboard,
}

/// Focus node IDs for every interactive element in the CompositeDashboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DashboardWidget {
    SearchInput,
    ScanButton,
    ClearButton,
    SwitchTurbo,
    SwitchVpn,
    SwitchAnalytics,
    SliderBandwidth,
    SliderThermal,
    ProgressBtn25,
    ProgressBtn50,
    ProgressBtn75,
    ProgressBtn100,
    TokenInput,
    DropdownRegion,
    RadioDev,
    RadioStaging,
    RadioProd,
    CheckNotify,
    CheckMirror,
    BtnDeploy,
    BtnVerify,
    BtnPurge,
    BtnHalt,
}

/// Section IDs for Ctrl+HJKL inter-card navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DashSection {
    SearchBar = 0,
    CoreEngine = 1,
    PipelineSync = 2,
    Security = 3,
    ActionDispatcher = 4,
}

/// Two-tier highlight layers: outer section + inner element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DashHighlight {
    /// Outer card/section highlight (glides around cards on Ctrl+HJKL)
    Section,
    /// Inner element highlight (glides between individual widgets on HJKL)
    Item,
}

#[derive(Debug)]
pub struct WidgetsDemoState {
    pub category: WidgetsCategory,
    // Interactive control states
    pub switch_turbo: bool,
    pub switch_analytics: bool,
    pub switch_vpn: bool,
    pub volume_slider: f32,
    pub threshold_slider: f32,
    pub progress_value: f32,
    pub checkbox_newsletter: bool,
    pub checkbox_telemetry: bool,
    pub radio_tier: Option<usize>,
    pub search_term: String,
    pub token_input: String,
    pub search_vim: VimBufferState,
    pub token_vim: VimBufferState,
    pub search_input_state: InputState,
    pub token_input_state: InputState,
    pub bandwidth_slider_state: SliderState,
    pub thermal_slider_state: SliderState,
    pub demo_region: DemoRegion,
    pub demo_log_level: DemoLogLevel,
    pub demo_region_dropdown_state: DropdownState,
    pub demo_log_dropdown_state: DropdownState,
    // Intuitive physics parameters
    pub response_time: f32,    // seconds — how long the animation takes
    pub bounce: f32,           // 0.0–1.0 — overshoot amount (0 = none)
    pub mass: f32,             // weight/inertia (1.0 = normal)
    pub velocity_kick: f32,    // initial impulse on click (0.0 = pure spring)
    // Sandbox test slider value
    pub test_slider_val: f32,
    // Dashboard vim navigation state
    pub dash_graph: FocusGraph<DashboardWidget>,
    pub dash_nav: Navigator<DashboardWidget>,
    pub dash_section_graph: FocusGraph<DashSection>,
    pub dash_section_nav: Navigator<DashSection>,
    pub dash_highlights: HighlightGroup<DashHighlight>,
    pub dash_key_handler: VimKeyHandler,
    pub last_section_widget: [DashboardWidget; 5],
    /// Two-tier focus level: Navigation (HJKL between widgets) vs TextEditing (keys go to VimBuffer)
    pub focus_level: FocusLevel,
    /// Directional scrolloff and spring-damped viewport scrolling manager
    pub dash_scrolloff: Scrolloff<DashboardWidget>,
    // Dynamic size animation demo steps
    pub scan_button_step: usize,
    pub deploy_button_step: usize,
    pub gallery_btn_step: usize,
}

impl Default for WidgetsDemoState {
    fn default() -> Self {
        use DashboardWidget::*;

        // Build the focus graph matching each card's internal layout
        let mut g = FocusGraph::new();

        // 1. Top search bar (horizontal row)
        g.connect_horizontal(SearchInput, ScanButton);
        g.connect_horizontal(ScanButton, ClearButton);

        // 2. Column 1: Core Engine card (vertical stack)
        g.connect_vertical(SwitchTurbo, SwitchVpn);
        g.connect_vertical(SwitchVpn, SwitchAnalytics);
        g.connect_vertical(SwitchAnalytics, SliderBandwidth);
        g.connect_vertical(SliderBandwidth, SliderThermal);

        // 3. Column 1: Pipeline Sync card (preset buttons horizontal strip)
        g.connect_horizontal(ProgressBtn25, ProgressBtn50);
        g.connect_horizontal(ProgressBtn50, ProgressBtn75);
        g.connect_horizontal(ProgressBtn75, ProgressBtn100);

        // 4. Column 2: Security card (token input -> dropdown region -> radios -> notify sandwich + mirror)
        g.connect_vertical(TokenInput, DropdownRegion);
        g.connect_branch_between(
            DropdownRegion,
            Direction::Down,
            &[RadioDev, RadioStaging, RadioProd],
            CheckNotify,
        );
        g.connect_vertical(CheckNotify, CheckMirror);

        // 5. Column 2: Action Dispatcher card (horizontal button strip)
        g.connect_horizontal(BtnDeploy, BtnVerify);
        g.connect_horizontal(BtnVerify, BtnPurge);
        g.connect_horizontal(BtnPurge, BtnHalt);

        // Section graph for Inter-Card navigation (Ctrl+HJKL and Pass 2 boundary fallback)
        let mut sg = FocusGraph::new();
        sg.connect_vertical(DashSection::SearchBar, DashSection::CoreEngine);
        sg.connect_vertical(DashSection::SearchBar, DashSection::Security);
        sg.connect_horizontal(DashSection::CoreEngine, DashSection::Security);
        sg.connect_vertical(DashSection::CoreEngine, DashSection::PipelineSync);
        sg.connect_vertical(DashSection::Security, DashSection::ActionDispatcher);
        sg.connect_horizontal(DashSection::PipelineSync, DashSection::ActionDispatcher);

        // Two-tier highlight group
        let mut highlights = HighlightGroup::new();
        // Outer section highlight (gentle, cushioned glide around cards)
        highlights.add(
            DashHighlight::Section,
            HighlightConfig::new()
                .with_motion(MotionPhysics::Gentle)
                .with_fill(egui::Color32::TRANSPARENT)
                .with_stroke(Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(130, 180, 255, 90)))
                .with_rounding(8.0)
                .with_padding(4.0),
        );
        // Inner element highlight (snappier, tracks focused widget)
        highlights.add(
            DashHighlight::Item,
            HighlightConfig::new()
                .with_motion(MotionPhysics::Custom(SpringParams::new(28.1, 0.57)))
                .with_fill(egui::Color32::TRANSPARENT)
                .with_stroke(Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(130, 130, 255, 240)))
                .with_rounding(6.0)
                .with_padding(3.0),
        );

        Self {
            category: WidgetsCategory::CompositeDashboard,
            switch_turbo: true,
            switch_analytics: false,
            switch_vpn: true,
            volume_slider: 68.0,
            threshold_slider: 42.0,
            progress_value: 0.65,
            checkbox_newsletter: true,
            checkbox_telemetry: false,
            radio_tier: Some(2),
            search_term: String::new(),
            token_input: String::new(),
            search_vim: VimBufferState::new(""),
            token_vim: VimBufferState::new(""),
            search_input_state: InputState::default(),
            token_input_state: InputState::default(),
            bandwidth_slider_state: SliderState::default(),
            thermal_slider_state: SliderState::default(),
            demo_region: DemoRegion::default(),
            demo_log_level: DemoLogLevel::default(),
            demo_region_dropdown_state: DropdownState::default(),
            demo_log_dropdown_state: DropdownState::default(),
            response_time: 0.10,
            bounce: 0.47,
            mass: 5.0,
            velocity_kick: 6.5,
            test_slider_val: 45.0,
            dash_graph: g,
            dash_nav: Navigator::new().with_initial_focus(SwitchTurbo),
            dash_section_graph: sg,
            dash_section_nav: Navigator::new().with_initial_focus(DashSection::CoreEngine),
            dash_highlights: highlights,
            dash_key_handler: VimKeyHandler::new().with_tab(false),
            last_section_widget: [
                SearchInput,
                SwitchTurbo,
                ProgressBtn25,
                TokenInput,
                BtnDeploy,
            ],
            focus_level: FocusLevel::Navigation,
            dash_scrolloff: Scrolloff::new(),
            scan_button_step: 0,
            deploy_button_step: 0,
            gallery_btn_step: 0,
        }
    }
}

impl WidgetsDemoState {
    /// Focuses a widget, updates branch memory, synchronizes section navigation,
    /// and records the last active widget for that section.
    pub fn record_widget_focus(&mut self, id: DashboardWidget) {
        let prev = self.dash_nav.focused().copied();
        self.dash_nav.set_focus_with_graph(Some(id), &self.dash_graph);
        let sec = widget_to_section(id);
        self.dash_section_nav.set_focus(Some(sec));
        self.last_section_widget[sec as usize] = id;
        if Some(id) != prev {
            self.dash_scrolloff.record_nav_event(Some(id), None);
        }
    }
}

/// Maps a DashboardWidget to the DashSection (card) it belongs to.
fn widget_to_section(w: DashboardWidget) -> DashSection {
    use DashboardWidget::*;
    match w {
        SearchInput | ScanButton | ClearButton => DashSection::SearchBar,
        SwitchTurbo | SwitchVpn | SwitchAnalytics | SliderBandwidth | SliderThermal
            => DashSection::CoreEngine,
        ProgressBtn25 | ProgressBtn50 | ProgressBtn75 | ProgressBtn100
            => DashSection::PipelineSync,
        TokenInput | DropdownRegion | RadioDev | RadioStaging | RadioProd | CheckNotify | CheckMirror
            => DashSection::Security,
        BtnDeploy | BtnVerify | BtnPurge | BtnHalt
            => DashSection::ActionDispatcher,
    }
}

/// Determines the appropriate boundary entry widget when navigating into `section` in movement direction `nav_dir`.
fn section_entry_widget(
    section: DashSection,
    nav_dir: Direction,
    last_widget: DashboardWidget,
) -> DashboardWidget {
    use DashboardWidget::*;
    match nav_dir {
        // Horizontal transitions: restore last-focused widget position in target section
        Direction::Left | Direction::Right => last_widget,

        // Vertical transitions: direction-aware physical edge entry
        Direction::Up => match section {
            DashSection::SearchBar => last_widget,
            DashSection::CoreEngine => SliderThermal, // moving UP -> enter via bottom-most element
            DashSection::PipelineSync => last_widget, // horizontal preset strip resumes active chip
            DashSection::Security => CheckMirror,     // moving UP -> enter via bottom-most element
            DashSection::ActionDispatcher => last_widget,
        },
        Direction::Down => match section {
            DashSection::SearchBar => SearchInput,
            DashSection::CoreEngine => SwitchTurbo,   // moving DOWN -> enter via top-most element
            DashSection::PipelineSync => last_widget, // horizontal preset strip resumes active chip
            DashSection::Security => TokenInput,      // moving DOWN -> enter via top-most element
            DashSection::ActionDispatcher => last_widget,
        },
    }
}

/// Converts intuitive (response_time, bounce, mass) into SpringParams (ω₀, ζ).
fn compute_spring_params(response_time: f32, bounce: f32, mass: f32) -> SpringParams {
    let omega_0 = (std::f32::consts::TAU / response_time.max(0.01)) / mass.sqrt().max(0.1);
    let zeta = 1.0 - bounce.clamp(0.0, 1.0) * 0.90; // bounce 0→ζ=1.0, bounce 1→ζ=0.10
    SpringParams::new(omega_0, zeta)
}

pub fn show(ui: &mut Ui, state: &mut WidgetsDemoState, palette: &ThemePalette) {
    let custom_spring_params = compute_spring_params(state.response_time, state.bounce, state.mass);
    let computed_omega = custom_spring_params.angular_frequency;
    let computed_zeta = custom_spring_params.damping_ratio;
    let velocity_kick = state.velocity_kick;

    ui.horizontal(|ui| {
        ui.heading("🧩 egui-widgets Showcase");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            Badge::new(format!(
                "ω₀: {:.0}, ζ: {:.2} | {:.0}ms, bounce: {:.0}%, mass: {:.1}",
                computed_omega, computed_zeta,
                state.response_time * 1000.0, state.bounce * 100.0, state.mass
            ))
                .accent()
                .palette(palette)
                .show(ui);
        });
    });
    ui.label("Polished design-system layer powered by ThemePalette and spring motion physics");
    ui.add_space(6.0);

    // Live Physics Tuner — compact
    Card::new()
        .title("🎛️ Motion Physics Tuner")
        .subtitle("Applies to every widget uniformly")
        .palette(palette)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Presets:");
                if Button::new("Instant").small().palette(palette).spring_params(custom_spring_params).show(ui).clicked() {
                    state.response_time = 0.08; state.bounce = 0.0; state.mass = 1.0; state.velocity_kick = 0.0;
                }
                if Button::new("Fast & Crisp").small().palette(palette).spring_params(custom_spring_params).show(ui).clicked() {
                    state.response_time = 0.12; state.bounce = 0.05; state.mass = 1.0; state.velocity_kick = 0.0;
                }
                if Button::new("Snappy Pop").small().palette(palette).spring_params(custom_spring_params).show(ui).clicked() {
                    state.response_time = 0.18; state.bounce = 0.40; state.mass = 1.0; state.velocity_kick = 0.0;
                }
                if Button::new("Smooth Glide").small().palette(palette).spring_params(custom_spring_params).show(ui).clicked() {
                    state.response_time = 0.30; state.bounce = 0.0; state.mass = 1.8; state.velocity_kick = 0.0;
                }
                if Button::new("Juicy Spring").small().palette(palette).spring_params(custom_spring_params).show(ui).clicked() {
                    state.response_time = 0.20; state.bounce = 0.65; state.mass = 1.0; state.velocity_kick = 3.0;
                }
            });

            ui.add_space(6.0);

            let bounce_hint = match () {
                _ if state.bounce < 0.05 => " (None)",
                _ if state.bounce < 0.25 => " (Subtle)",
                _ if state.bounce < 0.50 => " (Springy)",
                _ if state.bounce < 0.75 => " (Bouncy)",
                _ => " (Very Bouncy)",
            };
            let mass_hint = match () {
                _ if state.mass < 0.6 => " (Light)",
                _ if state.mass < 1.2 => " (Normal)",
                _ if state.mass < 2.5 => " (Heavy)",
                _ => " (Very Heavy)",
            };
            let kick_hint = if state.velocity_kick < 0.1 { " (Off)" } else { " (Flick)" };

            ui.columns(2, |cols| {
                cols[0].label("⏱ Response Time:");
                Slider::new(&mut state.response_time, 0.03..=0.80)
                    .step(0.01).suffix(" sec").palette(palette)
                    .show(&mut cols[0]);
                cols[0].add_space(4.0);
                cols[0].label("⚖️ Mass:");
                Slider::new(&mut state.mass, 0.3..=5.0)
                    .step(0.1).suffix(mass_hint).palette(palette)
                    .show(&mut cols[0]);

                cols[1].label("🏀 Bounce:");
                Slider::new(&mut state.bounce, 0.0..=1.0)
                    .step(0.01).suffix(bounce_hint).palette(palette)
                    .show(&mut cols[1]);
                cols[1].add_space(4.0);
                cols[1].label("💥 Velocity Kick:");
                Slider::new(&mut state.velocity_kick, 0.0..=10.0)
                    .step(0.5).suffix(kick_hint).palette(palette)
                    .show(&mut cols[1]);
            });

            ui.add_space(2.0);
            Badge::new(format!(
                "ω₀ = {:.1} rad/s, ζ = {:.2}",
                computed_omega, computed_zeta
            )).info().palette(palette).show(ui);
        });

    ui.add_space(10.0);

    // Category Tabs
    SegmentedTabs::new(&mut state.category)
        .tab(WidgetsCategory::ButtonsAndBadges, "🔘 Buttons & Badges")
        .tab(WidgetsCategory::SwitchesAndToggles, "🎚️ Switches & Toggles")
        .tab(WidgetsCategory::SlidersAndInputs, "🎛️ Sliders & Inputs")
        .tab(WidgetsCategory::SurfacesAndProgress, "📦 Cards & Progress")
        .tab(WidgetsCategory::CompositeDashboard, "🚀 All-In-One Dashboard")
        .palette(palette)
        .show(ui);

    ui.add_space(12.0);

    let dt = ui.input(|i| i.stable_dt).min(0.05);

    let pointer_in_dropdown = state.demo_region_dropdown_state.contains_pointer(ui.ctx());
    let scroll_area = egui::ScrollArea::vertical().enable_scrolling(!pointer_in_dropdown);
    let is_dashboard = state.category == WidgetsCategory::CompositeDashboard;
    let (scroll_area, applied_scroll) = if is_dashboard {
        let (sa, val) = state.dash_scrolloff.inject_into(scroll_area, dt);
        (sa, Some(val))
    } else {
        (scroll_area, None)
    };

    let scroll_output = scroll_area.show(ui, |ui| {
        match state.category {
            WidgetsCategory::ButtonsAndBadges => {
                Card::new()
                    .title("Button Variants & Micro-Interactions")
                    .subtitle("Spring-animated press compression and hover luminance glide")
                    .palette(palette)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let gallery_labels = [
                                "Primary Action",
                                "Dynamic Expanding Label (Smooth Spring Transition!)",
                                "Contracted Label",
                            ];
                            let gallery_text = gallery_labels[state.gallery_btn_step % gallery_labels.len()];
                            if Button::new(gallery_text)
                                .primary()
                                .icon("⚡")
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .show(ui)
                                .clicked()
                            {
                                state.gallery_btn_step += 1;
                            }

                            Button::new("Secondary")
                                .secondary()
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .show(ui);

                            Button::new("Success Action")
                                .success()
                                .icon("✓")
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .show(ui);

                            Button::new("Warning Note")
                                .warning()
                                .icon("⚠")
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .show(ui);

                            Button::new("Destructive Action")
                                .danger()
                                .icon("🗑")
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .show(ui);

                            Button::new("Outline")
                                .outline()
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .show(ui);

                            Button::new("Ghost")
                                .ghost()
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .show(ui);
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);

                        ui.label("Button Sizing & Badges:");
                        ui.horizontal(|ui| {
                            Button::new("Small").small().palette(palette).show(ui);
                            Button::new("Medium Standard").medium().badge("Pro").palette(palette).show(ui);
                            Button::new("Large CTA").large().shortcut("Ctrl+Enter").palette(palette).show(ui);
                        });
                    });

                ui.add_space(12.0);

                Card::new()
                    .title("Semantic Status Badges & Tags")
                    .subtitle("High-contrast technical outline pills and punchy solid labels")
                    .palette(palette)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Technical Outline Style (Option A — Default):").small().color(palette.subtext0));
                        ui.horizontal_wrapped(|ui| {
                            Badge::new("System Active").success().outline().dot(true).palette(palette).show(ui);
                            Badge::new("High CPU Load").warning().outline().dot(true).palette(palette).show(ui);
                            Badge::new("Connection Error").danger().outline().dot(true).palette(palette).show(ui);
                            Badge::new("Syncing...").info().outline().dot(true).palette(palette).show(ui);
                            Badge::new("v2.4.0-stable").neutral().outline().palette(palette).show(ui);
                        });
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Punchy Solid Style (Option B):").small().color(palette.subtext0));
                        ui.horizontal_wrapped(|ui| {
                            Badge::new("System Active").success().solid().dot(true).palette(palette).show(ui);
                            Badge::new("High CPU Load").warning().solid().dot(true).palette(palette).show(ui);
                            Badge::new("Connection Error").danger().solid().dot(true).palette(palette).show(ui);
                            Badge::new("Syncing...").info().solid().dot(true).palette(palette).show(ui);
                            Badge::new("v2.4.0-stable").neutral().solid().palette(palette).show(ui);
                        });
                    });
            }

            WidgetsCategory::SwitchesAndToggles => {
                Card::new()
                    .title("Spring Toggle Switches")
                    .subtitle("1D ODE physics with elastic thumb arrivals and track color morphing")
                    .palette(palette)
                    .show(ui, |ui| {
                        Switch::new(&mut state.switch_turbo)
                            .label("Turbo Processing Acceleration")
                            .large()
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);
                        ui.add_space(8.0);

                        Switch::new(&mut state.switch_vpn)
                            .label("Encrypted Tunnel Proxy")
                            .standard()
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);
                        ui.add_space(8.0);

                        Switch::new(&mut state.switch_analytics)
                            .label("Anonymous Usage Telemetry")
                            .compact()
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);
                    });

                ui.add_space(12.0);

                Card::new()
                    .title("Checkboxes & Radio Options")
                    .subtitle("Spring checkmark scale pop and radio dot expansion")
                    .palette(palette)
                    .show(ui, |ui| {
                        Checkbox::new(&mut state.checkbox_newsletter)
                            .label("Subscribe to product release updates")
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);
                        ui.add_space(6.0);

                        Checkbox::new(&mut state.checkbox_telemetry)
                            .label("Send anonymous crash diagnostic dumps")
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);

                        ui.label("Select Deployment Tier (Nullable — click selected to deselect):");
                        ui.horizontal(|ui| {
                            RadioButton::nullable(1, &mut state.radio_tier)
                                .label("Standard")
                                .palette(palette)
                                .show(ui);
                            RadioButton::nullable(2, &mut state.radio_tier)
                                .label("Professional")
                                .palette(palette)
                                .show(ui);
                            RadioButton::nullable(3, &mut state.radio_tier)
                                .label("Enterprise")
                                .palette(palette)
                                .show(ui);
                        });
                    });
            }

            WidgetsCategory::SlidersAndInputs => {
                Card::new()
                    .title("Numeric Sliders")
                    .subtitle("Spring-animated position glide on click, 1:1 tracking on drag")
                    .palette(palette)
                    .show(ui, |ui| {
                        Slider::new(&mut state.volume_slider, 0.0..=100.0)
                            .label("Playback Volume")
                            .suffix(" %")
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .click_momentum(velocity_kick)
                            .show(ui);
                        ui.add_space(8.0);

                        Slider::new(&mut state.threshold_slider, 0.0..=100.0)
                            .label("Detection Sensitivity (Stacked Layout)")
                            .suffix(" dB")
                            .stacked()
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .click_momentum(velocity_kick)
                            .show(ui);
                        ui.add_space(8.0);

                        Slider::new(&mut state.test_slider_val, 0.0..=100.0)
                            .label("Sandbox — click jump buttons →")
                            .suffix(" %")
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .click_momentum(velocity_kick)
                            .show(ui);
                        ui.horizontal(|ui| {
                            for (label, val) in [("0%", 0.0), ("25%", 25.0), ("50%", 50.0), ("75%", 75.0), ("100%", 100.0)] {
                                if Button::new(label).small().palette(palette).show(ui).clicked() {
                                    state.test_slider_val = val;
                                }
                            }
                            if Button::new("🎲").small().palette(palette).show(ui).clicked() {
                                state.test_slider_val = ((state.test_slider_val * 37.0 + 17.0) % 100.0).round();
                            }
                        });
                    });

                ui.add_space(12.0);

                Card::new()
                    .title("Text Inputs")
                    .subtitle("Spring-animated focus glow and clear button")
                    .palette(palette)
                    .show(ui, |ui| {
                        TextInput::new(&mut state.search_term)
                            .placeholder("Search components or assets...")
                            .icon("🔍")
                            .clear_button(true)
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);
                        ui.add_space(8.0);

                        TextInput::new(&mut state.token_input)
                            .placeholder("Enter secret API authorization token...")
                            .icon("🔑")
                            .password(true)
                            .clear_button(true)
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);
                    });

                ui.add_space(12.0);

                Card::new()
                    .title("Spring-Animated Dropdown Menus")
                    .subtitle("4-state micro-interactions, continuous open expansion, sliding item highlight, and Vim navigation")
                    .palette(palette)
                    .show(ui, |ui| {
                        ui.label("Cluster Deployment Region (Primary Variant):");
                        Dropdown::new(
                            &mut state.demo_region,
                            vec![
                                DropdownOption::new(DemoRegion::UsEast, "US-East (Virginia)").icon("🇺🇸").subtitle("Ping: 24ms"),
                                DropdownOption::new(DemoRegion::EuCentral, "EU-Central (Frankfurt)").icon("🇩🇪").subtitle("Ping: 82ms"),
                                DropdownOption::new(DemoRegion::ApSouth, "AP-South (Mumbai)").icon("🇮🇳").subtitle("Ping: 110ms").badge("Fast"),
                                DropdownOption::new(DemoRegion::SaEast, "SA-East (São Paulo)").icon("🇧🇷").subtitle("Ping: 160ms"),
                            ],
                        )
                        .primary()
                        .palette(palette)
                        .spring_params(custom_spring_params)
                        .with_state(&mut state.demo_region_dropdown_state)
                        .show(ui);

                        ui.add_space(8.0);
                        ui.label("Logging Severity Filter (Outline Variant):");
                        Dropdown::new(
                            &mut state.demo_log_level,
                            vec![
                                DropdownOption::new(DemoLogLevel::Debug, "Debug").icon("🐛"),
                                DropdownOption::new(DemoLogLevel::Info, "Info").icon("ℹ"),
                                DropdownOption::new(DemoLogLevel::Warn, "Warn").icon("⚠"),
                                DropdownOption::new(DemoLogLevel::Error, "Error").icon("🚨").badge("Alert"),
                            ],
                        )
                        .outline()
                        .palette(palette)
                        .spring_params(custom_spring_params)
                        .with_state(&mut state.demo_log_dropdown_state)
                        .show(ui);
                    });
            }

            WidgetsCategory::SurfacesAndProgress => {
                Card::new()
                    .title("Physical Progress Catchup Meters")
                    .subtitle("Smooth continuous ODE spring catchup on target changes")
                    .palette(palette)
                    .show(ui, |ui| {
                        ProgressBar::new(state.progress_value)
                            .label("Overall Job Execution")
                            .show_percentage(true)
                            .variant(ProgressVariant::Accent)
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);
                        ui.add_space(8.0);

                        ProgressBar::new(state.progress_value * 0.8)
                            .label("Database Migration")
                            .show_percentage(true)
                            .variant(ProgressVariant::Success)
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);
                        ui.add_space(8.0);

                        ProgressBar::new(state.progress_value * 0.4)
                            .label("Asset Compression")
                            .show_percentage(true)
                            .variant(ProgressVariant::Warning)
                            .palette(palette)
                            .spring_params(custom_spring_params)
                            .show(ui);

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label("Simulate Target:");
                            for pct in [0.10, 0.35, 0.65, 0.90, 1.00] {
                                if Button::new(format!("{:.0}%", pct * 100.0))
                                    .small()
                                    .palette(palette)
                                    .show(ui)
                                    .clicked()
                                {
                                    state.progress_value = pct;
                                }
                            }
                        });
                    });

                ui.add_space(12.0);

                Card::new()
                    .title("Interactive Card with Hover Lift")
                    .subtitle("Hover elevation lift and border glow morphing")
                    .interactive(true)
                    .palette(palette)
                    .show(ui, |ui| {
                        ui.label("Hover over this card to observe the spring-animated elevation lift and border highlight!");
                        ui.add_space(4.0);
                        Badge::new("Interactive Surface").accent().palette(palette).show(ui);
                    });
            }

            WidgetsCategory::CompositeDashboard => {
                use DashboardWidget::*;

                // ── 1. Input Phase: keyboard navigation + actions ──
                let ctx = ui.ctx().clone();

                // Ensure focus is never None — self-heal to active section or default
                if state.dash_nav.focused().is_none() {
                    let sec = state.dash_section_nav.focused().copied().unwrap_or(DashSection::CoreEngine);
                    let default_w = state.last_section_widget[sec as usize];
                    state.dash_nav.set_focus_with_graph(Some(default_w), &state.dash_graph);
                    state.dash_section_nav.set_focus(Some(sec));
                }

                let current_focused = state.dash_nav.focused().copied().unwrap_or(SwitchTurbo);
                let is_on_text_widget = matches!(current_focused, SearchInput | TokenInput | SliderBandwidth | SliderThermal);

                // Force-clear text editing when focus moves to a non-text widget
                if !is_on_text_widget && state.focus_level.is_text_editing() {
                    state.focus_level = FocusLevel::Navigation;
                    ctx.memory_mut(|m| m.stop_text_input());
                }

                // Check for slider-specific external enter/exit triggers
                let externally_entered = state.focus_level.is_navigation() && (
                    (current_focused == SliderBandwidth && state.bandwidth_slider_state.editing)
                    || (current_focused == SliderThermal && state.thermal_slider_state.editing)
                );
                let externally_exited = state.focus_level.is_text_editing() && (
                    (current_focused == SliderBandwidth && !state.bandwidth_slider_state.editing)
                    || (current_focused == SliderThermal && !state.thermal_slider_state.editing)
                );

                // Determine if the active VimBuffer is in clean Normal mode (no pending ops)
                let is_in_clean_normal = match current_focused {
                    SearchInput => state.search_vim.mode() == VimMode::Normal && state.search_vim.parser.pending_keys_label().is_empty(),
                    TokenInput => state.token_vim.mode() == VimMode::Normal && state.token_vim.parser.pending_keys_label().is_empty(),
                    SliderBandwidth => !state.bandwidth_slider_state.editing || (state.bandwidth_slider_state.vim_buffer.mode() == VimMode::Normal && state.bandwidth_slider_state.vim_buffer.parser.pending_keys_label().is_empty()),
                    SliderThermal => !state.thermal_slider_state.editing || (state.thermal_slider_state.vim_buffer.mode() == VimMode::Normal && state.thermal_slider_state.vim_buffer.parser.pending_keys_label().is_empty()),
                    _ => true,
                };

                let section_nav_triggered = state.dash_key_handler.handle_section_input(&ctx).is_some();

                let transition = check_modal_transition(
                    state.focus_level,
                    is_on_text_widget,
                    externally_entered,
                    is_in_clean_normal,
                    externally_exited,
                    section_nav_triggered,
                    &ctx,
                );

                match transition {
                    ModalTransition::EnteredText => {
                        state.focus_level = FocusLevel::TextEditing;
                        match current_focused {
                            SearchInput => {
                                state.search_vim.mode = VimMode::Insert;
                                state.search_vim.cursor = state.search_term.len();
                            }
                            TokenInput => {
                                state.token_vim.mode = VimMode::Insert;
                                state.token_vim.cursor = state.token_input.len();
                            }
                            SliderBandwidth => {
                                state.bandwidth_slider_state.editing = true;
                                state.bandwidth_slider_state.vim_buffer.mode = VimMode::Insert;
                                state.bandwidth_slider_state.vim_buffer.cursor = state.bandwidth_slider_state.edit_buffer.len();
                            }
                            SliderThermal => {
                                state.thermal_slider_state.editing = true;
                                state.thermal_slider_state.vim_buffer.mode = VimMode::Insert;
                                state.thermal_slider_state.vim_buffer.cursor = state.thermal_slider_state.edit_buffer.len();
                            }
                            _ => {}
                        }
                    }
                    ModalTransition::ExitedText => {
                        state.focus_level = FocusLevel::Navigation;
                        state.search_vim.mode = VimMode::Normal;
                        state.token_vim.mode = VimMode::Normal;
                        state.bandwidth_slider_state.vim_buffer.mode = VimMode::Normal;
                        state.bandwidth_slider_state.editing = false;
                        state.thermal_slider_state.vim_buffer.mode = VimMode::Normal;
                        state.thermal_slider_state.editing = false;
                        ctx.memory_mut(|m| m.stop_text_input());
                    }
                    ModalTransition::None => {}
                }

                // Suppress egui's default Tab focus cycling (unconditionally)
                ctx.input_mut(|i| {
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
                    i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
                });

                let is_dropdown_open = state.dash_nav.focused() == Some(&DropdownRegion) && state.demo_region_dropdown_state.is_open;

                let dir_input = if state.focus_level.is_navigation() && !is_dropdown_open {
                    state.dash_key_handler.get_nav_direction(&ctx)
                        .or_else(|| state.dash_key_handler.handle_section_input(&ctx))
                } else {
                    None
                };

                // 1. Direct Section Jumps via Ctrl+HJKL (Restores Last-Focused Widget in Target Section)
                if !is_dropdown_open {
                    if let Some(sec_event) = state.dash_key_handler.handle_section_nav(
                        &ctx,
                        &mut state.dash_section_nav,
                        &state.dash_section_graph,
                    ) {
                        state.focus_level = FocusLevel::Navigation;
                        ctx.memory_mut(|m| m.stop_text_input());
                        let entry = state.last_section_widget[sec_event.current as usize];
                        state.dash_nav.set_focus_with_graph(Some(entry), &state.dash_graph);
                    }
                }

                // 2. 2-Pass Hierarchical Navigation (HJKL / Arrows without Ctrl)
                // When actively focused inside a text input or open dropdown, skip UI navigation so HJKL keys operate INSIDE!
                if state.focus_level.is_navigation() && !is_dropdown_open {
                    if let Some(dir) = state.dash_key_handler.get_nav_direction(&ctx) {
                        ctx.memory_mut(|m| m.stop_text_input());
                        if let Some(result) = hierarchical_move(
                            dir,
                            &mut state.dash_nav, &state.dash_graph,
                            &mut state.dash_section_nav, &state.dash_section_graph,
                            |w| widget_to_section(*w),
                            |sec, dir| section_entry_widget(*sec, dir, state.last_section_widget[*sec as usize]),
                        ) {
                            state.last_section_widget[result.focused_section as usize] = result.focused_widget;
                        }
                    }
                }

                let current_focused_node = state.dash_nav.focused().copied();
                state.dash_scrolloff.record_nav_event(current_focused_node, dir_input);

                // Shift+H / Shift+L — adjust slider values when focused on a slider
                if state.focus_level.is_navigation() && !is_dropdown_open {
                    let focused = state.dash_nav.focused().copied();
                    let shift_dir = ctx.input(|i| {
                        if !i.modifiers.shift { return None; }
                        if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft) {
                            Some(-1.0f32)
                        } else if i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) {
                            Some(1.0f32)
                        } else {
                            None
                        }
                    });
                    if let Some(dir) = shift_dir {
                        let step = 5.0; // step per keypress
                        match focused {
                            Some(SliderBandwidth) => {
                                state.volume_slider = (state.volume_slider + dir * step).clamp(0.0, 100.0);
                                ctx.request_repaint();
                            }
                            Some(SliderThermal) => {
                                state.threshold_slider = (state.threshold_slider + dir * step).clamp(0.0, 100.0);
                                ctx.request_repaint();
                            }
                            _ => {}
                        }
                    }
                }

                // Action keys (F/Enter, D, Q/Esc) — suppressed when actively focused in text input
                let primary_state = if state.focus_level.is_text_editing() {
                    VimActionState::default()
                } else {
                    state.dash_key_handler.primary_action_state(&ctx)
                };
                let secondary_state = if state.focus_level.is_text_editing() {
                    VimActionState::default()
                } else {
                    state.dash_key_handler.secondary_action_state(&ctx)
                };
                let focused = state.dash_nav.focused().copied();

                let action = if primary_state.clicked || primary_state.double_clicked {
                    Some(VimAction::PrimaryClick)
                } else if secondary_state.clicked || secondary_state.double_clicked {
                    Some(VimAction::SecondaryClick)
                } else {
                    ctx.input(|i| {
                        if i.modifiers.ctrl {
                            None
                        } else if i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Q) {
                            Some(VimAction::Back)
                        } else {
                            None
                        }
                    })
                };

                if let Some(VimAction::Back) = action {
                    ctx.memory_mut(|m| m.stop_text_input());
                }


                // ── 2. Layout Phase: render widgets, collect focused rect ──
                let mut highlight_target: Option<Rect> = None;
                let mut highlight_rounding = Rounding::same(6.0);
                let mut section_target: Option<Rect> = None;
                let section_rounding = Rounding::same(8.0);
                let mut trigger_expand_scroll = false;
                let mut expand_scroll_rect: Option<Rect> = None;
                let active_section = state.dash_section_nav.focused().copied()
                    .unwrap_or(DashSection::CoreEngine);
                let pointer_moved = ui.input(|i| i.pointer.delta() != Vec2::ZERO);

                // Header
                ui.horizontal(|ui| {
                    ui.heading("🚀 Unified Control Center");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        Badge::new("System Nominal").success().dot(true).palette(palette).show(ui);
                        Badge::new("Live Telemetry").accent().dot(true).palette(palette).show(ui);
                    });
                });
                ui.label("HJKL navigate • [i]/Enter type text • [Esc] done • Ctrl+HJKL jump section • Shift+H/L adjust slider");
                ui.add_space(10.0);

                // ── Top Search Bar ──
                let (search_card_resp, _) = Card::new()
                    .focused(active_section == DashSection::SearchBar)
                    .spring_params(custom_spring_params)
                    .palette(palette)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("🔎 Quick Find:");
                            ui.add_space(6.0);

                            let is_search_focused = focused == Some(SearchInput);
                            let is_search_active = is_search_focused && state.focus_level.is_text_editing();
                            let search_w = (ui.available_width() - 250.0).max(180.0);
                            let resp = TextInput::new(&mut state.search_term)
                                .placeholder("Filter parameters, services, endpoints...")
                                .icon("🔍")
                                .clear_button(true)
                                .width(search_w)
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .with_state(&mut state.search_input_state)
                                .focused(is_search_active)
                                .vim_buffer(&mut state.search_vim)
                                .show(ui);
                            if (resp.hovered() && pointer_moved) || resp.clicked() {
                                state.record_widget_focus(SearchInput);
                                if resp.clicked() {
                                    state.focus_level = FocusLevel::TextEditing;
                                    if !resp.double_clicked() && !resp.triple_clicked() && !state.search_vim.mode.is_visual() {
                                        state.search_vim.mode = VimMode::Insert;
                                    }
                                }
                            }
                            if is_search_focused {
                                highlight_target = Some(resp.rect);
                                highlight_rounding = Rounding::same(6.0);
                            }

                            ui.add_space(8.0);

                            let is_scan_focused = focused == Some(ScanButton);
                            let is_scan_down = is_scan_focused && primary_state.is_down;
                            let is_scan_triggered = is_scan_focused && (primary_state.clicked || primary_state.double_clicked);
                            let scan_labels = ["Scan Network", "Scanning Endpoints...", "Scan Completed (32 Hosts Found)", "Reset Scan"];
                            let scan_icons = ["📡", "⏳", "✓", "🔄"];
                            let scan_text = scan_labels[state.scan_button_step % scan_labels.len()];
                            let scan_icon = scan_icons[state.scan_button_step % scan_icons.len()];
                            let resp = Button::new(scan_text)
                                .primary()
                                .icon(scan_icon)
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .id_source(egui::Id::new("dash_btn").with(ScanButton as u32))
                                .focused(is_scan_focused)
                                .pressed(is_scan_down)
                                .triggered(is_scan_triggered)
                                .show(ui);
                            if (resp.hovered() && pointer_moved) || resp.clicked() {
                                state.record_widget_focus(ScanButton);
                            }
                            if resp.clicked() || is_scan_triggered {
                                state.scan_button_step += 1;
                                state.progress_value = match state.scan_button_step % 4 {
                                    1 => 0.45,
                                    2 => 1.0,
                                    3 => 0.0,
                                    _ => 0.88,
                                };
                            }
                            if is_scan_focused {
                                highlight_target = Some(resp.rect);
                                highlight_rounding = Rounding::same(6.0);
                            }

                            ui.add_space(6.0);

                            let is_clear_focused = focused == Some(ClearButton);
                            let is_clear_down = is_clear_focused && primary_state.is_down;
                            let is_clear_triggered = is_clear_focused && (primary_state.clicked || primary_state.double_clicked);
                            let resp = Button::new("Clear")
                                .secondary()
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .id_source(egui::Id::new("dash_btn").with(ClearButton as u32))
                                .focused(is_clear_focused)
                                .pressed(is_clear_down)
                                .triggered(is_clear_triggered)
                                .show(ui);
                            if (resp.hovered() && pointer_moved) || resp.clicked() {
                                state.record_widget_focus(ClearButton);
                            }
                            if resp.clicked() || is_clear_triggered {
                                state.search_term.clear();
                            }
                            if is_clear_focused {
                                highlight_target = Some(resp.rect);
                                highlight_rounding = Rounding::same(6.0);
                            }
                        });
                    });
                if active_section == DashSection::SearchBar {
                    section_target = Some(search_card_resp.rect.expand(2.0));
                }

                ui.add_space(10.0);

                // ── Two-column layout ──
                ui.columns(2, |columns| {
                    // ════ Column 1: Core Engine & Services ════
                    columns[0].vertical(|ui| {
                        let (core_card_resp, _) = Card::new()
                            .title("⚡ Core Engine & Services")
                            .subtitle("Network protocols and hardware accelerators")
                            .focused(active_section == DashSection::CoreEngine)
                            .spring_params(custom_spring_params)
                            .palette(palette)
                            .show(ui, |ui| {
                                let is_turbo_focused = focused == Some(SwitchTurbo);
                                let is_turbo_down = is_turbo_focused && primary_state.is_down;
                                let is_turbo_triggered = is_turbo_focused && matches!(action, Some(VimAction::PrimaryClick | VimAction::Enter));
                                let resp = Switch::new(&mut state.switch_turbo)
                                    .label("Turbo Mode")
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .focused(is_turbo_focused)
                                    .pressed(is_turbo_down)
                                    .triggered(is_turbo_triggered)
                                    .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.clicked() {
                                    state.record_widget_focus(SwitchTurbo);
                                }
                                if is_turbo_focused { highlight_target = Some(resp.rect); }

                                let is_vpn_focused = focused == Some(SwitchVpn);
                                let is_vpn_down = is_vpn_focused && primary_state.is_down;
                                let is_vpn_triggered = is_vpn_focused && matches!(action, Some(VimAction::PrimaryClick | VimAction::Enter));
                                let resp = Switch::new(&mut state.switch_vpn)
                                    .label("Secure Gateway Tunnel")
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .focused(is_vpn_focused)
                                    .pressed(is_vpn_down)
                                    .triggered(is_vpn_triggered)
                                    .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.clicked() {
                                    state.record_widget_focus(SwitchVpn);
                                }
                                if is_vpn_focused { highlight_target = Some(resp.rect); }

                                let is_analytics_focused = focused == Some(SwitchAnalytics);
                                let is_analytics_down = is_analytics_focused && primary_state.is_down;
                                let is_analytics_triggered = is_analytics_focused && matches!(action, Some(VimAction::PrimaryClick | VimAction::Enter));
                                let resp = Switch::new(&mut state.switch_analytics)
                                    .label("Realtime Telemetry Ingestion")
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .focused(is_analytics_focused)
                                    .pressed(is_analytics_down)
                                    .triggered(is_analytics_triggered)
                                    .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.clicked() {
                                    state.record_widget_focus(SwitchAnalytics);
                                }
                                if is_analytics_focused { highlight_target = Some(resp.rect); }

                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);

                                ui.label("Bandwidth Throttle:");
                                let is_bandwidth_focused = focused == Some(SliderBandwidth);
                                let resp = Slider::new(&mut state.volume_slider, 0.0..=100.0)
                                    .suffix(" MB/s")
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .with_state(&mut state.bandwidth_slider_state)
                                    .focused(is_bandwidth_focused)
                                    .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.dragged() || resp.clicked() {
                                    state.record_widget_focus(SliderBandwidth);
                                    if state.bandwidth_slider_state.editing {
                                        state.focus_level = FocusLevel::TextEditing;
                                    }
                                }
                                if is_bandwidth_focused {
                                    highlight_target = Some(resp.rect);
                                    highlight_rounding = Rounding::same(6.0);
                                }

                                ui.add_space(4.0);
                                ui.label("Thermal Cutoff Threshold:");
                                let is_thermal_focused = focused == Some(SliderThermal);
                                let resp = Slider::new(&mut state.threshold_slider, 0.0..=100.0)
                                    .suffix(" °C")
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .with_state(&mut state.thermal_slider_state)
                                    .focused(is_thermal_focused)
                                    .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.dragged() || resp.clicked() {
                                    state.record_widget_focus(SliderThermal);
                                    if state.thermal_slider_state.editing {
                                        state.focus_level = FocusLevel::TextEditing;
                                    }
                                }
                                if is_thermal_focused {
                                    highlight_target = Some(resp.rect);
                                    highlight_rounding = Rounding::same(6.0);
                                }
                            });
                        if active_section == DashSection::CoreEngine {
                            section_target = Some(core_card_resp.rect.expand(2.0));
                        }

                        ui.add_space(10.0);

                        // Pipeline Sync Card
                        let (pipeline_card_resp, _) = Card::new()
                            .title("📊 Live Pipeline Synchronization")
                            .subtitle("Physical spring ODE catchup meter")
                            .focused(active_section == DashSection::PipelineSync)
                            .spring_params(custom_spring_params)
                            .palette(palette)
                            .show(ui, |ui| {
                                egui_widgets::ProgressBar::new(state.progress_value)
                                    .label("Cluster Ingestion Sync")
                                    .show_percentage(true)
                                    .variant(if state.progress_value > 0.8 {
                                        ProgressVariant::Success
                                    } else {
                                        ProgressVariant::Accent
                                    })
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .show(ui);

                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    for (id, label, val) in [
                                        (ProgressBtn25, "25%", 0.25),
                                        (ProgressBtn50, "50%", 0.50),
                                        (ProgressBtn75, "75%", 0.75),
                                        (ProgressBtn100, "100%", 1.00),
                                    ] {
                                        let is_btn_focused = focused == Some(id);
                                        let is_btn_down = is_btn_focused && primary_state.is_down;
                                        let is_btn_triggered = is_btn_focused && (primary_state.clicked || primary_state.double_clicked);
                                        let is_active = (state.progress_value - val).abs() < 0.02;
                                        let mut btn = Button::new(label)
                                            .min_size(egui::vec2(52.0, 26.0))
                                            .palette(palette)
                                            .spring_params(custom_spring_params)
                                            .id_source(egui::Id::new("dash_btn").with(id as u32))
                                            .focused(is_btn_focused)
                                            .pressed(is_btn_down)
                                            .triggered(is_btn_triggered);
                                        if is_active {
                                            btn = btn.primary();
                                        } else {
                                            btn = btn.secondary();
                                        }
                                        let resp = btn.show(ui);
                                        if (resp.hovered() && pointer_moved) || resp.clicked() {
                                            state.record_widget_focus(id);
                                        }
                                        if resp.clicked() || is_btn_triggered {
                                            state.progress_value = val;
                                        }
                                        if is_btn_focused {
                                            highlight_target = Some(resp.rect);
                                            highlight_rounding = Rounding::same(6.0);
                                        }
                                    }
                                });
                            });
                        if active_section == DashSection::PipelineSync {
                            section_target = Some(pipeline_card_resp.rect.expand(2.0));
                        }
                    });

                    // ════ Column 2: Security & Actions ════
                    columns[1].vertical(|ui| {
                        let (security_card_resp, _) = Card::new()
                            .title("🔒 Security & Deployment Tier")
                            .subtitle("Access keys and environment targeting")
                            .focused(active_section == DashSection::Security)
                            .spring_params(custom_spring_params)
                            .palette(palette)
                            .show(ui, |ui| {
                                // Token Input
                                let is_token_focused = focused == Some(TokenInput);
                                let is_token_active = is_token_focused && state.focus_level.is_text_editing();
                                let resp = TextInput::new(&mut state.token_input)
                                    .placeholder("Authorization secret key...")
                                    .icon("🔑")
                                    .password(true)
                                    .clear_button(true)
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .with_state(&mut state.token_input_state)
                                    .focused(is_token_active)
                                    .vim_buffer(&mut state.token_vim)
                                    .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.clicked() {
                                    state.record_widget_focus(TokenInput);
                                    if resp.clicked() {
                                        state.focus_level = FocusLevel::TextEditing;
                                        if !resp.double_clicked() && !resp.triple_clicked() && !state.token_vim.mode.is_visual() {
                                            state.token_vim.mode = VimMode::Insert;
                                        }
                                    }
                                }
                                if is_token_focused {
                                    highlight_target = Some(resp.rect);
                                    highlight_rounding = Rounding::same(6.0);
                                }

                                ui.add_space(10.0);
                                ui.label("Target Deployment Region:");
                                let is_dd_focused = focused == Some(DropdownRegion);
                                let is_dd_down = is_dd_focused && primary_state.is_down;
                                let is_dd_triggered = is_dd_focused && (primary_state.clicked || primary_state.double_clicked);
                                let resp = Dropdown::new(
                                    &mut state.demo_region,
                                    vec![
                                        DropdownOption::new(DemoRegion::UsEast, "US-East (Virginia)")
                                            .icon("🇺🇸")
                                            .status_dot(palette.success)
                                            .description("Primary production cluster")
                                            .subtitle("24ms"),
                                        DropdownOption::new(DemoRegion::UsWest, "US-West (Oregon)")
                                            .icon("🇺🇸")
                                            .status_dot(palette.success)
                                            .description("Backup failover zone")
                                            .subtitle("48ms"),
                                        DropdownOption::new(DemoRegion::EuCentral, "EU-Central (Frankfurt)")
                                            .icon("🇩🇪")
                                            .status_dot(palette.success)
                                            .description("GDPR compliant tier")
                                            .subtitle("82ms"),
                                        DropdownOption::new(DemoRegion::EuWest, "EU-West (Ireland)")
                                            .icon("🇮🇪")
                                            .status_dot(palette.warning)
                                            .description("Scheduled maintenance")
                                            .subtitle("75ms"),
                                        DropdownOption::new(DemoRegion::ApSouth, "AP-South (Mumbai)")
                                            .icon("🇮🇳")
                                            .status_dot(palette.success)
                                            .description("Direct peering node")
                                            .subtitle("110ms")
                                            .badge("Fast")
                                            .badge_color(palette.accent),
                                        DropdownOption::new(DemoRegion::ApNortheast, "AP-East (Tokyo)")
                                            .icon("🇯🇵")
                                            .status_dot(palette.success)
                                            .description("Asia-Pacific relay")
                                            .subtitle("135ms"),
                                        DropdownOption::new(DemoRegion::SaEast, "SA-East (São Paulo)")
                                            .icon("🇧🇷")
                                            .status_dot(palette.danger)
                                            .description("Degraded network routing")
                                            .subtitle("160ms"),
                                        DropdownOption::new(DemoRegion::AfSouth, "AF-South (Cape Town)")
                                            .icon("🇿🇦")
                                            .status_dot(palette.warning)
                                            .description("Edge acceleration node")
                                            .subtitle("195ms"),
                                    ],
                                )
                                .primary()
                                .width(310.0)
                                .item_height(38.0)
                                .auto_scroll(false)
                                .palette(palette)
                                .spring_params(custom_spring_params)
                                .id_source(egui::Id::new("dash_dd").with(DropdownRegion as u32))
                                .with_state(&mut state.demo_region_dropdown_state)
                                .focused(is_dd_focused)
                                .pressed(is_dd_down)
                                .triggered(is_dd_triggered)
                                .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.clicked() {
                                    state.record_widget_focus(DropdownRegion);
                                }
                                let should_scroll = (resp.just_opened() || resp.is_navigating()) && !resp.is_fully_visible();
                                if should_scroll {
                                    trigger_expand_scroll = true;
                                    expand_scroll_rect = Some(resp.expanded_rect());
                                }
                                if is_dd_focused {
                                    highlight_target = Some(resp.highlight_rect());
                                    highlight_rounding = Rounding::same(4.0);
                                }

                                ui.add_space(10.0);
                                ui.label("Target Cluster Tier:");
                                ui.horizontal(|ui| {
                                    for (id, value, label) in [
                                        (RadioDev, 1, "Dev"),
                                        (RadioStaging, 2, "Staging"),
                                        (RadioProd, 3, "Production"),
                                    ] {
                                        let is_radio_focused = focused == Some(id);
                                        let is_radio_down = is_radio_focused && primary_state.is_down;
                                        let is_radio_triggered = is_radio_focused && (primary_state.clicked || primary_state.double_clicked);
                                        let resp = RadioButton::nullable(value, &mut state.radio_tier)
                                            .label(label)
                                            .palette(palette)
                                            .focused(is_radio_focused)
                                            .pressed(is_radio_down)
                                            .triggered(is_radio_triggered)
                                            .show(ui);
                                        if (resp.hovered() && pointer_moved) || resp.clicked() {
                                            state.record_widget_focus(id);
                                        }
                                        if is_radio_focused {
                                            highlight_target = Some(resp.rect);
                                            highlight_rounding = Rounding::same(4.0);
                                        }
                                    }
                                });

                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(6.0);

                                // Checkbox: Notify
                                let is_notify_focused = focused == Some(CheckNotify);
                                let is_notify_down = is_notify_focused && primary_state.is_down;
                                let is_notify_triggered = is_notify_focused && (primary_state.clicked || primary_state.double_clicked);
                                let resp = Checkbox::new(&mut state.checkbox_newsletter)
                                    .label("Auto-notify on deployment rollback")
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .focused(is_notify_focused)
                                    .pressed(is_notify_down)
                                    .triggered(is_notify_triggered)
                                    .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.clicked() {
                                    state.record_widget_focus(CheckNotify);
                                }
                                if is_notify_focused { highlight_target = Some(resp.rect); }

                                ui.add_space(4.0);

                                // Checkbox: Mirror
                                let is_mirror_focused = focused == Some(CheckMirror);
                                let is_mirror_down = is_mirror_focused && primary_state.is_down;
                                let is_mirror_triggered = is_mirror_focused && (primary_state.clicked || primary_state.double_clicked);
                                let resp = Checkbox::new(&mut state.checkbox_telemetry)
                                    .label("Mirror replication to backup region")
                                    .palette(palette)
                                    .spring_params(custom_spring_params)
                                    .focused(is_mirror_focused)
                                    .pressed(is_mirror_down)
                                    .triggered(is_mirror_triggered)
                                    .show(ui);
                                if (resp.hovered() && pointer_moved) || resp.clicked() {
                                    state.record_widget_focus(CheckMirror);
                                }
                                if is_mirror_focused { highlight_target = Some(resp.rect); }
                            });
                        if active_section == DashSection::Security {
                            section_target = Some(security_card_resp.rect.expand(2.0));
                        }

                        ui.add_space(10.0);

                        // Action Dispatcher Card
                        let (action_card_resp, _) = Card::new()
                            .title("🚀 Action Dispatcher")
                            .subtitle("Trigger instant operations across cluster")
                            .focused(active_section == DashSection::ActionDispatcher)
                            .spring_params(custom_spring_params)
                            .palette(palette)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    let deploy_labels = [
                                        "Deploy Cluster",
                                        "Deploying v2.4.0 to Production Tier...",
                                        "Cluster Deployed (Active)",
                                    ];
                                    let deploy_label = deploy_labels[state.deploy_button_step % deploy_labels.len()];

                                    for (id, label, icon, style, prog_val) in [
                                        (BtnDeploy, deploy_label, "🚀", "primary", Some(1.0f32)),
                                        (BtnVerify, "Verify Status", "✓", "success", Some(0.95)),
                                        (BtnPurge, "Purge Cache", "🧹", "warning", Some(0.10)),
                                        (BtnHalt, "Halt Cluster", "🛑", "danger", Some(0.0)),
                                    ] {
                                        let is_btn_focused = focused == Some(id);
                                        let is_btn_down = is_btn_focused && primary_state.is_down;
                                        let is_btn_triggered = is_btn_focused && (primary_state.clicked || primary_state.double_clicked);
                                        let mut btn = Button::new(label)
                                            .icon(icon)
                                            .palette(palette)
                                            .spring_params(custom_spring_params)
                                            .id_source(egui::Id::new("dash_btn").with(id as u32))
                                            .focused(is_btn_focused)
                                            .pressed(is_btn_down)
                                            .triggered(is_btn_triggered);

                                        btn = match style {
                                            "primary" => btn.primary(),
                                            "success" => btn.success(),
                                            "warning" => btn.warning(),
                                            "danger" => btn.danger(),
                                            _ => btn,
                                        };
                                        let resp = btn.show(ui);
                                        if (resp.hovered() && pointer_moved) || resp.clicked() {
                                            state.record_widget_focus(id);
                                        }
                                        if resp.clicked() || is_btn_triggered {
                                            if id == BtnDeploy {
                                                state.deploy_button_step += 1;
                                            }
                                            if let Some(v) = prog_val { state.progress_value = v; }
                                        }
                                        if is_btn_focused {
                                            highlight_target = Some(resp.rect);
                                            highlight_rounding = Rounding::same(6.0);
                                        }
                                    }
                                });

                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    Badge::new("Latency: 14ms").info().dot(true).palette(palette).show(ui);
                                    Badge::new("Uptime: 99.98%").success().dot(true).palette(palette).show(ui);
                                    Badge::new("Nodes: 24/24").accent().dot(true).palette(palette).show(ui);
                                });
                            });
                        if active_section == DashSection::ActionDispatcher {
                            section_target = Some(action_card_resp.rect.expand(2.0));
                        }
                    });
                });

                // ── 3. Retarget + Update + Paint Phase ──
                // Synchronize highlight styling with active ThemePalette and physics tuner
                set_highlight_fill(&mut state.dash_highlights, &DashHighlight::Item, egui::Color32::TRANSPARENT);

                // Resolve target mode stroke color based on active VimMode
                let target_stroke_color = if state.focus_level.is_text_editing() {
                    match current_focused {
                        SearchInput => egui_widgets::resolve_vim_mode_color(state.search_vim.mode(), Some(palette), ui.visuals()),
                        TokenInput => egui_widgets::resolve_vim_mode_color(state.token_vim.mode(), Some(palette), ui.visuals()),
                        SliderBandwidth => egui_widgets::resolve_vim_mode_color(state.bandwidth_slider_state.vim_buffer.mode(), Some(palette), ui.visuals()),
                        SliderThermal => egui_widgets::resolve_vim_mode_color(state.thermal_slider_state.vim_buffer.mode(), Some(palette), ui.visuals()),
                        _ => palette.accent,
                    }
                } else {
                    palette.accent
                };

                let dt = ui.input(|i| i.stable_dt).min(0.05);
                if sync_highlight_stroke_color(&mut state.dash_highlights, &DashHighlight::Item, target_stroke_color, dt, 14.0) {
                    ui.ctx().request_repaint();
                }
                if let Some(item_layer) = state.dash_highlights.get_mut(&DashHighlight::Item) {
                    item_layer.set_motion(MotionPhysics::Custom(custom_spring_params));
                }

                // Section highlight styling
                set_highlight_fill(&mut state.dash_highlights, &DashHighlight::Section, egui::Color32::TRANSPARENT);
                if let Some(section_layer) = state.dash_highlights.get_mut(&DashHighlight::Section) {
                    section_layer.stroke = Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(palette.accent.r(), palette.accent.g(), palette.accent.b(), 100));
                }

                if let Some(target) = section_target {
                    state.dash_highlights.set_target_with_corner_rounding(
                        &DashHighlight::Section,
                        target,
                        section_rounding,
                    );
                }

                if let Some(target) = highlight_target {
                    state.dash_highlights.set_target_with_corner_rounding(
                        &DashHighlight::Item,
                        target,
                        highlight_rounding,
                    );
                }

                let is_dd_focused = state.dash_nav.focused() == Some(&DashboardWidget::DropdownRegion);

                if trigger_expand_scroll {
                    if let Some(exp_rect) = expand_scroll_rect {
                        state.dash_scrolloff.ensure_visible(
                            exp_rect,
                            ui.clip_rect(),
                            ui.min_rect().height(),
                        );
                    }
                } else if !is_dd_focused {
                    if let Some(target_rect) = highlight_target {
                        state.dash_scrolloff.adjust_for_target(
                            target_rect,
                            ui.clip_rect(),
                            ui.min_rect().height(),
                        );
                    }
                }

                state.dash_scrolloff.request_repaint_if_needed(ui.ctx());

                // Update spring physics, check if settled, and paint all highlight layers
                if update_and_paint(&mut state.dash_highlights, dt, ui.painter()) {
                    ui.ctx().request_repaint();
                }
            }
        }
    });

    if applied_scroll.is_some() {
        state.dash_scrolloff.sync_manual_scroll(&scroll_output);
    }
}
