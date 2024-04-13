extern crate regex;
extern crate rand;

use std::fs;
use std::io::Read;
use std::str::FromStr;
use clap::{Parser, Subcommand};
use libmuse::package::{search_for_packages, MPMPackage, find_package, FILE_NAME_STRING};
use libmuse::package_source::PackageSourceContent;
use libmuse::csharp_parse::compile_to_single_script;
use std::{collections::HashMap, env, path::PathBuf, path::Path};
use base64::{encode_config, decode_config, URL_SAFE_NO_PAD};
use semver::Version;
#[derive(Parser)]
#[command(name = "mpm", about = "A Rust-based package manager for Project Frontier", long_about = None)]
struct Args {
	#[command(subcommand)]
	command: MPMCommand,
}

#[derive(Subcommand)]
enum MPMCommand {
	Install {
		#[arg(short = 'c', long)]
		path: Option<PathBuf>,
	},
	/// Build command takes input and output paths
	Build {
		#[arg(short, long)]
		input: PathBuf,
		#[arg(short, long)]
		output: PathBuf,
	},
}

const CACHE_DIR_NAME: &str = ".mpm-cache";

fn encode_str_to_path_safe_b64(value: String) -> String {
	encode_config(value, URL_SAFE_NO_PAD)
}

fn decode_path_safe_b64_to_str(encoded: &str) -> String {
	String::from_utf8(decode_config(encoded, URL_SAFE_NO_PAD).unwrap()).unwrap()
}

#[tokio::main]
async fn main() {
	let args: Args = Args::parse();

	match args.command {
		MPMCommand::Install { 
			path, 
		} => {
			let cwd = env::current_dir().unwrap();
			let cwd_path: &Path = cwd.as_path();
			let mut mpm_packages: Vec<MPMPackage> = Vec::new();
			if path.is_some(){
				let package_path = path.unwrap();
				let mpm_package_opt = find_package(package_path.as_path());
				let mpm_package = mpm_package_opt.expect(format!("couldn't find '{}' at '{}'", FILE_NAME_STRING, package_path.to_str().unwrap()).as_str());
				mpm_packages.insert(mpm_packages.len(), mpm_package);
			}else{
				println!("Searching for muse-package.toml's");
				mpm_packages = search_for_packages(cwd_path);
			}


			let mut source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>> = HashMap::new();
			let cache_path_buff: PathBuf = cwd_path.join(CACHE_DIR_NAME);
			let cache_path: &Path = cache_path_buff.as_path();
			if cache_path.exists(){
				for dir_entry in fs::read_dir(cache_path).unwrap() {
					let dir_entry: fs::DirEntry = dir_entry.unwrap();
					let dir_path: PathBuf = dir_entry.path();
					let dir_name: String = decode_path_safe_b64_to_str(&dir_path.file_name().unwrap().to_str().unwrap());
					let source_url_key: PathBuf = PathBuf::from_str(&dir_name).unwrap();

					// println!("dir_name={}", source_url_key.to_str().unwrap());

					if dir_path.is_dir(){
						let mut version_cache: HashMap<Version, PackageSourceContent> = HashMap::new();
						for file_entry in fs::read_dir(dir_path).unwrap() {
							let file_entry: fs::DirEntry = file_entry.unwrap();
							let file_path: PathBuf = file_entry.path();
							let file_name = file_path.file_stem().unwrap().to_str().unwrap();
							// println!("file_name={}", file_name);
							let version_name: String = decode_path_safe_b64_to_str(file_name);
							let version: Version = Version::parse(&version_name).unwrap();
							let mut file: fs::File = fs::File::open(file_path).unwrap();
							let mut buffer: Vec<u8> = Vec::new();
							file.read_to_end(&mut buffer).unwrap();

							let data: bytes::Bytes = bytes::Bytes::from(buffer);
							let source_url = source_url_key.clone();
							version_cache.insert(version.clone(), PackageSourceContent{
								data,
								version,
								source_url
							});
							// println!("version_name={}", version_name);
						}
						source_cache.insert(source_url_key, version_cache);
					}

				}
			}
			for mpm_package in mpm_packages {
				source_cache = mpm_package.solve(source_cache).await;
			}
			if cache_path.exists() == false{
				let _ = fs::create_dir_all(cache_path).unwrap();
			}
			for (path_buf, version_cache) in source_cache {
				let dir_name: String = encode_str_to_path_safe_b64(path_buf.to_str().unwrap().to_string());
				let dir_name_path: PathBuf = PathBuf::from_str(&dir_name).unwrap();
				let sub_dir_path: PathBuf = cache_path.to_path_buf().join(dir_name_path);
				if sub_dir_path.exists() == false{
					let _ = fs::create_dir_all(sub_dir_path.clone()).unwrap();
				}
				for (version, psc) in version_cache {
					let file_name: String = encode_str_to_path_safe_b64(version.to_string());
					let mut file_name_path: PathBuf = PathBuf::from_str(&file_name).unwrap();
					file_name_path.set_extension("zip");
					let full_file_path: PathBuf = sub_dir_path.clone().join(file_name_path);
					fs::write(full_file_path, psc.data).expect("write fail");
				}
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
