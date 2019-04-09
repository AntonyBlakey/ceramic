use super::{layout::Bounds, window_data::WindowData};

pub trait Artist {
    fn calculate_bounds(&self, window: xcb::Window) -> Option<Bounds>;
    fn draw(&self, window: xcb::Window);
}
