use super::{
    layout::*, window_manager::WindowManager, window_selector::add_window_selector_labels,
};
use std::{cell::RefCell, rc::Rc};

pub fn configure(wm: &mut WindowManager) {
    for i in 1..=9 {
        // wm.add_workspace(&format!("{}", i), layouts(&wm.window_selector_running));
        wm.add_workspace(&format!("{}", i), layouts());
    }
}

// fn layouts(window_selector_is_running: &Rc<RefCell<bool>>) -> Vec<LayoutRoot> {
fn layouts() -> Vec<LayoutRoot> {
    vec![
        // standard_layout_root(
        //     "monad_tall_right_stack",
        //     monad_stack(Direction::Decreasing, Axis::X, 0.75, 1),
        // ),
        standard_layout_root(
            "monad_tall_right",
            monad(Direction::Decreasing, Axis::X, 0.75, 1),
        ),
        standard_layout_root(
            "monad_wide_top",
            monad(Direction::Increasing, Axis::Y, 0.75, 1),
        ),
    ]
}

fn standard_layout_root<A: Layout + 'static>(name: &str, child: A) -> LayoutRoot {
    root(
        name,
        add_window_selector_labels(avoid_struts(ignore_some_windows(add_gaps(
            5,
            5,
            add_focus_border(1, (0, 255, 0), child),
        )))),
    )
}
