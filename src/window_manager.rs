use crate::window::Window;
use std::collections::HashMap;

pub struct WindowManager {
    connection: xcb::Connection,
    windows: HashMap<xcb::Window, Window>,
}

impl WindowManager {
    pub fn run() {
        let mut me = Self::new();
        me.main_loop();
    }

    fn new() -> WindowManager {
        let (connection, _screen_number) = xcb::Connection::connect(None).unwrap();
        WindowManager {
            connection,
            windows: Default::default(),
        }
    }

    fn main_loop(&mut self) {
        // TODO: handle all screens
        let screen = self.connection.get_setup().roots().nth(0).unwrap();
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
        )];
        xcb::change_window_attributes_checked(&self.connection, screen.root(), &values)
            .request_check()
            .expect("Cannot install as window manager");

        while let Some(e) = self.connection.wait_for_event() {
            match e.response_type() {
                xcb::CONFIGURE_REQUEST => self.configure_request(unsafe { xcb::cast_event(&e) }),
                xcb::MAP_REQUEST => self.map_request(unsafe { xcb::cast_event(&e) }),
                xcb::CREATE_NOTIFY => self.create_notify(unsafe { xcb::cast_event(&e) }),
                xcb::DESTROY_NOTIFY => self.destroy_notify(unsafe { xcb::cast_event(&e) }),
                xcb::CONFIGURE_NOTIFY => self.configure_notify(unsafe { xcb::cast_event(&e) }),
                xcb::PROPERTY_NOTIFY => self.property_notify(unsafe { xcb::cast_event(&e) }),
                xcb::MAP_NOTIFY => self.map_notify(unsafe { xcb::cast_event(&e) }),
                xcb::UNMAP_NOTIFY => self.unmap_notify(unsafe { xcb::cast_event(&e) }),
                xcb::CLIENT_MESSAGE => self.client_message(unsafe { xcb::cast_event(&e) }),
                t => eprintln!("UNEXPECTED EVENT TYPE: {}", t),
            }
            self.connection.flush();
        }
    }

    fn create_notify(&mut self, e: &xcb::CreateNotifyEvent) {
        self.windows.insert(e.window(), Window::new(e.window()));
    }

    fn destroy_notify(&mut self, e: &xcb::DestroyNotifyEvent) {
        self.windows.remove(&e.window());
    }

    fn configure_request(&mut self, _: &xcb::ConfigureRequestEvent) {}

    fn map_request(&mut self, e: &xcb::MapRequestEvent) {
        let window = self.windows.get_mut(&e.window()).unwrap();
        xcb::xproto::map_window(&self.connection, window.id);
    }

    fn configure_notify(&mut self, _: &xcb::ConfigureNotifyEvent) {}

    fn property_notify(&mut self, _: &xcb::PropertyNotifyEvent) {}

    fn map_notify(&mut self, _: &xcb::MapNotifyEvent) {
        self.update_layout();
    }

    fn unmap_notify(&mut self, _: &xcb::UnmapNotifyEvent) {
        self.update_layout();
    }

    fn client_message(&mut self, _: &xcb::ClientMessageEvent) {}

    fn update_layout(&mut self) {
        let screen = self.connection.get_setup().roots().nth(0).unwrap();
        let width = screen.width_in_pixels();
        let height = screen.height_in_pixels();

        let rows = (self.windows.len() as f64).sqrt().ceil() as u16;
        let columns = (self.windows.len() / rows as usize) as u16;

        let screen_gap = 5;
        let window_gap = 5;

        let cell_width = (width - screen_gap * 2) / rows;
        let cell_height = (height - screen_gap * 2) / columns;

        let mut x = 0;
        let mut y = 0;

        for w in self.windows.values() {
            let values = [
                (xcb::xproto::CONFIG_WINDOW_X as u16, (screen_gap + cell_width * x + window_gap) as u32),
                (xcb::xproto::CONFIG_WINDOW_Y as u16, (screen_gap + cell_height * y + window_gap) as u32),
                (xcb::xproto::CONFIG_WINDOW_WIDTH as u16, (cell_width - 2 * window_gap) as u32),
                (xcb::xproto::CONFIG_WINDOW_HEIGHT as u16, (cell_height - 2 * window_gap) as u32),
            ];
            xcb::xproto::configure_window(&self.connection, w.id, &values);
            x += 1;
            if x == columns {
                x = 0;
                y += 1;
            }
        }
    }
}
