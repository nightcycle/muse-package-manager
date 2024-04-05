use clap::{Parser, Subcommand, ValueEnum};
use libmuse::package::{search_for_packages, MPMDirectory};
use std::env;

#[derive(ValueEnum, Clone, Debug)]
enum VersionBump {
    Major,
    Minor,
    Patch,
}


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
	/// Publishes the current package with a version bump
	Update {
		#[arg(short, long, value_enum)]
		version: VersionBump,
	},
}

fn main() {
	let args: Args = Args::parse();

	match args.command {
		MPMCommand::Install => {
			let cwd = env::current_dir().unwrap();
			let _result_configs: Vec<MPMDirectory> = search_for_packages(cwd.as_path());
			println!("Installing muse packages {:#?}", _result_configs);
		}
		MPMCommand::Update { version } => {
			match version {
			    VersionBump::Major => println!("Update muse packages - Major version"),
			    VersionBump::Minor => println!("Update muse packages - Minor version"),
			    VersionBump::Patch => println!("Update muse packages - Patch version"),
			}
		 }
	}
}
