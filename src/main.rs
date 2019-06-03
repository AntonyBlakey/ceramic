#![feature(vec_remove_item, trait_alias)]
#![recursion_limit = "128"]

mod artist;
mod commands;
mod config;
mod connection;
mod layout;
mod window_data;
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
    args.verbosity.setup_env_logger("ceramic").unwrap();
    window_manager::WindowManager::new(config::Configuration::new()).run();
}
