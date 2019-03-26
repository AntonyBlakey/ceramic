use super::window::Window;
use std::clone::Clone;
use std::rc::Rc;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Direction {
    Increasing,
    Decreasing,
}

pub type LayoutRect = euclid::Rect<u16>;

pub trait LayoutAlgorithm {
    fn boxed_clone(&self) -> Box<LayoutAlgorithm>;
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window]);
}

pub struct LayoutStep(Box<LayoutAlgorithm>);

impl std::ops::Deref for LayoutStep {
    type Target = Box<LayoutAlgorithm>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for LayoutStep {
    fn clone(&self) -> LayoutStep {
        LayoutStep(self.0.boxed_clone())
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct GridLayout {
    major_axis: Axis,
    major_direction: Direction,
    minor_direction: Direction,
}

impl Default for GridLayout {
    fn default() -> GridLayout {
        GridLayout {
            major_axis: Axis::X,
            major_direction: Direction::Increasing,
            minor_direction: Direction::Increasing,
        }
    }
}

impl LayoutAlgorithm for GridLayout {
    fn boxed_clone(&self) -> Box<LayoutAlgorithm> {
        Box::new(self.clone())
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window]) {
        if windows.is_empty() {
            return;
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
        for window in windows {
            let x = rect.origin.x + screen_gap + cell_width * column + window_gap;
            let y = rect.origin.y + screen_gap + cell_height * row + window_gap;
            window.set_geometry(x as u32, y as u32, w as u32, h as u32);
            column += 1;
            if column == columns {
                column = 0;
                row += 1;
            }
        }
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct SplitLayout {
    ratio: f64,
    axis: Axis,
    direction: Direction,
    layout_1: LayoutStep,
    layout_2: LayoutStep,
}

impl LayoutAlgorithm for SplitLayout {
    fn boxed_clone(&self) -> Box<LayoutAlgorithm> {
        Box::new(self.clone())
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window]) {
        let mut rect_1 = *rect;
        let mut rect_2 = *rect;

        match (&self.axis, &self.direction) {
            (Axis::X, Direction::Increasing) => {
                let split = (rect.size.width as f64 * self.ratio).floor() as u16;
                rect_1.size.width = split;
                rect_2.origin.x += split;
                rect_2.size.width -= split;
            }
            (Axis::X, Direction::Decreasing) => {
                let split = (rect.size.width as f64 * self.ratio).floor() as u16;
                rect_2.size.width = split;
                rect_1.origin.x += split;
                rect_1.size.width -= split;
            }
            (Axis::Y, Direction::Increasing) => {
                let split = (rect.size.height as f64 * self.ratio).floor() as u16;
                rect_1.size.height = split;
                rect_2.origin.y += split;
                rect_2.size.height -= split;
            }
            (Axis::Y, Direction::Decreasing) => {
                let split = (rect.size.height as f64 * self.ratio).floor() as u16;
                rect_2.size.height = split;
                rect_1.origin.y += split;
                rect_1.size.height -= split;
            }
        }

        self.layout_1.layout(&rect_1, windows);
        self.layout_2.layout(&rect_2, windows);
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct TabbedLayout {
    direction: Direction,
}

impl LayoutAlgorithm for TabbedLayout {
    fn boxed_clone(&self) -> Box<LayoutAlgorithm> {
        Box::new(self.clone())
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window]) {}
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct LinearLayout {
    axis: Axis,
    direction: Direction,
}

impl LayoutAlgorithm for LinearLayout {
    fn boxed_clone(&self) -> Box<LayoutAlgorithm> {
        Box::new(self.clone())
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window]) {}
}

//
//------------------------------------------------------------------
//

pub type LayoutFactory = Fn(&LayoutRect, &[&mut Window]) -> usize;

// Selects a layout dynamically
#[derive(Clone)]
pub struct DynamicLayout {
    layout_factory: Rc<LayoutFactory>,
    algorithms: Vec<LayoutStep>,
}

impl DynamicLayout {
    pub fn new(layout_factory: &Rc<LayoutFactory>, algorithms: &[LayoutStep]) -> DynamicLayout {
        DynamicLayout {
            layout_factory: layout_factory.clone(),
            algorithms: algorithms.iter().map(|x| x.clone()).collect(),
        }
    }

    pub fn switch_on_window_count(
        count: usize,
        lsmall: &LayoutStep,
        lbig: &LayoutStep,
    ) -> DynamicLayout {
        let layout_factory: Rc<LayoutFactory> = Rc::new(
            move |rect, windows| {
                if windows.len() <= count {
                    0
                } else {
                    1
                }
            },
        );
        Self::new(&layout_factory, &[lsmall.clone(), lbig.clone()])
    }

    pub fn switch_on_available_size(
        axis: Axis,
        size_break: u16,
        lsmall: &LayoutStep,
        lbig: &LayoutStep,
    ) -> DynamicLayout {
        let layout_factory: Rc<LayoutFactory> = Rc::new(move |rect, _| {
            if axis.extract_size(rect) < size_break {
                0
            } else {
                1
            }
        });
        Self::new(&layout_factory, &[lsmall.clone(), lbig.clone()])
    }

    pub fn switch_on_prorata_size(
        axis: Axis,
        size_break: u16,
        lsmall: &LayoutStep,
        lbig: &LayoutStep,
    ) -> DynamicLayout {
        let layout_factory: Rc<LayoutFactory> = Rc::new(move |rect, windows| {
            if (axis.extract_size(rect) / windows.len() as u16) < size_break {
                0
            } else {
                1
            }
        });
        Self::new(&layout_factory, &[lsmall.clone(), lbig.clone()])
    }
}

impl LayoutAlgorithm for DynamicLayout {
    fn boxed_clone(&self) -> Box<LayoutAlgorithm> {
        Box::new(self.clone())
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window]) {
        self.algorithms[(self.layout_factory)(rect, windows)].layout(rect, windows);
    }
}

//
//------------------------------------------------------------------
//

pub type Predicate = Fn(&LayoutRect, usize, &Window) -> bool;

#[derive(Clone)]
pub struct PredicateSelector {
    predicate: Rc<Predicate>,
    layout: LayoutStep,
}

impl PredicateSelector {
    pub fn passing(predicate: &Rc<Predicate>, layout: LayoutStep) -> PredicateSelector {
        PredicateSelector {
            predicate: predicate.clone(),
            layout: layout.clone(),
        }
    }

    pub fn failing(test: &Rc<Predicate>, layout: LayoutStep) -> PredicateSelector {
        let t = test.clone();
        let test: Rc<Predicate> =
            Rc::new(move |rect, index, window| t(rect, index, window) != true);
        Self::passing(&test, layout)
    }

    pub fn first(count: usize, layout: LayoutStep) -> PredicateSelector {
        let test: Rc<Predicate> = Rc::new(move |_, index, _| index < count);
        Self::passing(&test, layout)
    }

    pub fn all_but_first(count: usize, layout: LayoutStep) -> PredicateSelector {
        let test: Rc<Predicate> = Rc::new(move |_, index, _| index >= count);
        Self::passing(&test, layout)
    }
}

impl LayoutAlgorithm for PredicateSelector {
    fn boxed_clone(&self) -> Box<LayoutAlgorithm> {
        Box::new(self.clone())
    }
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window]) {
        // let filtered_windows = windows
        //     .iter()
        //     .enumerate()
        //     .filter(|(i, w)| (self.predicate)(rect, *i, *w))
        //     .collect::<Vec<_>>();
        // (self.layout).layout(rect, &filtered_windows, result);
    }
}
