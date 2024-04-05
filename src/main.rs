use clap::{Parser, Subcommand};
use libmuse::config::{search_for_packages, MPMDirectory};
use std::env;

#[derive(Parser)]
#[command(name = "mpm", about = "A Rust-based package manager for Project Frontier", long_about = None)]
struct Args {
	#[command(subcommand)]
	command: MPMCommand,
}

#[derive(Subcommand)]
enum MPMCommand {
	/// installs all the packages in the workspace
	Install,
	/// publishes the current package
	Update,
}

fn main() {
	let args: Args = Args::parse();

	match args.command {
		MPMCommand::Install => {
			let cwd = env::current_dir().unwrap();
			let _result_configs: Vec<MPMDirectory> = search_for_packages(cwd.as_path());
			println!("Installing muse packages {:#?}", _result_configs);
		}
		MPMCommand::Update => {
			println!("Update muse packages");
		}
	}
}
