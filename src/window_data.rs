use super::{commands::Commands, connection::*, layout::Bounds};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowData {
    window: xcb::Window,
    pub is_floating: bool,
    pub bounds: Bounds,
    pub border_width: u8,
    pub border_color: (u8, u8, u8),
    pub selector_label: String,
    pub leader_window: Option<xcb::Window>,
}

impl WindowData {
    pub fn new(window: xcb::Window) -> WindowData {
        WindowData {
            window,
            ..Default::default()
        }
    }

    pub fn window(&self) -> xcb::Window {
        self.window
    }

    pub fn configure(&self) {
        let connection = connection();
        if self.border_width > 0 {
            xcb::change_window_attributes(
                &connection,
                self.window,
                &[(
                    xcb::CW_BORDER_PIXEL,
                    ((self.border_color.0 as u32) << 16)
                        | ((self.border_color.1 as u32) << 8)
                        | self.border_color.2 as u32,
                )],
            );
        }
        xcb::configure_window(
            &connection,
            self.window,
            &[
                (
                    xcb::CONFIG_WINDOW_X as u16,
                    (self.bounds.origin.x - self.border_width as i16) as u32,
                ),
                (
                    xcb::CONFIG_WINDOW_Y as u16,
                    (self.bounds.origin.y - self.border_width as i16) as u32,
                ),
                (
                    xcb::CONFIG_WINDOW_WIDTH as u16,
                    self.bounds.size.width as u32,
                ),
                (
                    xcb::CONFIG_WINDOW_HEIGHT as u16,
                    self.bounds.size.height as u32,
                ),
                (
                    xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                    self.border_width as u32,
                ),
            ],
        );
    }
}

impl Commands for WindowData {
    fn get_commands(&self) -> Vec<String> {
        vec![String::from("close_focused_window")]
    }

    fn execute_command(&mut self, command: &str, _args: &[&str]) -> bool {
        match command {
            "close_focused_window" => {
                xcb::kill_client(&connection(), self.window);
                // destruction of window will trigger layout update
                false
            }
            _ => {
                eprintln!("Unhandled command: {}", command);
                false
            }
        }
    }
}
