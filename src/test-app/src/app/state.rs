use crate::scenes::spring_demo::SpringDemoState;
use crate::scenes::vim_nav_demo::VimNavDemoState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ActiveScene {
    #[default]
    Layout,
    Spring,
    VimNav,
    NavStack,
    Theme,
    Combined,
}

#[derive(Default)]
pub struct TestAppState {
    pub active_scene: ActiveScene,
    pub spring_demo: SpringDemoState,
    pub vim_nav_demo: VimNavDemoState,
}
