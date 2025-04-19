use std::{fs, process::Command, path::Path};

pub fn execute_project_if_needed(project_dir: &Path, output_dir: &Path) -> std::io::Result<()> {
    let main_rs = project_dir.join("src/main.rs");
    let lib_rs = project_dir.join("src/lib.rs");
    let cargo_toml = project_dir.join("Cargo.toml");

    if !cargo_toml.exists() {
        eprintln!("No Cargo.toml found at {:?}, skipping execution.", cargo_toml);
        return Ok(());
    }

    // Ensure the output directory exists
    fs::create_dir_all(output_dir)?;

    // Run `cargo run` if main.rs is present
    if main_rs.exists() {
        let output_file = output_dir.join("run_output.txt");
        println!("Executing `cargo run` for {:?}", project_dir);

        let output = Command::new("cargo")
            .arg("run")
            .current_dir(project_dir)
            .output()?;

        let combined_output = format!(
            "[STDOUT]\n{}\n[STDERR]\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        fs::write(&output_file, combined_output)?;
    }

    // Run `cargo test` only if lib.rs is present
    if lib_rs.exists() {
        let output_file = output_dir.join("test_output.txt");
        println!("Executing `cargo test` for {:?}", project_dir);

        let output = Command::new("cargo")
            .arg("test")
            .current_dir(project_dir)
            .output()?;

        let combined_output = format!(
            "[STDOUT]\n{}\n[STDERR]\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        fs::write(&output_file, combined_output)?;
    }

    Ok(())
}