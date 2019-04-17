use crate::{
    artist::Artist, commands::Commands, connection::*, layout::*, window_data::WindowData,
};

pub fn new(
    width: u8,
    color: (u8, u8, u8),
    focus_color: (u8, u8, u8),
    child: Box<Layout>,
) -> Box<AddBorder> {
    Box::new(AddBorder {
        width,
        color,
        focus_color,
        child,
    })
}

pub struct AddBorder {
    width: u8,
    color: (u8, u8, u8),
    focus_color: (u8, u8, u8),
    child: Box<Layout>,
}

impl Layout for AddBorder {
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
            window.border_width = self.width;
            if window.window() == focused_window {
                window.border_color = self.focus_color;
            } else {
                window.border_color = self.color;
            }
        }

        (new_windows, artists)
    }
}

impl Commands for AddBorder {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        self.child.execute_command(command, args)
    }
}
