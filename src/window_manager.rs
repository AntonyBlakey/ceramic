use super::{artist, connection::*, layout::*};
use std::rc::Rc;

pub trait Commandable {
    fn commands(&self) -> Vec<String> {
        Default::default()
    }
    fn execute(&mut self, command: String) {
        eprintln!("Unhandled command: {}", command);
    }
}

#[derive(Default)]
pub struct WindowManager {
    // decorators: HashMap<window::Id, Decorator>,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
}

impl Commandable for WindowManager {
    fn commands(&self) -> Vec<String> {
        Default::default()
    }
    fn execute(&mut self, command: String) {
        eprintln!("Unhandled command: {}", command);
    }
}

pub struct Workspace {
    pub name: String,
    pub layouts: Vec<LayoutRoot>,
    pub current_layout: usize,
    pub windows: Vec<WindowData>,
    pub focused_window: Option<usize>,
}

impl Commandable for Workspace {
    fn commands(&self) -> Vec<String> {
        Default::default()
    }
    fn execute(&mut self, command: String) {
        eprintln!("Unhandled command: {}", command);
    }
}

pub struct WindowData {
    pub id: xcb::Window,
    pub is_floating: bool,
    pub floating_frame: Option<LayoutRect>,
}

impl Commandable for WindowData {
    fn commands(&self) -> Vec<String> {
        Default::default()
    }
    fn execute(&mut self, command: String) {
        eprintln!("Unhandled command: {}", command);
    }
}

fn standard_layout_root<A: Layout + 'static>(name: &str, child: A) -> LayoutRoot {
    let add_focus_border = add_focus_border(2, (0, 255, 0), child);
    let add_gaps = add_gaps(5, 5, add_focus_border);
    let ignore_some_windows = ignore_some_windows(add_gaps);
    let avoid_struts = avoid_struts(ignore_some_windows);
    root(name, avoid_struts)
}

fn layouts() -> Vec<LayoutRoot> {
    vec![
        standard_layout_root(
            "monad_tall_right",
            monad(Direction::Decreasing, Axis::X, 0.75, 1),
        ),
        standard_layout_root(
            "monad_wide_top",
            monad(Direction::Increasing, Axis::Y, 0.75, 1),
        ),
    ]
}

pub fn run() {
    let mut wm = WindowManager::default();
    for i in 1..=9 {
        wm.add_workspace(&format!("{}", i), layouts());
    }
    wm.main_loop();
}

struct Decorator {
    id: xcb::Window,
    artist: Rc<artist::Artist>,
}

impl WindowManager {
    fn main_loop(&mut self) {
        // TODO: handle all screens
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY
                | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT
                | xcb::EVENT_MASK_PROPERTY_CHANGE,
        )];
        xcb::change_window_attributes_checked(&connection, screen.root(), &values)
            .request_check()
            .expect("Cannot install as window manager");
        self.set_initial_root_window_properties();

        // TODO: process all the pre-existing windows

        while let Some(e) = connection.wait_for_event() {
            match e.response_type() {
                xcb::CONFIGURE_REQUEST => self.configure_request(unsafe { xcb::cast_event(&e) }),
                xcb::PROPERTY_NOTIFY => self.property_notify(unsafe { xcb::cast_event(&e) }),
                xcb::MAP_REQUEST => self.map_request(unsafe { xcb::cast_event(&e) }),
                xcb::MAP_NOTIFY => self.map_notify(unsafe { xcb::cast_event(&e) }),
                xcb::UNMAP_NOTIFY => self.unmap_notify(unsafe { xcb::cast_event(&e) }),
                xcb::CLIENT_MESSAGE
                | xcb::CREATE_NOTIFY
                | xcb::DESTROY_NOTIFY
                | xcb::CONFIGURE_NOTIFY
                | xcb::MAPPING_NOTIFY => (),
                t => eprintln!("UNEXPECTED EVENT TYPE: {}", t),
            }
            connection.flush();
        }
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

    fn add_workspace(&mut self, name: &str, layouts: Vec<LayoutRoot>) {
        self.workspaces.push(Workspace {
            name: String::from(name),
            windows: Default::default(),
            focused_window: None,
            layouts,
            current_layout: 0,
        })
    }

    fn configure_request(&mut self, e: &xcb::ConfigureRequestEvent) {
        // TODO: apply rules
        // If the window isn't managed by us then act on the request for frame at least
        // println!("Configure Request: {:x}", e.window());
    }

    fn map_request(&mut self, e: &xcb::MapRequestEvent) {
        xcb::map_window(&connection(), e.window());
    }

    fn property_notify(&mut self, e: &xcb::PropertyNotifyEvent) {
        if e.atom() == *ATOM_CERAMIC_COMMAND && e.state() == xcb::PROPERTY_NEW_VALUE as u8 {
            let command = get_string_property(e.window(), e.atom());
            xcb::delete_property(&connection(), e.window(), e.atom());
            let ws = &mut self.workspaces[self.current_workspace];
            let layout_name = format!("layout/{}/", ws.layouts[ws.current_layout].name());
            if command.starts_with(layout_name.as_str()) {
                ws.layouts[ws.current_layout]
                    .execute(command.split_at(7 + layout_name.len()).1.to_owned());
                self.update_layout();
            } else if command.starts_with("goto_workspace/") {
                let workspace_name = command.split_at(14).1;
            } else if command.starts_with("move_window_to_workspace/") {
                let workspace_name = command.split_at(24).1;
            } else {
                match command.as_str() {
                    "goto_next_workspace" => {}
                    "goto_previous_workspace" => {}
                    "move_window_to_head" => {
                        let ws = &mut self.workspaces[self.current_workspace];
                        // Wrap around
                        let new_index = 0;
                        let window = ws.windows.remove(ws.focused_window.unwrap());
                        ws.windows.insert(new_index, window);
                        self.set_focused_window(Some(new_index));
                        self.update_layout();
                    }
                    "move_window_forward" => {
                        let ws = &mut self.workspaces[self.current_workspace];
                        // Wrap around
                        let new_index = (ws.focused_window.unwrap() + 1) % ws.windows.len();
                        let window = ws.windows.remove(ws.focused_window.unwrap());
                        ws.windows.insert(new_index, window);
                        self.set_focused_window(Some(new_index));
                        self.update_layout();
                    }
                    "move_window_backward" => {
                        let ws = &mut self.workspaces[self.current_workspace];
                        // Wrap around
                        let new_index =
                            (ws.focused_window.unwrap() + ws.windows.len() - 1) % ws.windows.len();
                        let window = ws.windows.remove(ws.focused_window.unwrap());
                        ws.windows.insert(new_index, window);
                        self.set_focused_window(Some(new_index));
                        self.update_layout();
                    }
                    "focus_next_window" => {
                        let ws = &mut self.workspaces[self.current_workspace];
                        // Wrap around
                        let new_index = (ws.focused_window.unwrap() + 1) % ws.windows.len();
                        self.set_focused_window(Some(new_index));
                        self.update_layout();
                    }
                    "focus_previous_window" => {
                        let ws = &self.workspaces[self.current_workspace];
                        // Wrap around
                        let new_index =
                            (ws.focused_window.unwrap() + ws.windows.len() - 1) % ws.windows.len();
                        self.set_focused_window(Some(new_index));
                        self.update_layout();
                    }
                    "move_window_to_window_X/" => {}
                    "swap_window_with_window_X/" => {}
                    "focus_window_X/" => {}
                    "close_window" => {
                        let ws = &self.workspaces[self.current_workspace];
                        let window = ws.windows[ws.focused_window.unwrap()].id;
                        xcb::kill_client(&connection(), window);
                    }
                    "quit" => {}
                    "reload" => {}
                    _ => eprintln!("Invalid command: {}", command),
                }
            }
        }
    }

    fn map_notify(&mut self, e: &xcb::MapNotifyEvent) {
        let ws = &mut self.workspaces[self.current_workspace];
        // TODO: maybe we don't want to focus the new window?
        let data = WindowData {
            id: e.window(),
            is_floating: false,
            floating_frame: None,
        };
        match ws.focused_window {
            Some(index) => {
                ws.windows.insert(index, data);
                self.set_focused_window(Some(index));
            }
            None => {
                ws.windows.insert(0, data);
                self.set_focused_window(Some(0));
            }
        }
        self.update_layout();
    }

    fn unmap_notify(&mut self, e: &xcb::UnmapNotifyEvent) {
        let ws = &mut self.workspaces[self.current_workspace];
        if let Some(pos) = ws.windows.iter().position(|w| w.id == e.window()) {
            ws.windows.remove(pos);
            if ws.windows.is_empty() {
                self.set_focused_window(None)
            } else {
                let new_fw = match ws.focused_window {
                    Some(index) => {
                        if pos < index {
                            Some(index - 1)
                        } else {
                            Some(index)
                        }
                    }
                    _ => None,
                };
                self.set_focused_window(new_fw);
            }
            self.update_layout();
        }
    }

    fn set_focused_window(&mut self, w: Option<usize>) {
        let ws = &mut self.workspaces[self.current_workspace];
        ws.focused_window = w;
        match w {
            Some(index) => {
                let window = ws.windows[index].id;
                let connection = connection();
                xcb::set_input_focus(
                    &connection,
                    xcb::INPUT_FOCUS_NONE as u8,
                    window,
                    xcb::CURRENT_TIME,
                );
                let screen = connection.get_setup().roots().nth(0).unwrap();
                set_window_property(screen.root(), *ATOM__NET_ACTIVE_WINDOW, window);
            }
            _ => {}
        }
    }

    fn update_layout(&mut self) {
        let ws = &self.workspaces[self.current_workspace];
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let windows = ws.windows.iter().collect::<Vec<&WindowData>>();
        let actions = ws.layouts[ws.current_layout].layout(
            &euclid::rect(0, 0, screen.width_in_pixels(), screen.height_in_pixels()),
            &windows,
        );
        for a in actions {
            match a {
                Action::Position {
                    id,
                    rect,
                    border_width,
                    border_color,
                } => {
                    if border_width > 0 {
                        xcb::change_window_attributes(
                            &connection,
                            id,
                            &[(xcb::CW_BORDER_PIXEL, border_color)],
                        );
                    }
                    xcb::configure_window(
                        &connection,
                        id,
                        &[
                            (
                                xcb::CONFIG_WINDOW_X as u16,
                                (rect.origin.x - border_width) as u32,
                            ),
                            (
                                xcb::CONFIG_WINDOW_Y as u16,
                                (rect.origin.y - border_width) as u32,
                            ),
                            (xcb::CONFIG_WINDOW_WIDTH as u16, rect.size.width as u32),
                            (xcb::CONFIG_WINDOW_HEIGHT as u16, rect.size.height as u32),
                            (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border_width as u32),
                        ],
                    );
                }
                _ => (),
            }
        }

        self.update_commands();
    }

    fn update_commands(&self) {
        let ws = &self.workspaces[self.current_workspace];
        let commands = ws.layouts[ws.current_layout].commands();
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        set_strings_property(
            screen.root(),
            *ATOM_CERAMIC_AVAILABLE_COMMANDS,
            &commands.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        );
    }
}
