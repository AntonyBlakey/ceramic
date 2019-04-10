use crate::{
    artist::Artist, commands::Commands, connection::*, layout::*, window_data::WindowData,
};

pub fn new<A: Layout>(width: u8, color: (u8, u8, u8), child: A) -> AddFocusBorder<A> {
    AddFocusBorder {
        width,
        color,
        child,
    }
}

#[derive(Clone)]
pub struct AddFocusBorder<A: Layout> {
    width: u8,
    color: (u8, u8, u8),
    child: A,
}

impl<A: Layout> Layout for AddFocusBorder<A> {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let focused_window = xcb::get_input_focus(&connection())
            .get_reply()
            .unwrap()
            .focus();

        let (mut new_windows, artists) = self.child.layout(rect, windows);

        for window in new_windows.iter_mut() {
            if window.window() == focused_window {
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

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        self.child.execute_command(command, args)
    }
}
