use core::str;
use std::fs;

use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use serde::Deserialize;
use semver::Version;
use super::package_source::{PackageSource, PackageSourceContent};
use std::collections::HashMap;

const FILE_NAME_STRING: &str = "muse-package.toml";

#[derive(Debug, Deserialize)]
struct RawMPMConfig {
	pub deprecated: Option<bool>,
	pub dependencies: HashMap<String, String>,
}

impl RawMPMConfig {
	fn new(config_file_path: &Path ) -> Self {
		let contents_result: std::prelude::v1::Result<String, std::io::Error> = fs::read_to_string(config_file_path);
		let contents_sting: String = contents_result.expect("bad path");
		let contents: &str = contents_sting.as_str();
		
		return toml::from_str(contents).expect("bad config file");
	}
}


#[derive(Debug, Deserialize)]
pub struct MPMDependency{
	pub name: String,
	pub path_buf: PathBuf,
	source: PackageSource,
}

impl MPMDependency {
	fn new(name: String, path_buf: PathBuf, value: String) -> Self {
		let source: PackageSource = PackageSource::new(value);
		return MPMDependency {
			name,
			path_buf,
			source
		};	
	}
	pub async fn solve(self: Self, original_source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>>) -> HashMap<PathBuf, HashMap<Version, PackageSourceContent>>{
		let mut new_source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>> = original_source_cache.clone();

		let content: String;
		(new_source_cache, content) = self.source.solve(self.name, new_source_cache.clone()).await;
		if self.path_buf.exists(){
			fs::remove_file(self.path_buf.clone()).expect("remove fail");
		}

		fs::write(self.path_buf, content).expect("write fail");

		return new_source_cache;
	}
}

#[derive(Debug, Deserialize)]
pub struct MPMPackage {
	pub name: String,
	pub config_path_buf: PathBuf,
	pub is_deprecated: Option<bool>,
	pub dependencies: Vec<MPMDependency>,
}

impl MPMPackage {
	fn new(config_file_path: &Path) -> Self {
		let raw_config: RawMPMConfig = RawMPMConfig::new(config_file_path);
		let name: String = config_file_path.parent() // Option<&Path>
			.and_then(|p| p.file_name()) // Option<&OsStr>
			.and_then(|os_str| os_str.to_str()) // Option<&str>
			.expect("Failed to extract directory name from path") // Panics if None
			.to_owned();

		let is_deprecated: Option<bool> = raw_config.deprecated;

		let mut dependencies: Vec<MPMDependency> = Vec::new();

		let dir_path_buf = config_file_path.parent().unwrap().to_path_buf();
		for (dep_name, dep_value) in raw_config.dependencies {
			let dep_file_name: String = format!("{}.cs", dep_name);
			let dep_path_buf = dir_path_buf.join(&dep_file_name);
			let dependency: MPMDependency = MPMDependency::new(
				dep_name.clone(), 
				dep_path_buf,
				dep_value.clone()
			);
			dependencies.push(dependency);
		}
		let config_path_buf: PathBuf = config_file_path.to_path_buf(); //.to_str().expect("string conversion fail").to_owned().to_string();

		return MPMPackage {
			name,
			config_path_buf,
			is_deprecated,
			dependencies
		};
	}

	pub async fn solve(self: Self, original_source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>>) -> HashMap<PathBuf, HashMap<Version, PackageSourceContent>>{
		let mut new_source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>> = original_source_cache.clone();

		for mpm_dependency in self.dependencies{
			new_source_cache = mpm_dependency.solve(new_source_cache).await;
		}

		return new_source_cache;
	}
}

/// Searches for files named `file_name` under the given `start_dir` directory and returns a Vec with the paths to the files found.
pub fn search_for_packages(start_dir: &Path) -> Vec<MPMPackage> {
	let mut found_configs: Vec<MPMPackage> = Vec::new();

	for entry in WalkDir::new(start_dir).follow_links(true).into_iter().filter_map(|e| e.ok())
	{
		let path: &Path = entry.path();
		if path
			.file_name()
			.and_then(|n: &std::ffi::OsStr| n.to_str())
			.map(|n| n == FILE_NAME_STRING)
			.unwrap_or(false)
		{
			found_configs.push(MPMPackage::new(path));
		}
	}

	return found_configs;
}
