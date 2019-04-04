pub trait Artist {
    fn draw(&self, context: &cairo::Context);
}
