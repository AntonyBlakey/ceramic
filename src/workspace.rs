use super::{
    artist::Artist, commands::Commands, connection::*, layout::layout_root::LayoutRoot, layout::*,
    window_data::WindowData, window_manager::WindowManager,
};

pub struct Workspace {
    pub name: String,
    pub layouts: Vec<LayoutRoot>,
    pub current_layout: usize,
    pub windows: Vec<WindowData>,
    pub focused_window: Option<usize>,
}

impl Workspace {
    pub fn add_window(&mut self, window: xcb::Window) {
        let data = WindowData::new(window);
        let is_managed = data.is_managed;
        match self.focused_window {
            Some(index) => {
                self.windows.insert(index, data);
                if is_managed {
                    self.set_focused_window(Some(index));
                }
            }
            None => {
                self.windows.insert(0, data);
                if is_managed {
                    self.set_focused_window(Some(0));
                }
            }
        }
    }

    pub fn remove_window(&mut self, window: xcb::Window) {
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
        }
    }

    pub fn update_layout(&mut self) -> Vec<Box<Artist>> {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let (new_windows, artists) = self.layouts[self.current_layout].layout(
            &Bounds::new(0, 0, screen.width_in_pixels(), screen.height_in_pixels()),
            self.windows.clone(),
        );

        // TODO: only configure changed windows

        self.windows = new_windows;

        for window in &self.windows {
            window.configure();
        }

        artists
    }

    fn set_focused_window(&mut self, w: Option<usize>) {
        self.focused_window = w;
        if let Some(index) = w {
            self.windows[index].set_input_focus();
        }
    }

    fn focusable_window_after(&self, index: usize) -> Option<usize> {
        // TODO: account for floating/non-floating

        for new_index in (index + 1)..self.windows.len() {
            if self.windows[new_index].is_managed {
                return Some(new_index);
            }
        }

        for new_index in 0..index {
            if self.windows[new_index].is_managed {
                return Some(new_index);
            }
        }

        None
    }

    fn focusable_window_before(&self, index: usize) -> Option<usize> {
        // TODO: account for floating/non-floating

        for new_index in (1..=index).rev() {
            if self.windows[new_index - 1].is_managed {
                return Some(new_index - 1);
            }
        }

        for new_index in ((index + 2)..=self.windows.len()).rev() {
            if self.windows[new_index - 1].is_managed {
                return Some(new_index - 1);
            }
        }

        None
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
                    commands.push(String::from("focus_on_window:"));
                    commands.push(String::from("move_focused_window_to_head"));
                    commands.push(String::from("move_focused_window_forward"));
                    commands.push(String::from("move_focused_window_backward"));
                    commands.push(String::from("focus_on_next_window"));
                    commands.push(String::from("focus_on_previous_window"));
                    // commands.push(String::from("move_focused_window_to_position_of_window:"));
                    // commands.push(String::from("swap_focused_window_with_window:"));
                }
                commands.extend(self.windows[index].get_commands().into_iter());
            } else {
                commands.push(String::from("focus_on_window:"));
                // if self.windows.len() > 1 {
                //     commands.push(String::from("move_to_head_after_focusing_on_window:"));
                // }
            }
        }
        commands
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        if command.starts_with("layout/") {
            self.layouts[self.current_layout].execute_command(command.split_at(7).1, args)
        } else {
            match command {
                "switch_to_next_layout" => {
                    eprintln!("switch to next layout");
                    self.current_layout = (self.current_layout + 1) % self.layouts.len();
                    Some(Box::new(|w| w.update_layout()))
                }
                "switch_to_previous_layout" => {
                    eprintln!("switch to previous layout");
                    self.current_layout =
                        (self.current_layout + self.layouts.len() - 1) % self.layouts.len();
                    Some(Box::new(|w| w.update_layout()))
                }
                "switch_to_layout_named:" => {
                    eprintln!("switch to layout named: {}", args[0]);
                    match self.layouts.iter().position(|l| l.name() == args[0]) {
                        Some(index) => {
                            self.current_layout = index;
                            Some(Box::new(|w| w.update_layout()))
                        }
                        None => None,
                    }
                }
                "focus_on_window:" => match args[0].parse::<u32>() {
                    Ok(window) => match self.windows.iter().position(|w| w.window() == window) {
                        Some(index) => {
                            self.set_focused_window(Some(index));
                            Some(Box::new(|w| w.update_layout()))
                        }
                        None => None,
                    },
                    Err(_) => None,
                },
                _ => match self.focused_window {
                    Some(index) => match command {
                        "move_focused_window_to_head" => {
                            // TODO: account for floating/non-floating
                            // Floating or stacked needs to re-order the actual windows
                            let window = self.windows.remove(index);
                            self.windows.insert(0, window);
                            self.set_focused_window(Some(0));
                            Some(Box::new(|w| w.update_layout()))
                        }
                        "move_focused_window_forward" => match self.focusable_window_after(index) {
                            Some(new_index) => {
                                // Floating or stacked needs to re-order the actual windows
                                self.windows.swap(index, new_index);
                                self.set_focused_window(Some(new_index));
                                Some(Box::new(|w| w.update_layout()))
                            }
                            None => None,
                        },
                        "move_focused_window_backward" => match self.focusable_window_before(index)
                        {
                            Some(new_index) => {
                                // Floating or stacked needs to re-order the actual windows
                                self.windows.swap(index, new_index);
                                self.set_focused_window(Some(new_index));
                                Some(Box::new(|w| w.update_layout()))
                            }
                            None => None,
                        },
                        "focus_on_next_window" => match self.focusable_window_after(index) {
                            Some(new_index) => {
                                // Floating or stacked needs to re-order the actual windows
                                self.set_focused_window(Some(new_index));
                                Some(Box::new(|w| w.update_layout()))
                            }
                            None => None,
                        },
                        "focus_on_previous_window" => {
                            match self.focusable_window_before(index) {
                                Some(new_index) => {
                                    // Floating or stacked needs to re-order the actual windows
                                    self.set_focused_window(Some(new_index));
                                    Some(Box::new(|w| w.update_layout()))
                                }
                                None => None,
                            }
                        }
                        // "move_focused_window_to_position_of_window:" => None,
                        // "swap_focused_window_with_window:" => None,
                        _ => self.windows[index].execute_command(command, args),
                    },
                    None => None,
                },
            }
        }
    }
}
