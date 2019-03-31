use super::{
    artist,
    connection::*,
    layout,
    layout::{Axis, Direction, Layout},
    window,
};
use maplit::hashmap;
use std::{collections::HashMap, rc::Rc};

#[derive(Default)]
pub struct WindowManager {
    windows: HashMap<window::Id, window::Window>,
    // decorators: HashMap<window::Id, Decorator>,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
}

#[derive(Clone)]
pub struct Workspace {
    pub name: String,
    pub layouts: Rc<HashMap<&'static str, Box<layout::Layout>>>,
    pub current_layout: &'static str,
    pub windows: Vec<window::Id>,
    pub focused_window: Option<window::Id>,
}

fn standard_layout_root<A: Default + Layout + 'static>(child: A) -> Box<layout::Layout> {
    let add_focus_border = layout::add_focus_border(2, (0, 255, 0), child);
    let add_gaps = layout::add_gaps(5, 5, add_focus_border);
    let ignore_some_windows = layout::ignore_some_windows(add_gaps);
    let avoid_struts = layout::avoid_struts(ignore_some_windows);
    let root = layout::root(avoid_struts);

    Box::new(root)
}

fn layouts() -> HashMap<&'static str, Box<layout::Layout>> {
    hashmap! {
        "monad_tall_right" => standard_layout_root(layout::monad(Direction::Decreasing, Axis::X, 0.75, 1)),
        "monad_wide_top" => standard_layout_root(layout::monad(Direction::Increasing, Axis::Y, 0.75, 1)),
    }
}

pub fn run() {
    let mut wm = WindowManager::default();
    for i in 1..=9 {
        wm.add_workspace(&format!("{}", i), layouts());
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
        self.set_initial_root_window_properties();

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
    pub fn switch_to_workspace(&mut self, workspace_number: usize) {
        // 1. Copy current workspace into temp
        // 2. Unmap all windows
        // 3. Overwrite workspace from temp
        // 4. Map new workspace's windows, set input focus, update layout
    }

    fn set_initial_root_window_properties(&self) {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let check_window_id = connection.generate_id();
        xcb::create_window(
            &connection,
            xcb::COPY_FROM_PARENT as u8,
            check_window_id,
            screen.root(),
            -100,
            -100,
            1,
            1,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[(xcb::CW_OVERRIDE_REDIRECT, 1)],
        );
        set_string_property(check_window_id, *ATOM__NET_WM_NAME, "ceramic");
        set_window_property(
            check_window_id,
            *ATOM__NET_SUPPORTING_WM_CHECK,
            check_window_id,
        );
        set_window_property(
            screen.root(),
            *ATOM__NET_SUPPORTING_WM_CHECK,
            check_window_id,
        );
        set_cardinal_property(screen.root(), *ATOM__NET_CURRENT_DESKTOP, 0);
        set_cardinal_property(
            screen.root(),
            *ATOM__NET_NUMBER_OF_DESKTOPS,
            self.workspaces.len() as u32,
        );
        set_strings_property(
            screen.root(),
            *ATOM__NET_DESKTOP_NAMES,
            &self
                .workspaces
                .iter()
                .map(|ws| ws.name.as_str())
                .collect::<Vec<_>>(),
        );
        set_atoms_property(
            screen.root(),
            *ATOM__NET_SUPPORTED,
            &[
                *ATOM__NET_SUPPORTING_WM_CHECK,
                *ATOM__NET_WM_NAME,
                *ATOM__NET_WM_DESKTOP,
                *ATOM__NET_WM_STRUT,
                *ATOM__NET_NUMBER_OF_DESKTOPS,
                *ATOM__NET_CURRENT_DESKTOP,
                *ATOM__NET_DESKTOP_NAMES,
                *ATOM__NET_ACTIVE_WINDOW,
            ],
        );
        connection.flush();
    }

    fn add_workspace(&mut self, name: &str, layouts: HashMap<&'static str, Box<layout::Layout>>) {
        let first_layout = *(layouts.keys().nth(0).unwrap());
        self.workspaces.push(Workspace {
            name: String::from(name),
            windows: Default::default(),
            focused_window: None,
            layouts: Rc::new(layouts),
            current_layout: first_layout,
        })
    }

    fn create_notify(&mut self, e: &xcb::CreateNotifyEvent) {
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
        // println!("Configure Request: {:x}", e.window());
    }

    fn map_request(&mut self, e: &xcb::MapRequestEvent) {
        self.windows[&e.window()].map();
        // TODO: give focus only to windows that want it
        // TODO: when restoring a workspace, don't do this
        // TODO: probaby just normally don't do this
        // TODO: maybe put this call into the update
        self.windows[&e.window()].set_input_focus();
    }

    fn configure_notify(&mut self, e: &xcb::ConfigureNotifyEvent) {
        // println!("Configure Notify: {:x}", e.window());
    }

    fn property_notify(&mut self, e: &xcb::PropertyNotifyEvent) {
        println!("Property Notify: {:x}", e.window());
    }

    fn map_notify(&mut self, e: &xcb::MapNotifyEvent) {
        // TODO: If this is the first mapping, apply rules
        // and insert in the right spot
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
        let actions = ws.layouts[&ws.current_layout].layout(
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
