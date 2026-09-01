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

pub struct TestAppState {
    pub active_scene: ActiveScene,
    pub spring_demo: SpringDemoState,
    pub vim_nav_demo: VimNavDemoState,
    pub nav_stack_demo: NavStackDemoState,
    pub widgets_demo: WidgetsDemoState,
    pub theme: ThemeState,
    pub cursor_autohide: egui_vim_nav::CursorAutohide,
    pub sidebar_expanded: bool,
    pub inspector_expanded: bool,
    pub ribbon_expanded: bool,
    pub combined_motion: spring_core::MotionPhysics,
}

impl Default for TestAppState {
    fn default() -> Self {
        Self {
            active_scene: Default::default(),
            spring_demo: Default::default(),
            vim_nav_demo: Default::default(),
            nav_stack_demo: Default::default(),
            widgets_demo: Default::default(),
            theme: Default::default(),
            cursor_autohide: Default::default(),
            sidebar_expanded: true,
            inspector_expanded: true,
            ribbon_expanded: true,
            combined_motion: spring_core::MotionPhysics::Responsive,
        }
    }
}
