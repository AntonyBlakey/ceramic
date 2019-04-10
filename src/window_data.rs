use super::{commands::Commands, connection::*, layout::Bounds, window_manager::WindowManager};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowData {
    window: xcb::Window,
    pub is_managed: bool,
    pub bounds: Bounds,
    pub border_width: u8,
    pub border_color: (u8, u8, u8),
    pub selector_label: String,
    pub leader_window: Option<xcb::Window>,
}

impl WindowData {
    pub fn new(window: xcb::Window) -> WindowData {
        let window_type = get_atoms_property(window, *ATOM__NET_WM_WINDOW_TYPE);
        let is_managed = !window_type.contains(&*ATOM__NET_WM_WINDOW_TYPE_DOCK);
        WindowData {
            window,
            is_managed,
            ..Default::default()
        }
    }

    pub fn window(&self) -> xcb::Window {
        self.window
    }

    pub fn set_input_focus(&self) {
        let connection = connection();
        xcb::set_input_focus(
            &connection,
            xcb::INPUT_FOCUS_NONE as u8,
            self.window,
            xcb::CURRENT_TIME,
        );
        let screen = connection.get_setup().roots().nth(0).unwrap();
        set_window_property(screen.root(), *ATOM__NET_ACTIVE_WINDOW, self.window);
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

    fn execute_command(
        &mut self,
        command: &str,
        _args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        match command {
            "close_focused_window" => {
                xcb::kill_client(&connection(), self.window);
                None
            }
            _ => {
                eprintln!("Unhandled command: {}", command);
                None
            }
        }
    }
}
