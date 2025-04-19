use std::fs;
use std::io::{self, Write};
use std::path::Path;

const GITIGNORE_CONTENT: &str = r#"
/target
/Cargo.lock
**/*.rs.bk
"#;

/// Generates the project in the given output directory using the provided parsed files,
/// and copies the source Markdown file into the generated project folder.
pub fn generate_project_with_dir(
    output_dir: &str,
    files: Vec<crate::parser::ParsedFile>,
    source_md: &Path,
) -> io::Result<()> {
    let out_path = Path::new(output_dir);
    fs::create_dir_all(out_path)?;

    // Write each extracted file.
    for file in files {
        let file_path = out_path.join(&file.path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = fs::File::create(file_path)?;
        f.write_all(file.content.as_bytes())?;
    }

    // Write a default .gitignore file if it doesn't exist.
    let gitignore_path = out_path.join(".gitignore");
    if !gitignore_path.exists() {
        let mut f = fs::File::create(gitignore_path)?;
        f.write_all(GITIGNORE_CONTENT.as_bytes())?;
    }

    // Copy the source Markdown file into the generated project directory.
    if let Some(md_filename) = source_md.file_name() {
        let dest = out_path.join(md_filename);
        fs::copy(source_md, dest)?;
    }
    Ok(())
}
