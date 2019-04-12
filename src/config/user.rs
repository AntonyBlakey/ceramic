use super::default;
use crate::config::*;

impl ConfigurationProvider for Configuration {
    // Must be provided - there is no default trait implementation
    fn root(&self) -> &ConfigurationProvider {
        self
    }

    fn classify_window(
        &self,
        window: xcb::Window,
        wm_instance_name: Option<&str>,
        wm_class_name: Option<&str>,
        net_wm_type: &[xcb::Atom],
        net_wm_state: &[xcb::Atom],
        wm_transient_for: Option<xcb::Window>,
    ) -> Option<WindowType> {
        if Some("St80") == wm_class_name {
            return Some(WindowType::FLOATING);
        }

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
