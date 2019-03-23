use std::collections::HashMap;
use crate::window::Window;

pub struct WindowManager<'a> {
    connection: xcb::Connection,
    windows: HashMap<xcb::Window, Window<'a>>
}

impl<'a> WindowManager<'a> {
    pub fn run() {
        let mut me = Self::new();
        me.setup();
        me.main_loop();
        me.teardown();
    }

    fn new() -> WindowManager<'a> {
        let (connection, _screen_number) = xcb::Connection::connect(None).unwrap();
        WindowManager { connection, windows: Default::default() }
    }

    fn setup(&mut self) {
        // TODO: handle all screens
        let root = self.connection.get_setup().roots().nth(0).unwrap().root();
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
        )];
        xcb::change_window_attributes_checked(&self.connection, root, &values)
            .request_check()
            .expect("Cannot install as window manager");
    }

    fn main_loop(&mut self) {
        while let Some(event) = self.connection.wait_for_event() {
            match event.response_type() {
                xcb::CLIENT_MESSAGE => {
                    let e: &xcb::ClientMessageEvent = unsafe { xcb::cast_event(&event) };
                    println!("CLIENT_MESSAGE");
                }
                xcb::PROPERTY_NOTIFY => {
                    let e: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(&event) };
                    println!("PROPERTY_NOTIFY");
                }
                xcb::CONFIGURE_REQUEST => {
                    let e: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(&event) };
                    println!("CONFIGURE_REQUEST");
                }
                xcb::CONFIGURE_NOTIFY => {
                    let e: &xcb::ConfigureNotifyEvent = unsafe { xcb::cast_event(&event) };
                    println!("CONFIGURE_NOTIFY");
                }
                xcb::MAP_REQUEST => {
                    let e: &xcb::MapRequestEvent = unsafe { xcb::cast_event(&event) };
                    xcb::xproto::map_window(&self.connection, e.window());
                    println!("MAP_REQUEST");
                }
                xcb::UNMAP_NOTIFY => {
                    let e: &xcb::UnmapNotifyEvent = unsafe { xcb::cast_event(&event) };
                    println!("UNMAP_NOTIFY");
                    self.forget_window(e.window());
                }
                xcb::DESTROY_NOTIFY => {
                    let e: &xcb::DestroyNotifyEvent = unsafe { xcb::cast_event(&event) };
                    println!("DESTROY_NOTIFY");
                }
                _ => {}
            }
        }
    }

    fn teardown(&mut self) {}

    fn get_window(&'a mut self, id: xcb::Window) -> &Window {
        let connection = &self.connection;
        self.windows.entry(id).or_insert_with(|| Window::new(connection, id))
    }

    fn forget_window(&mut self, id: xcb::Window) {
        self.windows.remove(&id);
    }
}
