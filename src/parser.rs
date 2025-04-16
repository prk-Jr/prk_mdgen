use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedFile {
    pub path: String,
    pub content: String,
}

/// Parses the given markdown content and returns a vector of ParsedFile (only for .rs, .toml, or .json files).
/// 
/// The parser supports several patterns:
/// 
/// 1. **XML-like code block**: `<code path="file_path"> ... </code>`
/// 2. **Hash marker pattern**: a header line like `### src/main.rs` followed by a code fence block.
/// 3. **Delimiter marker pattern**: a three‑line header consisting of a line of "=",
///    then the file name (e.g. `src/lib.rs`), then another line of "=",
///    followed by a code fence block.
/// 4. **Raw code block pattern**: a header comment like `// file: src/utils.rs` on its own line,
///    followed by a code fence block.
///
/// In this version, we first run each sub‐parser and then select the “best” group of parsed results.
/// Here “best” is defined as the one with the most extracted file blocks (you could adjust the logic as needed).
pub fn parse_content(content: &str) -> Vec<ParsedFile> {
    let group1 = parse_code_tag(content);
    let group2 = parse_hash_marker(content);
    let group3 = parse_delimiter_marker(content);
    let group4 = parse_raw_code_block(content);

    // Collect the non-empty groups.
    let mut groups: Vec<Vec<ParsedFile>> = vec![];
    if !group1.is_empty() { groups.push(group1); }
    if !group2.is_empty() { groups.push(group2); }
    if !group3.is_empty() { groups.push(group3); }
    if !group4.is_empty() { groups.push(group4); }

    if groups.is_empty() {
        vec![]
    } else {
        // Choose the group with the highest count.
        groups.into_iter().max_by_key(|g| g.len()).unwrap()
    }
}

/// Sub-parser 1: XML-like code block, e.g.:
///     <code path="Cargo.toml">
///     [package] ... 
///     </code>
fn parse_code_tag(content: &str) -> Vec<ParsedFile> {
    lazy_static! {
        static ref CODE_TAG_REGEX: Regex = Regex::new(
            r#"(?is)<code\s+path\s*=\s*"([^"\r\n]+?\.(?:rs|toml|json))">\s*(.*?)\s*</code>"#
        ).unwrap();
    }
    let mut results = Vec::new();
    for cap in CODE_TAG_REGEX.captures_iter(content) {
        let path = cap[1].trim().to_string();
        let code = cap[2].trim().to_string();
        results.push(ParsedFile { path, content: code });
    }
    results
}

/// Sub-parser 2: Hash marker pattern, e.g.:
///     ### src/main.rs
///     ```rust
///     // code block
///     ```
///
/// If no closing fence is found, capture to end-of-input.
fn parse_hash_marker(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();
    // Split the content into lines.
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;
    // Regex for the header line.
    lazy_static! {
        static ref HASH_HEADER_REGEX: Regex = Regex::new(r"^\s*#{1,6}\s+([^\s]+\.(?:rs|toml|json))\s*$").unwrap();
        static ref CODE_FENCE_REGEX: Regex = Regex::new(r"^\s*```(?:[a-zA-Z0-9]*)\s*$").unwrap();
    }
    while idx < lines.len() {
        let line = lines[idx];
        if let Some(cap) = HASH_HEADER_REGEX.captures(line) {
            let file_path = cap[1].trim().to_string();
            idx += 1;
            // Skip blank lines.
            while idx < lines.len() && lines[idx].trim().is_empty() {
                idx += 1;
            }
            if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                idx += 1; // Skip opening fence.
                let (code, new_idx) = extract_code_block(&lines, idx);
                idx = new_idx;
                results.push(ParsedFile { path: file_path, content: code.trim().to_string() });
                continue;
            }
        }
        idx += 1;
    }
    results
}

/// Sub-parser 3: Delimiter marker pattern, e.g.:
///     ========
///     src/lib.rs
///     ========
///     ```rust
///     // code block
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
        // Look for a line that contains only "=" characters.
        if line.trim().chars().all(|c| c == '=') && !line.trim().is_empty() {
            if idx + 2 < lines.len() {
                let candidate = lines[idx + 1].trim();
                if candidate.ends_with(".rs") || candidate.ends_with(".toml") || candidate.ends_with(".json") {
                    let delim_line = lines[idx + 2].trim();
                    if delim_line.chars().all(|c| c == '=') && !delim_line.is_empty() {
                        let file_path = candidate.to_string();
                        idx += 3; // Skip the header lines.
                        while idx < lines.len() && lines[idx].trim().is_empty() {
                            idx += 1;
                        }
                        if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                            idx += 1; // Skip opening fence.
                            let (code, new_idx) = extract_code_block(&lines, idx);
                            idx = new_idx;
                            results.push(ParsedFile { path: file_path, content: code.trim().to_string() });
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

/// Sub-parser 4: Raw code block pattern, e.g.:
///     // file: src/utils.rs
///     ```rust
///     // code block
///     ``` 
fn parse_raw_code_block(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;
    lazy_static! {
        static ref RAW_HEADER_REGEX: Regex = Regex::new(r"^\s*//\s*file:\s*([^\s]+\.(?:rs|toml|json))\s*$").unwrap();
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
                idx += 1; // Skip opening fence.
                let (code, new_idx) = extract_code_block(&lines, idx);
                idx = new_idx;
                results.push(ParsedFile { path: file_path, content: code.trim().to_string() });
                continue;
            }
        }
        idx += 1;
    }
    results
}

/// Helper: extracts code from lines starting at `idx` until a closing code fence is found (or EOF)
fn extract_code_block(lines: &[&str], mut idx: usize) -> (String, usize) {
    lazy_static! {
        static ref CODE_FENCE_REGEX: Regex = Regex::new(r"^\s*```(?:[a-zA-Z0-9]*)\s*$").unwrap();
    }
    let mut code_lines = Vec::new();
    while idx < lines.len() {
        if CODE_FENCE_REGEX.is_match(lines[idx]) {
            idx += 1; // Skip closing fence.
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
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file for code tag pattern");
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
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file for hash marker pattern");
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
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file for delimiter pattern");
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
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file for raw code block pattern");
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
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file even when closing fence is missing");
        assert_eq!(parsed[0].path, "src/missing.rs");
        assert!(parsed[0].content.contains("pub fn foo() {}"));
    }
}
