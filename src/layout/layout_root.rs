use crate::{artist::Artist, commands::Commands, layout::*, window_data::WindowData};

pub fn new(name: &str, child: Box<dyn Layout>) -> LayoutRoot {
    LayoutRoot::new(name, child)
}

pub struct LayoutRoot {
    name: String,
    child: Box<dyn Layout>,
}

impl LayoutRoot {
    pub fn new(name: &str, child: Box<dyn Layout>) -> LayoutRoot {
        LayoutRoot {
            name: name.to_owned(),
            child,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl Layout for LayoutRoot {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<dyn Artist>>) {
        self.child.layout(rect, windows)
    }
}

impl Commands for LayoutRoot {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        self.child.execute_command(command, args)
    }
}
