use crate::{artist::Artist, commands::Commands, layout::*, window_data::WindowData};

pub fn new() -> Box<GridLayout> {
    Box::new(GridLayout {})
}

pub struct GridLayout {}

impl Layout for GridLayout {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<dyn Artist>>) {
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
        clear_window_order(&mut new_windows);
        (new_windows, Default::default())
    }
}

impl Commands for GridLayout {}
