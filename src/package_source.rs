use std::io::copy;
use std::str::FromStr;
use zip::ZipArchive;
use semver::{VersionReq, Version};
use tempfile::tempdir;
use std::fs::File;
use std::io;
use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use super::csharp_parse::compile_to_single_script;
use anyhow::{Result, anyhow};

fn unzip_file_to_directory(zip_path: &Path, output_path: &Path){
	// Open the .zip file
	// println!("zip_path={}", zip_path.to_str().unwrap());
	let zip_file = File::open(zip_path).unwrap();
	let mut archive: ZipArchive<File> = ZipArchive::new(zip_file).unwrap();

	// Iterate through each entry in the .zip archive
	for i in 0..archive.len() {
		let mut file = archive.by_index(i).unwrap();
		let file_path = output_path.join(file.mangled_name());

		// If the file is a directory, create it
		if file.name().ends_with('/') {
			std::fs::create_dir_all(&file_path).unwrap();
		} else {
			// Ensure the file's parent directory exists
			if let Some(parent) = file_path.parent() {
				std::fs::create_dir_all(parent).unwrap();
			}

			// Write the file contents
			let mut outfile = File::create(&file_path).unwrap();
			let _ = io::copy(&mut file, &mut outfile);
		}
	}
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

fn get_psc_from_cache(
	source_url: PathBuf,
	version_req: VersionReq,
	source_cache: &HashMap<PathBuf, HashMap<Version, PackageSourceContent>>
) -> Option<PackageSourceContent>{
	let mut package_source_content_opt: Option<PackageSourceContent> = None;
	
	if source_cache.contains_key(&source_url){
		for (version, psc) in source_cache.get(&source_url).unwrap() {
			if version_req.matches(&version){
				package_source_content_opt = Some(psc.clone());
			}
		}
	}

	return package_source_content_opt;
}

fn save_psc_into_cache(original_package_source_content: PackageSourceContent, original_source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>>) -> HashMap<PathBuf, HashMap<Version, PackageSourceContent>>{
	let mut source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>> = original_source_cache.clone();
	let package_source_content: PackageSourceContent = original_package_source_content.clone();
	
	if source_cache.contains_key(&package_source_content.source_url) == false{
		source_cache.insert(package_source_content.source_url, HashMap::new());
	}

	let path: PathBuf = original_package_source_content.source_url.clone();

	let mut version_cache: HashMap<Version, PackageSourceContent> = source_cache.get(&path).unwrap().to_owned();
	version_cache.insert(package_source_content.version, original_package_source_content);
	source_cache.insert(path, version_cache);
	return source_cache
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum SourceType {
	Unknown,
	GitHubRelease,
}

#[derive(Debug, Clone)]
pub struct PackageSourceContent{
	pub data: bytes::Bytes,
	pub version: Version,
	pub source_url: PathBuf,
}

impl PackageSourceContent {
	pub async fn new(
		source_url: PathBuf,
		version_req: VersionReq,
		source_type: SourceType,
	) -> Self{
		let source_url_str: &str = source_url.to_str().unwrap();
		println!("downloading {}", &source_url_str);
		assert!(source_type == SourceType::GitHubRelease, "not a supported source");

		// let info_start: usize = self.source_url.find("").expect("URL does not contain 'https://github.com/'");
		let info_string: &str = &source_url_str["https://github.com/".len()..];
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
		let mut release_version_option: Option<Version> = None;

		for release in page.items{
			match Version::parse(&release.tag_name.replace("v", "")) {
				Ok(release_version) => {
					if version_req.matches(&release_version){
						release_tag_option = Some(release.tag_name.clone()); // Set release_tag here
						release_version_option = Some(release_version);
						break;
					}
				}
				Err(e) => {
					eprintln!("Failed to parse release with tag '{}': {}", release.tag_name, e);
				}
			}
		}
		

		let release_tag: String = release_tag_option.expect("no compatible release found");

		let release: octocrab::models::repos::Release = repos.releases()
			.get_by_tag(&release_tag)
			.await.unwrap();

		let zip_url: reqwest::Url = release.zipball_url.expect("bad zip url");
		// println!("zip_url={:#?}", zip_url.to_string());

		// Download the asset
		let client = reqwest::Client::new();
		let response: reqwest::Response = match client.get(zip_url.to_string())
			.header("User-Agent", "request").send().await {
				Ok(res) => res,
				Err(e) => panic!("{}", e),
			};

		let data: bytes::Bytes = response.bytes().await.unwrap();

		let version: Version = release_version_option.unwrap();
		return PackageSourceContent{
			data,
			version,
			source_url
		};
	}

	pub fn compile(self: Self, target_namespace_name: String, inner_path: String) -> String{
		println!("compiling {}", target_namespace_name);
		// Create a temporary directory
		let dir: tempfile::TempDir = tempdir().unwrap();
		let dir_path: &Path = dir.path();
		let file_path: std::path::PathBuf = dir_path.join("source.zip");

		let mut file: File = File::create(file_path.clone()).unwrap();
		let mut content: io::Cursor<bytes::Bytes> =  std::io::Cursor::new(self.data);
		copy(&mut content, &mut file).unwrap();

		let unzip_dir_path: std::path::PathBuf = dir_path.join("unzipped_directory");
		unzip_file_to_directory(file_path.as_path(), unzip_dir_path.as_path());

		let inner_dir_path: PathBuf = find_single_subdirectory(&unzip_dir_path).unwrap();
		let target_package_path: PathBuf = inner_dir_path.join(inner_path);

		// let source_namespace: String = target_package_path.file_stem().unwrap().to_str().unwrap().to_string();
		let tpdp_clone = target_package_path.clone();
		let target_path_debug_str: &str = tpdp_clone.to_str().unwrap();
		let mut scripts: HashMap<String, String> = HashMap::new();


		if target_package_path.is_file(){
			if let Some(name) = target_package_path.file_name().and_then(|n| n.to_str()) {
				let contents: String = fs::read_to_string(&target_package_path).unwrap();
				scripts.insert(name.to_owned(), contents);
			}
		}else if target_package_path.is_dir() {
			for entry in fs::read_dir(target_package_path).expect(&format!("can't read directory '{}'", target_path_debug_str)) {
				let entry = entry.unwrap();
				let path = entry.path();
				
				// Ensure the entry is a file
				if path.is_file() {
					// Get the file name as a String
					if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
						// Read the file's contents into a String
						let contents = fs::read_to_string(&path).unwrap();
						// Insert the file name and contents into the map
						scripts.insert(name.to_owned(), contents);
					}
				}
			}
		}

		
		return compile_to_single_script(
			format!("DO NOT EDIT!\n// downloaded from '{}' and compiled into single script using 'github.com/nightcycle/muse-package-manager'", self.source_url.to_str().unwrap()),
			target_namespace_name, 
			scripts
		);
	}
}

#[derive(Debug, Deserialize)]
pub struct PackageSource {
	pub source_url: PathBuf,
	pub version_req: VersionReq,
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

		let source_url: PathBuf = PathBuf::from_str(&value[..releases_start].to_string()).unwrap();

		let version_req: VersionReq = VersionReq::parse(version_string.as_str()).expect("bad version req");
		
		let source_type: SourceType = SourceType::GitHubRelease;

		return PackageSource{
			source_url,
			version_req,
			source_type,
			inner_path
		};
	}

	pub async fn solve(
		self: Self, 
		namespace_name: String,
		source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>>
	) -> (HashMap<PathBuf, HashMap<Version, PackageSourceContent>>, String){

		let content_option: Option<PackageSourceContent> = get_psc_from_cache(
			self.source_url.clone(),
			self.version_req.clone(),
			&source_cache
		);
		
		if content_option.is_some(){		
			let package_source_content: PackageSourceContent = content_option.unwrap();
			return (source_cache, package_source_content.compile(namespace_name, self.inner_path));
		}else{
			let package_source_content: PackageSourceContent = PackageSourceContent::new(
				self.source_url, 
				self.version_req, 
				self.source_type
			).await;

			
			let new_source_cache: HashMap<PathBuf, HashMap<Version, PackageSourceContent>> = save_psc_into_cache(package_source_content.clone(), source_cache);

			return (new_source_cache, package_source_content.compile(namespace_name, self.inner_path));
		}

	}
}