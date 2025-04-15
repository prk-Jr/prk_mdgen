use std::fs;

pub fn generate_sample_md() -> std::io::Result<()> {
    let sample_content = r#"
<code path="Cargo.toml">
[package]
name = "sample_project"
version = "0.1.0"
edition = "2021"
</code>

### src/main.rs
```rust
fn main() {
    println!("Hello, sample project!");
}
```
"#;
    fs::write("sample.md", sample_content.trim_start())?;
    println!("Generated sample.md");
    Ok(())
}

pub fn generate_prompt_md() -> std::io::Result<()> {
    let prompt_content = r#"
The following is an LLM prompt that defines the expected response format:

* The response should contain code blocks annotated with file paths using different patterns such as:
  - XML-like `<code path="...">` blocks.
  - Markdown headings (e.g. `### src/main.rs`) followed by code fences.
  - Delimiter markers (e.g. `======== src/lib.rs ========`) followed by code fences.
  - A raw file header comment (e.g. `// file: src/utils.rs`) before a code fence.

E.g.:
<code path="Cargo.toml">
[package]
name = "sample_project"
version = "0.1.0"
edition = "2021"
</code>

### src/main.rs
```rust
fn main() {
    println!("Hello, sample project!");
}
```
"#;
    fs::write("prompt.md", prompt_content.trim_start())?;
    println!("Generated prompt.md");
    Ok(())
}