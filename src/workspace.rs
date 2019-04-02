use super::{connection::*, layout::*, window_data::WindowData, window_manager::Commands};

pub struct Workspace {
    pub name: String,
    pub layouts: Vec<LayoutRoot>,
    pub current_layout: usize,
    pub windows: Vec<WindowData>,
    pub focused_window: Option<usize>,
}

impl Workspace {
    pub fn add_window(&mut self, window: xcb::Window) {
        let data = WindowData {
            id: window,
            is_floating: false,
            floating_frame: None,
        };
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
    }

    pub fn remove_window(&mut self, window: xcb::Window) {
        if let Some(pos) = self.windows.iter().position(|w| w.id == window) {
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

    pub fn update_layout(&mut self) -> Vec<Action> {
        let connection = connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let windows = self.windows.iter().collect::<Vec<&WindowData>>();
        let actions = self.layouts[self.current_layout].layout(
            &euclid::rect(0, 0, screen.width_in_pixels(), screen.height_in_pixels()),
            &windows,
        );

        for a in &actions {
            match a {
                Action::Position {
                    id,
                    rect,
                    border_width,
                    border_color,
                } => {
                    if let Some(pos) = self.windows.iter().position(|w| w.id == *id) {
                        self.windows[pos].configure(rect, *border_width, *border_color);
                    }
                }
                _ => (),
            }
        }

        actions
    }

    fn set_focused_window(&mut self, w: Option<usize>) {
        self.focused_window = w;
        if let Some(index) = w {
            self.windows[index].set_input_focus();
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
        if self.windows.len() > 0 {
            if let Some(index) = self.focused_window {
                // TODO: this should be the count of floating vs. non-floating *focusable* windows
                if self.windows.len() > 1 {
                    commands.push(String::from("focus_on_window_with_id:"));
                    commands.push(String::from("move_focused_window_to_head"));
                    commands.push(String::from("move_focused_window_forward"));
                    commands.push(String::from("move_focused_window_backward"));
                    commands.push(String::from("move_focus_to_next_window"));
                    commands.push(String::from("move_focus_to_previous_window"));
                    commands.push(String::from(
                        "move_focused_window_to_position_of_window_with_id:",
                    ));
                    commands.push(String::from("swap_focused_window_with_window_with_id:"));
                }
                commands.extend(self.windows[index].get_commands().into_iter());
            } else {
                commands.push(String::from("focus_on_window_with_id:"));
            }
        }
        commands
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) {
        if command.starts_with("layout/") {
            self.layouts[self.current_layout].execute_command(command.split_at(7).1, args);
        } else {
            match command {
                "switch_to_next_layout" => {}
                "switch_to_previous_layout" => {}
                "switch_to_layout_named:" => {}
                "focus_on_window_with_id:" => {}
                _ => {
                    if let Some(index) = self.focused_window {
                        match command {
                            "move_focused_window_to_head" => {
                                // Wrap around
                                let new_index = 0;
                                let window = self.windows.remove(index);
                                self.windows.insert(new_index, window);
                                self.set_focused_window(Some(new_index));
                            }
                            "move_focused_window_forward" => {
                                // Wrap around
                                let new_index = (index + 1) % self.windows.len();
                                let window = self.windows.remove(index);
                                self.windows.insert(new_index, window);
                                self.set_focused_window(Some(new_index));
                            }
                            "move_focused_window_backward" => {
                                // Wrap around
                                let new_index =
                                    (index + self.windows.len() - 1) % self.windows.len();
                                let window = self.windows.remove(index);
                                self.windows.insert(new_index, window);
                                self.set_focused_window(Some(new_index));
                            }
                            "move_focus_to_next_window" => {
                                // Wrap around
                                let new_index = (index + 1) % self.windows.len();
                                self.set_focused_window(Some(new_index));
                            }
                            "move_focus_to_previous_window" => {
                                // Wrap around
                                let new_index =
                                    (index + self.windows.len() - 1) % self.windows.len();
                                self.set_focused_window(Some(new_index));
                            }
                            "move_focused_window_to_position_of_window_with_id:" => {}
                            "swap_focused_window_with_window_with_id:" => {}
                            _ => self.windows[index].execute_command(command, args),
                        }
                    }
                }
            }
        }
    }
}
