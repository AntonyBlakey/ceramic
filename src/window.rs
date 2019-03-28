use super::{layout, window_manager::connection};

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
        xcb::xproto::map_window(&connection(), self.id);
    }

    pub fn set_is_focused(&self, is_focused: bool) {
        {
            let values = [(xcb::xproto::CW_BORDER_PIXEL, 0xff0000)];
            xcb::xproto::change_window_attributes(&connection(), self.id, &values);
        }
        {
            let values = [(xcb::xproto::CONFIG_WINDOW_BORDER_WIDTH as u16, 2)];
            xcb::xproto::configure_window(&connection(), self.id, &values);
        }
        xcb::xproto::set_input_focus(
            &connection(),
            xcb::xproto::INPUT_FOCUS_NONE as u8,
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

    pub fn set_geometry(&self, rect: &layout::LayoutRect) {
        let values = [
            (xcb::xproto::CONFIG_WINDOW_X as u16, rect.origin.x as u32),
            (xcb::xproto::CONFIG_WINDOW_Y as u16, rect.origin.y as u32),
            (
                xcb::xproto::CONFIG_WINDOW_WIDTH as u16,
                rect.size.width as u32,
            ),
            (
                xcb::xproto::CONFIG_WINDOW_HEIGHT as u16,
                rect.size.height as u32,
            ),
        ];
        xcb::xproto::configure_window(&connection(), self.id, &values);
    }
}
