use crate::{
    artist::Artist,
    commands::Commands,
    connection::connection,
    layout::*,
    window_data::WindowData,
};

pub fn new(child: Box<Layout>) -> Box<FloatingLayout> {
    Box::new(FloatingLayout { child })
}

pub struct FloatingLayout {
    child: Box<Layout>,
}

impl Layout for FloatingLayout {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let (mut floating_windows, tiled_windows): (Vec<WindowData>, Vec<WindowData>) = windows
            .into_iter()
            .partition(|w| w.is_floating);
        let connection = connection();
        for window in floating_windows.iter_mut() {
            if window.bounds.size.width == 0 && window.bounds.size.height == 0 {
                // TODO: get hints, not actual geometry
                if let Ok(geometry) = xcb::get_geometry(&connection, window.window()).get_reply() {
                    window.bounds = Bounds::new(
                        geometry.x(),
                        geometry.y(),
                        geometry.width(),
                        geometry.height(),
                    );
                }
            }
        }
        let (mut new_tiled_windows, artists) = self.child.layout(rect, tiled_windows);
        floating_windows.append(&mut new_tiled_windows);
        (floating_windows, artists)
    }
}

impl Commands for FloatingLayout {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        self.child.execute_command(command, args)
    }
}
