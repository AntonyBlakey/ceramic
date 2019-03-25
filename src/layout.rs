use super::window::Window;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Axis {
    X,
    Y,
}

impl Axis {
    pub fn extract_origin<T>(&self, rect: &euclid::Rect<T>) -> T {
        match self {
            X => rect.origin.x,
            Y => rect.origin.y,
        }
    }
    pub fn extract_size<T>(&self, rect: &euclid::Rect<T>) -> T {
        match self {
            X => rect.size.width,
            Y => rect.size.height,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    Decreasing,
    Increasing,
}

pub type LayoutRect = euclid::Rect<u16>;

pub trait LayoutAlgorithm {
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window], result: &mut LayoutResult);
}

pub trait Graphic {}

pub struct LayoutResult {}

impl LayoutResult {
    pub fn add_window(&mut self, rect: &LayoutRect, window: &mut Window) {}

    pub fn add_graphic(&mut self, rect: &LayoutRect, graphic: &Rc<Graphic>) {}
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

impl LayoutAlgorithm for GridLayout {
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window], result: &mut LayoutResult) {
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
    layout_1: Rc<LayoutAlgorithm>,
    layout_2: Rc<LayoutAlgorithm>,
}

impl LayoutAlgorithm for SplitLayout {
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window], result: &mut LayoutResult) {
        let rect_1 = rect;
        let rect_2 = rect;

        match (self.axis, self.direction) {
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

        self.layout_1.clone().layout(&rect_1, windows, result);
        self.layout_2.clone().layout(&rect_2, windows, result);
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
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window], result: &mut LayoutResult) {}
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
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window], result: &mut LayoutResult) {}
}

//
//------------------------------------------------------------------
//

pub type LayoutFactory = Fn(&LayoutRect, &[&mut Window]) -> Rc<LayoutAlgorithm>;

// Selects a layout dynamically
#[derive(Clone)]
pub struct DynamicLayout {
    layout_factory: Rc<LayoutFactory>,
}

impl DynamicLayout {
    pub fn new(layout_factory: &Rc<LayoutFactory>) -> DynamicLayout {
        DynamicLayout {
            layout_factory: layout_factory.clone(),
        }
    }

    pub fn switch_on_window_count(
        count: usize,
        layout_1: &Rc<LayoutAlgorithm>,
        layout_2: &Rc<LayoutAlgorithm>,
    ) -> DynamicLayout {
        let l_1 = layout_1.clone();
        let l_2 = layout_2.clone();
        let layout_factory: Rc<LayoutFactory> = Rc::new(move |rect, windows| {
            if windows.len() <= count {
                l_1.clone()
            } else {
                l_2.clone()
            }
        });
        Self::new(&layout_factory)
    }

    pub fn switch_on_available_size(
        axis: Axis,
        size_break: u16,
        layout_1: &Rc<LayoutAlgorithm>,
        layout_2: &Rc<LayoutAlgorithm>,
    ) -> DynamicLayout {
        let l_1 = layout_1.clone();
        let l_2 = layout_2.clone();
        let layout_factory: Rc<LayoutFactory> = Rc::new(move |rect, _| {
            if axis.extract_size(rect) < size_break {
                l_1.clone()
            } else {
                l_2.clone()
            }
        });
        Self::new(&layout_factory)
    }

    pub fn switch_on_prorata_size(
        axis: Axis,
        size_break: u16,
        layout_1: &Rc<LayoutAlgorithm>,
        layout_2: &Rc<LayoutAlgorithm>,
    ) -> DynamicLayout {
        let l_1 = layout_1.clone();
        let l_2 = layout_2.clone();
        let layout_factory: Rc<LayoutFactory> = Rc::new(move |rect, windows| {
            if (axis.extract_size(rect) / windows.len() as u16) < size_break {
                l_1.clone()
            } else {
                l_2.clone()
            }
        });
        Self::new(&layout_factory)
    }
}

impl LayoutAlgorithm for DynamicLayout {
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window], result: &mut LayoutResult) {
        (self.layout_factory)(rect, windows)
            .clone()
            .layout(rect, windows, result);
    }
}

//
//------------------------------------------------------------------
//

pub type Predicate = Fn(&LayoutRect, usize, &Window) -> bool;

#[derive(Clone)]
pub struct PredicateSelector {
    predicate: Rc<Predicate>,
    layout: Rc<LayoutAlgorithm>,
}

impl PredicateSelector {
    pub fn passing(predicate: &Rc<Predicate>, layout: &Rc<LayoutAlgorithm>) -> PredicateSelector {
        PredicateSelector {
            predicate: predicate.clone(),
            layout: layout.clone(),
        }
    }

    pub fn failing(test: &Rc<Predicate>, layout: &Rc<LayoutAlgorithm>) -> PredicateSelector {
        let t = test.clone();
        let test: Rc<Predicate> =
            Rc::new(move |rect, index, window| t(rect, index, window) != true);
        Self::passing(&test, layout)
    }

    pub fn first(count: usize, layout: &Rc<LayoutAlgorithm>) -> PredicateSelector {
        let test: Rc<Predicate> = Rc::new(|_, index, _| index < count);
        Self::passing(&test, layout)
    }

    pub fn all_but_first(count: usize, layout: &Rc<LayoutAlgorithm>) -> PredicateSelector {
        let test: Rc<Predicate> = Rc::new(|_, index, _| index >= count);
        Self::passing(&test, layout)
    }
}

impl LayoutAlgorithm for PredicateSelector {
    fn layout(&self, rect: &LayoutRect, windows: &[&mut Window], result: &mut LayoutResult) {
        let filtered_windows = windows
            .iter_mut()
            .enumerate()
            .filter_map(|(i, &mut w)| {
                if (self.predicate)(rect, i, w) {
                    Some(w)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        (self.layout).layout(rect, &filtered_windows, result);
    }
}
