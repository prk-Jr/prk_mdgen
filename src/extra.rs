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
"#;
    fs::write("sample.md", sample_content.trim_start())?;
    println!("Generated sample.md");
    Ok(())
}

pub fn generate_prompt_md() -> std::io::Result<()> {
    let prompt_content = r#"

Expected Format:
Each code block should be associated with a file path using one of the following annotation styles:

XML-style tags:

xml
Copy
Edit
<code path="Cargo.toml">
...
</code>
Markdown heading with file path:

md
Copy
Edit
### src/main.rs
```rust
...
Delimiter-based markers:

python-repl
Copy
Edit
======== src/lib.rs ========
```rust
...
Raw file comment before a code fence:

rust
Copy
Edit
// file: src/utils.rs
...
Important Notes:
Every file must be properly annotated with its relative path.

Each Markdown input should define a complete Rust project, including a Cargo.toml.

If src/main.rs exists, the project will be built and run using cargo run.

If src/lib.rs exists, it will be tested using cargo test.

The output of these commands will be captured and stored in files such as run_output.log and test_output.log.

Example:
<code path="Cargo.toml"> [package] name = "sample_project" version = "0.1.0" edition = "2021" </code>
src/main.rs
rust
Copy
Edit
fn main() {
    println!("Hello, sample project!");
}
"#;

    fs::write("prompt.md", prompt_content.trim_start())?;
    println!("Generated prompt.md");
    Ok(())
}
