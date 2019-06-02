use super::{
    artist::Artist, commands::Commands, connection::*, layout::layout_root::LayoutRoot, layout::*,
    window_data::WindowData,
};

pub struct Workspace {
    pub name: String,
    pub is_visible: bool,
    pub layouts: Vec<LayoutRoot>,
    pub current_layout: usize,
    pub windows: Vec<WindowData>, // top .. bottom (floating .. tiled) ordering
    pub number_of_floating_windows: usize,
    pub focused_window_index: Option<usize>,
}

impl Workspace {
    pub fn new(name: &str, layouts: Vec<LayoutRoot>) -> Workspace {
        Workspace {
            name: name.into(),
            is_visible: false,
            layouts,
            current_layout: 0,
            windows: Default::default(),
            number_of_floating_windows: 0,
            focused_window_index: None,
        }
    }

    pub fn show(&mut self) {
        self.is_visible = true;
        let connection = connection();
        for window_data in &self.windows {
            // TODO: check if this affects the ordering
            xcb::map_window(&connection, window_data.window());
        }
        self.synchronize_focused_window_with_os();
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        let connection = connection();
        for window_data in &self.windows {
            xcb::unmap_window(&connection, window_data.window());
        }
    }

    pub fn add_window(&mut self, window: xcb::Window, is_floating: bool) {
        if self.find_window(window).is_some() {
            return;
        }

        let mut data = WindowData::new(window);
        data.is_floating = is_floating;

        if let Ok(geometry) = xcb::get_geometry(&connection(), window).get_reply() {
            data.bounds = Bounds::new(
                geometry.x(),
                geometry.y(),
                geometry.width(),
                geometry.height(),
            );
        }

        self.add_window_data(data);
    }

    pub fn add_window_data(&mut self, window: WindowData) {
        let new_index = match self.focused_window_index {
            Some(index) if self.windows[index].is_floating == window.is_floating => index,
            _ => {
                if window.is_floating {
                    0
                } else {
                    self.number_of_floating_windows
                }
            }
        };

        if window.is_floating {
            self.number_of_floating_windows += 1;
        }

        self.windows.insert(new_index, window);
        self.set_focused_window(Some(new_index));
    }

    pub fn remove_window(&mut self, window: xcb::Window, force: bool) -> Option<WindowData> {
        if !force && !self.is_visible {
            return None;
        }
        self.find_window(window)
            .map(|index| self.remove_window_index(index))
    }

    pub fn remove_focused_window(&mut self) -> Option<WindowData> {
        self.focused_window_index
            .map(|index| self.remove_window_index(index))
    }

    pub fn request_configure(&mut self, e: &xcb::ConfigureRequestEvent) {
        if let Some(index) = self.find_window(e.window()) {
            let mut window = &mut self.windows[index];
            if window.is_floating {
                if e.value_mask() & xcb::CONFIG_WINDOW_X as u16 != 0 {
                    window.bounds.origin.x = e.x();
                }
                if e.value_mask() & xcb::CONFIG_WINDOW_Y as u16 != 0 {
                    window.bounds.origin.y = e.y();
                }
                if e.value_mask() & xcb::CONFIG_WINDOW_WIDTH as u16 != 0 {
                    window.bounds.size.width = e.width();
                }
                if e.value_mask() & xcb::CONFIG_WINDOW_HEIGHT as u16 != 0 {
                    window.bounds.size.height = e.height();
                }
                window.configure();
            }
        }
    }

    pub fn update_layout(&mut self, bounds: &Bounds) -> Vec<Box<Artist>> {
        let (new_windows, artists) =
            self.layouts[self.current_layout].layout(bounds, self.windows.clone());

        // TODO: only configure changed windows

        self.windows = new_windows;

        let mut ordered_windows = self.windows.iter().collect::<Vec<_>>();
        ordered_windows.sort_by(|a, b| a.order.unwrap_or(0).cmp(&b.order.unwrap_or(0)));

        let connection = connection();
        for pair in ordered_windows.windows(2) {
            let below = pair[0];
            let above = pair[1];
            xcb::configure_window(
                &connection,
                above.window(),
                &[
                    (
                        xcb::CONFIG_WINDOW_STACK_MODE as u16,
                        xcb::STACK_MODE_ABOVE as u32,
                    ),
                    (xcb::CONFIG_WINDOW_SIBLING as u16, below.window() as u32),
                ],
            );
        }

        for window in &self.windows {
            window.configure();
        }

        artists
    }

    fn remove_window_index(&mut self, index: usize) -> WindowData {
        let old_window = self.windows.remove(index);
        if old_window.is_floating {
            self.number_of_floating_windows -= 1
        }
        let focused_index = self.focused_window_index.unwrap();
        self.set_focused_window(if index < focused_index {
            Some(index - 1)
        } else if focused_index == self.windows.len() {
            None
        } else {
            Some(index)
        });
        old_window
    }

    fn wrapped_next_index_in_layer(&self, index: usize) -> usize {
        if self.windows[index].is_floating {
            if index == self.number_of_floating_windows - 1 {
                0
            } else {
                index + 1
            }
        } else {
            if index == self.windows.len() - 1 {
                self.number_of_floating_windows
            } else {
                index + 1
            }
        }
    }

    fn wrapped_previous_index_in_layer(&self, index: usize) -> usize {
        if self.windows[index].is_floating {
            if index == 0 {
                self.number_of_floating_windows - 1
            } else {
                index - 1
            }
        } else {
            if index == self.number_of_floating_windows {
                self.windows.len() - 1
            } else {
                index - 1
            }
        }
    }

    fn find_window(&self, window: xcb::Window) -> Option<usize> {
        self.windows
            .iter()
            .position(|window_data| window_data.window() == window)
    }

    fn set_focused_window(&mut self, w: Option<usize>) {
        self.focused_window_index = w;
        self.synchronize_focused_window_with_os();
    }

    fn synchronize_focused_window_with_os(&self) {
        if !self.is_visible {
            return;
        }

        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        if let Some(index) = self.focused_window_index {
            xcb::set_input_focus(
                &connection,
                xcb::INPUT_FOCUS_POINTER_ROOT as u8,
                self.windows[index].window(),
                xcb::CURRENT_TIME,
            );
            set_window_property(
                screen.root(),
                *ATOM__NET_ACTIVE_WINDOW,
                self.windows[index].window(),
            );
        } else {
            xcb::set_input_focus(
                &connection,
                xcb::INPUT_FOCUS_POINTER_ROOT as u8,
                xcb::INPUT_FOCUS_POINTER_ROOT,
                xcb::CURRENT_TIME,
            );
            xcb::delete_property(&connection, screen.root(), *ATOM__NET_ACTIVE_WINDOW);
        }
    }
}

impl Commands for Workspace {
    fn get_commands(&self) -> Vec<String> {
        let mut commands: Vec<String> = self.layouts[self.current_layout]
            .get_commands()
            .iter()
            .map(|c| format!("layout/{}", c))
            .collect();
        if self.layouts.len() > 1 {
            commands.push(String::from("switch_to_next_layout"));
            commands.push(String::from("switch_to_previous_layout"));
            commands.push(String::from("switch_to_layout_named:"));
        }
        if !self.windows.is_empty() {
            if let Some(index) = self.focused_window_index {
                if self.windows[index].is_floating {
                    commands.push(String::from("tile_focused_window"));
                } else {
                    commands.push(String::from("float_focused_window"));
                }
                // TODO: this should be the count of *focusable* windows
                if self.windows.len() > 1 {
                    commands.push(String::from("focus_on_window:"));
                    commands.push(String::from("move_focused_window_to_head"));
                    commands.push(String::from("move_focused_window_forward"));
                    commands.push(String::from("move_focused_window_backward"));
                    commands.push(String::from("focus_on_next_window_in_layer"));
                    commands.push(String::from("focus_on_next_window"));
                    commands.push(String::from("focus_on_previous_window_in_layer"));
                    commands.push(String::from("focus_on_previous_window"));
                }
                commands.extend(self.windows[index].get_commands().into_iter());
            }
        }
        commands
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        if command.starts_with("layout/") {
            self.layouts[self.current_layout].execute_command(command.split_at(7).1, args)
        } else {
            match command {
                "switch_to_next_layout" => {
                    self.current_layout = (self.current_layout + 1) % self.layouts.len();
                    true
                }
                "switch_to_previous_layout" => {
                    self.current_layout =
                        (self.current_layout + self.layouts.len() - 1) % self.layouts.len();
                    true
                }
                "switch_to_layout_named:" => {
                    match self.layouts.iter().position(|l| l.name() == args[0]) {
                        Some(index) => {
                            self.current_layout = index;
                            true
                        }
                        None => false,
                    }
                }
                "focus_on_window:" => match args[0].parse::<u32>() {
                    Ok(window) => match self.windows.iter().position(|w| w.window() == window) {
                        Some(index) => {
                            self.set_focused_window(Some(index));
                            true
                        }
                        None => false,
                    },
                    Err(_) => false,
                },
                _ => match self.focused_window_index {
                    Some(index) => match command {
                        "move_focused_window_to_head" => {
                            let new_index = if self.windows[index].is_floating {
                                0
                            } else {
                                self.number_of_floating_windows
                            };
                            self.windows.swap(index, new_index);
                            self.set_focused_window(Some(new_index));
                            true
                        }
                        "move_focused_window_forward" => {
                            let new_index = self.wrapped_next_index_in_layer(index);
                            self.windows.swap(index, new_index);
                            self.set_focused_window(Some(new_index));
                            true
                        }
                        "move_focused_window_backward" => {
                            let new_index = self.wrapped_previous_index_in_layer(index);
                            self.windows.swap(index, new_index);
                            self.set_focused_window(Some(new_index));
                            true
                        }
                        "focus_on_next_window" => {
                            self.set_focused_window(Some(self.wrapped_next_index_in_layer(index)));
                            true
                        }
                        "focus_on_previous_window" => {
                            self.set_focused_window(Some(
                                self.wrapped_previous_index_in_layer(index),
                            ));
                            true
                        }
                        "float_focused_window" => {
                            if !self.windows[index].is_floating {
                                let new_index = 0;
                                self.windows[index].is_floating = true;
                                self.windows.swap(index, new_index);
                                self.set_focused_window(Some(new_index));
                                true
                            } else {
                                false
                            }
                        }
                        "tile_focused_window" => {
                            if self.windows[index].is_floating {
                                let new_index = self.number_of_floating_windows;
                                self.number_of_floating_windows -= 1;
                                self.windows[index].is_floating = false;
                                self.windows.swap(index, new_index);
                                self.set_focused_window(Some(new_index));
                                true
                            } else {
                                false
                            }
                        }
                        _ => self.windows[index].execute_command(command, args),
                    },
                    None => false,
                },
            }
        }
    }
}
