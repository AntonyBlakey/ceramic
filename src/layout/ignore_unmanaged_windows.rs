use crate::{
    artist::Artist, commands::Commands, layout::*, window_data::WindowData,
    window_manager::WindowManager,
};

pub fn new<A: Layout>(child: A) -> IgnoreUnmanagedWindows<A> {
    IgnoreUnmanagedWindows { child }
}

#[derive(Clone)]
pub struct IgnoreUnmanagedWindows<A: Layout> {
    child: A,
}

impl<A: Layout> Layout for IgnoreUnmanagedWindows<A> {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let (managed, unmanaged): (Vec<WindowData>, Vec<WindowData>) =
            windows.into_iter().partition(|w| w.is_managed);
        let (mut new_windows, artists) = self.child.layout(rect, managed);
        new_windows.extend(unmanaged.into_iter());
        (new_windows, artists)
    }
}

impl<A: Layout> Commands for IgnoreUnmanagedWindows<A> {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Option<Box<Fn(&mut WindowManager)>> {
        self.child.execute_command(command, args)
    }
}
