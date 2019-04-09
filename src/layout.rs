use super::{
    artist::Artist, commands::Commands, connection::*, window_data::WindowData,
    window_manager::WindowManager,
};

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
    pub fn extract_origin(&self, rect: &Bounds) -> i16 {
        match self {
            Axis::X => rect.origin.x,
            Axis::Y => rect.origin.y,
        }
    }
    pub fn extract_size(&self, rect: &Bounds) -> u16 {
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

pub trait Layout: Commands {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>);
}

pub fn root<A: Layout + 'static>(name: &str, child: A) -> LayoutRoot {
    LayoutRoot::new(name, child)
}

pub fn avoid_struts<A: Layout>(child: A) -> AvoidStruts<A> {
    AvoidStruts { child }
}

pub fn ignore_some_windows<A: Layout>(child: A) -> IgnoreSomeWindows<A> {
    IgnoreSomeWindows { child }
}

pub fn add_gaps<A: Layout>(screen_gap: u16, window_gap: u16, child: A) -> AddGaps<A> {
    AddGaps {
        screen_gap,
        window_gap,
        child,
    }
}

pub fn add_focus_border<A: Layout>(width: u8, color: (u8, u8, u8), child: A) -> AddFocusBorder<A> {
    AddFocusBorder {
        width,
        color,
        child,
    }
}

pub fn grid() -> GridLayout {
    GridLayout {}
}

pub fn stack() -> StackLayout {
    StackLayout {}
}

pub fn linear(direction: Direction, axis: Axis) -> LinearLayout {
    LinearLayout { direction, axis }
}

pub fn split<A: Layout, B: Layout>(
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

pub fn monad_stack(
    direction: Direction,
    axis: Axis,
    ratio: f64,
    count: usize,
) -> SplitLayout<LinearLayout, StackLayout> {
    split(
        direction,
        axis,
        ratio,
        count,
        linear(direction, axis),
        stack(),
    )
}

//
//------------------------------------------------------------------
//

pub struct LayoutRoot {
    name: String,
    child: Box<Layout>,
}

impl LayoutRoot {
    pub fn new<T: Layout + 'static>(name: &str, child: T) -> LayoutRoot {
        LayoutRoot {
            name: name.to_owned(),
            child: Box::new(child),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn layout(
        &self,
        rect: &Bounds,
        windows: &[WindowData],
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        self.child.layout(rect, &windows)
    }

    pub fn get_commands(&self) -> Vec<String> {
        self.child
            .get_commands()
            .iter()
            .map(|s| format!("{}/{}", self.name, s))
            .collect()
    }

    pub fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        if command.starts_with(self.name.as_str()) {
            self.child
                .execute_command(command.split_at(self.name.len() + 1).1, args)
        } else {
            eprintln!("Command not valid for layout {} : {}", self.name, command);
            None
        }
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct IgnoreSomeWindows<A: Layout> {
    child: A,
}

impl<A: Layout> Layout for IgnoreSomeWindows<A> {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        self.child.layout(
            rect,
            &windows
                .iter()
                .filter(|w| {
                    let window_type = get_atoms_property(w.id(), *ATOM__NET_WM_WINDOW_TYPE);
                    !window_type.contains(&*ATOM__NET_WM_WINDOW_TYPE_DOCK)
                })
                .cloned()
                .collect::<Vec<WindowData>>(),
        )
    }
}

impl<A: Layout> Commands for IgnoreSomeWindows<A> {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        self.child.execute_command(command, args)
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct AvoidStruts<A: Layout> {
    child: A,
}

impl<A: Layout> Layout for AvoidStruts<A> {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let mut r = *rect;

        for window in windows {
            let struts = get_cardinals_property(window.id(), *ATOM__NET_WM_STRUT);
            if struts.len() == 4 {
                let left = struts[0];
                let right = struts[1];
                let top = struts[2];
                let bottom = struts[3];
                r.origin.x += left as i16;
                r.size.width -= (left + right) as u16;
                r.origin.y += top as i16;
                r.size.height -= (top + bottom) as u16;
            }
        }

        self.child.layout(&r, windows)
    }
}

impl<A: Layout> Commands for AvoidStruts<A> {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        self.child.execute_command(command, args)
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct AddGaps<A: Layout> {
    screen_gap: u16,
    window_gap: u16,
    child: A,
}

impl<A: Layout> Layout for AddGaps<A> {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let r = Bounds::new(
            rect.origin.x + self.screen_gap as i16,
            rect.origin.y + self.screen_gap as i16,
            rect.size.width - 2 * self.screen_gap,
            rect.size.height - 2 * self.screen_gap,
        );

        let (mut new_windows, artists) = self.child.layout(&r, windows);

        for window in new_windows.iter_mut() {
            window.bounds.origin.x += self.window_gap as i16;
            window.bounds.origin.y += self.window_gap as i16;
            window.bounds.size.width -= 2 * self.window_gap;
            window.bounds.size.height -= 2 * self.window_gap;
        }

        (new_windows, artists)
    }
}

impl<A: Layout> Commands for AddGaps<A> {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        self.child.execute_command(command, args)
    }
}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct AddFocusBorder<A: Layout> {
    width: u8,
    color: (u8, u8, u8),
    child: A,
}

impl<A: Layout> Layout for AddFocusBorder<A> {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let focused_window = xcb::get_input_focus(&connection())
            .get_reply()
            .unwrap()
            .focus();

        let (mut new_windows, artists) = self.child.layout(rect, windows);

        for window in new_windows.iter_mut() {
            if window.id() == focused_window {
                window.border_width = self.width;
                window.border_color = self.color;
            } else {
                window.border_width = 0;
            }
        }

        (new_windows, artists)
    }
}

impl<A: Layout> Commands for AddFocusBorder<A> {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        self.child.execute_command(command, args)
    }
}

//
//------------------------------------------------------------------
//

struct StackIndicatorArtist {
    axis: Axis,
    window: xcb::Window,
}

impl Artist for StackIndicatorArtist {
    fn calculate_bounds(&self, window: xcb::Window) -> Option<Bounds> {
        match xcb::get_geometry(&connection(), self.window).get_reply() {
            Ok(geometry) => Some(match self.axis {
                Axis::X => Bounds::new(geometry.x() - 8, geometry.y(), 4, geometry.height()),
                Axis::Y => Bounds::new(geometry.x(), geometry.y() - 8, geometry.width(), 4),
            }),
            _ => None,
        }
    }

    fn draw(&self, window: xcb::Window) {
        if let Ok(geometry) = xcb::get_geometry(&connection(), window).get_reply() {
            if let Ok(surface) = get_cairo_surface(window) {
                let context = cairo::Context::new(&surface);
                context.set_source_rgb(0.125, 0.375, 0.5);
                context.move_to(0.0, 0.0);
                context.line_to(geometry.width() as f64, 0.0);
                context.line_to(geometry.width() as f64, geometry.height() as f64);
                context.line_to(0.0, geometry.height() as f64);
                context.close_path();
                context.fill();
            }
        }
    }
}

#[derive(Clone)]
pub struct StackLayout {}

impl Layout for StackLayout {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        if windows.is_empty() {
            return Default::default();
        }

        let axis = rect.size.largest_axis();

        let artists: Vec<Box<Artist>> = vec![Box::new(StackIndicatorArtist {
            window: windows[0].id(),
            axis: axis,
        })];

        let r = match axis {
            Axis::X => Bounds::new(
                rect.origin.x + 8,
                rect.origin.y,
                rect.size.width - 8,
                rect.size.height,
            ),
            Axis::Y => Bounds::new(
                rect.origin.x,
                rect.origin.y + 8,
                rect.size.width,
                rect.size.height - 8,
            ),
        };

        let mut new_windows = windows.to_vec();
        for window in new_windows.iter_mut() {
            window.bounds = r;
        }

        (new_windows, artists)
    }
}

impl Commands for StackLayout {}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct GridLayout {}

impl Layout for GridLayout {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
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

        let width = cell_width - 2 * window_gap;
        let height = cell_height - 2 * window_gap;

        let mut new_windows = windows.to_vec();
        for window in new_windows.iter_mut() {
            let x = rect.origin.x + (screen_gap + cell_width * column + window_gap) as i16;
            let y = rect.origin.y + (screen_gap + cell_height * row + window_gap) as i16;
            column += 1;
            if column == columns {
                column = 0;
                row += 1;
            }
            window.bounds = Bounds::new(x, y, width, height);
        }

        (new_windows, Default::default())
    }
}

impl Commands for GridLayout {}

//
//------------------------------------------------------------------
//

#[derive(Clone)]
pub struct SplitLayout<A, B> {
    axis: Axis,
    direction: Direction,
    ratio: f64,
    count: usize,
    children: (A, B),
}

impl<A: Layout, B: Layout> Layout for SplitLayout<A, B> {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
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
            let (mut new_windows, mut artists) = self.children.0.layout(&rect_1, w1);
            let (mut new_windows_2, mut artists_2) = self.children.1.layout(&rect_2, w2);
            new_windows.extend(new_windows_2.drain(0..));
            artists.extend(artists_2.drain(0..));
            (new_windows, artists)
        } else {
            self.children.0.layout(&rect_1, windows)
        }
    }
}

impl<A: Layout, B: Layout> Commands for SplitLayout<A, B> {
    fn get_commands(&self) -> Vec<String> {
        let c0 = self.children.0.get_commands();
        let c1 = self.children.1.get_commands();

        let mut result = Vec::with_capacity(c0.len() + c1.len() + 4);

        result.push(String::from("increase_count"));
        if self.count > 1 {
            result.push(String::from("decrease_count"));
        }
        if self.ratio < 0.9 {
            result.push(String::from("increase_ratio"));
        }
        if self.ratio > 0.1 {
            result.push(String::from("decrease_ratio"));
        }

        result.extend(c0.iter().map(|c| format!("0/{}", c)));
        result.extend(c1.iter().map(|c| format!("1/{}", c)));

        result
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        if command.starts_with("0/") {
            self.children.0.execute_command(command.split_at(2).1, args)
        } else if command.starts_with("1/") {
            self.children.1.execute_command(command.split_at(2).1, args)
        } else {
            match command {
                "increase_count" => self.count += 1,
                "decrease_count" if self.count > 1 => self.count -= 1,
                "increase_ratio" if self.ratio < 0.9 => self.ratio += 0.05,
                "decrease_ratio" if self.ratio > 0.1 => self.ratio -= 0.05,
                _ => (),
            }
            Some(Box::new(|wm| wm.update_layout()))
        }
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

impl Layout for LinearLayout {
    fn layout(&self, rect: &Bounds, windows: &[WindowData]) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        if windows.is_empty() {
            return Default::default();
        }

        let mut new_windows = windows.to_vec();
        let mut r = *rect;
        match self.axis {
            Axis::X => {
                r.size.width = r.size.width / windows.len() as u16;
                for window in new_windows.iter_mut() {
                    window.bounds = r;
                    r.origin.x += r.size.width as i16;
                }
            }
            Axis::Y => {
                r.size.height = r.size.height / windows.len() as u16;
                for window in new_windows.iter_mut() {
                    window.bounds = r;
                    r.origin.y += r.size.height as i16;
                }
            }
        };

        (new_windows, Default::default())
    }
}

impl Commands for LinearLayout {}

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
