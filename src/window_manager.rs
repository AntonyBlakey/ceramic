use super::{artist, connection::*, layout::*, workspace::Workspace};
use std::{collections::HashMap, rc::Rc};

pub trait Commands {
    fn get_commands(&self) -> Vec<String> {
        Default::default()
    }
    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        eprintln!("Unhandled command: {}", command);
        None
    }
}

#[derive(Default)]
pub struct WindowManager {
    workspaces: Vec<Workspace>,
    current_workspace: usize,
    decorations: HashMap<xcb::Window, Rc<artist::Artist>>,
    selector_command: Option<String>,
}

struct WindowSelectorArtist {
    labels: Vec<String>,
    windows: Vec<xcb::Window>,
}

struct Point {
    x: u16,
    y: u16,
}

impl WindowSelectorArtist {
    const FONT_FACE: &'static str = "Noto Sans Mono";
    const FONT_SIZE: u16 = 12;

    const MARGIN: Point = Point { x: 6, y: 2 };
    const LABEL_PADDING: Point = Point { x: 4, y: 2 };
    const LABEL_TO_NAME_GAP: u16 = 6;
    const LINE_SPACING: u16 = 2;

    fn configure_label_font(&self, context: &cairo::Context) {
        context.select_font_face(
            Self::FONT_FACE,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Bold,
        );
        context.set_font_size(Self::FONT_SIZE as f64);
    }

    fn configure_name_font(&self, context: &cairo::Context) {
        context.select_font_face(
            Self::FONT_FACE,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        context.set_font_size(Self::FONT_SIZE as f64);
    }
}

impl artist::Artist for WindowSelectorArtist {
    fn calculate_bounds(&self, window: xcb::Window) -> Option<LayoutRect> {
        match xcb::get_geometry(&connection(), self.windows[0]).get_reply() {
            Ok(geometry) => {
                if let Ok(surface) = get_cairo_surface(window) {
                    let context = cairo::Context::new(&surface);

                    self.configure_label_font(&context);
                    let font_extents = context.font_extents();
                    let line_height = font_extents.height.ceil() as u16;

                    let mut label_width = 0;
                    for label in &self.labels {
                        let text_extents = context.text_extents(label);
                        label_width = label_width.max(text_extents.width.ceil() as u16);
                    }

                    self.configure_name_font(&context);
                    let mut name_width = 0;
                    for window in &self.windows {
                        let name = get_string_property(*window, *ATOM__NET_WM_NAME);
                        let text_extents = context.text_extents(&name);
                        name_width = name_width.max(text_extents.width.ceil() as u16);
                    }

                    return Some(euclid::rect(
                        geometry.x() as u16,
                        geometry.y() as u16,
                        Self::MARGIN.x
                            + Self::LABEL_PADDING.x
                            + label_width
                            + Self::LABEL_PADDING.x
                            + Self::LABEL_TO_NAME_GAP
                            + name_width
                            + Self::MARGIN.x,
                        Self::MARGIN.y
                            + self.labels.len() as u16
                                * (Self::LABEL_PADDING.y
                                    + line_height
                                    + Self::LABEL_PADDING.y
                                    + Self::LINE_SPACING)
                            - Self::LINE_SPACING
                            + Self::MARGIN.y,
                    ));
                }
            }
            _ => {}
        }

        None
    }

    fn draw(&self, window: xcb::Window) {
        if let Ok(surface) = get_cairo_surface(window) {
            let context = cairo::Context::new(&surface);

            self.configure_label_font(&context);
            let font_extents = context.font_extents();
            let line_height = font_extents.height.ceil() as u16;
            let ascent = font_extents.ascent;

            let mut label_width = 0;
            for label in &self.labels {
                let text_extents = context.text_extents(label);
                label_width = label_width.max(text_extents.width.ceil() as u16);
            }

            {
                let mut top = Self::MARGIN.y;
                let left = Self::MARGIN.x;
                let right = left + Self::LABEL_PADDING.x + label_width + Self::LABEL_PADDING.x;
                for label in &self.labels {
                    let bottom = top + Self::LABEL_PADDING.y + line_height + Self::LABEL_PADDING.y;

                    context.set_source_rgb(0.4, 0.0, 0.0);
                    context.move_to(left as f64, top as f64);
                    context.line_to(right as f64, top as f64);
                    context.line_to(right as f64, bottom as f64);
                    context.line_to(left as f64, bottom as f64);
                    context.close_path();
                    context.fill();

                    context.set_source_rgb(1.0, 1.0, 1.0);
                    context.move_to(
                        (left + Self::LABEL_PADDING.x) as f64,
                        (top + Self::LABEL_PADDING.y) as f64 + ascent,
                    );
                    context.show_text(label);

                    top = bottom + Self::LINE_SPACING;
                }
            }

            {
                self.configure_name_font(&context);
                let mut top = Self::MARGIN.y;
                let left = Self::MARGIN.x
                    + Self::LABEL_PADDING.x
                    + label_width
                    + Self::LABEL_PADDING.x
                    + Self::LABEL_TO_NAME_GAP;
                context.set_source_rgb(0.2, 0.2, 0.5);
                for window in &self.windows {
                    let bottom = top + Self::LABEL_PADDING.y + line_height + Self::LABEL_PADDING.y;

                    let name = get_string_property(*window, *ATOM__NET_WM_NAME);
                    context.move_to(left as f64, (top + Self::LABEL_PADDING.y) as f64 + ascent);
                    context.show_text(&name);

                    top = bottom + Self::LINE_SPACING;
                }
            }
        }
    }
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
        // standard_layout_root(
        //     "monad_tall_right_stack",
        //     monad_stack(Direction::Decreasing, Axis::X, 0.75, 1),
        // ),
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

impl WindowManager {
    fn main_loop(&mut self) {
        // TODO: handle all screens
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY
                | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT
                | xcb::EVENT_MASK_PROPERTY_CHANGE
                | xcb::EVENT_MASK_KEY_PRESS
                | xcb::EVENT_MASK_KEY_RELEASE,
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
            if let Some(c) = &self.selector_command {
                let command = c.clone();
                eprintln!("Run a selector for: {}", command);
                self.selector_command = None;
                if let Some(keysym) = self.run_window_selector_event_loop() {
                    // dispatch command + keysym selected window
                    let args = vec!["12345678"];
                    self.execute_command(&command, &args);
                }
            }
        }
    }

    fn run_window_selector_event_loop(&mut self) -> Option<xcb::Keysym> {
        eprintln!("Enter selector loop");
        self.grab_keyboard();
        let connection = connection();
        let mut key_press_count = 0;
        let mut first_key_down: Option<xcb::Keysym> = None;
        while let Some(e) = connection.wait_for_event() {
            eprintln!("Selector loop event: {}", e.response_type());
            match e.response_type() & 0x7f {
                xcb::KEY_PRESS => {
                    let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&e) };
                    if key_press_count == 0 {
                        eprintln!("Begin key listening");
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
                        eprintln!("End key listening");
                        break;
                    }
                }
                _ => self.dispatch_wm_event(&e),
            }
        }
        self.ungrab_keyboard();
        eprintln!("Exit selector loop");
        first_key_down
    }

    fn grab_keyboard(&self) {
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
            Ok(_) => eprintln!("Grabbed yeyboard"),
            Err(x) => eprintln!("Failed to grab keyboard: {:?}", x),
        }
        connection.flush();
    }

    fn ungrab_keyboard(&self) {
        let connection = connection();
        xcb::ungrab_keyboard(&connection, xcb::CURRENT_TIME);
        eprintln!("Ungrabbed keyboard");
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
        if e.count() == 0 {
            let window = e.window();
            if let Some(artist) = self.decorations.get(&window) {
                artist.draw(window);
            }
        }
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
        if !self.decorations.contains_key(&e.window()) {
            self.workspaces[self.current_workspace].add_window(e.window());
            self.update_layout();
        }
    }

    fn unmap_notify(&mut self, e: &xcb::UnmapNotifyEvent) {
        if !self.decorations.contains_key(&e.window()) {
            self.workspaces[self.current_workspace].remove_window(e.window());
            self.update_layout();
        } else {
            self.decorations.remove(&e.window());
        }
    }

    // Public because it is called by the command result functions
    pub fn update_layout(&mut self) {
        let mut actions = self.workspaces[self.current_workspace].update_layout();
        self.add_selector_actions(&mut actions);
        self.process_actions(&actions);
        self.update_commands();
    }

    fn add_selector_actions(&self, actions: &mut Vec<Action>) {
        if let Some(_command) = &self.selector_command {
            let mut selector_chars = "ASDFGHJKLQWERTYUIOPZXCVBNM1234567890".chars();
            let mut selector_artists: Vec<(LayoutRect, WindowSelectorArtist)> = Vec::new();
            for action in actions.iter() {
                match action {
                    Action::Position {
                        id,
                        rect,
                        border_width: _,
                        border_color: _,
                    } => match selector_chars.next() {
                        Some(c) => {
                            let label = format!("{}", c);
                            match selector_artists.iter_mut().find(|(r, _)| {
                                r.origin.x == rect.origin.x && r.origin.y == rect.origin.y
                            }) {
                                Some((_, artist)) => {
                                    artist.labels.push(label);
                                    artist.windows.push(*id);
                                }
                                None => {
                                    selector_artists.push((
                                        *rect,
                                        WindowSelectorArtist {
                                            labels: vec![label],
                                            windows: vec![*id],
                                        },
                                    ));
                                }
                            }
                        }
                        None => {}
                    },
                    _ => (),
                }
            }

            for (_, artist) in selector_artists {
                actions.push(Action::Draw {
                    artist: Rc::new(artist),
                });
            }
        }
    }

    fn process_actions(&mut self, actions: &Vec<Action>) {
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
        for window in self.decorations.keys().clone() {
            xcb::destroy_window(&connection, *window);
        }

        for action in actions {
            match action {
                Action::Draw { artist } => {
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
                            self.decorations.insert(new_id, artist.clone());
                        }
                        None => {
                            xcb::destroy_window(&connection, new_id);
                        }
                    }
                }
                _ => {}
            }
        }
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
            match self.execute_command(command, &args) {
                Some(f) => f(self),
                None => (),
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
        match args.get(0) {
            Some(&"{window}") => {
                self.selector_command = Some(command.to_owned());
                self.update_layout();
                None
            }
            _ => match command {
                "switch_to_workspace_named:" => None,
                "move_focused_window_to_workspace_named:" => None,
                "switch_to_next_workspace" => None,
                "switch_to_previous_workspace" => None,
                "quit" => None,
                "reload" => None,
                _ => self.workspaces[self.current_workspace].execute_command(command, args),
            },
        }
    }
}
