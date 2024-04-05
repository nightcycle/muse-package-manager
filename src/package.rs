use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use serde::Deserialize;
use semver::VersionReq;
use std::collections::HashMap;
use regex::Regex;

const FILE_NAME_STRING: &str = "muse-package.toml";

#[derive(Debug, Deserialize)]
pub enum SourceType {
	Unknown,
	GitHubRelease,
}


#[derive(Debug, Deserialize)]
pub struct PackageSource {
	pub source_url: String,
	pub version: VersionReq,
	pub source_type: SourceType,
	pub inner_path: String
}

impl PackageSource {
	fn new(value: String) -> Self {
		let tag_start: usize = value.find("/tag/").expect("URL does not contain '/tag/'");
		let version_start: usize = tag_start + "/tag/".len();
		let version_end: usize = value[version_start..].find('/').expect("Could not find the end of the version segment") + version_start;
		
		let source_url: String = value[..tag_start + "/tag".len()].to_string();
		let version_string: String = value[version_start..version_end].to_string().to_lowercase().replace("v", "");
		let inner_path: String = value[version_end + 1..].to_string();

		let version: VersionReq = VersionReq::parse(version_string.as_str()).expect("bad version req");
		
		let mut source_type: SourceType = SourceType::Unknown;
		if Regex::new(r"https://github\.com/.+?/.+?/releases").unwrap().is_match(&source_url){
			source_type = SourceType::GitHubRelease;
		}

		return PackageSource{
			source_url,
			version,
			source_type,
			inner_path
		};
	}
}

#[derive(Debug, Deserialize)]
struct RawMPMConfig {
	pub deprecated: bool,
	pub public: bool,
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
	pub package_source: PackageSource,
}

impl MPMDependency {
	fn new(name: String, value: String) -> Self {
		let package_source = PackageSource::new(value);
		return MPMDependency {
			name,
			package_source
		};	
	}
}

#[derive(Debug, Deserialize)]
pub struct MPMDirectory {
	pub name: String,
	pub path: String,
	pub is_deprecated: bool,
	pub is_public: bool,
	pub dependencies: Vec<MPMDependency>,
}

impl MPMDirectory {
	fn new(config_file_path: &Path) -> Self {
		let raw_config: RawMPMConfig = RawMPMConfig::new(config_file_path);
		let name: String = config_file_path.parent() // Option<&Path>
			.and_then(|p| p.file_name()) // Option<&OsStr>
			.and_then(|os_str| os_str.to_str()) // Option<&str>
			.expect("Failed to extract directory name from path") // Panics if None
			.to_owned();

		let is_deprecated: bool = raw_config.deprecated;
		let is_public: bool = raw_config.public;

		let mut dependencies: Vec<MPMDependency> = Vec::new();

		for (dep_name, dep_value) in raw_config.dependencies {
			let dependency = MPMDependency::new(dep_name.clone(), dep_value.clone());
			dependencies.push(dependency);
		}
		let path: String = config_file_path.to_str().expect("string conversion fail").to_owned().to_string();

		return MPMDirectory {
			name,
			path,
			is_deprecated,
			is_public,
			dependencies
		};
	}
}

/// Searches for files named `file_name` under the given `start_dir` directory and returns a Vec with the paths to the files found.
pub fn search_for_packages(start_dir: &Path) -> Vec<MPMDirectory> {
	let mut found_configs: Vec<MPMDirectory> = Vec::new();

	for entry in WalkDir::new(start_dir).follow_links(true).into_iter().filter_map(|e| e.ok())
	{
		let path: &Path = entry.path();
		if path
			.file_name()
			.and_then(|n: &std::ffi::OsStr| n.to_str())
			.map(|n| n == FILE_NAME_STRING)
			.unwrap_or(false)
		{
			found_configs.push(MPMDirectory::new(path));
		}
	}

	return found_configs;
}
