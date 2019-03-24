pub struct Window {
    pub id: xcb::Window,
}

impl Window {
    pub fn new(id: xcb::Window) -> Window {
        Window { id }
    }
}
