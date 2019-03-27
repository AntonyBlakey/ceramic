use super::{layout::LayoutStep, window, window::Window};

pub struct Workspace {
    pub name: String,
    pub layout: LayoutStep,
    pub windows: Vec<window::Id>,
}

impl Workspace {
    pub fn add_window(&mut self, window: &Window) {
        self.windows.push(window.id());
    }
    pub fn remove_window(&mut self, window: &Window) {
        self.windows.remove_item(&window.id());
    }
}
