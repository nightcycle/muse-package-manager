use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use serde::Deserialize;
use semver::{VersionReq, Version};
use std::collections::HashMap;
use tempfile::tempdir;
use std::fs::File;
use std::io::copy;
use zip::ZipArchive;
use anyhow::{Result, anyhow};

const FILE_NAME_STRING: &str = "muse-package.toml";

fn unzip_file_to_directory(zip_path: &Path, output_path: &Path) -> Result<()> {
	// Open the .zip file
	let zip_file = File::open(zip_path)?;
	let mut archive = ZipArchive::new(zip_file)?;

	// Iterate through each entry in the .zip archive
	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		let file_path = output_path.join(file.sanitized_name());

		// If the file is a directory, create it
		if file.name().ends_with('/') {
			std::fs::create_dir_all(&file_path)?;
		} else {
			// Ensure the file's parent directory exists
			if let Some(parent) = file_path.parent() {
				std::fs::create_dir_all(parent)?;
			}

			// Write the file contents
			let mut outfile = File::create(&file_path)?;
			io::copy(&mut file, &mut outfile)?;
		}
	}

	Ok(())
}

fn find_single_subdirectory(path: &Path) -> Result<PathBuf> {
	let mut directories: Vec<PathBuf> = Vec::new();

	// Iterate over the entries in the directory.
	let entries = fs::read_dir(path)?
		.filter_map(Result::ok) // Filter out Err results and unwrap Ok values.
		.filter(|entry| entry.path().is_dir()); // Consider only directories.

	// Collect directories
	for entry in entries {
		directories.push(entry.path());
		if directories.len() > 1 {
			// If more than one directory is found, return an error.
			return Err(anyhow!("More than one subdirectory found in {}", path.display()));
		}
	}

	// Check how many directories were found and act accordingly.
	match directories.len() {
		1 => Ok(directories[0].clone()), // Return the first (and only) directory.
		0 => Err(anyhow!("No subdirectories found in {}", path.display())),
		_ => unreachable!(), // This case is already handled above, so it's unreachable.
	}
 }

#[derive(Debug, Deserialize, PartialEq)]
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
	pub fn new(value: String) -> Self {
		let tag_start: usize = value.find("/tag/").expect("URL does not contain '/tag/'");
		let version_start: usize = tag_start + "/tag/".len();
		let version_end: usize = value[version_start..].find('/').expect("Could not find the end of the version segment") + version_start;
		
		let version_string: String = value[version_start..version_end].to_string().to_lowercase().replace("v", "");
		let inner_path: String = value[version_end + 1..].to_string();

		let releases_start: usize = value.find("/releases/").expect("URL does not contain '/releases/'");

		let source_url: String = value[..releases_start].to_string();

		let version: VersionReq = VersionReq::parse(version_string.as_str()).expect("bad version req");
		
		let source_type: SourceType = SourceType::GitHubRelease;

		return PackageSource{
			source_url,
			version,
			source_type,
			inner_path
		};
	}
	pub async fn download(self: Self){
		assert!(self.source_type == SourceType::GitHubRelease, "not a supported source");

		// let info_start: usize = self.source_url.find("").expect("URL does not contain 'https://github.com/'");
		let info_string: &str = &self.source_url["https://github.com/".len()..];
		let mid_slash_start: usize = info_string.find("/").expect("bad github url");

		let owner: &str = &info_string[..mid_slash_start];
		let repo: &str = &info_string[(mid_slash_start+1)..];

		let client: std::sync::Arc<octocrab::Octocrab> = octocrab::instance();

		let repos = client.repos(owner, repo);
		let page: octocrab::Page<octocrab::models::repos::Release> = repos
			.releases()
			.list()
			// Optional Parameters
			.per_page(100)
			// .page(5u32)
			// Send the request
			.send()
			.await.unwrap();

		let mut release_tag_option: Option<String> = None;

		for release in page.items{
			match Version::parse(&release.tag_name.replace("v", "")) {
				Ok(release_version) => {
					if self.version.matches(&release_version){
						release_tag_option = Some(release.tag_name.clone()); // Set release_tag here
						break;
					}
				}
				Err(e) => {
					eprintln!("Failed to parse release with tag '{}': {}", release.tag_name, e);
				}
			}
		}
		

		let release_tag: String = release_tag_option.expect("no compatible release found");
		println!("best={:#?}", release_tag);

		let release: octocrab::models::repos::Release = repos.releases()
			.get_by_tag(&release_tag)
			.await.unwrap();


		let zip_url = release.zipball_url.expect("bad zip url");
		println!("zip_url={:#?}", zip_url.to_string());


		// Create a temporary directory
		let dir: tempfile::TempDir = tempdir().unwrap();
		let file_path: std::path::PathBuf = dir.path().join("repo.zip");
		let unzip_dir_path: std::path::PathBuf = dir.path().join("unzipped_directory");
		// Download the asset
		let response: reqwest::Response = reqwest::get(zip_url.to_string()).await.unwrap();
			
		let mut file: File = File::create(file_path).unwrap();
		let mut content =  std::io::Cursor::new(response.bytes().await.unwrap());
		copy(&mut content, &mut file).unwrap();
	
		unzip_file_to_directory(file_path.as_path(), unzip_dir_path.as_path())?;

		let inner_dir_path = find_single_subdirectory(&unzip_dir_path).unwrap();
		let inner_package_path = inner_dir_path.join(self.inner_path);

		println!("to inner path: {}", inner_package_path);
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
	pub source: PackageSource,
}

impl MPMDependency {
	fn new(name: String, value: String) -> Self {
		let source: PackageSource = PackageSource::new(value);
		return MPMDependency {
			name,
			source
		};	
	}
}

#[derive(Debug, Deserialize)]
pub struct MPMPackage {
	pub name: String,
	pub path: String,
	pub is_deprecated: bool,
	pub is_public: bool,
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

		let is_deprecated: bool = raw_config.deprecated;
		let is_public: bool = raw_config.public;

		let mut dependencies: Vec<MPMDependency> = Vec::new();

		for (dep_name, dep_value) in raw_config.dependencies {
			let dependency: MPMDependency = MPMDependency::new(dep_name.clone(), dep_value.clone());
			dependencies.push(dependency);
		}
		let path: String = config_file_path.to_str().expect("string conversion fail").to_owned().to_string();

		return MPMPackage {
			name,
			path,
			is_deprecated,
			is_public,
			dependencies
		};
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
