use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use rayon::prelude::*;

/// Finds all markdown files in the given directory that match the pattern `{name}.md`
pub fn find_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut md_files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        entries.filter_map(|entry| entry.ok())
            .for_each(|entry| {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "md" {
                            md_files.push(path);
                        }
                    }
                }
            });
    }
    md_files
}

/// Reads the entire content of the specified file.
pub fn read_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

/// Extracts the project name from the markdown file's filename (without extension).
pub fn extract_project_name(path: &Path) -> Option<String> {
    path.file_stem().and_then(|os_str| os_str.to_str()).map(|s| s.to_string())
}
