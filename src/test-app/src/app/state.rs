use crate::scenes::spring_demo::SpringDemoState;

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
}
