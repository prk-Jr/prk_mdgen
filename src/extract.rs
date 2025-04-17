use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use ignore::WalkBuilder;

pub struct ExtractConfig {
    /// Root directory to scan
    pub root: PathBuf,
    /// Optional path to a `.gitignore` or other ignore file
    pub ignore_file: Option<PathBuf>,
    /// Comma‑separated skip patterns (folder or file names)
    pub extra_ignores: Vec<String>,
    /// Optional hint (`"rust"`, `"flutter"`, `"node"`, etc.)
    pub project_type: Option<String>,
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

        // Skip by project_type (optional)
        if !should_include(&path, &config) {
            continue;
        }

        files.push(path);
    }

    // 3) Sort and apply --skip filters
    files.sort();
    files.retain(|path| {
        // Compute path relative to root
        let rel = path.strip_prefix(&config.root).unwrap_or(path);
        let rel_str = rel.to_string_lossy();

        // If any skip pattern matches any path component or a prefix, drop it
        !config.extra_ignores.iter().any(|pat| {
            rel_str.starts_with(pat)
                || rel.components().any(|c| *c.as_os_str() == **pat)
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
        let rel = path.strip_prefix(&config.root).unwrap();
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
        md.push_str(&format!("### <file> {} </file>\n", rel.display()));
        md.push_str(&format!("```{}\n", lang));
        let content = fs::read_to_string(&path)?;
        md.push_str(&content);
        md.push_str("```\n\n");
    }

    Ok(md)
}

/// Decide inclusion by project_type hint (optional) or by extension.
fn should_include(path: &Path, config: &ExtractConfig) -> bool {
    let rel = path.strip_prefix(&config.root).unwrap_or(path);
    let s = rel.to_string_lossy();

    // Always include core manifests
    if s == "Cargo.toml" || s == "pubspec.yaml" || s == "package.json" {
        return true;
    }

    if let Some(pt) = &config.project_type {
        match pt.as_str() {
            "flutter" => {
                // Only keep pubspec.yaml plus lib/
                return s == "pubspec.yaml" || s.starts_with("lib/");
            }
            "rust" => {
                // Keep Cargo.toml plus src/
                return s == "Cargo.toml" || s.starts_with("src/");
            }
            "node" => {
                // Keep package.json plus src/
                return s == "package.json" || s.starts_with("src/");
            }
            _ => {}
        }
    }

    // Fallback: include by common source extensions
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
        let rel = path.strip_prefix(root).unwrap();
        let parts: Vec<String> = rel.iter().map(|p| p.to_string_lossy().into()).collect();
        let common = last_parts
            .iter()
            .zip(&parts)
            .take_while(|(a, b)| a == b)
            .count();

        // Pop off any branches we backtracked from
        last_parts.truncate(common);

        // Print new branches
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
