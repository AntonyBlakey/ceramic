mod default;
mod user;

use super::{layout::*, workspace::Workspace};

pub trait ConfigurationProvider {
    fn root(&self) -> &ConfigurationProvider;

    fn workspaces(&self) -> Vec<Workspace> {
        default::workspaces(self.root())
    }

    fn layouts(&self) -> Vec<layout_root::LayoutRoot> {
        default::layouts(self.root())
    }

    fn layout_root(&self, name: &str, child: Box<Layout>) -> layout_root::LayoutRoot {
        default::layout_root(self.root(), name, child)
    }

    fn classify_window(
        &self,
        window: xcb::Window,
        wm_instance_name: Option<&str>,
        wm_class_name: Option<&str>,
        net_wm_type: &[xcb::Atom],
        net_wm_state: &[xcb::Atom],
        wm_transient_for: Option<xcb::Window>,
    ) -> Option<bool> {
        default::classify_window(
            self.root(),
            window,
            wm_instance_name,
            wm_class_name,
            net_wm_type,
            net_wm_state,
            wm_transient_for,
        )
    }
}

pub struct Configuration {}

impl Configuration {
    pub fn new() -> Box<ConfigurationProvider> {
        Box::new(Self {})
    }
}
