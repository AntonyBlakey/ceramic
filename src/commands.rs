pub trait Commands {
    fn get_commands(&self) -> Vec<String> {
        Default::default()
    }

    fn execute_command(&mut self, command: &str, _args: &[&str]) -> bool {
        eprintln!("Unhandled command: {}", command);
        false
    }
}
