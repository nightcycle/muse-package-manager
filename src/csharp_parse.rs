use pest_derive::Parser;
use pest::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "csharp.pest"]
struct CSharpParser;

pub fn compile_to_single_script(
	final_namespace: String, 
	dependency_scripts: HashMap<String, String>
) -> String{
	
	for (_dep_name, dep_value) in &dependency_scripts {

		let parse_result = CSharpParser::parse(Rule::using_directive, &dep_value)
		.expect("unsuccessful parse") // Handle parsing error
		.next().unwrap(); // Get the first parsed element, if exists

		println!("parse_result={:#?}", parse_result);
	}

	return String::from("test");
}