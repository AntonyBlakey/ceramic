use super::{layout, window};

pub struct Workspace {
    pub name: String,
    pub layout: Box<layout::Layout>,
    pub saved_windows: Vec<window::Id>,
    pub windows: Vec<window::Id>,
}

impl Workspace {
    pub fn add_window(&mut self, window: &window::Window) {
        self.windows.push(window.id());
    }
    pub fn remove_window(&mut self, window: &window::Window) {
        self.windows.remove_item(&window.id());
    }
}
