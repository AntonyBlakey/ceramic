use super::{
    connection::*,
    layout::LayoutRect,
    window_manager::{Commands, WindowManager},
};

pub struct WindowData {
    pub id: xcb::Window,
    pub is_floating: bool,
    pub floating_frame: Option<LayoutRect>,
}

impl WindowData {
    pub fn set_input_focus(&self) {
        let connection = connection();
        xcb::set_input_focus(
            &connection,
            xcb::INPUT_FOCUS_NONE as u8,
            self.id,
            xcb::CURRENT_TIME,
        );
        let screen = connection.get_setup().roots().nth(0).unwrap();
        set_window_property(screen.root(), *ATOM__NET_ACTIVE_WINDOW, self.id);
    }

    pub fn configure(&self, rect: &LayoutRect, border_width: u16, border_color: u32) {
        let connection = connection();
        if border_width > 0 {
            xcb::change_window_attributes(
                &connection,
                self.id,
                &[(xcb::CW_BORDER_PIXEL, border_color)],
            );
        }
        xcb::configure_window(
            &connection,
            self.id,
            &[
                (
                    xcb::CONFIG_WINDOW_X as u16,
                    (rect.origin.x - border_width) as u32,
                ),
                (
                    xcb::CONFIG_WINDOW_Y as u16,
                    (rect.origin.y - border_width) as u32,
                ),
                (xcb::CONFIG_WINDOW_WIDTH as u16, rect.size.width as u32),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, rect.size.height as u32),
                (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border_width as u32),
            ],
        );
    }
}

impl Commands for WindowData {
    fn get_commands(&self) -> Vec<String> {
        vec![String::from("close_focused_window")]
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        match command {
            "close_focused_window" => {
                xcb::kill_client(&connection(), self.id);
                None
            }
            _ => {
                eprintln!("Unhandled command: {}", command);
                None
            }
        }
    }
}
