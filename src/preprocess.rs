use alloc::string::{String, ToString};
use alloc::vec::Vec;
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

struct Preprocessor {
	comment_re: Regex,
	module: Regex,
	import: Regex,
}

impl Preprocessor {
	fn new() -> Self {
		Self {
			comment_re: Regex::new(r"//.*|/\*.*?\*/").unwrap(),
			module: Regex::new(r"^\s*module\s+([a-zA-Z_]\w*)\s*;").unwrap(),
			import: Regex::new(r#"import\s+(?:"([^"]+)"|(\w[\w.]*))\s*;"#).unwrap(),
		}
	}

	fn preprocess(&self, content: &str) -> (Option<String>, Vec<PathBuf>) {
		// Remove comments first
		let clean_content = self.comment_re.replace_all(content, "");

		let module_name = 'module: {
			for line in clean_content.lines() {
				if let Some(caps) = self.module.captures(line) {
					break 'module Some(caps[1].to_string());
				}
			}
			None
		};

		let mut imports = Vec::new();
		for cap in self.import.captures_iter(&clean_content) {
			if let Some(matched) = cap.get(1).or_else(|| cap.get(2)) {
				let raw_path = matched.as_str();
				let path = if cap.get(1).is_some() {
					PathBuf::from(raw_path)
				} else {
					let mut path = PathBuf::new();
					let parts = raw_path.split('.');
					for part in parts {
						path.push(part);
					}
					path
				};
				imports.push(path);
			}
		}

		(module_name, imports)
	}
}

static PREPROCESSOR: LazyLock<Preprocessor> = LazyLock::new(Preprocessor::new);

pub fn preprocess(content: &str) -> (Option<String>, Vec<PathBuf>) {
	PREPROCESSOR.preprocess(content)
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloc::vec;

	#[test]
	fn test() {
		let content = r#"
module test;

import common;
import "utils/math";
import utils.math;
		"#;
		let (module_name, imports) = preprocess(content);
		assert_eq!(module_name, Some("test".into()));
		assert_eq!(
			imports,
			vec![
				PathBuf::from("common"),
				"utils/math".into(),
				"utils/math".into(),
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
				PathBuf::from("test")
			]
		);
	}
}