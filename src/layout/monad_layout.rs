use crate::layout::*;

pub fn new_linear(direction: Direction, axis: Axis, ratio: f64, count: usize) -> Box<Layout> {
    split_layout::new(
        direction,
        axis,
        ratio,
        count,
        linear_layout::new(direction, axis),
        linear_layout::new(Direction::Increasing, axis.orthogonal()),
    )
}

pub fn new_stack(direction: Direction, axis: Axis, ratio: f64, count: usize) -> Box<Layout> {
    split_layout::new(
        direction,
        axis,
        ratio,
        count,
        linear_layout::new(direction, axis),
        stack_layout::new(),
    )
}
