use super::{artist, window, window_manager};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

impl Default for Axis {
    fn default() -> Self {
        Axis::X
    }
}

impl Axis {
    pub fn extract_origin<T>(&self, rect: &euclid::Rect<T>) -> T
    where
        T: Copy,
    {
        match self {
            Axis::X => rect.origin.x,
            Axis::Y => rect.origin.y,
        }
    }
    pub fn extract_size<T>(&self, rect: &euclid::Rect<T>) -> T
    where
        T: Copy,
    {
        match self {
            Axis::X => rect.size.width,
            Axis::Y => rect.size.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Increasing,
    Decreasing,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Increasing
    }
}

pub type LayoutRect = euclid::Rect<u16>;

pub trait Layout {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action>;
}

pub enum Action {
    Draw {
        artist: Rc<artist::Artist>,
        rect: LayoutRect,
    },
    Position {
        id: window::Id,
        rect: LayoutRect,
        border_width: u16,
        border_color: u32,
    },
}

pub fn root<A: Default + Layout>() -> LayoutRoot<A> {
    Default::default()
}

pub fn spacing<A: Default + Layout>() -> SpacingLayout<A> {
    Default::default()
}

pub fn focus_border<A: Default + Layout>() -> AddFocusBorder<A> {
    Default::default()
}

pub fn grid() -> GridLayout {
    Default::default()
}

pub fn linear() -> LinearLayout {
    Default::default()
}

pub fn split<A: Default + Layout, B: Default + Layout>() -> SplitLayout<A, B> {
    Default::default()
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct SpacingLayout<A: Default> {
    screen_gap: u16,
    window_gap: u16,
    child: A,
}

impl<A: Default> SpacingLayout<A> {
    pub fn set_screen_gap(mut self, screen_gap: u16) -> Self {
        self.screen_gap = screen_gap;
        self
    }
    pub fn set_window_gap(mut self, window_gap: u16) -> Self {
        self.screen_gap = window_gap;
        self
    }
    pub fn set_child(mut self, child: A) -> Self {
        self.child = child;
        self
    }
}

impl<A: Default + Layout> Layout for SpacingLayout<A> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        let mut r = *rect;
        r.origin.x += self.screen_gap;
        r.origin.y += self.screen_gap;
        r.size.width -= 2 * self.screen_gap;
        r.size.height -= 2 * self.screen_gap;
        let mut actions = self.child.layout(&r, windows);
        for a in &mut actions {
            match a {
                Action::Position {
                    id: _,
                    rect: r,
                    border_width: _,
                    border_color: _,
                } => {
                    r.origin.x += self.window_gap;
                    r.origin.y += self.window_gap;
                    r.size.width -= 2 * self.window_gap;
                    r.size.height -= 2 * self.window_gap;
                }
                _ => {}
            }
        }
        actions
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct LayoutRoot<A: Default> {
    commands: HashMap<String, Rc<Box<Fn(&mut window_manager::WindowManager, &mut A)>>>,
    child: A,
}

impl<A: Default> LayoutRoot<A> {
    pub fn set_child(mut self, child: A) -> Self {
        self.child = child;
        self
    }
    // pub fn make(child: A) -> LayoutRoot<A> {
    //     LayoutRoot {
    //         child,
    //         commands: Default::default(),
    //     }
    // }
    // pub fn add_command<F: Fn(&mut window_manager::WindowManager, &mut A) + 'static>(
    //     &mut self,
    //     name: &'static str,
    //     f: F,
    // ) {
    //     self.commands
    //         .insert(String::from(name), Rc::new(Box::new(f)));
    // }
}

impl<A: Default + Layout> Layout for LayoutRoot<A> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        self.child.layout(rect, windows)
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct AddFocusBorder<A: Default> {
    width: u16,
    color: u32,
    child: A,
}

impl<A: Default> AddFocusBorder<A> {
    pub fn set_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }
    pub fn set_color(mut self, red: u8, green: u8, blue: u8) -> Self {
        self.color = (red << 16 & green << 8 + blue).into();
        self
    }
    pub fn set_child(mut self, child: A) -> Self {
        self.child = child;
        self
    }
    // pub fn make(width: u16, color: u32, child: A) -> AddFocusBorder<A> {
    //     AddFocusBorder {
    //         width,
    //         color,
    //         child,
    //     }
    // }
}

impl<A: Default + Layout> Layout for AddFocusBorder<A> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        let focused_window = xcb::xproto::get_input_focus(&window_manager::connection())
            .get_reply()
            .unwrap()
            .focus();
        let mut actions = self.child.layout(rect, windows);
        for a in &mut actions {
            match a {
                Action::Position {
                    id,
                    rect: _,
                    border_width,
                    border_color,
                } if *id == focused_window => {
                    *border_width = self.width;
                    *border_color = self.color;
                }
                _ => {}
            }
        }
        actions
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct GridLayout {}

impl Layout for GridLayout {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        if windows.is_empty() {
            return Default::default();
        }

        let columns = (windows.len() as f64).sqrt().ceil() as u16;
        let rows = (windows.len() as u16 + columns - 1) / columns;

        let screen_gap = 5;
        let window_gap = 5;

        let cell_width = (rect.size.width - screen_gap * 2) / columns;
        let cell_height = (rect.size.height - screen_gap * 2) / rows;

        let mut row = 0;
        let mut column = 0;

        let w = cell_width - 2 * window_gap;
        let h = cell_height - 2 * window_gap;
        windows
            .iter()
            .map(|window| {
                let x = rect.origin.x + screen_gap + cell_width * column + window_gap;
                let y = rect.origin.y + screen_gap + cell_height * row + window_gap;
                column += 1;
                if column == columns {
                    column = 0;
                    row += 1;
                }
                Action::Position {
                    id: window.id(),
                    rect: euclid::rect(x, y, w, h),
                    border_width: 0,
                    border_color: 0,
                }
            })
            .collect()
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct SplitLayout<A, B> {
    axis: Axis,
    direction: Direction,
    ratio: f64,
    count: usize,
    children: (A, B),
}

impl<A: Default + Layout, B: Default + Layout> SplitLayout<A, B> {
    pub fn set_axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
    pub fn set_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    pub fn set_ratio(mut self, ratio: f64) -> Self {
        self.ratio = ratio;
        self
    }
    pub fn set_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
    pub fn set_children(mut self, child_a: A, child_b: B) -> Self {
        self.children = (child_a, child_b);
        self
    }

    // pub fn make(width: u16, color: u32, child: A) -> AddFocusBorder<A> {
    // pub fn make(
    //     axis: Axis,
    //     direction: Direction,
    //     ratio: f64,
    //     count: usize,
    //     children: (A, B),
    // ) -> SplitLayout<A, B> {
    //     SplitLayout {
    //         axis,
    //         direction,
    //         ratio,
    //         count,
    //         children,
    //     }
    // }

    // pub fn make_left_to_right(ratio: f64, count: usize, children: (A, B)) -> SplitLayout<A, B> {
    //     Self::make(Axis::X, Direction::Increasing, ratio, count, children)
    // }

    // pub fn make_right_to_left(ratio: f64, count: usize, children: (A, B)) -> SplitLayout<A, B> {
    //     Self::make(Axis::X, Direction::Decreasing, ratio, count, children)
    // }

    // pub fn make_top_to_bottom(ratio: f64, count: usize, children: (A, B)) -> SplitLayout<A, B> {
    //     Self::make(Axis::Y, Direction::Increasing, ratio, count, children)
    // }

    // pub fn make_bottom_to_top(ratio: f64, count: usize, children: (A, B)) -> SplitLayout<A, B> {
    //     Self::make(Axis::Y, Direction::Decreasing, ratio, count, children)
    // }
}

impl<A: Layout, B: Layout> Layout for SplitLayout<A, B> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        if windows.is_empty() {
            return Default::default();
        }

        let mut rect_1 = *rect;
        let mut rect_2 = *rect;

        match self.axis {
            Axis::X => {
                rect_1.size.width = (rect.size.width as f64 * self.ratio).floor() as u16;
                rect_2.size.width = rect.size.width - rect_1.size.width;
                match self.direction {
                    Direction::Increasing => rect_2.origin.x = rect_1.max_x(),
                    Direction::Decreasing => rect_1.origin.x = rect_2.max_x(),
                }
            }
            Axis::Y => {
                rect_1.size.height = (rect.size.height as f64 * self.ratio).floor() as u16;
                rect_2.size.height = rect.size.height - rect_1.size.height;
                match self.direction {
                    Direction::Increasing => rect_2.origin.y = rect_1.max_y(),
                    Direction::Decreasing => rect_1.origin.y = rect_2.max_y(),
                }
            }
        }

        if windows.len() > self.count {
            let (w1, w2) = windows.split_at(self.count);
            let mut result = self.children.0.layout(&rect_1, w1);
            result.append(&mut self.children.1.layout(&rect_2, w2));
            result
        } else {
            self.children.0.layout(&rect_1, windows)
        }
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct LinearLayout {
    axis: Axis,
    direction: Direction,
}

impl LinearLayout {
    pub fn set_axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
    pub fn set_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    //     pub fn make(axis: Axis, direction: Direction) -> LinearLayout {
    //         LinearLayout { axis, direction }
    //     }

    //     pub fn make_left_to_right() -> LinearLayout {
    //         Self::make(Axis::X, Direction::Increasing)
    //     }

    //     pub fn make_right_to_left() -> LinearLayout {
    //         Self::make(Axis::X, Direction::Decreasing)
    //     }

    //     pub fn make_top_to_bottom() -> LinearLayout {
    //         Self::make(Axis::Y, Direction::Increasing)
    //     }

    //     pub fn make_bottom_to_top() -> LinearLayout {
    //         Self::make(Axis::Y, Direction::Decreasing)
    //     }
}

impl Layout for LinearLayout {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        let mut result = Vec::with_capacity(windows.len());

        match self.axis {
            Axis::X => {
                let mut r = *rect;
                r.size.width = r.size.width / windows.len() as u16;
                for w in windows {
                    result.push(Action::Position {
                        id: w.id(),
                        rect: r,
                        border_width: 0,
                        border_color: 0,
                    });
                    r.origin.x += r.size.width;
                }
            }
            Axis::Y => {
                let mut r = *rect;
                r.size.height = r.size.height / windows.len() as u16;
                for w in windows {
                    result.push(Action::Position {
                        id: w.id(),
                        rect: r,
                        border_width: 0,
                        border_color: 0,
                    });
                    r.origin.y += r.size.height;
                }
            }
        }

        result
    }
}

//
//------------------------------------------------------------------
//

// pub type BoxedLayoutPredicate = Box<Fn(&LayoutRect, &[&window::Window]) -> bool>;

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
//     fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
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

// pub type BoxedWindowPredicate = Box<Fn(&LayoutRect, usize, &window::Window) -> bool>;

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
//     fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
//         let filtered_windows: Vec<&window::Window> = windows
//             .iter()
//             .enumerate()
//             .filter(|&(i, w)| (self.predicate)(rect, i, w))
//             .map(|(_, &w)| w)
//             .collect();
//         (self.child).layout(rect, &filtered_windows)
//     }
// }
