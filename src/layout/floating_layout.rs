use crate::{artist::Artist, commands::Commands, layout::*, window_data::WindowData};

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
        let (mut floating_windows, tiled_windows): (Vec<WindowData>, Vec<WindowData>) =
            windows.into_iter().partition(|w| w.is_floating);
        compute_window_order(&mut floating_windows);
        let (mut new_tiled_windows, artists) = self.child.layout(rect, tiled_windows);
        let floating_window_order_offset = new_tiled_windows.len() as i16;
        for window in &mut floating_windows {
            window.order = window.order.map(|order| order + floating_window_order_offset);
            // TODO: pick this up at creating time?
            if window.bounds.size.height == 0 || window.bounds.size.width == 0 {
                if let Ok(geometry) = xcb::get_geometry(&connection(), window.window()).get_reply() {
                    window.bounds.origin.x = geometry.x();
                    window.bounds.origin.y = geometry.y();
                    window.bounds.size.width = geometry.width();
                    window.bounds.size.height = geometry.height();
                }
            }
        }
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
