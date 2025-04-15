use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, PartialEq)]
pub struct ParsedFile {
    pub path: String,
    pub content: String,
}

/// Parses the given markdown content and returns a vector of ParsedFile (only for .rs, .toml, or .json files).
///
/// It supports several patterns:
///
/// 1. XML-like code block:
///    `<code path="file_path"> ... </code>`
///
/// 2. Hash marker pattern:
///    A header line like `### src/main.rs` followed by a code fence block.
///    The code fence must start with a line containing only "```" (optionally with "rust"),
///    then the code lines, and it ends when a line containing only "```" is found.
///    If no closing fence is found, all remaining lines are captured.
///
/// 3. Delimiter marker pattern:
///    A three‑line header consisting of:
///      - Line 1: a line of equals signs (only "=" characters),
///      - Line 2: the file name (e.g. `src/lib.rs`),
///      - Line 3: another line of equals signs,
///    followed by a code fence block (same as above).
///
/// 4. Raw code block pattern:
///    A header comment like `// file: src/utils.rs` on its own line,
///    followed by a code fence block (same as above).
///
/// Returns a vector of ParsedFile with the file path and captured code.
pub fn parse_content(content: &str) -> Vec<ParsedFile> {
    let mut results = Vec::new();

    // Pattern 1: <code path="..."> ... </code>
    // The pattern now uses the (?is) flags to be case-insensitive and allow '.' to match newlines.
    lazy_static! {
        static ref CODE_TAG_REGEX: Regex = Regex::new(
            r#"(?is)<code\s+path\s*=\s*"([^"\r\n]+?\.(?:rs|toml|json))">\s*(.*?)\s*</code>"#
        ).unwrap();
    }
    
    
    for cap in CODE_TAG_REGEX.captures_iter(content) {
        let path = cap[1].trim().to_string();
        let code = cap[2].trim().to_string();
        results.push(ParsedFile { path, content: code });
    }

    // Split the input into lines.
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;

    // Compile header regexes for hash marker and raw header.
    lazy_static! {
        // Hash marker: starts with 1-6 '#' followed by whitespace and file name.
        static ref HASH_HEADER_REGEX: Regex = Regex::new(r"^\s*#{1,6}\s+([^\s]+\.(?:rs|toml|json))\s*$").unwrap();
        // Raw code block header: // file: path
        static ref RAW_HEADER_REGEX: Regex = Regex::new(r"^\s*//\s*file:\s*([^\s]+\.(?:rs|toml|json))\s*$").unwrap();
        // Code fence: a line that starts with ``` (possibly with "rust") and nothing else.
        static ref CODE_FENCE_REGEX: Regex = Regex::new(r"^\s*```(?:rust)?\s*$").unwrap();
    }

    // Helper function to extract a code block given an index at the opening fence.
    // Returns (code_string, next index to process)
    fn extract_code_block(lines: &[&str], mut idx: usize) -> (String, usize) {
        let mut code_lines = Vec::new();
        while idx < lines.len() {
            if CODE_FENCE_REGEX.is_match(lines[idx]) {
                idx += 1; // Skip the closing fence.
                break;
            } else {
                code_lines.push(lines[idx]);
                idx += 1;
            }
        }
        (code_lines.join("\n"), idx)
    }

    // Main loop: scan lines for headers then process the corresponding code block.
    while idx < lines.len() {
        let line = lines[idx];

        // --- Hash marker processing ---
        if let Some(cap) = HASH_HEADER_REGEX.captures(line) {
            let file_path = cap[1].trim().to_string();
            idx += 1;
            // Skip blank lines to find the code fence.
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
        // --- Delimiter marker processing ---
        else if line.trim().chars().all(|c| c == '=') && !line.trim().is_empty() {
            // Check for three-line delimiter header.
            if idx + 2 < lines.len() {
                let candidate = lines[idx + 1].trim();
                // Check if candidate looks like a file name with allowed extension.
                if candidate.ends_with(".rs") || candidate.ends_with(".toml") || candidate.ends_with(".json") {
                    let delim_line = lines[idx + 2].trim();
                    if delim_line.chars().all(|c| c == '=') && !delim_line.is_empty() {
                        // We have matched the three-line header.
                        let file_path = candidate.to_string();
                        idx += 3; // Skip the three header lines.
                        // Skip blank lines until code fence.
                        while idx < lines.len() && lines[idx].trim().is_empty() {
                            idx += 1;
                        }
                        if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                            idx += 1; // Skip opening code fence.
                            let (code, new_idx) = extract_code_block(&lines, idx);
                            idx = new_idx;
                            results.push(ParsedFile { path: file_path, content: code.trim().to_string() });
                            continue;
                        }
                    }
                }
            }
        }
        // --- Raw code block processing ---
        else if let Some(cap) = RAW_HEADER_REGEX.captures(line) {
            let file_path = cap[1].trim().to_string();
            idx += 1;
            while idx < lines.len() && lines[idx].trim().is_empty() {
                idx += 1;
            }
            if idx < lines.len() && CODE_FENCE_REGEX.is_match(lines[idx]) {
                idx += 1;
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
        assert_eq!(parsed.len(), 1, "Expected one parsed file for code tag pattern");
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
        assert!(parsed[0].content.contains("pub fn util() {}"));
    }

    #[test]
    fn test_hash_marker_no_closing_fence() {
        let md = r#"
            ### src/missing.rs
            ```rust
            // Some code without a closing fence
            pub fn foo() {}
        "#;
        let parsed = parse_content(md);
        assert_eq!(parsed.len(), 1, "Expected one parsed file even when closing fence is missing");
        assert_eq!(parsed[0].path, "src/missing.rs");
        assert!(parsed[0].content.contains("pub fn foo() {}"));
    }
}