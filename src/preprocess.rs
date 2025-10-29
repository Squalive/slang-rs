mod comment_strip_iter;

use alloc::string::String;
use alloc::vec::Vec;
use comment_strip_iter::CommentReplaceExt;

pub fn preprocess(content: &str) -> (Option<String>, Vec<String>) {
	let mut module_name = None;
	let mut imports = Vec::new();

	for line in content.lines().replace_comments() {
		let line = line.as_ref().trim();
		if line.is_empty() {
			continue;
		}

		if module_name.is_none() {
			if let Some(name) = parse_module(line) {
				module_name = Some(name);
				continue;
			}
		}

		if let Some(path) = parse_import(line) {
			imports.push(path);
		}
	}

	(module_name, imports)
}

pub enum FileType {
	Code,
	Module(String),
	IncludeDependencies,
}

pub fn get_file_type(content: &str) -> FileType {
	for line in content.lines().replace_comments() {
		let line = line.as_ref().trim();
		if line.is_empty() {
			continue;
		}

		if let Some(name) = parse_module(line) {
			return FileType::Module(name);
		}

		if line.starts_with("implementing") {
			return FileType::IncludeDependencies;
		}
	}

	FileType::Code
}

fn parse_module(line: &str) -> Option<String> {
	// Check if line starts with "module" followed by whitespace
	if !line.starts_with("module") {
		return None;
	}

	let after_module = &line[6..]; // Skip "module"
	let after_module = after_module.trim_start();

	if after_module.is_empty() {
		return None;
	}

	// Parse identifier until we hit whitespace or semicolon
	let mut identifier = String::new();
	for ch in after_module.chars() {
		if ch.is_whitespace() || ch == ';' {
			break;
		}
		if identifier.is_empty() {
			// First character must be alphabetic or underscore
			if ch.is_ascii_alphabetic() || ch == '_' {
				identifier.push(ch);
			} else {
				return None;
			}
		} else {
			// Subsequent characters can be alphanumeric or underscore
			if ch.is_ascii_alphanumeric() || ch == '_' {
				identifier.push(ch);
			} else {
				break;
			}
		}
	}

	if identifier.is_empty() {
		None
	} else {
		Some(identifier)
	}
}

fn parse_import(line: &str) -> Option<String> {
	// Check if line starts with "import" followed by whitespace
	if !line.starts_with("import") {
		return None;
	}

	let after_import = &line[6..]; // Skip "import"
	let after_import = after_import.trim_start();

	if after_import.is_empty() {
		return None;
	}

	let next_char = after_import.chars().next()?;

	if next_char == '"' {
		// Quoted path: import "path/to/file"
		let mut path = String::new();
		let mut chars = after_import[1..].chars(); // Skip the opening quote

		while let Some(ch) = chars.next() {
			if ch == '/' {
				return None;
			}
			if ch == '"' {
				// Found closing quote
				break;
			}
			path.push(ch);
		}

		Some(path)
	} else {
		// Dot-separated identifier: import some.module.name
		let mut identifier = String::new();
		let mut chars = after_import.chars();

		// Parse the entire identifier (until semicolon or end)
		while let Some(ch) = chars.next() {
			if ch == '.' {
				return None;
			}
			if ch == ';' || ch.is_whitespace() {
				break;
			}
			identifier.push(ch);
		}

		// Convert dot-separated identifier to path
		Some(identifier)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloc::string::ToString;
	use alloc::vec;

	#[test]
	fn test() {
		let content = r#"
module test;

import common;
import "math";
import math;
		"#;
		let (module_name, imports) = preprocess(content);
		assert_eq!(module_name, Some("test".into()));
		assert_eq!(
			imports,
			vec![
				"common".to_string(),
				"math".to_string(),
				"math".to_string(),
			],
		);
	}

	#[test]
	fn module_test() {
		let content = "module test;";

		let (module_name, imports) = preprocess(content);
		assert_eq!(module_name, Some("test".into()));
		assert_eq!(imports.len(), 0);

		let (module_name, imports) = preprocess("import test;");
		assert_eq!(module_name, None);
		assert_eq!(
			imports,
			vec![
				"test".to_string()
			]
		);
	}
}