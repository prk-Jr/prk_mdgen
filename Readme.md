# 🦀 Markdown to Rust Project Generator

This tool parses specially formatted Markdown files to generate fully structured Rust projects — and even executes them (via `cargo run` or `cargo test` depending on the code)!

---

## 📦 What It Does

This CLI tool scans all Markdown (`.md`) files in the current directory, extracts embedded Rust code annotated with file paths, and:

- **Generates complete Rust projects** (files, folders, `Cargo.toml`, etc.)
- **Builds and runs projects** with `main.rs` using `cargo run`
- **Runs tests** on projects with `lib.rs` using `cargo test`
- Saves execution output in `run_output.txt` or `test_output.txt` respectively

---

## 🧠 Supported Markdown Formats

You can annotate code blocks in your Markdown using several supported patterns:

### ✅ Supported Patterns

1. **XML-style tag**
   ```xml
   <code path="src/main.rs">
   fn main() {
       println!("Hello!");
   }
   </code>
   ```

2. **Markdown heading**
   ```md
   ### src/lib.rs
   ```rust
   pub fn add(a: i32, b: i32) -> i32 {
       a + b
   }
   ```
   ```

3. **Delimiter marker**
   ```
   ======== src/utils.rs ========
   ```rust
   pub fn double(x: i32) -> i32 {
       x * 2
   }
   ```
   ```

4. **Raw comment before code**
   ```rust
   // file: src/math.rs
   pub fn square(x: i32) -> i32 {
       x * x
   }
   ```
5. **File befor code**
```md
   ### <file> src/main.rs </file>
```rust
use std::sync::Arc;
use std::error::Error;
use std::future::Future;
   ```
```

---

## 🚀 Getting Started

### 🛠 Prerequisites

- Rust & Cargo installed (`https://rustup.rs`)
- `rayon` and `clap` as dependencies

### 📥 Installation


```bash
cargo install prk_mdgen
```

---

## 🧪 Usage

Run the parser in the directory containing Markdown files:

```bash
prk_mdgen
```

You can customize it using the CLI options:

```bash
USAGE:
    prk_mdgen [OPTIONS]

OPTIONS:
    -o, --output-dir <DIR>     Output directory for generated projects [default: output]
    -p, --pattern <PATTERN>    Force a specific pattern (code-tag, hash, delimiter, raw, file-code)
    -c, --command <COMMAND>    Generate a sample or prompt markdown file (sample, prompt)
```

### 🔍 Example

```bash
# Generate a prompt template for an LLM
prk_mdgen prompt

# Process all .md files and write projects to ./output
prk_mdgen -o ./output

# Force pattern detection to use XML code tags only
prk_mdgen --pattern code-tag

# Process all .md files and write projects to ./output and then run `cargo run` and `cargo test`
prk_mdgen -0 ./output -e
```

---

## 📂 Output Structure

For each Markdown file found:

```text
output/
└── my_project/
    ├── Cargo.toml
    ├── src/
    │   └── main.rs
    └── run_output.txt   # if main.rs exists
```

---

## 🧪 Execution Behavior

- If `src/main.rs` is present: runs `cargo run`, output is saved to `run_output.txt`
- If `src/lib.rs` is present: runs `cargo test`, output is saved to `test_output.txt`

---

## 📚 Development

To test parsing or project generation without writing a full Markdown file, you can generate starter files:

```bash
prk_mdgen sample
```

### 🔧 Add New Parsing Styles?

Look inside `parser.rs` and update `MdPatternType` logic to support new formats.

---

## 🧑‍💻 Contributing

Pull requests welcome! Please include:

- A relevant test Markdown file
- Use of at least one of the supported patterns
- Clean `cargo fmt`-ed code

---

## 🪪 License

MIT

---

## ❤️ Made with Rust

This tool is built with love in Rust using:

- [`clap`](https://docs.rs/clap/) for CLI parsing
- [`rayon`](https://docs.rs/rayon/) for parallel file processing
