use std::collections::HashMap;
use regex::Regex;
use std::collections::HashSet;


fn get_matching_lines(input: &str, pattern: &Regex) -> Vec<String> {
	input
	    .lines()
	    .filter_map(|line| {
		   if pattern.is_match(line) {
			  Some(line.to_string())
		   } else {
			  None
		   }
	    })
	    .collect()
}

// fn generate_translations(input: &str, pattern: &Regex) -> HashMap<String, String> {
// 	let mut results = HashMap::new();
// 	let letters: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"; // Include only letters
// 	let letter_dist = Uniform::new_inclusive(0, letters.len() - 1);

// 	for line in input.lines() {
// 	    if pattern.is_match(line) {
// 		   let key = line.to_string(); //line.replace(" ", ""); // Remove unimportant whitespace
// 		   let value: String = (0..32)
// 		   .map(|_| {
// 			  let idx = thread_rng().sample(letter_dist); // Sample a position
// 			  letters.chars().nth(idx).unwrap() // Get the character at the sampled position
// 		   })
// 		   .collect();
// 		   results.insert(key, value);
// 	    }
// 	}
// 	results
//  }

 fn deduplicate(vec: Vec<String>) -> Vec<String> {
	let set: HashSet<_> = vec.into_iter().collect(); // Deduplicate
	let unique_vec: Vec<String> = set.into_iter().collect(); // Convert back to Vec
	unique_vec
 }

 fn extract_indented_lines(input: &str) -> String {
	input
	    .lines()
	    .filter(|line| line.starts_with("\t"))
	    .collect::<Vec<&str>>()
	    .join("\n")
 }
 

 fn remove_matching_lines(input: &str, pattern: &Regex) -> String {
	input
	    .lines()
	    .filter(|line| !pattern.is_match(line)) // Keep lines that do NOT match the pattern
	    .collect::<Vec<&str>>()
	    .join("\n")
 }

pub fn compile_to_single_script(
	header_comment: String,
	target_namespace: String, 
	dependency_scripts: HashMap<String, String>
) -> String{
	
	let namespace_pattern: Regex = Regex::new(r"namespace\s+[A-Za-z0-9]").unwrap();
	let using_pattern: Regex = Regex::new(r"using [A-Za-z]+").unwrap();

	let mut mega_script: String = String::new();

	let mut prior_namespaces: Vec<String> = Vec::new();
	for (_dep_name, dep_value) in &dependency_scripts {
		mega_script.push_str(&format!("\n{}", dep_value));
		prior_namespaces.append(& mut get_matching_lines(dep_value, &namespace_pattern));
	}
	let cleaned_namespaces: Vec<String> = prior_namespaces
		.iter()
		.map(|s| s.replace("namespace ", ""))
		.collect();
	// println!("prior_namespaces={:#?}", cleaned_namespaces);

	let mut usage_list: Vec<String> = get_matching_lines(&mega_script, &using_pattern);
	usage_list = deduplicate(usage_list);
	mega_script = remove_matching_lines(&mega_script, &using_pattern);

	mega_script = extract_indented_lines(mega_script.as_str());

	let mut header_string: String = String::new();

	for usage_line in &usage_list {
		let mut is_safe: bool = true;
		for namespace in &cleaned_namespaces {
			if usage_line.contains(&format!("using {}", namespace)){
				is_safe = false;
			}
		}
		if is_safe {
			header_string.push_str(&format!("\n{}", &usage_line));
		}
	}
	mega_script = header_string + &format!("\nnamespace {}", &target_namespace.as_str()) + "\n{\n" + &mega_script + "\n}";
	// println!("usage_line={:#?}", cleaned_namespaces);
	

	// let namespace_pattern: Regex = Regex::new(r"namespace\s+[A-Za-z0-9]").unwrap();
	// let mut namespace_dict: HashMap<String, String> = generate_translations(&mega_script, &namespace_pattern);
	// namespace_dict.insert(format!("namespace {}", source_namespace), target_namespace);
	// println!("namespace_dict={:#?}", namespace_dict);

	// for (key, value) in namespace_dict {
	// 	mega_script = mega_script.replace(&key, &format!("namespace {}",value));
	// 	mega_script = mega_script.replace(&key.replace("namespace ", "using "), &format!("using {}",value));
	// }
	
	return format!("// {}\n{}", header_comment, mega_script);
}