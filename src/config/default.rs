use crate::{config::*, connection::*, layout::*, workspace::Workspace};

pub fn workspaces(configuration: &ConfigurationProvider) -> Vec<Workspace> {
    ["1", "2", "3", "4", "5", "6", "7", "8", "9"]
        .iter()
        .map(|name| Workspace::new(name, configuration.layouts()))
        .collect()
}

pub fn layouts(configuration: &ConfigurationProvider) -> Vec<layout_root::LayoutRoot> {
    vec![
        configuration.layout_root(
            "monad_tall_right",
            monad_layout::new_linear(Direction::Decreasing, Axis::X, 0.75, 1),
        ),
        configuration.layout_root(
            "monad_wide_top",
            monad_layout::new_linear(Direction::Increasing, Axis::Y, 0.75, 1),
        ),
        configuration.layout_root(
            "monad_stacked",
            monad_layout::new_stack(Direction::Decreasing, Axis::X, 0.75, 1),
        ),
        configuration.layout_root("stacked", stack_layout::new()),
    ]
}

pub fn layout_root(
    _configuration: &ConfigurationProvider,
    name: &str,
    child: Box<Layout>,
) -> layout_root::LayoutRoot {
    layout_root::new(
        name,
        add_window_selector_labels::new(add_border::new(
            1,
            (127, 127, 127),
            (0, 255, 0),
            floating_layout::new(add_gaps::new(5, 5, child)),
        )),
    )
}

pub fn classify_window(
    _configuration: &ConfigurationProvider,
    _window: xcb::Window,
    wm_instance_name: Option<&str>,
    _wm_class_name: Option<&str>,
    net_wm_type: &[xcb::Atom],
    net_wm_state: &[xcb::Atom],
    wm_transient_for: Option<xcb::Window>,
) -> Option<bool> {
    // TODO: override_redirect

    if let Some(_owner) = wm_transient_for {
        // TODO: is this really transient in the sense that we mean?
        return Some(true);
    }

    if net_wm_type.is_empty() {
        if net_wm_state.contains(&*ATOM__NET_WM_STATE_ABOVE) {
            Some(true)
        } else {
            if wm_instance_name.is_none() {
                Some(true)
            } else {
                Some(false)
            }
        }
    } else if net_wm_type.contains(&*ATOM__NET_WM_WINDOW_TYPE_NORMAL) {
        Some(false)
    } else if net_wm_type.contains(&*ATOM__NET_WM_WINDOW_TYPE_DIALOG) {
        Some(true)
    } else if net_wm_type.contains(&*ATOM__NET_WM_WINDOW_TYPE_SPLASH) {
        Some(true)
    } else {
        None
    }
}
