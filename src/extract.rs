use std::{fs, path::{Path, PathBuf}};
use anyhow::{Result, Context};
use ignore::WalkBuilder;
use crate::MdPatternCli;

pub struct ExtractConfig {
    pub root: PathBuf,
    pub ignore_file: Option<PathBuf>,
    pub extra_ignores: Vec<String>,
    pub project_type: Option<String>,
    pub pattern: Option<MdPatternCli>,
}

/// Simple project tree generator with no params — uses current dir
pub fn generate_tree_markdown() -> Result<String> {
    let root = std::env::current_dir().context("Failed to get current directory")?;

    let mut builder = WalkBuilder::new(&root);
    builder.git_ignore(true).git_exclude(true).hidden(true);
    let walker = builder.build();

    let mut files = Vec::new();
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        files.push(entry.into_path());
    }

    files.sort();

    let tree = build_tree(&files, &root);

    let mut md = String::new();
    md.push_str("# Project structure\n\n");
    md.push_str("```\n");
    md.push_str(&tree);
    md.push_str("```\n");

    Ok(md)
}

/// Walks the directory, applies ignores & skips, builds a tree and dumps every file into Markdown.
pub fn extract_to_markdown(config: ExtractConfig) -> Result<String> {
    // 1) Build the walker with .gitignore etc.
    let mut builder = WalkBuilder::new(&config.root);
    if let Some(ignore) = &config.ignore_file {
        builder.add_ignore(ignore);
    }
    builder.git_ignore(true).git_exclude(true).hidden(true);
    let walker = builder.build();

    // 2) Collect all candidate files
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.into_path();
        if !should_include(&path, &config) {
            continue;
        }
        files.push(path);
    }

    // Early exit if no files
    if files.is_empty() {
        return Ok("# Project structure\n\n*No files found*\n".to_string());
    }

    // 3) Sort and apply --skip filters
    files.sort();
    files.retain(|path| {
        let rel = path.strip_prefix(&config.root)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| path.clone());
        let rel_str = rel.to_string_lossy();
        !config.extra_ignores.iter().any(|pat| {
            rel_str.starts_with(pat) || rel.components().any(|c| *c.as_os_str() == **pat)
        })
    });

    // 4) Build an ASCII tree
    let tree = build_tree(&files, &config.root);

    // 5) Emit Markdown
    let mut md = String::new();
    md.push_str("# Project structure\n\n");
    md.push_str("```\n");
    md.push_str(&tree);
    md.push_str("```\n\n");

    for path in files {
        // compute relative path, normalize separators
        let rel = path.strip_prefix(&config.root)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| path.clone());
        let rel_raw = rel.to_string_lossy().to_string();
        let rel_str = rel_raw.replace('\\', "/");
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang = match ext {
            "rs" => "rust",
            "toml" => "toml",
            "json" => "json",
            "dart" => "dart",
            "js" => "javascript",
            "ts" => "typescript",
            _ => "",
        };
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read file: {:?}", path))?;

        // build the block
        let block = match config.pattern {
            Some(MdPatternCli::CodeTag) => format!(
                "<code path=\"{0}\">\n{1}\n</code>\n\n", 
                rel_str, content.trim()
            ),
            Some(MdPatternCli::Hash) => format!(
                "### {0}\n{1}", rel_str, fenced(&rel_str, lang, &content)
            ),
            Some(MdPatternCli::Delimiter) => format!(
                "========\n{0}\n========\n{1}", rel_str, fenced(&rel_str, lang, &content)
            ),
            Some(MdPatternCli::Raw) => format!(
                "// file: {0}\n{1}", rel_str, fenced(&rel_str, lang, &content)
            ),
            Some(MdPatternCli::FileCode) => format!(
                "<file> {0} </file>\n<code>\n{1}\n</code>\n\n", 
                rel_str, content.trim()
            ),
            Some(MdPatternCli::FileFence) | None => format!(
                "### <file> {0} </file>\n{1}", rel_str, fenced(&rel_str, lang, &content)
            ),
        };
        md.push_str(&block);
    }

    Ok(md)
}

/// Helper to produce a fenced code block with language and content
fn fenced(rel: &str, lang: &str, content: &str) -> String {
    format!("```{}\n{}\n```\n\n", lang, content.trim())
}

/// Decide inclusion by project_type hint (optional) or by extension.
fn should_include(path: &Path, config: &ExtractConfig) -> bool {
    let rel = path.strip_prefix(&config.root).unwrap_or(path);
    let s = rel.to_string_lossy();

    if s == "Cargo.toml" || s == "pubspec.yaml" || s == "package.json" {
        return true;
    }

    if let Some(pt) = &config.project_type {
        match pt.as_str() {
            "flutter" => return s == "pubspec.yaml" || s.starts_with("lib/"),
            "rust" => return s == "Cargo.toml" || s.starts_with("src/"),
            "node" => return s == "package.json" || s.starts_with("src/"),
            _ => {}
        }
    }

    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "toml" | "json" | "js" | "ts" | "dart")
    )
}

/// Build a simple ASCII tree representation from a sorted list of file paths.
fn build_tree(files: &[PathBuf], root: &Path) -> String {
    let mut tree = String::new();
    let mut last_parts: Vec<String> = Vec::new();

    for path in files {
        let rel = path.strip_prefix(root).unwrap_or(path);
        let parts: Vec<String> = rel.iter().map(|p| p.to_string_lossy().into()).collect();
        let common = last_parts.iter().zip(&parts).take_while(|(a, b)| a == b).count();

        last_parts.truncate(common);
        for part in &parts[common..] {
            for _ in 0..last_parts.len() {
                tree.push_str("    ");
            }
            tree.push_str("├── ");
            tree.push_str(part);
            tree.push('\n');
            last_parts.push(part.clone());
        }
    }

    tree
}
