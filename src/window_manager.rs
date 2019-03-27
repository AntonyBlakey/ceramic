use super::{layout, layout::{LayoutStep, LayoutAlgorithm, LayoutAction}, window, window::Window, workspace::Workspace};
use std::{collections::HashMap, rc::Rc};

#[derive(Default)]
pub struct WindowManager {
    pub windows: HashMap<window::Id, Window>,
    pub workspaces: Vec<Workspace>,
    pub current_workspace: usize,
}

static mut CONNECTION: Option<Rc<xcb::Connection>> = None;
pub fn connection() -> Rc<xcb::Connection> {
    unsafe {
        if CONNECTION.is_none() {
            let (connection, _screen_number) = xcb::Connection::connect(None).unwrap();
            CONNECTION = Some(Rc::new(connection));
        }
        CONNECTION.clone().unwrap()
    }
}

pub fn run() {
    let mut wm = WindowManager::default();
    let layout = layout::GridLayout::default();
    for name in &["a", "s", "d", "f"] {
        wm.add_workspace(name, layout.to_layout_step());
    }
    wm.main_loop();
}

impl WindowManager {
    fn main_loop(&mut self) {
        // TODO: handle all screens
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
        )];
        xcb::change_window_attributes_checked(&connection, screen.root(), &values)
            .request_check()
            .expect("Cannot install as window manager");

        // TODO: process all the pre-existing windows

        while let Some(e) = connection.wait_for_event() {
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
            connection.flush();
        }
    }

    fn add_workspace(&mut self, name: &str, layout: LayoutStep) {
        self.workspaces.push(Workspace {
            name: String::from(name),
            windows: Default::default(),
            layout,
        })
    }

    fn create_notify(&mut self, e: &xcb::CreateNotifyEvent) {
        self.windows.insert(e.window(), Window::new(e.window()));
        let window = &self.windows[&e.window()];
        self.workspaces[self.current_workspace].add_window(window);
    }

    fn destroy_notify(&mut self, e: &xcb::DestroyNotifyEvent) {
        let window = &self.windows[&e.window()];
        for ws in &mut self.workspaces {
            ws.remove_window(window);
        }
        self.windows.remove(&e.window());
    }

    fn configure_request(&mut self, e: &xcb::ConfigureRequestEvent) {
        println!("Configure Request: {:x}", e.window());
    }

    fn map_request(&mut self, e: &xcb::MapRequestEvent) {
        self.windows[&e.window()].map();
    }

    fn configure_notify(&mut self, e: &xcb::ConfigureNotifyEvent) {
        println!("Configure Notify: {:x}", e.window());
    }

    fn property_notify(&mut self, e: &xcb::PropertyNotifyEvent) {
        println!("Property Notify: {:x}", e.window());
    }

    fn map_notify(&mut self, e: &xcb::MapNotifyEvent) {
        self.windows.get_mut(&e.window()).unwrap().map_notify();
        self.update_layout();
    }

    fn unmap_notify(&mut self, e: &xcb::UnmapNotifyEvent) {
        self.windows.get_mut(&e.window()).unwrap().unmap_notify();
        self.update_layout();
    }

    fn client_message(&mut self, e: &xcb::ClientMessageEvent) {
        println!("Client Message: {:x}", e.window());
    }

    fn update_layout(&mut self) {
        let ws = &self.workspaces[self.current_workspace];
        let windows: Vec<&Window> = ws.windows.iter().map(|id|&self.windows[id]).collect();
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let actions = ws
            .layout
            .layout(&euclid::rect(0, 0, screen.width_in_pixels(), screen.height_in_pixels()), &windows);
        for a in actions {
            match a {
                LayoutAction::Position { id, rect } =>
                    self.windows[&id].set_geometry(&rect),
                _ => (),
            }
        }
    }
}
