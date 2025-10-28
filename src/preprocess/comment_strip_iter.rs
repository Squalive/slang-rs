use aho_corasick::AhoCorasick;
use alloc::borrow::Cow;
use alloc::string::String;
use core::str::Lines;
use once_cell::sync::Lazy;

// All patterns we need to match
static PATTERNS: &[&str] = &["//", "/*", "*/", "\""];
static AC: Lazy<AhoCorasick> = Lazy::new(|| AhoCorasick::new(PATTERNS).expect("Failed to build AhoCorasick"));

#[derive(PartialEq, Eq, Clone, Copy)]
enum CommentState {
	None,
	Block(usize),
	Quote,
}

pub struct CommentReplaceIter<'a> {
	lines: Lines<'a>,
	state: CommentState,
	buffer: String,
}

impl<'a> CommentReplaceIter<'a> {
	fn new(lines: Lines<'a>) -> Self {
		Self {
			lines,
			state: CommentState::None,
			buffer: String::with_capacity(128),
		}
	}
}

impl<'a> Iterator for CommentReplaceIter<'a> {
	type Item = Cow<'a, str>;

	fn next(&mut self) -> Option<Self::Item> {
		let line = self.lines.next()?;

		// Enhanced fast path
		if self.state == CommentState::None && !has_special_chars(line) {
			return Some(Cow::Borrowed(line));
		}

		self.buffer.clear();
		let mut last_end = 0;
		let mut current_state = self.state;

		for mat in AC.find_iter(line) {
			let pattern = PATTERNS[mat.pattern().as_usize()];
			let start = mat.start();
			let end = mat.end();

			// Handle content before this match based on current state
			if last_end < start {
				match current_state {
					CommentState::Block(_) => {
						// In block comment - replace with spaces
						self.buffer.extend(std::iter::repeat(' ').take(start - last_end));
					}
					_ => {
						// Not in block - keep content as-is
						self.buffer.push_str(&line[last_end..start]);
					}
				}
			}

			// Handle the matched pattern based on current state
			match (current_state, pattern) {
				// None state transitions
				(CommentState::None, "//") => {
					// Line comment - replace rest of line with spaces and return
					self.buffer.extend(std::iter::repeat(' ').take(line.len() - start));
					self.state = current_state;
					return Some(Cow::Owned(self.buffer.clone()));
				}
				(CommentState::None, "/*") => {
					current_state = CommentState::Block(1);
					self.buffer.push_str("  "); // Replace "/*" with spaces
				}
				(CommentState::None, "\"") => {
					current_state = CommentState::Quote;
					self.buffer.push('"');
				}
				(CommentState::None, "*/") => {
					// "*/" in None state - keep as-is (not in a block comment)
					self.buffer.push_str("*/");
				}

				// Block state transitions
				(CommentState::Block(depth), "/*") => {
					current_state = CommentState::Block(depth + 1);
					self.buffer.push_str("  ");
				}
				(CommentState::Block(depth), "*/") => {
					if depth == 1 {
						current_state = CommentState::None;
					} else {
						current_state = CommentState::Block(depth - 1);
					}
					self.buffer.push_str("  ");
				}
				(CommentState::Block(_), "//") => {
					// "//" in block comment - replace with spaces
					self.buffer.push_str("  ");
				}
				(CommentState::Block(_), "\"") => {
					// Quote in block comment - replace with space
					self.buffer.push(' ');
				}

				// Quote state transitions
				(CommentState::Quote, "\"") => {
					current_state = CommentState::None;
					self.buffer.push('"');
				}
				(CommentState::Quote, "//") |
				(CommentState::Quote, "/*") |
				(CommentState::Quote, "*/") => {
					// Any comment pattern in quote state - keep as-is
					self.buffer.push_str(pattern);
				}

				// Catch-all for any unexpected combinations
				_ => {
					// This should never happen with our pattern set, but keep the pattern as-is
					self.buffer.push_str(pattern);
				}
			}

			last_end = end;
		}

		// Handle remaining content after last match
		if last_end < line.len() {
			match current_state {
				CommentState::Block(_) => {
					self.buffer.extend(std::iter::repeat(' ').take(line.len() - last_end));
				}
				_ => {
					self.buffer.push_str(&line[last_end..]);
				}
			}
		}

		self.state = current_state;

		// Return borrowed if no changes were made
		if self.buffer == line {
			Some(Cow::Borrowed(line))
		} else {
			Some(Cow::Owned(self.buffer.clone()))
		}
	}
}

/// Fast check for presence of any special characters that might trigger processing
#[inline]
fn has_special_chars(s: &str) -> bool {
	s.chars().any(|c| matches!(c, '/' | '"' | '*'))
}

pub trait CommentReplaceExt<'a> {
	fn replace_comments(self) -> CommentReplaceIter<'a>;
}

impl<'a> CommentReplaceExt<'a> for Lines<'a> {
	fn replace_comments(self) -> CommentReplaceIter<'a> {
		CommentReplaceIter::new(self)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloc::vec::Vec;

	#[test]
	fn comment_test() {
		const INPUT: &str = r"
not commented
// line commented
not commented
/* block commented on a line */
not commented
// line comment with a /* block comment unterminated
not commented
/* block comment
   spanning lines */
not commented
/* block comment
   spanning lines and with // line comments
   even with a // line commented terminator */
not commented
";

		let processed: Vec<_> = INPUT.lines().replace_comments().collect();
		let original: Vec<_> = INPUT.lines().collect();

		for (i, (processed_line, original_line)) in processed.iter().zip(original.iter()).enumerate() {
			assert_eq!(
				processed_line.len(),
				original_line.len(),
				"Line {} length mismatch: processed '{}' vs original '{}'",
				i, processed_line, original_line
			);

			if *original_line != "not commented" {
				assert!(
					processed_line.chars().all(|c| c == ' ') || *processed_line == *original_line,
					"Line {} should be all spaces or unchanged: '{}'",
					i, processed_line
				);
			}
		}
	}

	#[test]
	fn partial_tests() {
		const PARTIAL_TESTS: [(&str, &str); 11] = [
			(
				"1.0 /* block comment with a partial line comment on the end *// 2.0",
				"1.0                                                           / 2.0",
			),
			(
				"1.0 /* block comment with a partial block comment on the end */* 2.0",
				"1.0                                                            * 2.0",
			),
			(
				"1.0 /* block comment 1 *//* block comment 2 */ * 2.0",
				"1.0                                            * 2.0",
			),
			(
				"1.0 /* block comment with real line comment after */// line comment",
				"1.0                                                                ",
			),
			("*/", "*/"),
			(
				r#"#import "embedded://file.wgsl""#,
				r#"#import "embedded://file.wgsl""#,
			),
			(
				r#"// #import "embedded://file.wgsl""#,
				r#"                                 "#,
			),
			(
				r#"/* #import "embedded://file.wgsl" */"#,
				r#"                                    "#,
			),
			(
				r#"/* #import "embedded:*/file.wgsl" */"#,
				r#"                       file.wgsl" */"#,
			),
			(
				r#"#import "embedded://file.wgsl" // comment"#,
				r#"#import "embedded://file.wgsl"           "#,
			),
			(
				r#"#import "embedded:/* */ /* /**/* / / /// * / //*/*/ / */*file.wgsl""#,
				r#"#import "embedded:/* */ /* /**/* / / /// * / //*/*/ / */*file.wgsl""#,
			),
		];

		for (input, expected) in PARTIAL_TESTS.iter() {
			let result = input.lines().replace_comments().next().unwrap();
			assert_eq!(&result, expected, "Failed for input: {}", input);
		}
	}

	#[test]
	fn test_comment_becomes_spaces() {
		let input = "let a/**/b =3u;";
		let expected = "let a    b =3u;";
		let result = input.lines().replace_comments().next().unwrap();
		assert_eq!(&result, expected);
	}

	#[test]
	fn test_nested_block_comments() {
		let input = "test /* outer /* inner */ outer */ test";
		let expected = "test                               test";
		let result = input.lines().replace_comments().next().unwrap();
		assert_eq!(&result, expected);
	}

	#[test]
	fn test_quotes_with_comments() {
		let input = r#"text "string // with comment" text"#;
		let expected = r#"text "string // with comment" text"#;
		let result = input.lines().replace_comments().next().unwrap();
		assert_eq!((&result).trim(), expected.trim());
	}

	#[test]
	fn test_quotes_in_comments() {
		let input = r#"/* "comment inside string" */"#;
		let expected = "                              ";
		let result = input.lines().replace_comments().next().unwrap();
		assert_eq!((&result).trim(), expected.trim());
	}

	#[test]
	fn test_fast_path() {
		let simple_lines = [
			"just some code",
			"let x = 5;",
			"fn main() {}",
		];

		for line in simple_lines {
			let result = line.lines().replace_comments().next().unwrap();
			assert!(matches!(result, Cow::Borrowed(_)), "Fast path failed for: {}", line);
			assert_eq!(&result, line);
		}
	}

	#[test]
	fn test_unterminated_block_comment() {
		let input = "start /* unclosed comment";
		let expected = "start                       ";
		let result = input.lines().replace_comments().next().unwrap();
		assert_eq!((&result).trim(), expected.trim());
	}

	#[test]
	fn test_multiple_lines_persist_state() {
		let input = "line1 /* comment\nline2 still in comment\nline3 */ out";
		let mut iter = input.lines().replace_comments();

		assert_eq!(&iter.next().unwrap(), "line1           ");
		assert_eq!(&iter.next().unwrap(), "                      "); // This should be all spaces
		assert_eq!(&iter.next().unwrap(), "         out"); // This should have the comment removed
	}
}