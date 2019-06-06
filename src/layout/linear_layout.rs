use crate::{artist::Artist, commands::Commands, layout::*, window_data::WindowData};

pub fn new(direction: Direction, axis: Axis) -> Box<LinearLayout> {
    Box::new(LinearLayout { direction, axis })
}

pub struct LinearLayout {
    axis: Axis,
    direction: Direction,
}

impl Layout for LinearLayout {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<dyn Artist>>) {
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
        clear_window_order(&mut new_windows);
        (new_windows, Default::default())
    }
}

impl Commands for LinearLayout {}
