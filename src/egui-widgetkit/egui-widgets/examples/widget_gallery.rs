//! Interactive showcase gallery of all spring-animated, theme-aware widgets.
//!
//! Run with:
//! ```bash
//! cargo run -p egui-widgets --example widget_gallery
//! ```

use eframe::egui;
use egui_themes::{ThemePreset, ThemeState};
use egui_widgets::{
    Badge, Button, ButtonVariant, Card, Checkbox, ProgressBar,
    ProgressVariant, RadioButton, SegmentedTabs, Slider, Switch, TextInput,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum GalleryTab {
    Buttons,
    Toggles,
    Inputs,
    Surfaces,
    Dashboard,
}

struct WidgetGalleryApp {
    theme: ThemeState,
    current_tab: GalleryTab,
    // Widget state variables
    switch_wifi: bool,
    switch_bluetooth: bool,
    switch_dark_mode: bool,
    slider_volume: f32,
    slider_brightness: f32,
    progress_val: f32,
    checkbox_agree: bool,
    radio_choice: usize,
    search_query: String,
    password_input: String,
}

impl Default for WidgetGalleryApp {
    fn default() -> Self {
        Self {
            theme: ThemeState::new(ThemePreset::CatppuccinMocha),
            current_tab: GalleryTab::Buttons,
            switch_wifi: true,
            switch_bluetooth: false,
            switch_dark_mode: true,
            slider_volume: 65.0,
            slider_brightness: 80.0,
            progress_val: 0.64,
            checkbox_agree: true,
            radio_choice: 1,
            search_query: String::new(),
            password_input: String::new(),
        }
    }
}

impl eframe::App for WidgetGalleryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt).min(0.05);
        self.theme.update(dt, ctx);
        self.theme.apply_to_ctx(ctx);

        let p = self.theme.current.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎨 egui-widgets Component Gallery");
            ui.label("Theme-aware, spring-animated UI components with zero global state");
            ui.add_space(8.0);

            // Top control bar: Theme preset selector
            ui.horizontal(|ui| {
                ui.label("Active Theme:");
                for preset in [
                    ThemePreset::CatppuccinMocha,
                    ThemePreset::TokyoNight,
                    ThemePreset::Dracula,
                    ThemePreset::Gruvbox,
                    ThemePreset::CatppuccinLatte,
                ] {
                    let is_active = self.theme.active_preset == Some(preset);
                    if Button::new(preset.name())
                        .small()
                        .variant(if is_active {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Secondary
                        })
                        .palette(&p)
                        .show(ui)
                        .clicked()
                    {
                        self.theme.set_preset(preset);
                    }
                }
            });

            ui.add_space(10.0);

            // Tab navigation
            SegmentedTabs::new(&mut self.current_tab)
                .tab(GalleryTab::Buttons, "Buttons & Badges")
                .tab(GalleryTab::Toggles, "Toggles & Selectors")
                .tab(GalleryTab::Inputs, "Sliders & Inputs")
                .tab(GalleryTab::Surfaces, "Cards & Progress")
                .tab(GalleryTab::Dashboard, "🚀 All-In-One Dashboard")
                .palette(&p)
                .show(ui);

            ui.add_space(12.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.current_tab {
                    GalleryTab::Buttons => {
                        ui.heading("Buttons & Semantic Variants");
                        ui.label("Spring-animated press compression bounce and hover luminance glide");
                        ui.add_space(8.0);

                        ui.horizontal_wrapped(|ui| {
                            Button::new("Primary Action").primary().icon("⚡").palette(&p).show(ui);
                            Button::new("Secondary").secondary().palette(&p).show(ui);
                            Button::new("Success").success().icon("✓").palette(&p).show(ui);
                            Button::new("Warning").warning().icon("⚠").palette(&p).show(ui);
                            Button::new("Danger Action").danger().icon("🗑").palette(&p).show(ui);
                            Button::new("Outline").outline().palette(&p).show(ui);
                            Button::new("Ghost").ghost().palette(&p).show(ui);
                        });

                        ui.add_space(12.0);
                        ui.heading("Button Sizes & Badges");
                        ui.horizontal(|ui| {
                            Button::new("Small").small().palette(&p).show(ui);
                            Button::new("Medium Standard").medium().badge("Pro").palette(&p).show(ui);
                            Button::new("Large CTA").large().shortcut("Ctrl+S").palette(&p).show(ui);
                        });

                        ui.add_space(16.0);
                        ui.heading("Status Badges & Tags");
                        ui.horizontal_wrapped(|ui| {
                            Badge::new("Online").success().dot(true).palette(&p).show(ui);
                            Badge::new("Maintenance").warning().dot(true).palette(&p).show(ui);
                            Badge::new("Offline").danger().dot(true).palette(&p).show(ui);
                            Badge::new("v1.2.0").info().palette(&p).show(ui);
                            Badge::new("Experimental").neutral().palette(&p).show(ui);
                        });
                    }
                    GalleryTab::Toggles => {
                        ui.heading("Spring Toggle Switches");
                        ui.label("1D analytical spring ODE thumb translation and track morphing");
                        ui.add_space(8.0);

                        Switch::new(&mut self.switch_wifi)
                            .label("Wi-Fi Wireless Network")
                            .standard()
                            .palette(&p)
                            .show(ui);
                        ui.add_space(6.0);

                        Switch::new(&mut self.switch_bluetooth)
                            .label("Bluetooth Discovery Mode")
                            .compact()
                            .palette(&p)
                            .show(ui);
                        ui.add_space(6.0);

                        Switch::new(&mut self.switch_dark_mode)
                            .label("Dark Mode High Contrast")
                            .large()
                            .palette(&p)
                            .show(ui);

                        ui.add_space(16.0);
                        ui.heading("Checkboxes & Radio Buttons");
                        Checkbox::new(&mut self.checkbox_agree)
                            .label("I acknowledge the software terms & conditions")
                            .palette(&p)
                            .show(ui);
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            RadioButton::new(1, &mut self.radio_choice)
                                .label("Standard Route")
                                .palette(&p)
                                .show(ui);
                            RadioButton::new(2, &mut self.radio_choice)
                                .label("Express Route")
                                .palette(&p)
                                .show(ui);
                            RadioButton::new(3, &mut self.radio_choice)
                                .label("Custom Route")
                                .palette(&p)
                                .show(ui);
                        });
                    }
                    GalleryTab::Inputs => {
                        ui.heading("Numeric Sliders");
                        ui.label("Interactive drag with spring-scaling knob thumb on hover");
                        ui.add_space(8.0);

                        Slider::new(&mut self.slider_volume, 0.0..=100.0)
                            .label("Speaker Volume")
                            .suffix(" %")
                            .palette(&p)
                            .show(ui);
                        ui.add_space(10.0);

                        Slider::new(&mut self.slider_brightness, 0.0..=100.0)
                            .label("Display Brightness")
                            .suffix(" nits")
                            .palette(&p)
                            .show(ui);

                        ui.add_space(16.0);
                        ui.heading("Text Inputs & Search");
                        TextInput::new(&mut self.search_query)
                            .placeholder("Search components, tokens, or symbols...")
                            .icon("🔍")
                            .clear_button(true)
                            .palette(&p)
                            .show(ui);
                        ui.add_space(8.0);

                        TextInput::new(&mut self.password_input)
                            .placeholder("Enter secret token...")
                            .icon("🔒")
                            .password(true)
                            .palette(&p)
                            .show(ui);
                    }
                    GalleryTab::Surfaces => {
                        ui.heading("Cards & Surfaces");
                        ui.label("Theme-styled containers with optional spring elevation lift");
                        ui.add_space(8.0);

                        Card::new()
                            .title("📦 Package Manager")
                            .subtitle("3 dependencies up to date")
                            .interactive(true)
                            .palette(&p)
                            .show(ui, |ui| {
                                ui.label("egui-layout v0.1.0");
                                ui.label("egui-spring v0.1.0");
                                ui.label("spring-core v0.1.0");
                            });

                        ui.add_space(16.0);
                        ui.heading("Spring Progress Bars");
                        ui.label("Physical catchup animation with smooth acceleration");
                        ui.add_space(8.0);

                        ProgressBar::new(self.progress_val)
                            .label("System Sync Status")
                            .show_percentage(true)
                            .variant(ProgressVariant::Accent)
                            .palette(&p)
                            .show(ui);
                        ui.add_space(6.0);

                        ProgressBar::new(self.progress_val * 0.8)
                            .variant(ProgressVariant::Success)
                            .palette(&p)
                            .show(ui);
                        ui.add_space(6.0);

                        ProgressBar::new(self.progress_val * 0.5)
                            .variant(ProgressVariant::Warning)
                            .palette(&p)
                            .show(ui);

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if Button::new("Set 25%").small().palette(&p).show(ui).clicked() {
                                self.progress_val = 0.25;
                            }
                            if Button::new("Set 50%").small().palette(&p).show(ui).clicked() {
                                self.progress_val = 0.50;
                            }
                            if Button::new("Set 75%").small().palette(&p).show(ui).clicked() {
                                self.progress_val = 0.75;
                            }
                            if Button::new("Set 100%").small().palette(&p).show(ui).clicked() {
                                self.progress_val = 1.00;
                            }
                        });
                    }
                    GalleryTab::Dashboard => {
                        ui.horizontal(|ui| {
                            ui.heading("🚀 Composite Control Center");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                Badge::new("All Systems Go").success().dot(true).palette(&p).show(ui);
                                Badge::new("Live Telemetry").accent().palette(&p).show(ui);
                            });
                        });
                        ui.label("Every widget composed together in a realistic dashboard");
                        ui.add_space(10.0);

                        // Search & Action Header
                        Card::new().palette(&p).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                TextInput::new(&mut self.search_query)
                                    .placeholder("Filter cluster telemetry, endpoints, or keys...")
                                    .icon("🔍")
                                    .clear_button(true)
                                    .palette(&p)
                                    .show(ui);

                                if Button::new("Scan Cluster").secondary().icon("📡").palette(&p).show(ui).clicked() {
                                    self.progress_val = 0.88;
                                }
                            });
                        });

                        ui.add_space(10.0);

                        ui.columns(2, |cols| {
                            cols[0].vertical(|ui| {
                                Card::new()
                                    .title("⚙️ Engine Parameters")
                                    .subtitle("Hardware acceleration & tunnels")
                                    .palette(&p)
                                    .interactive(true)
                                    .show(ui, |ui| {
                                        Switch::new(&mut self.switch_wifi).label("Wi-Fi Direct Uplink").palette(&p).show(ui);
                                        ui.add_space(6.0);
                                        Switch::new(&mut self.switch_dark_mode).label("High Contrast Mode").palette(&p).show(ui);
                                        ui.add_space(8.0);
                                        Slider::new(&mut self.slider_volume, 0.0..=100.0)
                                            .label("Volume Level")
                                            .suffix(" %")
                                            .palette(&p)
                                            .show(ui);
                                        ui.add_space(8.0);
                                        ProgressBar::new(self.progress_val)
                                            .label("Pipeline Buffer Sync")
                                            .show_percentage(true)
                                            .palette(&p)
                                            .show(ui);
                                    });
                            });

                            cols[1].vertical(|ui| {
                                Card::new()
                                    .title("🔒 Security & Deployment")
                                    .subtitle("Environment targeting & credentials")
                                    .palette(&p)
                                    .interactive(true)
                                    .show(ui, |ui| {
                                        TextInput::new(&mut self.password_input)
                                            .placeholder("Authorization API token...")
                                            .icon("🔑")
                                            .password(true)
                                            .palette(&p)
                                            .show(ui);
                                        ui.add_space(8.0);
                                        ui.horizontal(|ui| {
                                            RadioButton::new(1, &mut self.radio_choice).label("Dev").palette(&p).show(ui);
                                            RadioButton::new(2, &mut self.radio_choice).label("Staging").palette(&p).show(ui);
                                            RadioButton::new(3, &mut self.radio_choice).label("Prod").palette(&p).show(ui);
                                        });
                                        ui.add_space(8.0);
                                        Checkbox::new(&mut self.checkbox_agree)
                                            .label("Enable multi-region replication")
                                            .palette(&p)
                                            .show(ui);
                                        ui.add_space(10.0);
                                        ui.horizontal_wrapped(|ui| {
                                            if Button::new("Deploy").primary().icon("🚀").palette(&p).show(ui).clicked() {
                                                self.progress_val = 1.0;
                                            }
                                            if Button::new("Purge").warning().icon("🧹").palette(&p).show(ui).clicked() {
                                                self.progress_val = 0.15;
                                            }
                                            Button::new("Halt").danger().icon("🛑").palette(&p).show(ui);
                                        });
                                    });
                            });
                        });
                    }
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 680.0])
            .with_title("egui-widgets Component Gallery"),
        ..Default::default()
    };
    eframe::run_native(
        "egui-widgets-gallery",
        options,
        Box::new(|_cc| Box::new(WidgetGalleryApp::default())),
    )
}
