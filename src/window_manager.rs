use super::layout::Layout; // Trait import for function access
use super::{artist, layout, window};
use std::{clone::Clone, collections::HashMap, rc::Rc};

#[derive(Default)]
pub struct WindowManager {
    windows: HashMap<window::Id, window::Window>,
    // decorators: HashMap<window::Id, Decorator>,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
}

pub struct Workspace {
    pub name: String,
    pub layout: Box<layout::Layout>,
    pub focused_window: Option<window::Id>,
    pub windows: Vec<window::Id>,
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
    let layout = layout::root(layout::focus_border(
        1,
        (0, 255, 0),
        layout::spacing(
            5,
            5,
            layout::split_right_to_left(
                0.75,
                1,
                layout::linear_right_to_left(),
                layout::linear_top_to_bottom(),
            ),
        ),
    ));
    for name in &["0", "1", "2", "3"] {
        wm.add_workspace(name, layout.clone());
    }
    wm.main_loop();
}

struct Decorator {
    id: window::Id,
    artist: Rc<artist::Artist>,
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
                xcb::CREATE_NOTIFY => self.create_notify(unsafe { xcb::cast_event(&e) }),
                xcb::DESTROY_NOTIFY => self.destroy_notify(unsafe { xcb::cast_event(&e) }),
                xcb::CONFIGURE_REQUEST => self.configure_request(unsafe { xcb::cast_event(&e) }),
                xcb::CONFIGURE_NOTIFY => self.configure_notify(unsafe { xcb::cast_event(&e) }),
                xcb::PROPERTY_NOTIFY => self.property_notify(unsafe { xcb::cast_event(&e) }),
                xcb::MAP_REQUEST => self.map_request(unsafe { xcb::cast_event(&e) }),
                xcb::MAP_NOTIFY => self.map_notify(unsafe { xcb::cast_event(&e) }),
                xcb::UNMAP_NOTIFY => self.unmap_notify(unsafe { xcb::cast_event(&e) }),
                xcb::CLIENT_MESSAGE => self.client_message(unsafe { xcb::cast_event(&e) }),
                t => eprintln!("UNEXPECTED EVENT TYPE: {}", t),
            }
            connection.flush();
        }
    }

    pub fn focus_down_stack(&mut self) {}
    pub fn focus_up_stack(&mut self) {}
    pub fn focus_on_selected(&mut self) {}
    pub fn swap_focused_with_selected(&mut self) {}
    pub fn move_focused_window_up_stack(&mut self) {}
    pub fn move_focused_window_down_stack(&mut self) {}
    pub fn move_focused_window_to_head(&mut self) {}

    fn add_workspace<A: Layout + Clone + 'static>(&mut self, name: &str, layout: A) {
        self.workspaces.push(Workspace {
            name: String::from(name),
            windows: Default::default(),
            focused_window: None,
            layout: Box::new(layout),
        })
    }

    fn create_notify(&mut self, e: &xcb::CreateNotifyEvent) {
        // TODO: don't blindly insert at the end
        // TODO: apply rules
        self.windows
            .insert(e.window(), window::Window::new(e.window()));
        self.workspaces[self.current_workspace]
            .windows
            .push(e.window());
    }

    fn destroy_notify(&mut self, e: &xcb::DestroyNotifyEvent) {
        let window = &self.windows[&e.window()];
        for ws in &mut self.workspaces {
            ws.windows.remove_item(&window.id());
        }
        self.windows.remove(&e.window());
    }

    fn configure_request(&mut self, e: &xcb::ConfigureRequestEvent) {
        // TODO: apply rules
        // If the window isn't managed by us then act on the request for frame at least
        println!(
            "Configure Request: {:x} {} {} {} {} {} {}",
            e.window(),
            e.x(),
            e.y(),
            e.width(),
            e.height(),
            e.border_width(),
            e.value_mask()
        );
    }

    fn switch_to_workspace(&mut self) {
        // 1. Copy current workspace into temp
        // 2. Unmap all windows
        // 3. Overwrite workspace from temp
        // 4. Map new workspace's windows, set input focus, update layout

    }

    fn map_request(&mut self, e: &xcb::MapRequestEvent) {
        self.windows[&e.window()].map();
        self.windows[&e.window()].set_input_focus();
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
        // TODO: move focus if required
        self.update_layout();
    }

    fn client_message(&mut self, e: &xcb::ClientMessageEvent) {
        println!("Client Message: {:x}", e.window());
    }

    fn update_layout(&mut self) {
        let ws = &self.workspaces[self.current_workspace];
        let windows: Vec<&window::Window> = ws
            .windows
            .iter()
            .map(|id| &self.windows[id])
            .filter(|w| w.is_mapped())
            .collect();
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let actions = ws.layout.layout(
            &euclid::rect(0, 0, screen.width_in_pixels(), screen.height_in_pixels()),
            &windows,
        );
        for a in actions {
            match a {
                layout::Action::Position {
                    id,
                    rect,
                    border_width,
                    border_color,
                } => self.windows[&id].set_geometry(&rect, border_width, border_color),
                _ => (),
            }
        }
    }
}
