use std::{collections::HashMap, rc::Rc};

struct Window {
    id: xcb::Window,
    is_mapped: bool,
}

trait LayoutAlgorithm {
    fn layout(&self, wm: &mut WindowManager);
}

struct Workspace {
    name: String,
    layout_algorithm: Rc<LayoutAlgorithm>,
    windows: HashMap<xcb::Window, Window>,
}

pub struct WindowManager {
    connection: xcb::Connection,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
}

struct GridLayout;
impl LayoutAlgorithm for GridLayout {
    fn layout(&self, wm: &mut WindowManager) {
        let screen = wm.connection.get_setup().roots().nth(0).unwrap();
        let width = screen.width_in_pixels();
        let height = screen.height_in_pixels();

        let ws = wm.workspaces.get_mut(wm.current_workspace).unwrap();

        let mapped_windows: Vec<&Window> = ws.windows.values().filter(|w| w.is_mapped).collect();

        if mapped_windows.is_empty() {
            return;
        }

        let columns = (mapped_windows.len() as f64).sqrt().ceil() as u16;
        let rows = (mapped_windows.len() as u16 + columns - 1) / columns;

        let screen_gap = 5;
        let window_gap = 5;

        let cell_width = (width - screen_gap * 2) / columns;
        let cell_height = (height - screen_gap * 2) / rows;

        let mut row = 0;
        let mut column = 0;

        let w = cell_width - 2 * window_gap;
        let h = cell_height - 2 * window_gap;
        for window in mapped_windows {
            let x = screen_gap + cell_width * column + window_gap;
            let y = screen_gap + cell_height * row + window_gap;
            let values = [
                (xcb::xproto::CONFIG_WINDOW_X as u16, x as u32),
                (xcb::xproto::CONFIG_WINDOW_Y as u16, y as u32),
                (xcb::xproto::CONFIG_WINDOW_WIDTH as u16, w as u32),
                (xcb::xproto::CONFIG_WINDOW_HEIGHT as u16, h as u32),
            ];
            xcb::xproto::configure_window(&wm.connection, window.id, &values);
            column += 1;
            if column == columns {
                column = 0;
                row += 1;
            }
        }
    }
}

impl WindowManager {
    pub fn run() {
        let mut me = Self::new();
        me.main_loop();
    }

    fn new() -> WindowManager {
        let (connection, _screen_number) = xcb::Connection::connect(None).unwrap();

        let layout = std::rc::Rc::new(GridLayout {});

        WindowManager {
            connection,
            workspaces: ["a", "s", "d", "f"]
                .iter()
                .map(|&n| Workspace {
                    name: String::from(n),
                    windows: HashMap::new(),
                    layout_algorithm: layout.clone(),
                })
                .collect(),
            current_workspace: 0,
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
        self.workspaces[self.current_workspace].windows.insert(
            e.window(),
            Window {
                id: e.window(),
                is_mapped: false,
            },
        );
    }

    fn destroy_notify(&mut self, e: &xcb::DestroyNotifyEvent) {
        let ws = self.workspaces.get_mut(self.current_workspace).unwrap();
        ws.windows.remove(&e.window());
    }

    fn configure_request(&mut self, _: &xcb::ConfigureRequestEvent) {}

    fn map_request(&mut self, e: &xcb::MapRequestEvent) {
        let window = self.workspaces[self.current_workspace]
            .windows
            .get_mut(&e.window())
            .unwrap();
        xcb::xproto::map_window(&self.connection, window.id);
    }

    fn configure_notify(&mut self, _: &xcb::ConfigureNotifyEvent) {}

    fn property_notify(&mut self, _: &xcb::PropertyNotifyEvent) {}

    fn map_notify(&mut self, e: &xcb::MapNotifyEvent) {
        self.workspaces[self.current_workspace]
            .windows
            .get_mut(&e.window())
            .unwrap()
            .is_mapped = true;
        self.update_layout();
    }

    fn unmap_notify(&mut self, e: &xcb::UnmapNotifyEvent) {
        self.workspaces[self.current_workspace]
            .windows
            .get_mut(&e.window())
            .unwrap()
            .is_mapped = false;
        self.update_layout();
    }

    fn client_message(&mut self, _: &xcb::ClientMessageEvent) {}

    fn update_layout(&mut self) {
        self.workspaces[self.current_workspace]
            .layout_algorithm
            .clone()
            .layout(self);
    }
}
