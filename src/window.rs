use super::window_manager::connection;
use super::layout::LayoutRect;

pub type Id = xcb::Window;

pub struct Window {
    id: Id,
    is_mapped: bool,
}

impl Window {
    pub fn new(id: Id) -> Window {
        Window {
            id,
            is_mapped: false,
        }
    }

    pub fn id(&self) -> xcb::Window {
        self.id
    }

    pub fn is_mapped(&self) -> bool {
        self.is_mapped
    }

    pub fn map(&self) {
        xcb::xproto::map_window(&connection(), self.id);
    }

    pub fn map_notify(&mut self) {
        self.is_mapped = true;
    }

    pub fn unmap_notify(&mut self) {
        self.is_mapped = false;
    }

    pub fn set_geometry(&self, rect: &LayoutRect) {
        let values = [
            (xcb::xproto::CONFIG_WINDOW_X as u16, rect.origin.x as u32),
            (xcb::xproto::CONFIG_WINDOW_Y as u16, rect.origin.y as u32),
            (xcb::xproto::CONFIG_WINDOW_WIDTH as u16, rect.size.width as u32),
            (xcb::xproto::CONFIG_WINDOW_HEIGHT as u16, rect.size.height as u32),
        ];
        xcb::xproto::configure_window(&connection(), self.id, &values);
    }
}
