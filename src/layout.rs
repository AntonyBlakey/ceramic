pub mod add_border;
pub mod add_gaps;
pub mod add_window_selector_labels;
pub mod floating_layout;
pub mod grid_layout;
pub mod layout_root;
pub mod linear_layout;
pub mod monad_layout;
pub mod split_layout;
pub mod stack_layout;

use super::{artist::Artist, commands::Commands, connection::connection, window_data::WindowData};
use std::collections::HashSet;

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub const fn new(x: i16, y: i16) -> Position {
        Position { x, y }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const fn new(width: u16, height: u16) -> Size {
        Size { width, height }
    }

    pub fn largest_axis(&self) -> Axis {
        if self.width > self.height {
            Axis::X
        } else {
            Axis::Y
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub struct Bounds {
    pub origin: Position,
    pub size: Size,
}

impl Bounds {
    pub const fn new(x: i16, y: i16, width: u16, height: u16) -> Bounds {
        Bounds {
            origin: Position::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn max_x(&self) -> i16 {
        (self.origin.x as i32 + self.size.width as i32) as i16
    }

    pub fn max_y(&self) -> i16 {
        (self.origin.y as i32 + self.size.height as i32) as i16
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Axis {
    X,
    Y,
}

impl Axis {
    // pub fn extract_origin(&self, rect: &Bounds) -> i16 {
    //     match self {
    //         Axis::X => rect.origin.x,
    //         Axis::Y => rect.origin.y,
    //     }
    // }

    // pub fn extract_size(&self, rect: &Bounds) -> u16 {
    //     match self {
    //         Axis::X => rect.size.width,
    //         Axis::Y => rect.size.height,
    //     }
    // }

    pub fn orthogonal(&self) -> Self {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Direction {
    Increasing,
    Decreasing,
}

pub trait Layout: Commands {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<Artist>>);
}

pub fn clear_window_order(windows: &mut [WindowData]) {
    for window in windows.iter_mut() {
        window.order = None;
    }
}

pub fn compute_window_order(windows: &mut [WindowData]) {
    // unordered -> heads to top, others to bottom

    let mut order = 1000;
    for window in windows.iter_mut().rev().take_while(|w| w.order.is_none()) {
        window.order = Some(order);
        order -= 1;
    }

    let mut order = -1000;
    for window in windows.iter_mut().filter(|w| w.order.is_none()) {
        window.order = Some(order);
        order += 1;
    }

    // focus -> to absolute top

    let connection = connection();
    let focused_window = xcb::get_input_focus(&connection)
        .get_reply()
        .unwrap()
        .focus();

    if let Some(window) = windows.iter_mut().find(|w| w.window() == focused_window) {
        window.order = Some(2000);
    }

    // normalize order as [0.. 

    let mut sorted_windows = windows.iter_mut().collect::<Vec<_>>();
    sorted_windows.sort_by(|a, b| a.order.cmp(&b.order));
    let mut order = 0;
    for window in sorted_windows {
        window.order = Some(order);
        order += 1;
    }
}

//
//------------------------------------------------------------------
//

// pub type BoxedLayoutPredicate = Box<Fn(&Bounds, &[&window::Window]) -> bool>;

// #[derive(Clone)]
// pub struct DynamicLayout<A, B> {
//     pub predicate: Rc<BoxedLayoutPredicate>,
//     pub children: (A, B),
// }

// impl<A: Layout, B: Layout> DynamicLayout<A, B> {
//     pub fn make(predicate: BoxedLayoutPredicate, children: (A, B)) -> DynamicLayout<A, B> {
//         DynamicLayout {
//             predicate: Rc::new(predicate),
//             children,
//         }
//     }

//     pub fn switch_on_window_count(count: usize, children: (A, B)) -> DynamicLayout<A, B> {
//         Self::make(Box::new(move |_, windows| windows.len() <= count), children)
//     }

//     pub fn switch_on_available_size(
//         axis: Axis,
//         size_break: u16,
//         children: (A, B),
//     ) -> DynamicLayout<A, B> {
//         Self::make(
//             Box::new(move |rect, _| axis.extract_size(rect) < size_break),
//             children,
//         )
//     }

//     pub fn switch_on_prorata_size(
//         axis: Axis,
//         size_break: u16,
//         children: (A, B),
//     ) -> DynamicLayout<A, B> {
//         Self::make(
//             Box::new(move |rect, windows| {
//                 (axis.extract_size(rect) / windows.len() as u16) < size_break
//             }),
//             children,
//         )
//     }
// }

// impl<A: Layout, B: Layout> Layout for DynamicLayout<A, B> {
//     fn layout(&self, rect: &Bounds, windows: &[&window::Window], artists: &mut Vec<Box<Artist>>) {
//         if (self.predicate)(rect, windows) {
//             self.children.0.layout(rect, windows)
//         } else {
//             self.children.1.layout(rect, windows)
//         }
//     }
// }

//
//------------------------------------------------------------------
//

// pub type BoxedWindowPredicate = Box<Fn(&Bounds, usize, &window::Window) -> bool>;

// #[derive(Clone)]
// pub struct PredicateSelector<A> {
//     pub predicate: Rc<BoxedWindowPredicate>,
//     pub child: A,
// }

// impl<A: Layout> PredicateSelector<A> {
//     pub fn passing(predicate: BoxedWindowPredicate, child: A) -> PredicateSelector<A> {
//         PredicateSelector {
//             predicate: Rc::new(predicate),
//             child,
//         }
//     }

//     pub fn failing(test: BoxedWindowPredicate, child: A) -> PredicateSelector<A> {
//         Self::passing(
//             Box::new(move |rect, index, window| !test(rect, index, window)),
//             child,
//         )
//     }

//     pub fn first(count: usize, child: A) -> PredicateSelector<A> {
//         Self::passing(Box::new(move |_, index, _| index < count), child)
//     }

//     pub fn all_but_first(count: usize, child: A) -> PredicateSelector<A> {
//         Self::passing(Box::new(move |_, index, _| index >= count), child)
//     }
// }

// impl<A: Layout> Layout for PredicateSelector<A> {
//     fn layout(&self, rect: &Bounds, windows: &[&window::Window], artists: &mut Vec<Box<Artist>>) {
//         let filtered_windows: Vec<&window::Window> = windows
//             .iter()
//             .enumerate()
//             .filter(|&(i, w)| (self.predicate)(rect, i, w))
//             .map(|(_, &w)| w)
//             .collect();
//         (self.child).layout(rect, &filtered_windows)
//     }
// }
