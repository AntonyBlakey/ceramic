use crate::{
    artist::Artist,
    commands::Commands,
    connection::{get_cardinals_property, ATOM__NET_WM_STRUT},
    layout::*,
    window_data::WindowData,
    window_manager::WindowManager,
};

pub fn new<A: Layout>(child: A) -> AvoidStruts<A> {
    AvoidStruts { child }
}

#[derive(Clone)]
pub struct AvoidStruts<A: Layout> {
    child: A,
}

impl<A: Layout> Layout for AvoidStruts<A> {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<Artist>>) {
        let mut r = *rect;

        for window in &windows {
            let struts = get_cardinals_property(window.window(), *ATOM__NET_WM_STRUT);
            if struts.len() == 4 {
                let left = struts[0];
                let right = struts[1];
                let top = struts[2];
                let bottom = struts[3];
                r.origin.x += left as i16;
                r.size.width -= (left + right) as u16;
                r.origin.y += top as i16;
                r.size.height -= (top + bottom) as u16;
            }
        }

        self.child.layout(&r, windows)
    }
}

impl<A: Layout> Commands for AvoidStruts<A> {
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
