extern crate regex;
extern crate rand;

use std::fs;
use clap::{Parser, Subcommand};
use libmuse::package::{search_for_packages, MPMPackage};
use libmuse::package_source::PackageSourceContent;
use libmuse::csharp_parse::compile_to_single_script;
use std::{collections::HashMap, env, path::PathBuf};

#[derive(Parser)]
#[command(name = "mpm", about = "A Rust-based package manager for Project Frontier", long_about = None)]
struct Args {
	#[command(subcommand)]
	command: MPMCommand,
}

#[derive(Subcommand)]
enum MPMCommand {
	Install,
	/// Build command takes input and output paths
	Build {
		#[arg(short, long)]
		input: PathBuf,
		#[arg(short, long)]
		output: PathBuf,
	},
}

#[tokio::main]
async fn main() {
	let args: Args = Args::parse();

	match args.command {
		MPMCommand::Install => {
			let cwd = env::current_dir().unwrap();
			println!("Searching for muse-package.toml's");
			let mpm_packages: Vec<MPMPackage> = search_for_packages(cwd.as_path());
			let mut source_cache: HashMap<String, PackageSourceContent> = HashMap::new();

			for mpm_package in mpm_packages {
				source_cache = mpm_package.solve(source_cache).await;
			}
		},
		MPMCommand::Build { 
			input, 
			output 
		} => {
			// let source_namespace_name: String = input.file_stem().unwrap().to_str().unwrap().to_string();
			let target_namespace_name: String = output.file_stem().unwrap().to_str().unwrap().to_string();
			let mut scripts: HashMap<String, String> = HashMap::new();

			println!("Building from {:?} to {:?}", input, output);
			for entry in fs::read_dir(input).unwrap() {
				let entry = entry.unwrap();
				let path = entry.path();
				
				// Ensure the entry is a file
				if path.is_file() {
					// Get the file name as a String
					if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
						// Read the file's contents into a String
						let contents: String = fs::read_to_string(&path).unwrap();
						// Insert the file name and contents into the map
						scripts.insert(name.to_owned(), contents);
					}
				}
			}
			let content: String = compile_to_single_script(
				String::from("DO NOT EDIT!\nCompiled using 'github.com/nightcycle/muse-package-manager'"),
				target_namespace_name, 
				scripts
			);
			if output.exists(){
				fs::remove_file(output.clone()).expect("bad remove");
			}

			fs::write(&output, content).expect("Unable to write to output file");
		},
	}
}
