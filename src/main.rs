#![feature(vec_remove_item)]

mod layout;
mod window;
mod window_manager;
mod workspace;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

fn main() {
    let args = Args::from_args();
    args.verbosity.setup_env_logger("commando").unwrap();
    window_manager::WindowManager::run();
}
