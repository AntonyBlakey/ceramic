pub struct Window<'a> {
    connection: &'a xcb::Connection,
    id: xcb::Window,
}

impl<'a> Window<'a> {
    pub fn new(connection: &'a xcb::Connection, id: xcb::Window) -> Window {
        Window { connection, id }
    }
}