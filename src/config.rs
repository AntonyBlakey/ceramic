use super::{layout::*, window_manager::WindowManager};

pub fn configure(wm: &mut WindowManager) {
    for i in 1..=3 {
        wm.add_workspace(&format!("{}", i), layouts());
    }
}

fn layouts() -> Vec<layout_root::LayoutRoot> {
    vec![
        // standard_layout_root(
        //     "monad_tall_right_stack",
        //     monad_stack(Direction::Decreasing, Axis::X, 0.75, 1),
        // ),
        standard_layout_root(
            "monad_tall_right",
            monad_layout::new_linear(Direction::Decreasing, Axis::X, 0.75, 1),
        ),
        standard_layout_root(
            "monad_wide_top",
            monad_layout::new_linear(Direction::Increasing, Axis::Y, 0.75, 1),
        ),
    ]
}

fn standard_layout_root<A: Layout + 'static>(name: &str, child: A) -> layout_root::LayoutRoot {
    layout_root::new(
        name,
        add_window_selector_labels::new(add_focus_border::new(
            1,
            (0, 255, 0),
            floating_layout::new(add_gaps::new(5, 5, child)),
        )),
    )
}
