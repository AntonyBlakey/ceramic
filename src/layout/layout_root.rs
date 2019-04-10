use crate::{
    artist::Artist, commands::Commands, layout::*, window_data::WindowData,
};

pub fn new<A: Layout + 'static>(name: &str, child: A) -> LayoutRoot {
    LayoutRoot::new(name, child)
}

pub struct LayoutRoot {
    name: String,
    child: Box<Layout>,
}

impl LayoutRoot {
    pub fn new<T: Layout + 'static>(name: &str, child: T) -> LayoutRoot {
        LayoutRoot {
            name: name.to_owned(),
            child: Box::new(child),
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
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
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
