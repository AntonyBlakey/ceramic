use super::window_manager::WindowManager;

pub trait Commands {
    fn get_commands(&self) -> Vec<String> {
        Default::default()
    }
    // TODO: can simplify to a bool return indicating if update_layout is required
    fn execute_command(&mut self, command: &str, _args: &[&str]) -> bool {
        eprintln!("Unhandled command: {}", command);
        false
    }
}
