#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ActiveScene {
    #[default]
    Layout,
    Spring,
    VimNav,
    Theme,
    Combined,
}

#[derive(Default)]
pub struct TestAppState {
    pub active_scene: ActiveScene,
}
