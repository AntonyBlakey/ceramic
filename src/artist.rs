use super::layout;

pub trait Artist {
    fn calculate_bounds(&self, window: xcb::Window) -> Option<layout::LayoutRect>;
    fn draw(&self, window: xcb::Window);
}
