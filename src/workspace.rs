use super::{
    artist::Artist, commands::Commands, connection::*, layout::layout_root::LayoutRoot, layout::*,
    window_data::WindowData,
};

pub struct Workspace {
    pub name: String,
    pub layouts: Vec<LayoutRoot>,
    pub current_layout: usize,
    pub windows: Vec<WindowData>,
    pub focused_window: Option<usize>,
}

impl Workspace {
    pub fn new(name: &str, layouts: Vec<LayoutRoot>) -> Workspace {
        Workspace {
            name: name.into(),
            layouts,
            current_layout: 0,
            windows: Default::default(),
            focused_window: None,
        }
    }

    pub fn show(&self) {
        let connection = connection();
        self.windows.iter().for_each(|w| {
            xcb::map_window(&connection, w.window());
        });
        self.synchronize_focused_window_with_os();
    }

    pub fn hide(&self) {
        let connection = connection();
        self.windows.iter().for_each(|w| {
            xcb::unmap_window(&connection, w.window());
        });
    }

    pub fn notify_window_mapped(&mut self, window: xcb::Window, is_floating: bool) -> bool {
        if self.windows.iter().find(|w| w.window() == window).is_some() {
            return false;
        }
        let mut data = WindowData::new(window);
        data.is_floating = is_floating;
        match self.focused_window {
            Some(index) => {
                self.windows.insert(index, data);
                self.set_focused_window(Some(index));
            }
            None => {
                self.windows.insert(0, data);
                self.set_focused_window(Some(0));
            }
        }
        return true;
    }

    pub fn notify_window_unmapped(&mut self, _window: xcb::Window) -> bool {
        return false;
    }

    pub fn notify_window_destroyed(&mut self, window: xcb::Window) -> bool {
        if let Some(pos) = self.windows.iter().position(|w| w.window() == window) {
            self.windows.remove(pos);
            if self.windows.is_empty() {
                self.set_focused_window(None)
            } else {
                let new_fw = match self.focused_window {
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
            return true;
        } else {
            return false;
        }
    }

    pub fn request_configure(&mut self, e: &xcb::ConfigureRequestEvent) -> bool {
        if let Some(window) = self.windows.iter_mut().find(|w| w.window() == e.window()) {
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
        false
    }

    pub fn update_layout(&mut self, bounds: &Bounds) -> Vec<Box<Artist>> {
        let (new_windows, artists) =
            self.layouts[self.current_layout].layout(bounds, self.windows.clone());

        // TODO: only configure changed windows

        self.windows = new_windows;

        for window in &self.windows {
            window.configure();
        }

        artists
    }

    fn set_focused_window(&mut self, w: Option<usize>) {
        self.focused_window = w;
        match self.focused_window {
            Some(index) => {
                if 0 < index && self.windows[index].is_floating {
                    let prefix = self.windows.drain(0..index).collect::<Vec<_>>();
                    let mut insertion_position = self.insertion_position_for_tiled_window();
                    for window in prefix.into_iter() {
                        self.windows.insert(insertion_position, window);
                        insertion_position += 1;
                    }
                    self.focused_window = Some(0);
                }
            }
            _ => {}
        }
        self.synchronize_focused_window_with_os();
    }

    fn synchronize_focused_window_with_os(&self) {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let windows = self
            .windows
            .iter()
            .map(|w| w.window())
            .collect::<Vec<xcb::Window>>();
        if let Some(index) = self.focused_window {
            xcb::set_input_focus(
                &connection,
                xcb::INPUT_FOCUS_NONE as u8,
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
                xcb::INPUT_FOCUS_NONE as u8,
                xcb::NONE,
                xcb::CURRENT_TIME,
            );
            xcb::delete_property(&connection, screen.root(), *ATOM__NET_ACTIVE_WINDOW);
        }
        for index in (1..windows.len()).rev() {
            let values = [
                (
                    xcb::CONFIG_WINDOW_STACK_MODE as u16,
                    xcb::STACK_MODE_ABOVE as u32,
                ),
                (xcb::CONFIG_WINDOW_SIBLING as u16, windows[index] as u32),
            ];
            xcb::configure_window(&connection, windows[index - 1], &values);
        }
    }

    fn first_floating_window(&self) -> Option<usize> {
        if !self.windows.is_empty() && self.windows[0].is_floating {
            Some(0)
        } else {
            None
        }
    }

    fn first_tiled_window(&self) -> Option<usize> {
        self.windows.iter().position(|w| !w.is_floating)
    }

    fn insertion_position_for_tiled_window(&self) -> usize {
        self.first_tiled_window().unwrap_or(self.windows.len())
    }

    fn first_window_in_layer(&self, index: usize) -> usize {
        if self.windows[index].is_floating {
            0
        } else {
            self.first_tiled_window().unwrap()
        }
    }

    fn last_window_in_layer(&self, index: usize) -> usize {
        if self.windows[index].is_floating {
            self.first_tiled_window().unwrap() - 1
        } else {
            self.windows.len() - 1
        }
    }

    fn next_window_in_layer_after(&self, index: usize) -> usize {
        if index + 1 < self.windows.len()
            && self.windows[index + 1].is_floating == self.windows[index].is_floating
        {
            index + 1
        } else {
            self.first_window_in_layer(index)
        }
    }

    fn previous_window_in_layer_before(&self, index: usize) -> usize {
        if 0 < index && self.windows[index - 1].is_floating == self.windows[index].is_floating {
            index - 1
        } else {
            self.last_window_in_layer(index)
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
            if let Some(index) = self.focused_window {
                // TODO: this should be the count of floating vs. non-floating *focusable* windows
                if self.windows.len() > 1 {
                    // commands.push(String::from("move_to_head_after_focusing_on_window:"));
                    commands.push(String::from("float_window:"));
                    commands.push(String::from("tile_window:"));
                    commands.push(String::from("focus_on_window:"));
                    commands.push(String::from("move_focused_window_to_head"));
                    commands.push(String::from("move_focused_window_forward"));
                    commands.push(String::from("move_focused_window_backward"));
                    commands.push(String::from("focus_on_next_window_in_layer"));
                    commands.push(String::from("focus_on_next_window"));
                    commands.push(String::from("focus_on_previous_window_in_layer"));
                    commands.push(String::from("focus_on_previous_window"));
                    // commands.push(String::from("move_focused_window_to_position_of_window:"));
                    // commands.push(String::from("swap_focused_window_with_window:"));
                }
                commands.extend(self.windows[index].get_commands().into_iter());
            } else {
                commands.push(String::from("float_window:"));
                commands.push(String::from("tile_window:"));
                commands.push(String::from("focus_on_window:"));
                // if self.windows.len() > 1 {
                //     commands.push(String::from("move_to_head_after_focusing_on_window:"));
                // }
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
                "tile_window:" => match args[0].parse::<u32>() {
                    Ok(window) => match self.windows.iter().position(|w| w.window() == window) {
                        Some(index) => {
                            if self.windows[index].is_floating {
                                let new_index = self.insertion_position_for_tiled_window();
                                self.windows[index].is_floating = true;
                                self.windows.swap(index, new_index);
                                self.set_focused_window(Some(new_index));
                                true
                            } else {
                                false
                            }
                        }
                        None => false,
                    },
                    Err(_) => false,
                },
                "float_window:" => match args[0].parse::<u32>() {
                    Ok(window) => match self.windows.iter().position(|w| w.window() == window) {
                        Some(index) => {
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
                        None => false,
                    },
                    Err(_) => false,
                },
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
                _ => match self.focused_window {
                    Some(index) => match command {
                        "move_focused_window_to_head" => {
                            let new_index = self.first_window_in_layer(index);
                            self.windows.swap(index, new_index);
                            self.set_focused_window(Some(new_index));
                            true
                        }
                        "move_focused_window_forward" => {
                            let new_index = self.next_window_in_layer_after(index);
                            self.windows.swap(index, new_index);
                            self.set_focused_window(Some(new_index));
                            true
                        }
                        "move_focused_window_backward" => {
                            let new_index = self.previous_window_in_layer_before(index);
                            self.windows.swap(index, new_index);
                            self.set_focused_window(Some(new_index));
                            true
                        }
                        "focus_on_next_window" => {
                            self.set_focused_window(Some(self.next_window_in_layer_after(index)));
                            true
                        }
                        "focus_on_previous_window" => {
                            self.set_focused_window(Some(
                                self.previous_window_in_layer_before(index),
                            ));
                            true
                        }
                        // "move_focused_window_to_position_of_window:" => None,
                        // "swap_focused_window_with_window:" => None,
                        _ => self.windows[index].execute_command(command, args),
                    },
                    None => false,
                },
            }
        }
    }
}
