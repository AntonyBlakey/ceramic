use super::{artist, connection::*, layout::*, workspace::Workspace};
use std::rc::Rc;

pub trait Commands {
    fn get_commands(&self) -> Vec<String> {
        Default::default()
    }
    fn execute_command(&mut self, command: &str, args: &[&str]) {
        eprintln!("Unhandled command: {}", command);
    }
}

#[derive(Default)]
pub struct WindowManager {
    // decorators: HashMap<window::Id, Decorator>,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
}

fn standard_layout_root<A: Layout + 'static>(name: &str, child: A) -> LayoutRoot {
    let add_focus_border = add_focus_border(1, (0, 255, 0), child);
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

        self.run_default_event_loop();
    }

    fn run_default_event_loop(&mut self) {
        let connection = connection();
        while let Some(e) = connection.wait_for_event() {
            self.dispatch_wm_event(&e);
        }
    }

    fn run_selector_event_loop(&mut self) -> Option<xcb::Keysym> {
        self.grab_keyboard();
        let connection = connection();
        let mut key_press_count = 0;
        let mut first_key_down: Option<xcb::Keysym> = None;
        while let Some(e) = connection.wait_for_event() {
            match e.response_type() & 0x7f {
                xcb::KEY_PRESS => {
                    let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&e) };
                    if key_press_count == 0 {
                        let keycode = press_event.detail();
                        let state = press_event.state();
                        first_key_down = Some(0);
                    } else {
                        first_key_down = None;
                    }
                    key_press_count += 1;
                }
                xcb::KEY_RELEASE => {
                    key_press_count -= 1;
                    if key_press_count == 0 {
                        break;
                    }
                }
                _ => self.dispatch_wm_event(&e),
            }
        }
        self.ungrab_keyboard();
        first_key_down
    }

    fn grab_keyboard(&self) {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        xcb::grab_keyboard(
            &connection,
            false,
            screen.root(),
            xcb::CURRENT_TIME,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_SYNC as u8,
        );
        connection.flush();
    }

    fn ungrab_keyboard(&self) {
        let connection = connection();
        xcb::ungrab_keyboard(&connection, xcb::CURRENT_TIME);
        connection.flush();
    }

    fn dispatch_wm_event(&mut self, e: &xcb::GenericEvent) {
        match e.response_type() & 0x7f {
            xcb::EXPOSE => self.expose(unsafe { xcb::cast_event(e) }),
            xcb::CONFIGURE_REQUEST => self.configure_request(unsafe { xcb::cast_event(e) }),
            xcb::PROPERTY_NOTIFY => self.property_notify(unsafe { xcb::cast_event(e) }),
            xcb::MAP_REQUEST => self.map_request(unsafe { xcb::cast_event(e) }),
            xcb::MAP_NOTIFY => self.map_notify(unsafe { xcb::cast_event(e) }),
            xcb::UNMAP_NOTIFY => self.unmap_notify(unsafe { xcb::cast_event(e) }),
            xcb::CLIENT_MESSAGE
            | xcb::CREATE_NOTIFY
            | xcb::DESTROY_NOTIFY
            | xcb::CONFIGURE_NOTIFY
            | xcb::MAPPING_NOTIFY => (),
            // expose => decorator draw (we never move decorators)
            // key press => more than one down -> cancel select
            // key release => if key press count == 1 && key is mapped then select window else no action, release grab
            t => eprintln!("UNEXPECTED EVENT TYPE: {}", t),
        }
        connection().flush();
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

    fn expose(&mut self, e: &xcb::ExposeEvent) {
        eprintln!("Expose {:x}", e.window());

        let connection = connection();
        let gc_id = connection.generate_id();
        xcb::create_gc(&connection, gc_id, e.window(), &[]);
        let rect = xcb::Rectangle::new(e.x() as i16, e.y() as i16, e.width(), e.height());
        xcb::poly_fill_rectangle(&connection, e.window(), gc_id, &[rect]);
        xcb::free_gc(&connection, gc_id);
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
            self.parse_and_dispatch_command(command.as_str());
        }
    }

    fn map_notify(&mut self, e: &xcb::MapNotifyEvent) {
        self.workspaces[self.current_workspace].add_window(e.window());
        self.update_layout();
    }

    fn unmap_notify(&mut self, e: &xcb::UnmapNotifyEvent) {
        self.workspaces[self.current_workspace].remove_window(e.window());
        self.update_layout();
    }

    fn update_layout(&mut self) {
        let actions = self.workspaces[self.current_workspace].update_layout();
        // TODO: add select-a-window decorations
        // TODO: create decorations as windows
        self.update_commands();
    }

    fn update_commands(&self) {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        set_strings_property(
            screen.root(),
            *ATOM_CERAMIC_AVAILABLE_COMMANDS,
            &self
                .get_commands()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>(),
        );
    }

    fn parse_and_dispatch_command(&mut self, command_string: &str) {
        let mut tokens = command_string.split(' ');
        if let Some(command) = tokens.next() {
            let args = tokens.collect::<Vec<_>>();
            self.execute_command(command, &args);
        }
    }
}

impl Commands for WindowManager {
    fn get_commands(&self) -> Vec<String> {
        let mut commands = self.workspaces[self.current_workspace].get_commands();
        if self.workspaces.len() > 1 {
            commands.push(String::from("switch_to_workspace_named:"));
            commands.push(String::from("move_focused_window_to_workspace_named:"));
            commands.push(String::from("switch_to_next_workspace"));
            commands.push(String::from("switch_to_previous_workspace"));
        }
        commands.push(String::from("quit"));
        commands.push(String::from("reload"));
        commands
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) {
        match command {
            "switch_to_workspace_named:" => {}
            "move_focused_window_to_workspace_named:" => {}
            "switch_to_next_workspace" => {}
            "switch_to_previous_workspace" => {}
            "quit" => {}
            "reload" => {}
            _ => {
                self.workspaces[self.current_workspace].execute_command(command, args);
                self.update_layout();
            }
        }
    }
}
