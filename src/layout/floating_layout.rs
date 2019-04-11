use crate::{
    artist::Artist,
    commands::Commands,
    layout::*,
    window_data::{WindowData, WindowType},
};

pub fn new<A: Layout>(child: A) -> FloatingLayout<A> {
    FloatingLayout { child }
}

#[derive(Clone)]
pub struct FloatingLayout<A: Layout> {
    child: A,
}

impl<A: Layout> Layout for FloatingLayout<A> {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let (mut floating_windows, tiled_windows) = windows
            .into_iter()
            .partition(|w| w.window_type != WindowType::TILED);
        let (mut new_tiled_windows, artists) = self.child.layout(rect, tiled_windows);
        floating_windows.append(&mut new_tiled_windows);
        (floating_windows, artists)
    }
}

impl<A: Layout> Commands for FloatingLayout<A> {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        self.child.execute_command(command, args)
    }
}
