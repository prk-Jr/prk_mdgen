use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn integration_test_generate_project() {
    // Prepare a temporary directory with an example markdown file.
    let tmp_dir = tempfile::tempdir().unwrap();
    let md_path = tmp_dir.path().join("demo.md");
    let md_content = r#"
        <code path="Cargo.toml">
        [package]
        name = "demo_project"
        version = "0.1.0"
        </code>

        ### src/main.rs
        ```rust
        fn main() {
            println!("Hello, Demo!");
        }
        ```
    "#;
    fs::write(&md_path, md_content).unwrap();

    // Change working directory.
    let orig_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();

    // Run the application binary.
    let mut cmd = Command::cargo_bin("prk_md_parser").unwrap();
    cmd.assert().success();

    // Check that the output folder was created.
    let output_path = tmp_dir.path().join("output").join("demo");
    assert!(Path::new(&output_path.join("Cargo.toml")).exists());
    assert!(Path::new(&output_path.join("src/main.rs")).exists());

    // Restore original working directory.
    std::env::set_current_dir(orig_dir).unwrap();
}