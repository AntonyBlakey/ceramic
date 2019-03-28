use super::{artist, window};
use std::{clone::Clone, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
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

pub type LayoutRect = euclid::Rect<u16>;

pub trait Layout {
    fn to_layout_step(&self) -> Step;
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action>;
}

pub struct Step(Box<Layout>);

impl std::ops::Deref for Step {
    type Target = Box<Layout>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for Step {
    fn clone(&self) -> Step {
        self.0.to_layout_step()
    }
}

pub enum Action {
    Draw {
        artist: Rc<artist::Artist>,
        rect: LayoutRect,
    },
    Position {
        id: window::Id,
        rect: LayoutRect,
    },
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct SpacingLayout {
    screen_gap: u16,
    window_gap: u16,
    layout: Step,
}

impl SpacingLayout {
    pub fn make(screen_gap: u16, window_gap: u16, layout: Step) -> Step {
        SpacingLayout {
            screen_gap,
            window_gap,
            layout,
        }
        .to_layout_step()
    }
}

impl Layout for SpacingLayout {
    fn to_layout_step(&self) -> Step {
        Step(Box::new(self.clone()))
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        let mut r = *rect;
        r.origin.x += self.screen_gap;
        r.origin.y += self.screen_gap;
        r.size.width -= 2 * self.screen_gap;
        r.size.height -= 2 * self.screen_gap;
        let mut actions = self.layout.layout(&r, windows);
        for a in &mut actions {
            match a {
                Action::Position { id: _, rect: r } => {
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

#[derive(Clone)]
pub struct GridLayout {}

impl GridLayout {
    pub fn make() -> Step {
        GridLayout {}.to_layout_step()
    }
}

impl Layout for GridLayout {
    fn to_layout_step(&self) -> Step {
        Step(Box::new(self.clone()))
    }
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
                }
            })
            .collect()
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct SplitLayout {
    axis: Axis,
    direction: Direction,
    ratio: f64,
    count: usize,
    layout_1: Step,
    layout_2: Step,
}

impl SplitLayout {
    pub fn make(
        axis: Axis,
        direction: Direction,
        ratio: f64,
        count: usize,
        layout_1: Step,
        layout_2: Step,
    ) -> Step {
        SplitLayout {
            axis,
            direction,
            ratio,
            count,
            layout_1,
            layout_2,
        }
        .to_layout_step()
    }

    pub fn make_left_to_right(ratio: f64, count: usize, layout_1: Step, layout_2: Step) -> Step {
        Self::make(
            Axis::X,
            Direction::Increasing,
            ratio,
            count,
            layout_1,
            layout_2,
        )
    }

    pub fn make_right_to_left(ratio: f64, count: usize, layout_1: Step, layout_2: Step) -> Step {
        Self::make(
            Axis::X,
            Direction::Decreasing,
            ratio,
            count,
            layout_1,
            layout_2,
        )
    }

    pub fn make_top_to_bottom(ratio: f64, count: usize, layout_1: Step, layout_2: Step) -> Step {
        Self::make(
            Axis::Y,
            Direction::Increasing,
            ratio,
            count,
            layout_1,
            layout_2,
        )
    }

    pub fn make_bottom_to_top(ratio: f64, count: usize, layout_1: Step, layout_2: Step) -> Step {
        Self::make(
            Axis::Y,
            Direction::Decreasing,
            ratio,
            count,
            layout_1,
            layout_2,
        )
    }
}

impl Layout for SplitLayout {
    fn to_layout_step(&self) -> Step {
        Step(Box::new(self.clone()))
    }
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
            let mut result = self.layout_1.layout(&rect_1, w1);
            result.append(&mut self.layout_2.layout(&rect_2, w2));
            result
        } else {
            self.layout_1.layout(&rect_1, windows)
        }
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct StackedLayout {}

impl StackedLayout {
    pub fn make() -> Step {
        StackedLayout {}.to_layout_step()
    }
}

impl Layout for StackedLayout {
    fn to_layout_step(&self) -> Step {
        Step(Box::new(self.clone()))
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        Default::default()
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct TabbedLayout {}

impl TabbedLayout {
    pub fn make() -> Step {
        TabbedLayout {}.to_layout_step()
    }
}

impl Layout for TabbedLayout {
    fn to_layout_step(&self) -> Step {
        Step(Box::new(self.clone()))
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        Default::default()
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct LinearLayout {
    axis: Axis,
    direction: Direction,
}

impl LinearLayout {
    pub fn make(axis: Axis, direction: Direction) -> Step {
        LinearLayout { axis, direction }.to_layout_step()
    }

    pub fn make_left_to_right() -> Step {
        Self::make(Axis::X, Direction::Increasing)
    }

    pub fn make_right_to_left() -> Step {
        Self::make(Axis::X, Direction::Decreasing)
    }

    pub fn make_top_to_bottom() -> Step {
        Self::make(Axis::Y, Direction::Increasing)
    }

    pub fn make_bottom_to_top() -> Step {
        Self::make(Axis::Y, Direction::Decreasing)
    }
}

impl Layout for LinearLayout {
    fn to_layout_step(&self) -> Step {
        Step(Box::new(self.clone()))
    }
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

pub type LayoutFactory = Fn(&LayoutRect, &[&window::Window]) -> usize;

// Selects a layout dynamically
#[derive(Clone)]
pub struct DynamicLayout {
    layout_factory: Rc<LayoutFactory>,
    algorithms: Vec<Step>,
}

impl DynamicLayout {
    pub fn make(layout_factory: &Rc<LayoutFactory>, algorithms: &[Step]) -> Step {
        DynamicLayout {
            layout_factory: layout_factory.clone(),
            algorithms: algorithms.iter().map(|x| x.clone()).collect(),
        }
        .to_layout_step()
    }

    pub fn switch_on_window_count(count: usize, lsmall: &Step, lbig: &Step) -> Step {
        let layout_factory: Rc<LayoutFactory> = Rc::new(
            move |rect, windows| {
                if windows.len() <= count {
                    0
                } else {
                    1
                }
            },
        );
        Self::make(&layout_factory, &[lsmall.clone(), lbig.clone()])
    }

    pub fn switch_on_available_size(
        axis: Axis,
        size_break: u16,
        lsmall: &Step,
        lbig: &Step,
    ) -> Step {
        let layout_factory: Rc<LayoutFactory> = Rc::new(move |rect, _| {
            if axis.extract_size(rect) < size_break {
                0
            } else {
                1
            }
        });
        Self::make(&layout_factory, &[lsmall.clone(), lbig.clone()])
    }

    pub fn switch_on_prorata_size(axis: Axis, size_break: u16, lsmall: &Step, lbig: &Step) -> Step {
        let layout_factory: Rc<LayoutFactory> = Rc::new(move |rect, windows| {
            if (axis.extract_size(rect) / windows.len() as u16) < size_break {
                0
            } else {
                1
            }
        });
        Self::make(&layout_factory, &[lsmall.clone(), lbig.clone()])
    }
}

impl Layout for DynamicLayout {
    fn to_layout_step(&self) -> Step {
        Step(Box::new(self.clone()))
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        self.algorithms[(self.layout_factory)(rect, windows)].layout(rect, windows)
    }
}

//
//------------------------------------------------------------------
//

pub type BoxedPredicate = Box<Fn(&LayoutRect, usize, &window::Window) -> bool>;

#[derive(Clone)]
pub struct PredicateSelector {
    predicate: Rc<BoxedPredicate>,
    layout: Step,
}

impl PredicateSelector {
    pub fn passing(predicate: BoxedPredicate, layout: Step) -> Step {
        PredicateSelector {
            predicate: Rc::new(predicate),
            layout: layout.clone(),
        }
        .to_layout_step()
    }

    pub fn failing(test: BoxedPredicate, layout: Step) -> Step {
        Self::passing(
            Box::new(move |rect, index, window| !test(rect, index, window)),
            layout,
        )
    }

    pub fn first(count: usize, layout: Step) -> Step {
        Self::passing(Box::new(move |_, index, _| index < count), layout)
    }

    pub fn all_but_first(count: usize, layout: Step) -> Step {
        Self::passing(Box::new(move |_, index, _| index >= count), layout)
    }
}

impl Layout for PredicateSelector {
    fn to_layout_step(&self) -> Step {
        Step(Box::new(self.clone()))
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&window::Window]) -> Vec<Action> {
        let filtered_windows: Vec<&window::Window> = windows
            .iter()
            .enumerate()
            .filter(|&(i, w)| (self.predicate)(rect, i, w))
            .map(|(_, &w)| w)
            .collect();
        (self.layout).layout(rect, &filtered_windows)
    }
}
