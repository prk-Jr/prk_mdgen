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

## Expected Format:

Each code block should be associated with a file path using **one of the following annotation styles**:

1. **XML-style tags**:
   ```xml
   <code path="Cargo.toml">
   ...
   </code>
   ```

2. **Markdown heading with file path**:
   ```md
   ### src/main.rs
   ```rust
   ...
   ```

3. **Delimiter-based markers**:
   ```
   ======== src/lib.rs ========
   ```rust
   ...
   ```

4. **Raw file comment before a code fence**:
   ```rust
   // file: src/utils.rs
   ...
   ```

## Important Notes:

- Every file must be properly annotated with its relative path.
- Each Markdown input should define a complete Rust project, including a `Cargo.toml`.
- If `src/main.rs` exists, the project will be built and run using `cargo run`.
- If `src/lib.rs` exists, it will be tested using `cargo test`.
- The output of these commands will be captured and stored in files such as `run_output.txt` and `test_output.txt`.

## Example:

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
