use super::window_manager::connection;

pub struct Window {
    id: xcb::Window,
    is_mapped: bool,
}

impl Window {
    pub fn new(id: xcb::Window) -> Window {
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

    pub fn set_geometry(&self, x: u32, y: u32, w: u32, h: u32) {
        let values = [
            (xcb::xproto::CONFIG_WINDOW_X as u16, x),
            (xcb::xproto::CONFIG_WINDOW_Y as u16, y),
            (xcb::xproto::CONFIG_WINDOW_WIDTH as u16, w),
            (xcb::xproto::CONFIG_WINDOW_HEIGHT as u16, h),
        ];
        xcb::xproto::configure_window(&connection(), self.id, &values);
    }
}
