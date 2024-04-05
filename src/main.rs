use clap::{Parser, Subcommand};
use libmuse::package::{search_for_packages, MPMPackage};
use std::env;

// #[derive(ValueEnum, Clone, Debug)]
// enum VersionBump {
//     Major,
//     Minor,
//     Patch,
// }


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
	// /// Publishes the current package with a version bump
	// Update {
	// 	#[arg(short, long, value_enum)]
	// 	version: VersionBump,
	// },
}

#[tokio::main]
async fn main() {
	let args: Args = Args::parse();

	match args.command {
		MPMCommand::Install => {
			let cwd = env::current_dir().unwrap();
			let mpm_packages: Vec<MPMPackage> = search_for_packages(cwd.as_path());
			
			println!("Installing muse packages {:#?}", mpm_packages);
			for mpm_package in mpm_packages {
				for mpm_dependency in mpm_package.dependencies{
					mpm_dependency.source.download().await;
				}
			}
		}
		// MPMCommand::Update { version } => {
		// 	match version {
		// 	    VersionBump::Major => println!("Update muse packages - Major version"),
		// 	    VersionBump::Minor => println!("Update muse packages - Minor version"),
		// 	    VersionBump::Patch => println!("Update muse packages - Patch version"),
		// 	}
		//  }
	}
}
