use super::{layout, connection::*};

pub type Id = xcb::Window;

#[derive(Default)]
pub struct Window {
    id: Id,
    is_mapped: bool,
}

impl Window {
    pub fn new(id: Id) -> Window {
        Window {
            id,
            ..Default::default()
        }
    }

    pub fn id(&self) -> xcb::Window {
        self.id
    }

    pub fn is_mapped(&self) -> bool {
        self.is_mapped
    }

    pub fn map(&self) {
        xcb::map_window(&connection(), self.id);
    }

    pub fn set_input_focus(&self) {
        xcb::set_input_focus(
            &connection(),
            xcb::INPUT_FOCUS_NONE as u8,
            self.id,
            xcb::CURRENT_TIME,
        );
    }

    pub fn map_notify(&mut self) {
        self.is_mapped = true;
    }

    pub fn unmap_notify(&mut self) {
        self.is_mapped = false;
    }

    pub fn set_geometry(&self, rect: &layout::LayoutRect, border_width: u16, border_color: u32) {
        if border_width > 0 {
            xcb::change_window_attributes(
                &connection(),
                self.id,
                &[(xcb::CW_BORDER_PIXEL, border_color)],
            );
        }
        xcb::configure_window(
            &connection(),
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
                (
                    xcb::CONFIG_WINDOW_WIDTH as u16,
                    (rect.size.width + 2 * border_width) as u32,
                ),
                (
                    xcb::CONFIG_WINDOW_HEIGHT as u16,
                    (rect.size.height + 2 * border_width) as u32,
                ),
                (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border_width as u32),
            ],
        );
    }
}
