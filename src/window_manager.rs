use super::{
    artist::Artist,
    commands::Commands,
    config::ConfigurationProvider,
    connection::*,
    layout::{Bounds, Position},
    workspace::Workspace,
};
use std::collections::HashMap;

pub struct WindowManager {
    configuration: Box<dyn ConfigurationProvider>,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
    unmanaged_windows: Vec<xcb::Window>,
    decorations: HashMap<xcb::Window, Box<dyn Artist>>,
}

impl WindowManager {
    pub fn new(configuration: Box<dyn ConfigurationProvider>) -> WindowManager {
        let workspaces = configuration.workspaces();
        WindowManager {
            configuration,
            workspaces,
            current_workspace: Default::default(),
            unmanaged_windows: Default::default(),
            decorations: Default::default(),
        }
    }

    pub fn run(&mut self) {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY
                | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT
                | xcb::EVENT_MASK_PROPERTY_CHANGE,
        )];
        xcb::change_window_attributes_checked(connection, screen.root(), &values)
            .request_check()
            .expect("Cannot install as window manager");
        self.set_initial_root_window_properties();

        for w in xcb::query_tree(connection, screen.root())
            .get_reply()
            .expect("Cannot get list of existing windows")
            .children()
        {
            self.absorb_window(*w);
        }

        self.workspaces[self.current_workspace].show();

        self.run_default_event_loop();
    }

    pub fn do_command(&mut self, command: &str, args: &[&str]) {
        // eprintln!("execute command: {} {:?}", command, args);
        if self.execute_command(command, args) {
            self.update_layout();
        }
    }

    fn set_initial_root_window_properties(&self) {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let check_window_id = connection.generate_id();
        xcb::create_window(
            connection,
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
        while let Some(e) = wait_for_event() {
            self.dispatch_wm_event(&e);
        }
    }

    fn run_keygrab_event_loop(&mut self) -> Option<String> {
        log::debug!("Enter grab loop");
        let mut key_press_count = 0;
        let mut selected_label: Option<String> = None;
        grab_keyboard();
        let key_symbols = xcb_util::keysyms::KeySymbols::new(connection());
        while let Some(e) = wait_for_event() {
            match e.response_type() & 0x7f {
                xcb::KEY_PRESS => {
                    let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&e) };
                    log::debug!("KEY_PRESS in grab loop");
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
                    // We may have to eat some release(s) from the triggering keystroke
                    log::debug!("KEY_RELEASE in grab loop");
                    if key_press_count > 0 {
                        key_press_count -= 1;
                        if key_press_count == 0 {
                            break;
                        }
                    }
                }
                _ => {
                    self.dispatch_wm_event(&e);
                }
            }
        }
        ungrab_keyboard();
        log::debug!("Exit grab loop with {:?}", selected_label);
        selected_label
    }

    const MINIMUM_RESIZE_WIDTH: u16 = 20;
    const MINIMUM_RESIZE_HEIGHT: u16 = 20;

    const WINDOW_MOVE_KEY_MASK: xcb::KeyButMask = xcb::KEY_BUT_MASK_MOD_1;
    const WINDOW_RESIZE_KEY_MASK: xcb::KeyButMask =
        xcb::KEY_BUT_MASK_SHIFT | xcb::KEY_BUT_MASK_MOD_1;

    fn run_window_move_event_loop(&mut self, e: &xcb::ButtonPressEvent) {
        // TODO: lock out commands?
        let window = e.event();

        let origin = match self.workspaces[self.current_workspace]
            .windows
            .iter_mut()
            .find(|w| w.window() == window)
        {
            Some(window_data) => window_data.bounds.origin,
            None => return,
        };
        let mouse_down = Position::new(e.root_x(), e.root_y());

        self.do_command("float_window:", &[format!("{}", window).as_str()]);

        while let Some(e) = wait_for_event() {
            match e.response_type() & 0x7f {
                xcb::BUTTON_RELEASE => {
                    break;
                }

                xcb::MOTION_NOTIFY => {
                    let e: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&e) };
                    if let Some(window_data) = self.workspaces[self.current_workspace]
                        .windows
                        .iter_mut()
                        .find(|w| w.window() == window)
                    {
                        let dx = e.root_x() - mouse_down.x;
                        let dy = e.root_y() - mouse_down.y;
                        window_data.bounds.origin = Position::new(origin.x + dx, origin.y + dy);

                        window_data.configure();
                        connection().flush();
                    }
                }
                _ => self.dispatch_wm_event(&e),
            }
        }
    }

    fn run_window_resize_event_loop(&mut self, e: &xcb::ButtonPressEvent) {
        // TODO: lock out commands?
        let window = e.event();

        let mut x = e.root_x();
        let mut y = e.root_y();

        self.do_command("float_window:", &[format!("{}", window).as_str()]);

        let mut adjust_origin_x = 0;
        let mut adjust_origin_y = 0;
        let mut adjust_size_width = 0;
        let mut adjust_size_height = 0;
        if let Some(window_data) = self.workspaces[self.current_workspace]
            .windows
            .iter()
            .find(|w| w.window() == window)
        {
            if e.event_x() < window_data.bounds.size.width as i16 / 3 {
                adjust_origin_x = 1;
                adjust_size_width = -1;
            } else if window_data.bounds.size.width as i16 * 2 / 3 < e.event_x() {
                adjust_size_width = 1;
            }

            if e.event_y() < window_data.bounds.size.height as i16 / 3 {
                adjust_origin_y = 1;
                adjust_size_height = -1;
            } else if window_data.bounds.size.height as i16 * 2 / 3 < e.event_y() {
                adjust_size_height = 1;
            }
        }

        while let Some(e) = wait_for_event() {
            match e.response_type() & 0x7f {
                xcb::BUTTON_RELEASE => {
                    break;
                }

                xcb::MOTION_NOTIFY => {
                    let e: &xcb::MotionNotifyEvent = unsafe { xcb::cast_event(&e) };
                    if let Some(window_data) = self.workspaces[self.current_workspace]
                        .windows
                        .iter_mut()
                        .find(|w| w.window() == window)
                    {
                        // TODO: simplify this logic if possible

                        let mut dx = e.root_x() - x;
                        let mut new_width =
                            window_data.bounds.size.width as i16 + dx * adjust_size_width;
                        if new_width < Self::MINIMUM_RESIZE_WIDTH as i16 {
                            new_width = Self::MINIMUM_RESIZE_WIDTH as i16;
                            dx = adjust_size_width
                                * (Self::MINIMUM_RESIZE_WIDTH as i16
                                    - window_data.bounds.size.width as i16);
                        }
                        x += dx;
                        window_data.bounds.origin.x += dx * adjust_origin_x;
                        window_data.bounds.size.width = new_width as u16;

                        let mut dy = e.root_y() - y;
                        let mut new_height =
                            window_data.bounds.size.height as i16 + dy * adjust_size_height;
                        if new_height < Self::MINIMUM_RESIZE_HEIGHT as i16 {
                            new_height = Self::MINIMUM_RESIZE_HEIGHT as i16;
                            dy = adjust_size_height
                                * (Self::MINIMUM_RESIZE_HEIGHT as i16
                                    - window_data.bounds.size.height as i16);
                        }
                        y += dy;
                        window_data.bounds.origin.y += dy * adjust_origin_y;
                        window_data.bounds.size.height = new_height as u16;

                        window_data.configure();
                        connection().flush();
                    }
                }
                _ => self.dispatch_wm_event(&e),
            }
        }
    }

    fn dispatch_wm_event(&mut self, e: &xcb::GenericEvent) {
        match e.response_type() & 0x7f {
            xcb::BUTTON_PRESS => {
                let e: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(e) };

                if e.state() == 0 {
                    self.do_command("focus_on_window:", &[format!("{}", e.event()).as_str()]);
                    xcb::ungrab_pointer(connection(), xcb::CURRENT_TIME);
                    xcb::send_event(
                        &connection(),
                        true,
                        e.event(),
                        xcb::EVENT_MASK_BUTTON_PRESS,
                        e,
                    );
                } else if e.state() == Self::WINDOW_MOVE_KEY_MASK as u16 {
                    self.run_window_move_event_loop(e);
                } else if e.state() == Self::WINDOW_RESIZE_KEY_MASK as u16 {
                    self.run_window_resize_event_loop(e);
                }
            }

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
                let e: &xcb::MapRequestEvent = unsafe { xcb::cast_event(e) };

                xcb::map_window(&connection(), e.window());
            }

            xcb::MAP_NOTIFY => {
                let e: &xcb::MapNotifyEvent = unsafe { xcb::cast_event(e) };

                if self.decorations.contains_key(&e.window()) {
                    return;
                }

                self.absorb_window(e.window());
                self.update_layout();
            }

            xcb::UNMAP_NOTIFY => {
                let e: &xcb::UnmapNotifyEvent = unsafe { xcb::cast_event(e) };

                self.unmanaged_windows.remove_item(&e.window());

                xcb::ungrab_button(
                    connection(),
                    xcb::BUTTON_INDEX_1 as u8,
                    e.window(),
                    xcb::MOD_MASK_ANY as u16,
                );
                self.workspaces[self.current_workspace].remove_window(e.window(), false);
                self.update_layout()
            }

            xcb::DESTROY_NOTIFY => {
                let e: &xcb::DestroyNotifyEvent = unsafe { xcb::cast_event(e) };

                if !self.decorations.contains_key(&e.window()) {
                    self.workspaces.iter_mut().for_each(|ws| {
                        ws.remove_window(e.window(), true);
                    });
                    self.update_layout();
                }
            }

            xcb::CONFIGURE_REQUEST => {
                let e: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(e) };
                if !self.decorations.contains_key(&e.window()) {
                    // TODO: unmanaged windows should be configured in response
                    self.workspaces.iter_mut().for_each(|ws| {
                        ws.request_configure(e);
                    });
                    self.update_layout();
                }
            }

            xcb::CLIENT_MESSAGE
            | xcb::CREATE_NOTIFY
            | xcb::CONFIGURE_NOTIFY
            | xcb::MAPPING_NOTIFY => (),

            _ => (), //eprintln!("UNEXPECTED EVENT TYPE: {}", e.response_type()),
        }

        connection().flush();
    }

    fn update_layout(&mut self) {
        let screen = connection().get_setup().roots().nth(0).unwrap();
        let mut bounds = Bounds::new(0, 0, screen.width_in_pixels(), screen.height_in_pixels());

        for window in &self.unmanaged_windows {
            let struts = get_cardinals_property(*window, *ATOM__NET_WM_STRUT);
            if struts.len() == 4 {
                let left = struts[0];
                let right = struts[1];
                let top = struts[2];
                let bottom = struts[3];
                bounds.origin.x += left as i16;
                bounds.size.width -= (left + right) as u16;
                bounds.origin.y += top as i16;
                bounds.size.height -= (top + bottom) as u16;
            }
        }

        let artists = self.workspaces[self.current_workspace].update_layout(&bounds);
        self.set_artists(artists);
        self.set_root_window_available_commands_property();
    }

    fn set_artists(&mut self, artists: Vec<Box<dyn Artist>>) {
        let screen = connection().get_setup().roots().nth(0).unwrap();
        let root = screen.root();
        let root_visual = screen.root_visual();

        let values = [
            (xcb::CW_BACK_PIXEL, screen.white_pixel()),
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE),
            (xcb::CW_OVERRIDE_REDIRECT, 1),
        ];

        // TODO: reuse decoration windows

        for window in self.decorations.keys().copied() {
            xcb::destroy_window(connection(), window);
        }
        self.decorations.clear();

        for artist in artists {
            let new_id = connection().generate_id();
            xcb::create_window(
                connection(),
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
                        connection(),
                        new_id,
                        &[
                            (xcb::CONFIG_WINDOW_X as u16, bounds.origin.x as u32),
                            (xcb::CONFIG_WINDOW_Y as u16, bounds.origin.y as u32),
                            (xcb::CONFIG_WINDOW_WIDTH as u16, bounds.size.width as u32),
                            (xcb::CONFIG_WINDOW_HEIGHT as u16, bounds.size.height as u32),
                        ],
                    );
                    xcb::map_window(connection(), new_id);
                    self.decorations.insert(new_id, artist);
                }
                None => {
                    xcb::destroy_window(connection(), new_id);
                }
            }
        }
    }

    fn set_workspace(&mut self, workspace: usize) -> bool {
        if workspace != self.current_workspace {
            self.workspaces[self.current_workspace].hide();
            self.current_workspace = workspace;
            self.workspaces[self.current_workspace].show();
            let screen = connection().get_setup().roots().nth(0).unwrap();
            set_cardinal_property(
                screen.root(),
                *ATOM__NET_CURRENT_DESKTOP,
                self.current_workspace as u32,
            );
            connection().flush();
            true
        } else {
            false
        }
    }

    fn set_root_window_available_commands_property(&self) {
        let screen = connection().get_setup().roots().nth(0).unwrap();
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
                    "{selected_window}" => {
                        self.do_command("layout/show_window_selector_labels", &[]);
                        let selected_label = self.run_keygrab_event_loop();
                        self.do_command("layout/hide_window_selector_labels", &[]);
                        selected_label
                            .and_then(|label| {
                                self.workspaces[self.current_workspace]
                                    .windows
                                    .iter()
                                    .find(|w| w.selector_label == label)
                                    .map(|w| format!("{}", w.window()))
                            })
                            .ok_or(())
                    }
                    "{focused_window}" => self.workspaces[self.current_workspace]
                        .focused_window_index
                        .map(|index| {
                            format!(
                                "{}",
                                self.workspaces[self.current_workspace].windows[index].window()
                            )
                        })
                        .ok_or(()),
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

    fn absorb_window(&mut self, window: xcb::Window) {
        match self.classify_window(window) {
            None => self.unmanaged_windows.push(window),
            Some(is_floating) => {
                // TODO: use symbolic representations in the config
                xcb::grab_button(
                    connection(),
                    false,
                    window,
                    (xcb::EVENT_MASK_BUTTON_1_MOTION
                        | xcb::EVENT_MASK_BUTTON_PRESS
                        | xcb::EVENT_MASK_BUTTON_RELEASE) as u16,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::NONE,
                    xcb::NONE,
                    xcb::BUTTON_INDEX_1 as u8,
                    Self::WINDOW_MOVE_KEY_MASK as u16,
                );
                xcb::grab_button(
                    connection(),
                    false,
                    window,
                    (xcb::EVENT_MASK_BUTTON_1_MOTION
                        | xcb::EVENT_MASK_BUTTON_PRESS
                        | xcb::EVENT_MASK_BUTTON_RELEASE) as u16,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::NONE,
                    xcb::NONE,
                    xcb::BUTTON_INDEX_1 as u8,
                    Self::WINDOW_RESIZE_KEY_MASK as u16,
                );
                // click-to-focus
                xcb::grab_button(
                    connection(),
                    true,
                    window,
                    xcb::EVENT_MASK_BUTTON_PRESS as u16,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::NONE,
                    xcb::NONE,
                    xcb::BUTTON_INDEX_1 as u8,
                    0,
                );
                self.workspaces[self.current_workspace].add_window(window, is_floating);
            }
        }
    }

    fn classify_window(&self, window: xcb::Window) -> Option<bool> {
        if let Ok(attributes) = xcb::get_window_attributes(connection(), window).get_reply() {
            if attributes.override_redirect() {
                return None;
            }
        };
        let wm_transient_for = get_window_property(window, xcb::ATOM_WM_TRANSIENT_FOR);
        let wm_class = get_ascii_strings_property(window, xcb::ATOM_WM_CLASS);
        let (instance_name, class_name) = if wm_class.len() == 2 {
            (wm_class.get(0), wm_class.get(1))
        } else {
            (None, None)
        };
        let net_wm_type = get_atoms_property(window, *ATOM__NET_WM_WINDOW_TYPE);
        let net_wm_state = get_atoms_property(window, *ATOM__NET_WM_STATE);

        let result = self.configuration.classify_window(
            window,
            instance_name.map(|s| s.as_str()),
            class_name.map(|s| s.as_str()),
            &net_wm_type,
            &net_wm_state,
            wm_transient_for,
        );
        result
    }
}

impl Commands for WindowManager {
    fn get_commands(&self) -> Vec<String> {
        let mut commands = self.workspaces[self.current_workspace].get_commands();
        if self.workspaces.len() > 1 {
            commands.push(String::from("move_focused_window_to_workspace_named:"));
            commands.push(String::from("switch_to_workspace_named:"));
        }
        commands.push(String::from("quit"));
        commands
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        match command {
            "move_focused_window_to_workspace_named:" => {
                match self.workspaces.iter().position(|ws| ws.name == args[0]) {
                    Some(new_workspace) if new_workspace != self.current_workspace => {
                        if let Some(window_data) =
                            self.workspaces[self.current_workspace].remove_focused_window()
                        {
                            xcb::unmap_window(connection(), window_data.window());
                            self.workspaces[new_workspace].add_window_data(window_data);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            "switch_to_workspace_named:" => {
                match self.workspaces.iter().position(|ws| ws.name == args[0]) {
                    Some(new_workspace) => self.set_workspace(new_workspace),
                    _ => false,
                }
            }
            "quit" => false,
            _ => self.workspaces[self.current_workspace].execute_command(command, args),
        }
    }
}
