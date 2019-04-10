use crate::{
    artist::Artist, commands::Commands, layout::*, window_data::WindowData,
};

pub fn new<A: Layout>(screen_gap: u16, window_gap: u16, child: A) -> AddGaps<A> {
    AddGaps {
        screen_gap,
        window_gap,
        child,
    }
}

#[derive(Clone)]
pub struct AddGaps<A: Layout> {
    screen_gap: u16,
    window_gap: u16,
    child: A,
}

impl<A: Layout> Layout for AddGaps<A> {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let r = Bounds::new(
            rect.origin.x + self.screen_gap as i16,
            rect.origin.y + self.screen_gap as i16,
            rect.size.width - 2 * self.screen_gap,
            rect.size.height - 2 * self.screen_gap,
        );

        let (mut new_windows, artists) = self.child.layout(&r, windows);

        for window in new_windows.iter_mut() {
            window.bounds.origin.x += self.window_gap as i16;
            window.bounds.origin.y += self.window_gap as i16;
            window.bounds.size.width -= 2 * self.window_gap;
            window.bounds.size.height -= 2 * self.window_gap;
        }

        (new_windows, artists)
    }
}

impl<A: Layout> Commands for AddGaps<A> {
    fn get_commands(&self) -> Vec<String> {
        self.child.get_commands()
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        self.child.execute_command(command, args)
    }
}
