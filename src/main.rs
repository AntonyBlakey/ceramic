#![feature(inner_deref, type_alias_enum_variants)]

mod config;
mod window_manager;

use std::path::PathBuf;
use structopt::StructOpt;
use window_manager::WindowManager;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    #[structopt(parse(from_os_str))]
    config: Vec<PathBuf>,
}

fn main() {
    let args = Args::from_args();
    args.verbosity.setup_env_logger("commando").unwrap();
    WindowManager::new(&collect_files(&args.config)).run();
}

fn collect_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut result = Vec::new();

    for path in paths.iter().map(|p| p.canonicalize().unwrap()) {
        if path.is_file() {
            result.push(path);
        } else if path.is_dir() {
            result.extend(path.read_dir().unwrap().map(|e| e.unwrap().path()));
        }
    }

    result
}
