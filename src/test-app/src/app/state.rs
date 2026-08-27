use egui_themes::ThemeState;
use crate::scenes::nav_stack_demo::NavStackDemoState;
use crate::scenes::spring_demo::SpringDemoState;
use crate::scenes::vim_nav_demo::VimNavDemoState;
use crate::scenes::widgets_demo::WidgetsDemoState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ActiveScene {
    Layout,
    Spring,
    VimNav,
    NavStack,
    Theme,
    #[default]
    Widgets,
    Combined,
}

#[derive(Default)]
pub struct TestAppState {
    pub active_scene: ActiveScene,
    pub spring_demo: SpringDemoState,
    pub vim_nav_demo: VimNavDemoState,
    pub nav_stack_demo: NavStackDemoState,
    pub widgets_demo: WidgetsDemoState,
    pub theme: ThemeState,
    pub cursor_autohide: egui_vim_nav::CursorAutohide,
}
