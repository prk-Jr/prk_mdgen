# 🦀 Markdown to Rust Project Generator

This tool parses specially formatted Markdown files to generate fully structured Rust projects — and even executes them (via `cargo run` or `cargo test` depending on the code)!

It can also extract a real Rust project from disk into a Markdown spec file using the `extract` command!

---

## 📦 What It Does

This CLI tool supports two main workflows:

### 🛠 1. **Project Generation**

- Scans all Markdown (`.md`) files in the current directory
- Extracts embedded Rust code annotated with file paths
- **Generates full Rust projects** (files, folders, `Cargo.toml`, etc.)
- **Builds and runs** `main.rs` projects using `cargo run`
- **Tests** `lib.rs` projects using `cargo test`
- Saves execution output to `run_output.txt` or `test_output.txt`

### 📤 2. **Project Extraction**

- Takes an existing Rust codebase and **generates a single Markdown file**
- All `.rs` files are converted to annotated code blocks with file paths
- Supports `.gitignore` and additional `--skip` rules
- Great for documentation, LLM prompts, or reproducible specs

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

5. **File Fence**
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

### 📥 Installation

```bash
cargo install prk_mdgen
```

---

## 🧪 Usage

### 🔁 Markdown → Rust (Default Mode)

```bash
prk_mdgen
```

Generate Rust projects from all `.md` files in the current directory.

### 📤 Extract Rust → Markdown

```bash
prk_mdgen extract -o ./docs --skip target,.git,tests
```

This will scan the current Rust project and generate `docs/codebase.md` with annotated code blocks for each file.

---

### 🔧 Additional CLI Options

```bash
USAGE:
    prk_mdgen [OPTIONS]

OPTIONS:
    -o, --output-dir <DIR>     Output directory [default: output]
    -p, --pattern <PATTERN>    Force a specific pattern (code-tag, hash, delimiter, raw, file-code, file-fence)
    -c, --command <COMMAND>    sample | prompt | extract
    -e, --execute              Run `cargo run` or `cargo test` on generated projects
        --skip <ITEMS>         Comma-separated list of files or folders to skip
        --project-type <TYPE>  (Optional) Language hint during extraction (e.g. rust, node, flutter)
```

---

### 🔍 Example Workflows

```bash
# Generate a prompt template
prk_mdgen prompt

# Generate sample Markdown to test parsing
prk_mdgen sample

# Extract an existing Rust project to Markdown
prk_mdgen extract -o ./docs --skip target,.git

# Generate projects from Markdown and run them
prk_mdgen -o ./output -e
```

---

## 📂 Output Structure

When generating a project from Markdown:

```text
output/
└── my_project/
    ├── Cargo.toml
    ├── src/
    │   └── main.rs
    └── run_output.txt   # if main.rs exists
```

When extracting a project to Markdown:

```text
output/
└── codebase.md   # contains annotated code blocks for each source file
```

---

## 🧪 Execution Behavior

- If `src/main.rs` is present: runs `cargo run`, output is saved to `run_output.txt`
- If `src/lib.rs` is present: runs `cargo test`, output is saved to `test_output.txt`

---

## 📚 Development

To test parsing or generation logic:

```bash
prk_mdgen sample
```

To add new parsing formats, see `parser.rs` and extend the `MdPatternType` enum and detection logic.

---

## 🧑‍💻 Contributing

Pull requests welcome! Please:

- Add a Markdown test case
- Use one or more supported code block patterns
- Run `cargo fmt` before committing

---

## 🪪 License

MIT

---

## ❤️ Made with Rust

This tool is built with love in Rust using:

- [`clap`](https://docs.rs/clap/) for CLI parsing
- [`rayon`](https://docs.rs/rayon/) for parallel file processing
