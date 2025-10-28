mod comment_strip_iter;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use comment_strip_iter::CommentReplaceExt;
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

struct Preprocessor {
	module: Regex,
	import: Regex,
}

impl Preprocessor {
	fn new() -> Self {
		Self {
			module: Regex::new(r"^\s*module\s+([a-zA-Z_]\w*)\s*;").unwrap(),
			import: Regex::new(r#"import\s+(?:"([^"]+)"|(\w[\w.]*))\s*;"#).unwrap(),
		}
	}

	fn preprocess(&self, content: &str) -> (Option<String>, Vec<PathBuf>) {
		let lines = content.lines();

		let mut module_name = None;
		let mut imports = Vec::new();

		for line in lines.replace_comments() {
			if module_name.is_none() {
				if let Some(caps) = self.module.captures(&line) {
					module_name = Some(caps[1].to_string());
				}
			}
			if let Some(cap) = self.import.captures(&line) {
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