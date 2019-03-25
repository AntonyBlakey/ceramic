use super::window_manager::WindowManager;

pub trait LayoutAlgorithm {
    fn layout(&self, wm: &mut WindowManager);
}

pub struct GridLayout;

impl LayoutAlgorithm for GridLayout {
    fn layout(&self, wm: &mut WindowManager) {
        let screen = wm.connection.get_setup().roots().nth(0).unwrap();
        let width = screen.width_in_pixels();
        let height = screen.height_in_pixels();

        let ws = wm.workspaces.get_mut(wm.current_workspace).unwrap();

        let windows = &wm.windows;
        let mapped_windows: Vec<&_> = ws
            .windows
            .iter()
            .map(|id| &windows[id])
            .filter(|w| w.is_mapped())
            .collect();

        if mapped_windows.is_empty() {
            return;
        }

        let columns = (mapped_windows.len() as f64).sqrt().ceil() as u16;
        let rows = (mapped_windows.len() as u16 + columns - 1) / columns;

        let screen_gap = 5;
        let window_gap = 5;

        let cell_width = (width - screen_gap * 2) / columns;
        let cell_height = (height - screen_gap * 2) / rows;

        let mut row = 0;
        let mut column = 0;

        let w = cell_width - 2 * window_gap;
        let h = cell_height - 2 * window_gap;
        for window in mapped_windows {
            let x = screen_gap + cell_width * column + window_gap;
            let y = screen_gap + cell_height * row + window_gap;
            window.set_geometry(x as u32, y as u32, w as u32, h as u32);
            column += 1;
            if column == columns {
                column = 0;
                row += 1;
            }
        }
    }
}