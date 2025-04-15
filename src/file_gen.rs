use std::fs;
use std::io::{self, Write};
use std::path::Path;

const GITIGNORE_CONTENT: &str = r#"
/target
/Cargo.lock
**/*.rs.bk
"#;

/// Creates the output project structure based on project_name and writes the extracted files.
pub fn generate_project(project_name: &str, files: Vec<crate::parser::ParsedFile>) -> io::Result<()> {
    let output_dir = Path::new("output").join(project_name);
    fs::create_dir_all(&output_dir)?;

    for file in files {
        // Create subdirectories if necessary.
        let file_path = output_dir.join(&file.path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = fs::File::create(file_path)?;
        f.write_all(file.content.as_bytes())?;
    }

    // Write a default .gitignore file if one doesn't exist.
    let gitignore_path = output_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let mut file = fs::File::create(gitignore_path)?;
        file.write_all(GITIGNORE_CONTENT.as_bytes())?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedFile;
    use tempfile::tempdir;
    use std::path::Path;

    #[test]
    fn test_generate_project() {
        let tmp_dir = tempdir().unwrap();
        let output_path = tmp_dir.path().join("output");
        fs::create_dir_all(&output_path).unwrap();

        // Set a temporary current directory.
        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp_dir).unwrap();

        let files = vec![
            ParsedFile {
                path: "Cargo.toml".to_string(),
                content: "[package]\nname = \"test_project\"".to_string(),
            },
            ParsedFile {
                path: "src/main.rs".to_string(),
                content: "fn main() { println!(\"Hello\"); }".to_string(),
            },
        ];

        generate_project("test_project", files).expect("Failed to generate project");
        let cargo_path = Path::new("output/test_project/Cargo.toml");
        let main_path = Path::new("output/test_project/src/main.rs");
        let gitignore_path = Path::new("output/test_project/.gitignore");

        assert!(cargo_path.exists());
        assert!(main_path.exists());
        assert!(gitignore_path.exists());

        // Restore the original working directory.
        std::env::set_current_dir(orig_dir).unwrap();
    }
}