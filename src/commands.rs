use super::window_manager;

pub trait Commands {
    fn get_commands(&self) -> Vec<String> {
        Default::default()
    }
    fn execute_command(
        &mut self,
        command: &str,
        _args: &[&str],
    ) -> Option<Box<Fn(&mut window_manager::WindowManager)>> {
        eprintln!("Unhandled command: {}", command);
        None
    }
}
