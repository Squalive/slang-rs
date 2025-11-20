mod comment_strip_iter;

use alloc::string::String;
use alloc::vec::Vec;
use comment_strip_iter::CommentReplaceExt;

pub struct PreprocessedShader {
    pub module_name: Option<String>,
    pub imports: Vec<String>,
    pub includes: Vec<String>,
}

pub fn preprocess(content: &str) -> PreprocessedShader {
    let mut module_name = None;
    let mut imports = Vec::new();
    let mut includes = Vec::new();

    for line in content.lines().replace_comments() {
        let line = line.as_ref().trim();
        if line.is_empty() {
            continue;
        }

        if module_name.is_none()
            && let Some(name) = parse_module(line)
        {
            module_name = Some(name);
            continue;
        }

        if let Some(path) = parse_import(line) {
            imports.push(path);
        }

        if let Some(path) = parse_include(line) {
            includes.push(path);
        }
    }

    PreprocessedShader {
        module_name,
        imports,
        includes,
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
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
        let chars = after_import[1..].chars(); // Skip the opening quote

        for ch in chars {
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
        let chars = after_import.chars();

        // Parse the entire identifier (until semicolon or end)
        for ch in chars {
            if ch == ';' || ch.is_whitespace() {
                break;
            }
            identifier.push(ch);
        }

        // Convert dot-separated identifier to path
        Some(identifier)
    }
}

fn parse_include(line: &str) -> Option<String> {
    const INCLUDE: &str = "__include";

    if !line.starts_with(INCLUDE) {
        return None;
    }

    let after_import = &line[INCLUDE.len() + 1..];
    let after_import = after_import.trim_start();

    if after_import.is_empty() {
        return None;
    }

    let next_char = after_import.chars().next()?;

    if next_char == '"' {
        // Quoted path: __include "path/to/file";
        let mut path = String::new();
        let chars = after_import[1..].chars();

        for ch in chars {
            if ch == '"' {
                // Found closing quote
                break;
            }
            path.push(ch);
        }

        Some(path)
    } else {
        None
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
        let shader = preprocess(content);
        assert_eq!(shader.module_name, Some("test".into()));
        assert_eq!(
            shader.imports,
            vec!["common".to_string(), "math".to_string(), "math".to_string(),],
        );
    }

    #[test]
    fn module_test() {
        let content = "module test;";

        let shader = preprocess(content);
        assert_eq!(shader.module_name, Some("test".into()));
        assert_eq!(shader.imports.len(), 0);

        let shader = preprocess("import test;");
        assert_eq!(shader.module_name, None);
        assert_eq!(shader.imports, vec!["test".to_string()]);
    }

    #[test]
    fn comment_test() {
        let content = r#"
module test2;
// module test2;

import common;
import "math";
//import math;
		"#;
        let shader = preprocess(content);
        assert_eq!(shader.module_name, Some("test2".into()));
        assert_eq!(
            shader.imports,
            vec!["common".to_string(), "math".to_string()],
        );
    }

    #[test]
    fn file_type_test() {
        let content = r#" implementing test;"#;

        let ty = get_file_type(content);
        assert_eq!(ty, FileType::IncludeDependencies);
    }

    #[test]
    fn include_test() {
        let content = r#"
__include "prelude/mesh";
            "#;

        let shader = preprocess(content);
        assert_eq!(shader.includes, vec!["prelude/mesh".to_string()]);
    }
}
