use super::{
    artist,
    connection::*,
    layout,
    layout::{Axis, Direction, Layout},
};
use std::{collections::HashMap, rc::Rc};

#[derive(Default)]
pub struct WindowManager {
    windows: HashMap<xcb::Window, Window>,
    // decorators: HashMap<window::Id, Decorator>,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
}

pub struct Window {
    pub id: xcb::Window,
    pub is_floating: bool,
    pub floating_frame: Option<layout::LayoutRect>,
}

pub struct Workspace {
    pub name: String,
    pub layouts: Vec<Box<layout::Layout>>,
    pub current_layout: usize,
    pub windows: Vec<xcb::Window>,
    pub focused_window: Option<xcb::Window>,
}

fn standard_layout_root<A: Default + Layout + 'static>(child: A) -> Box<layout::Layout> {
    let add_focus_border = layout::add_focus_border(2, (0, 255, 0), child);
    let add_gaps = layout::add_gaps(5, 5, add_focus_border);
    let ignore_some_windows = layout::ignore_some_windows(add_gaps);
    let avoid_struts = layout::avoid_struts(ignore_some_windows);
    let root = layout::root(avoid_struts);

    Box::new(root)
}

fn layouts() -> Vec<Box<layout::Layout>> {
    vec![
        standard_layout_root(layout::monad(Direction::Decreasing, Axis::X, 0.75, 1)),
        standard_layout_root(layout::monad(Direction::Increasing, Axis::Y, 0.75, 1)),
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
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_PROPERTY_CHANGE,
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
                xcb::PROPERTY_NOTIFY => self.property_notify(unsafe { xcb::cast_event(&e) }),
                xcb::MAP_REQUEST => self.map_request(unsafe { xcb::cast_event(&e) }),
                xcb::MAP_NOTIFY => self.map_notify(unsafe { xcb::cast_event(&e) }),
                xcb::UNMAP_NOTIFY => self.unmap_notify(unsafe { xcb::cast_event(&e) }),
                xcb::CLIENT_MESSAGE => (),
                xcb::CONFIGURE_NOTIFY => (),
                xcb::MAPPING_NOTIFY => (),
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

    fn add_workspace(&mut self, name: &str, layouts: Vec<Box<layout::Layout>>) {
        self.workspaces.push(Workspace {
            name: String::from(name),
            windows: Default::default(),
            focused_window: None,
            layouts,
            current_layout: 0,
        })
    }

    fn create_notify(&mut self, e: &xcb::CreateNotifyEvent) {
        self.windows.insert(
            e.window(),
            Window {
                id: e.window(),
                is_floating: false,
                floating_frame: None,
            },
        );
    }

    fn destroy_notify(&mut self, e: &xcb::DestroyNotifyEvent) {
        self.windows.remove(&e.window());
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
            println!("COMMAND: {}", command);
        }
    }

    fn map_notify(&mut self, e: &xcb::MapNotifyEvent) {
        let ws = &mut self.workspaces[self.current_workspace];
        // TODO: maybe we don't want to focus the new window?
        match ws.focused_window {
            Some(id) => {
                let index = ws.windows.iter().position(|x| *x == id).unwrap();
                ws.windows.insert(index, e.window());
            }
            None => {
                ws.windows.insert(0, e.window());
            }
        }
        self.set_focused_window(Some(e.window()));
        self.update_layout();
    }

    fn unmap_notify(&mut self, e: &xcb::UnmapNotifyEvent) {
        let ws = &mut self.workspaces[self.current_workspace];
        let mut fw = ws.focused_window;
        match fw {
            Some(id) if id == e.window() => {
                if ws.windows.len() == 1 {
                    ws.windows.remove(0);
                    fw = None;
                } else {
                    // TODO: the next window to focus might not be as simple as this
                    let index = ws.windows.iter().position(|x| *x == id).unwrap();
                    ws.windows.remove(index);
                    fw = Some(ws.windows[index.min(ws.windows.len() - 1)]);
                }
            }
            _ => {
                ws.windows.remove_item(&e.window());
            }
        };
        self.set_focused_window(fw);
        self.update_layout();
    }

    fn set_focused_window(&mut self, w: Option<xcb::Window>) {
        if self.workspaces[self.current_workspace].focused_window != w {
            self.workspaces[self.current_workspace].focused_window = w;
            match w {
                Some(id) => {
                    let connection = connection();
                    xcb::set_input_focus(
                        &connection,
                        xcb::INPUT_FOCUS_NONE as u8,
                        id,
                        xcb::CURRENT_TIME,
                    );
                    let screen = connection.get_setup().roots().nth(0).unwrap();
                    set_window_property(screen.root(), *ATOM__NET_ACTIVE_WINDOW, id);
                }
                _ => {}
            }
        }
    }

    fn update_layout(&mut self) {
        let ws = &self.workspaces[self.current_workspace];
        let windows: Vec<&Window> = ws.windows.iter().map(|id| &self.windows[id]).collect();
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let actions = ws.layouts[ws.current_layout].layout(
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
    }
}
