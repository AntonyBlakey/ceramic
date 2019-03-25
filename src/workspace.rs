use super::{layout::LayoutAlgorithm, window::Window};
use std::rc::Rc;

pub struct Workspace {
    pub name: String,
    pub layout_algorithm: Rc<LayoutAlgorithm>,
    pub windows: Vec<xcb::Window>,
}

impl Workspace {
    pub fn add_window(&mut self, window: &Window) {
        self.windows.push(window.id());
    }
    pub fn remove_window(&mut self, window: &Window) {
        self.windows.remove_item(&window.id());
    }
}
