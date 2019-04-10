use super::{
    artist::Artist, commands::Commands, config, connection::*, layout::LayoutRoot,
    workspace::Workspace,
};
use std::collections::HashMap;

pub fn run() {
    let mut wm = WindowManager::default();
    config::configure(&mut wm);

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
    wm.set_initial_root_window_properties();

    // TODO: process all the pre-existing windows

    wm.run_default_event_loop();
}

#[derive(Default)]
pub struct WindowManager {
    workspaces: Vec<Workspace>,
    current_workspace: usize,
    decorations: HashMap<xcb::Window, Box<Artist>>,
}

impl WindowManager {
    // public because it is called by the config
    pub fn add_workspace(&mut self, name: &str, layouts: Vec<LayoutRoot>) {
        self.workspaces.push(Workspace {
            name: String::from(name),
            windows: Default::default(),
            focused_window: None,
            layouts,
            current_layout: 0,
        })
    }
    // public because it is called by the command result functions
    pub fn update_layout(&mut self) {
        let artists = self.workspaces[self.current_workspace].update_layout();
        self.update_decorators(artists);
        self.update_available_commands();
    }

    pub fn do_command(&mut self, command: &str, args: &[&str]) {
        eprintln!("execute command: {} {:?}", command, args);
        match self.execute_command(command, args) {
            Some(f) => f(self),
            None => (),
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

    fn run_default_event_loop(&mut self) {
        let connection = connection();
        while let Some(e) = connection.wait_for_event() {
            self.dispatch_wm_event(&e);
        }
    }

    fn run_keygrab_event_loop(&mut self) -> Option<String> {
        let mut key_press_count = 0;
        let mut selected_label: Option<String> = None;
        grab_keyboard();
        allow_events();
        let connection = connection();
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&connection);
        while let Some(e) = connection.wait_for_event() {
            match e.response_type() & 0x7f {
                xcb::KEY_PRESS => {
                    let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&e) };
                    if key_press_count == 0 {
                        let keycode = press_event.detail();
                        let keysym = key_symbols.get_keysym(keycode, 0);
                        if keysym != xcb::base::NO_SYMBOL {
                            let cstr = unsafe {
                                std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
                            };
                            selected_label =
                                cstr.to_str().ok().map(|s| s.to_owned().to_uppercase());
                        }
                    } else {
                        selected_label = None;
                    }
                    key_press_count += 1;
                }
                xcb::KEY_RELEASE => {
                    key_press_count -= 1;
                    if key_press_count == 0 {
                        break;
                    }
                }
                _ => {
                    self.dispatch_wm_event(&e);
                }
            }
            allow_events();
        }
        ungrab_keyboard();
        allow_events();
        selected_label
    }

    fn dispatch_wm_event(&mut self, e: &xcb::GenericEvent) {
        match e.response_type() & 0x7f {
            xcb::EXPOSE => {
                let e: &xcb::ExposeEvent = unsafe { xcb::cast_event(e) };
                if e.count() == 0 {
                    let window = e.window();
                    if let Some(artist) = self.decorations.get(&window) {
                        artist.draw(window);
                    }
                }
            }

            xcb::PROPERTY_NOTIFY => {
                let e: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(e) };
                if e.atom() == *ATOM_CERAMIC_COMMAND && e.state() == xcb::PROPERTY_NEW_VALUE as u8 {
                    let command = get_string_property(e.window(), e.atom());
                    xcb::delete_property(&connection(), e.window(), e.atom());
                    self.parse_and_dispatch_command(command.as_str());
                }
            }

            xcb::MAP_REQUEST => {
                // TODO: maybe we don't need redirection, only notification?
                let e: &xcb::MapRequestEvent = unsafe { xcb::cast_event(e) };
                xcb::map_window(&connection(), e.window());
            }

            xcb::MAP_NOTIFY => {
                let e: &xcb::MapNotifyEvent = unsafe { xcb::cast_event(e) };
                if !self.decorations.contains_key(&e.window()) {
                    self.workspaces[self.current_workspace].add_window(e.window());
                    self.update_layout();
                }
            }

            xcb::UNMAP_NOTIFY => {
                let e: &xcb::UnmapNotifyEvent = unsafe { xcb::cast_event(e) };
                if !self.decorations.contains_key(&e.window()) {
                    self.workspaces[self.current_workspace].remove_window(e.window());
                    self.update_layout();
                }
            }

            xcb::CONFIGURE_REQUEST
            | xcb::CLIENT_MESSAGE
            | xcb::CREATE_NOTIFY
            | xcb::DESTROY_NOTIFY
            | xcb::CONFIGURE_NOTIFY
            | xcb::MAPPING_NOTIFY => (),
            t => eprintln!("UNEXPECTED EVENT TYPE: {}", t),
        }
        connection().flush();
    }

    fn update_decorators(&mut self, artists: Vec<Box<Artist>>) {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let root = screen.root();
        let root_visual = screen.root_visual();

        let values = [
            (xcb::CW_BACK_PIXEL, screen.white_pixel()),
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE),
            (xcb::CW_OVERRIDE_REDIRECT, 1),
        ];

        // TODO: reuse decoration windows

        for window in self.decorations.keys().copied() {
            xcb::destroy_window(&connection, window);
        }
        self.decorations.clear();

        for artist in artists {
            let new_id = connection.generate_id();
            xcb::create_window(
                &connection,
                xcb::COPY_FROM_PARENT as u8,
                new_id,
                root,
                -1,
                -1,
                1,
                1,
                0,
                xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                root_visual,
                &values,
            );
            match artist.calculate_bounds(new_id) {
                Some(bounds) => {
                    xcb::configure_window(
                        &connection,
                        new_id,
                        &[
                            (xcb::CONFIG_WINDOW_X as u16, bounds.origin.x as u32),
                            (xcb::CONFIG_WINDOW_Y as u16, bounds.origin.y as u32),
                            (xcb::CONFIG_WINDOW_WIDTH as u16, bounds.size.width as u32),
                            (xcb::CONFIG_WINDOW_HEIGHT as u16, bounds.size.height as u32),
                        ],
                    );
                    xcb::map_window(&connection, new_id);
                    self.decorations.insert(new_id, artist);
                }
                None => {
                    xcb::destroy_window(&connection, new_id);
                }
            }
        }
    }

    fn update_available_commands(&self) {
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
            if let Ok(args) = tokens
                .map(|token| match token {
                    "{window}" => {
                        self.do_command("show_window_selector_labels", &[]);
                        let selected_label = self.run_keygrab_event_loop();
                        self.do_command("hide_window_selector_labels", &[]);
                        selected_label
                            .and_then(|label| {
                                self.workspaces[self.current_workspace]
                                    .windows
                                    .iter()
                                    .find(|w| w.selector_label == label)
                                    .map(|w| format!("{}", w.id()))
                            })
                            .ok_or(())
                    }
                    _ => Ok(token.to_owned()),
                })
                .collect::<Result<Vec<String>, ()>>()
            {
                self.do_command(
                    command,
                    &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                );
            }
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

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        match command {
            "switch_to_workspace_named:" => None,
            "move_focused_window_to_workspace_named:" => None,
            "switch_to_next_workspace" => None,
            "switch_to_previous_workspace" => None,
            "quit" => None,
            "reload" => None,
            _ => self.workspaces[self.current_workspace].execute_command(command, args),
        }
    }
}

fn grab_keyboard() {
    let connection = connection();
    let screen = connection.get_setup().roots().nth(0).unwrap();
    match xcb::grab_keyboard(
        &connection,
        false,
        screen.root(),
        xcb::CURRENT_TIME,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_SYNC as u8,
    )
    .get_reply()
    {
        Ok(_) => (),
        Err(x) => eprintln!("Failed to grab keyboard: {:?}", x),
    }
    connection.flush();
}

fn ungrab_keyboard() {
    let connection = connection();
    xcb::ungrab_keyboard(&connection, xcb::CURRENT_TIME);
    connection.flush();
}

fn allow_events() {
    let connection = connection();
    xcb::xproto::allow_events(
        &connection,
        xcb::ALLOW_SYNC_KEYBOARD as u8,
        xcb::CURRENT_TIME,
    );
    connection.flush();
}
