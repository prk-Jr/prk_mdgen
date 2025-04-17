use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MdPatternType {
    CodeTag,    // <code path="..."> ... </code>
    HashMarker, // ### filename followed by code fence
    Delimiter,  // ====\nfilename\n==== followed by code fence
    Raw,        // // file: filename followed by code fence
    FileCode,   // <file> filename </file> / <code> ... </code>
    FileFence,  // <file>…</file> heading + fenced block
}

/// Parses the given markdown content and returns a vector of ParsedFile.
///
/// If `forced` is provided, only that pattern is used; otherwise the parser
/// automatically selects the pattern with the most extracted file blocks.
pub fn parse_content(content: &str, forced: Option<MdPatternType>) -> Vec<ParsedFile> {
    // Trim the content to remove any leading/trailing whitespace.
    let content = content.trim();

    // Run each sub-parser.
    let group1 = parse_code_tag(content);
    let group2 = parse_hash_marker(content);
    let group3 = parse_delimiter_marker(content);
    let group4 = parse_raw_code_block(content);
    let group5 = parse_file_code(content);
    let group6 = parse_file_fence(content);

    // If a pattern type is forced, return that group (or an empty vector if none).
    if let Some(forced_type) = forced {
        return match forced_type {
            MdPatternType::CodeTag => group1,
            MdPatternType::HashMarker => group2,
            MdPatternType::Delimiter => group3,
            MdPatternType::Raw => group4,
            MdPatternType::FileCode => group5,
            MdPatternType::FileFence => group6,
        };
    }

    // Merge the results from all sub-parsers.
    let mut all = Vec::new();
    all.extend(group1);
    all.extend(group2);
    all.extend(group3);
    all.extend(group4);
    all.extend(group5);
    all.extend(group6);

    // Optional: deduplicate by file path if the same file is defined in multiple patterns.
    all.sort_by(|a, b| a.path.cmp(&b.path));
    all.dedup_by(|a, b| a.path == b.path);

    all
}

/// Sub-parser 1: XML-like code block pattern.
/// Example:
///     <code path="Cargo.toml">
///     [package]
///     name = "example"
///     </code>
fn parse_code_tag(content: &str) -> Vec<ParsedFile> {
    lazy_static! {
        static ref CODE_TAG_REGEX: Regex = Regex::new(
            r#"(?is)<code\s+path\s*=\s*"([^"\r\n]+?\.(?:rs|toml|json))">\s*(.*?)\s*</code>"#
        )
        .unwrap();
    }
    let mut results = Vec::new();
    for cap in CODE_TAG_REGEX.captures_iter(content) {
        let path = cap[1].trim().to_string();
        let mut code = cap[2].trim().to_string();

        // If the captured code starts with a code fence, remove it.
        if code.starts_with("```") {
            // Remove the first line (the opening fence with optional language).
            if let Some(pos) = code.find('\n') {
                code = code[pos..].trim_start().to_string();
            }
            // If the code ends with a closing fence, remove it.
            if code.ends_with("```") {
                if let Some(pos) = code.rfind("```") {
                    code = code[..pos].trim_end().to_string();
                }
            }
        }

        results.push(ParsedFile {
            path,
            content: code,
        });
    }
    results
}

/// Sub-parser 2: Hash marker pattern.
/// Example:
///     ### src/main.rs
///     ```rust
///     fn main() { ... }
///     ```
fn parse_hash_marker(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;
    lazy_static! {
        static ref HASH_HEADER_REGEX: Regex =
            Regex::new(r"^\s*#{1,6}\s+([^\s]+\.(?:rs|toml|json))\s*$").unwrap();
        static ref CODE_FENCE_REGEX: Regex = Regex::new(r"^\s*```(?:[a-zA-Z0-9]*)\s*$").unwrap();
    }
    while idx < lines.len() {
        let line = lines[idx];
        if let Some(cap) = HASH_HEADER_REGEX.captures(line) {
            let file_path = cap[1].trim().to_string();
            idx += 1;
            while idx < lines.len() && lines[idx].trim().is_empty() {
                idx += 1;
            }
            if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                idx += 1; // skip opening fence
                let (code, new_idx) = extract_code_block(&lines, idx);
                idx = new_idx;
                results.push(ParsedFile {
                    path: file_path,
                    content: code.trim().to_string(),
                });
                continue;
            }
        }
        idx += 1;
    }
    results
}

/// Sub-parser 3: Delimiter marker pattern.
/// Example:
///     ========
///     src/lib.rs
///     ========
///     ```rust
///     pub fn lib_function() {}
///     ```
fn parse_delimiter_marker(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;
    lazy_static! {
        static ref CODE_FENCE_REGEX: Regex = Regex::new(r"^\s*```(?:[a-zA-Z0-9]*)\s*$").unwrap();
    }
    while idx < lines.len() {
        let line = lines[idx];
        if line.trim().chars().all(|c| c == '=') && !line.trim().is_empty() {
            if idx + 2 < lines.len() {
                let candidate = lines[idx + 1].trim();
                if candidate.ends_with(".rs")
                    || candidate.ends_with(".toml")
                    || candidate.ends_with(".json")
                {
                    let delim_line = lines[idx + 2].trim();
                    if delim_line.chars().all(|c| c == '=') && !delim_line.is_empty() {
                        let file_path = candidate.to_string();
                        idx += 3; // skip header lines
                        while idx < lines.len() && lines[idx].trim().is_empty() {
                            idx += 1;
                        }
                        if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                            idx += 1; // skip opening fence
                            let (code, new_idx) = extract_code_block(&lines, idx);
                            idx = new_idx;
                            results.push(ParsedFile {
                                path: file_path,
                                content: code.trim().to_string(),
                            });
                            continue;
                        }
                    }
                }
            }
        }
        idx += 1;
    }
    results
}

/// Sub-parser 4: Raw code block pattern.
/// Example:
///     // file: src/utils.rs
///     ```rust
///     pub fn util() {}
///     ```
fn parse_raw_code_block(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;
    lazy_static! {
        static ref RAW_HEADER_REGEX: Regex =
            Regex::new(r"^\s*//\s*file:\s*([^\s]+\.(?:rs|toml|json))\s*$").unwrap();
        static ref CODE_FENCE_REGEX: Regex = Regex::new(r"^\s*```(?:[a-zA-Z0-9]*)\s*$").unwrap();
    }
    while idx < lines.len() {
        let line = lines[idx];
        if let Some(cap) = RAW_HEADER_REGEX.captures(line) {
            let file_path = cap[1].trim().to_string();
            idx += 1;
            while idx < lines.len() && lines[idx].trim().is_empty() {
                idx += 1;
            }
            if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                idx += 1; // skip opening fence
                let (code, new_idx) = extract_code_block(&lines, idx);
                idx = new_idx;
                results.push(ParsedFile {
                    path: file_path,
                    content: code.trim().to_string(),
                });
                continue;
            }
        }
        idx += 1;
    }
    results
}

/// Sub-parser 5: File/Code pattern using <file>...</file> and <code>...</code>
/// Example:
///     <file> Cargo.toml </file>
///     <code>
///     [package]
///     name = "trait_enforcement_demo"
///     ...
///     </code>
fn parse_file_code(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();
    lazy_static! {
        static ref FILE_TAG_REGEX: Regex =
            Regex::new(r#"(?is)<file>\s*([^<>\r\n]+?\.(?:rs|toml|json))\s*</file>"#).unwrap();
        static ref CODE_BLOCK_REGEX: Regex =
            Regex::new(r#"(?is)<code>\s*(.*?)\s*</code>"#).unwrap();
    }
    let mut files = Vec::new();
    for cap in FILE_TAG_REGEX.captures_iter(content) {
        files.push(cap[1].trim().to_string());
    }
    let mut codes = Vec::new();
    for cap in CODE_BLOCK_REGEX.captures_iter(content) {
        codes.push(cap[1].trim().to_string());
    }
    let count = files.len().min(codes.len());
    for i in 0..count {
        results.push(ParsedFile {
            path: files[i].clone(),
            content: codes[i].clone(),
        });
    }
    results
}

/// Pattern 6: “File‑tag heading” + fenced code block.
/// E.g.
/// ### <file> src/lib.rs </file>
/// ```rust
/// pub fn foo() {}
/// ```
fn parse_file_fence(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;

    lazy_static! {
        // ### <file> path </file>
        static ref FILE_HEADING_REGEX: Regex = Regex::new(
            r"(?i)^\s*#{1,6}\s*<file>\s*([^\s<>]+?\.(?:rs|toml|json))\s*</file>\s*$"
        ).unwrap();
        static ref CODE_FENCE_REGEX: Regex = Regex::new(r"^\s*```(?:[^\n]*)\s*$").unwrap();
    }

    while idx < lines.len() {
        if let Some(cap) = FILE_HEADING_REGEX.captures(lines[idx]) {
            let file_path = cap[1].trim().to_string();
            idx += 1;
            // skip blank lines
            while idx < lines.len() && lines[idx].trim().is_empty() {
                idx += 1;
            }
            // expect opening fence
            if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                idx += 1; // skip the ``` line
                let mut code_lines = Vec::new();
                // collect until closing fence or EOF
                while idx < lines.len() && !CODE_FENCE_REGEX.is_match(lines[idx]) {
                    code_lines.push(lines[idx]);
                    idx += 1;
                }
                // skip closing fence if present
                if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                    idx += 1;
                }
                let code = code_lines.join("\n").trim().to_string();
                results.push(ParsedFile {
                    path: file_path,
                    content: code,
                });
                continue;
            }
        }
        idx += 1;
    }
    results
}

/// Helper: extracts code lines from `lines` starting at idx until a closing code fence is found (or EOF).
fn extract_code_block(lines: &[&str], mut idx: usize) -> (String, usize) {
    lazy_static! {
        static ref CODE_FENCE_REGEX: Regex = Regex::new(r"^\s*```(?:[a-zA-Z0-9]*)\s*$").unwrap();
    }
    let mut code_lines = Vec::new();
    while idx < lines.len() {
        if CODE_FENCE_REGEX.is_match(lines[idx]) {
            idx += 1;
            break;
        } else {
            code_lines.push(lines[idx]);
            idx += 1;
        }
    }
    (code_lines.join("\n"), idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_code_tag_pattern() {
        let md = indoc! {r#"
            <code path="Cargo.toml">
            [package]
            name = "example"
            </code>
        "#};
        let parsed = parse_content(md, None);
        assert_eq!(
            parsed.len(),
            1,
            "Expected one parsed file for code tag pattern"
        );
        assert_eq!(parsed[0].path, "Cargo.toml");
        assert!(parsed[0].content.contains("[package]"));
    }

    #[test]
    fn test_hash_marker_pattern() {
        let md = indoc! {r#"
            ### src/main.rs
            ```rust
            fn main() { println!("Hello, world!"); }
            ```
        "#};
        let parsed = parse_content(md, None);
        assert_eq!(
            parsed.len(),
            1,
            "Expected one parsed file for hash marker pattern"
        );
        assert_eq!(parsed[0].path, "src/main.rs");
        assert!(parsed[0].content.contains("fn main()"));
    }

    #[test]
    fn test_delimiter_pattern() {
        let md = indoc! {r#"
            ========
            src/lib.rs
            ========
            ```rust
            pub fn lib_function() {}
            ```
        "#};
        let parsed = parse_content(md, None);
        assert_eq!(
            parsed.len(),
            1,
            "Expected one parsed file for delimiter pattern"
        );
        assert_eq!(parsed[0].path, "src/lib.rs");
        assert!(parsed[0].content.contains("lib_function"));
    }

    #[test]
    fn test_raw_code_block_pattern() {
        let md = indoc! {r#"
            // file: src/utils.rs
            ```rust
            pub fn util() {}
        "#};
        let parsed = parse_content(md, None);
        assert_eq!(
            parsed.len(),
            1,
            "Expected one parsed file for raw code block pattern"
        );
        assert_eq!(parsed[0].path, "src/utils.rs");
        assert!(parsed[0].content.contains("pub fn util() {}"));
    }

    #[test]
    fn test_hash_marker_no_closing_fence() {
        let md = indoc! {r#"
            ### src/missing.rs
            ```rust
            // Some code without a closing fence
            pub fn foo() {}
        "#};
        let parsed = parse_content(md, None);
        assert_eq!(
            parsed.len(),
            1,
            "Expected one parsed file even when closing fence is missing"
        );
        assert_eq!(parsed[0].path, "src/missing.rs");
        assert!(parsed[0].content.contains("pub fn foo() {}"));
    }

    #[test]
    fn test_file_code_pattern() {
        let md = indoc! {r#"
            <file> Cargo.toml </file>
            <code>
            [package]
            name = "trait_enforcement_demo"
            version = "0.1.0"
            edition = "2021"
            </code>
            
            <file> src/main.rs </file>
            <code>
            // Write the main Rust code here
            </code>
            
            <file> src/lib.rs </file>
            <code>
            // If needed, add trait definitions or supporting modules here
            </code>
        "#};
        let parsed = parse_content(md, Some(MdPatternType::FileCode));
        assert_eq!(
            parsed.len(),
            3,
            "Expected three parsed file blocks for file/code pattern"
        );
        assert_eq!(parsed[0].path, "Cargo.toml");
        assert!(parsed[0].content.contains("[package]"));
    }

    #[test]
    fn test_file_fence_pattern() {
        let md = indoc::indoc! {r#"
        ### <file> src/lib.rs </file>
        ```rust
        pub fn hello() { println!("hello"); }
        ```
    "#};

        let parsed = parse_content(md, Some(MdPatternType::FileFence));
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].path, "src/lib.rs");
        assert!(parsed[0].content.contains("println!(\"hello\")"));
    }
}
