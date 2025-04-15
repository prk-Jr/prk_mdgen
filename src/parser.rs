use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, PartialEq)]
pub struct ParsedFile {
    pub path: String,
    pub content: String,
}

/// Parses the given markdown content and returns a vector of ParsedFile (only for .rs, .toml, .json files).
///
/// Supported patterns:
///
/// 1. `<code path="file_path"> ... </code>`
/// 2. Code fence blocks with preceding marker (e.g. `### file_path.rs`).
/// 3. Delimiter-based marker: lines like `======== file_path.rs ========` before a code block.
/// 4. Raw code block with a header comment (e.g. `// file: src/main.rs`).
pub fn parse_content(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();

    // Pattern 1: <code path="..."> ... </code>
    lazy_static! {
        static ref CODE_TAG_REGEX: Regex = Regex::new(
            r#"<code\s+path\s*=\s*"([^"]+\.(?:rs|toml|json))">\s*(?s:(.*?))\s*</code>"#
        ).unwrap();
    }
    for cap in CODE_TAG_REGEX.captures_iter(content) {
        let path = cap[1].trim().to_string();
        let code = cap[2].trim().to_string();
        results.push(ParsedFile { path, content: code });
    }

    // Pattern 2: Lines starting with ### (or 1-6 hashes) followed by file path
    // on its own line, then a ```rust code fence on its own line,
    // then the code block, and ending with a closing ``` line.
    lazy_static! {
        static ref HASH_MARKER_REGEX: Regex = Regex::new(
            r"(?m)^\s*#{1,6}\s+([^\s]+?\.(?:rs|toml|json))\s*$\r?\n\s*```(?:rust)?\s*$\r?\n(?s:(.*?))\s*```"
        ).unwrap();
    }
    for cap in HASH_MARKER_REGEX.captures_iter(content) {
        let path = cap[1].trim().to_string();
        let code = cap[2].trim().to_string();
        results.push(ParsedFile { path, content: code });
    }

    // Pattern 3: Delimiter-based marker (======== file_path.rs ========) followed by a code fence.
    lazy_static! {
        static ref DELIMITER_REGEX: Regex = Regex::new(
            r"(?m)^\s*=+\s*([^\s]+?\.(?:rs|toml|json))\s*=+\s*$\r?\n\s*```(?:rust)?\s*$\r?\n(?s:(.*?))\s*```"
        ).unwrap();
    }
    for cap in DELIMITER_REGEX.captures_iter(content) {
        let path = cap[1].trim().to_string();
        let code = cap[2].trim().to_string();
        results.push(ParsedFile { path, content: code });
    }

    // Pattern 4: A raw code block directly annotated with ```rust,
    // preceded by a header comment (e.g. // file: src/main.rs) on its own line.
    // In this case, if there's no closing fence, we capture until end-of-input.
    lazy_static! {
        static ref RAW_CODE_BLOCK_REGEX: Regex = Regex::new(
            r"(?m)^\s*//\s*file:\s*([^\s]+?\.(?:rs|toml|json))\s*$\r?\n\s*```(?:rust)?\s*$\r?\n(?s:(.*))"
        ).unwrap();
    }
    for cap in RAW_CODE_BLOCK_REGEX.captures_iter(content) {
        let path = cap[1].trim().to_string();
        // Capture until end-of-input when no closing fence is provided.
        let code = cap[2].trim().to_string();
        results.push(ParsedFile { path, content: code });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_tag_pattern() {
        let md = r#"
            <code path="Cargo.toml">
            [package]
            name = "example"
            </code>
        "#;
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].path, "Cargo.toml");
        assert!(parsed[0].content.contains("[package]"));
    }

    #[test]
    fn test_hash_marker_pattern() {
        let md = r#"
            ### src/main.rs
            ```rust
            fn main() { println!("Hello, world!"); }
            ```
        "#;
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file for hash marker pattern");
        assert_eq!(parsed[0].path, "src/main.rs");
        assert!(parsed[0].content.contains("fn main()"));
    }

    #[test]
    fn test_delimiter_pattern() {
        let md = r#"
            ========
            src/lib.rs
            ========
            ```rust
            pub fn lib_function() {}
            ```
        "#;
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file for delimiter pattern");
        assert_eq!(parsed[0].path, "src/lib.rs");
        assert!(parsed[0].content.contains("lib_function"));
    }

    #[test]
    fn test_raw_code_block_pattern() {
        let md = r#"
            // file: src/utils.rs
            ```rust
            pub fn util() {}
        "#;
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file for raw code block pattern");
        assert_eq!(parsed[0].path, "src/utils.rs");
        assert!(parsed[0].content.contains("util"));
    }
}