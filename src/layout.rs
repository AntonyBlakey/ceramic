use super::{artist, connection::*, window};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
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

pub fn root<A: Default + Layout>(child: A) -> LayoutRoot<A> {
    LayoutRoot {
        child: child,
        ..Default::default()
    }
}

pub fn avoid_struts<A: Default + Layout>(child: A) -> AvoidStruts<A> {
    AvoidStruts { child: child }
}

pub fn ignore_some_windows<A: Default + Layout>(child: A) -> IgnoreSomeWindows<A> {
    IgnoreSomeWindows { child: child }
}
pub fn add_gaps<A: Default + Layout>(screen_gap: u16, window_gap: u16, child: A) -> AddGaps<A> {
    AddGaps {
        screen_gap,
        window_gap,
        child: child,
    }
}

pub fn add_focus_border<A: Default + Layout>(
    width: u16,
    color: (u8, u8, u8),
    child: A,
) -> AddFocusBorder<A> {
    AddFocusBorder {
        width,
        color,
        child: child,
    }
}
pub fn grid() -> GridLayout {
    Default::default()
}

pub fn linear(direction: Direction, axis: Axis) -> LinearLayout {
    LinearLayout { direction, axis }
}

pub fn split<A: Default + Layout, B: Default + Layout>(
    direction: Direction,
    axis: Axis,
    ratio: f64,
    count: usize,
    child_a: A,
    child_b: B,
) -> SplitLayout<A, B> {
    SplitLayout {
        direction,
        axis,
        ratio,
        count,
        children: (child_a, child_b),
    }
}

pub fn monad(
    direction: Direction,
    axis: Axis,
    ratio: f64,
    count: usize,
) -> SplitLayout<LinearLayout, LinearLayout> {
    split(
        direction,
        axis,
        ratio,
        count,
        linear(direction, axis),
        linear(Direction::Increasing, axis.orthogonal()),
    )
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct LayoutRoot<A: Default> {
    child: A,
}

impl<A: Default + Layout> Layout for LayoutRoot<A> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        self.child.layout(rect, &windows)
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct IgnoreSomeWindows<A: Default> {
    child: A,
}

impl<A: Default + Layout> Layout for IgnoreSomeWindows<A> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        self.child.layout(
            rect,
            &windows
                .iter()
                .filter(|w| {
                    let window_type = get_atoms_property(w.id(), *ATOM__NET_WM_WINDOW_TYPE);
                    !window_type.contains(&*ATOM__NET_WM_WINDOW_TYPE_DOCK)
                })
                .map(|&w| w)
                .collect::<Vec<_>>(),
        )
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct AvoidStruts<A: Default> {
    child: A,
}

impl<A: Default + Layout> Layout for AvoidStruts<A> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        let mut r = *rect;

        for window in windows {
            let struts = get_cardinals_property(window.id(), *ATOM__NET_WM_STRUT);
            if struts.len() == 4 {
                let left = struts[0] as u16;
                let right = struts[1] as u16;
                let top = struts[2] as u16;
                let bottom = struts[3] as u16;
                r.origin.x += left;
                r.size.width -= left + right;
                r.origin.y += top;
                r.size.height -= top + bottom;
            }
        }

        self.child.layout(&r, &windows)
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone, Default)]
pub struct AddGaps<A: Default> {
    pub screen_gap: u16,
    pub window_gap: u16,
    child: A,
}

impl<A: Default + Layout> Layout for AddGaps<A> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        let mut r = *rect;

        r.origin.x += self.screen_gap;
        r.origin.y += self.screen_gap;
        r.size.width -= 2 * self.screen_gap;
        r.size.height -= 2 * self.screen_gap;
        let mut actions = self.child.layout(&r, &windows);

        for a in &mut actions {
            match a {
                Action::Position {
                    id: _,
                    rect,
                    border_width: _,
                    border_color: _,
                } => {
                    rect.origin.x += self.window_gap;
                    rect.origin.y += self.window_gap;
                    rect.size.width -= 2 * self.window_gap;
                    rect.size.height -= 2 * self.window_gap;
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
pub struct AddFocusBorder<A: Default> {
    pub width: u16,
    pub color: (u8, u8, u8),
    child: A,
}

impl<A: Default + Layout> Layout for AddFocusBorder<A> {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        let mut actions = self.child.layout(rect, windows);

        let focused_window = xcb::get_input_focus(&connection())
            .get_reply()
            .unwrap()
            .focus();

        let red = self.color.0 as u32;
        let green = self.color.1 as u32;
        let blue = self.color.2 as u32;
        // NOTE: without parens this doesn't do what you expect!
        let color = (red << 16) + (green << 8) + blue;

        for a in &mut actions {
            match a {
                Action::Position {
                    id,
                    rect: _,
                    border_width,
                    border_color,
                } if *id == focused_window => {
                    *border_width = self.width;
                    *border_color = color;
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

impl Layout for LinearLayout {
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        if windows.is_empty() {
            return Default::default();
        }

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
