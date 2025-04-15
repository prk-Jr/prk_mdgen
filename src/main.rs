mod parser;
mod scanner;
mod file_gen;
mod extra;

use std::env;
use std::process;
use extra::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for subcommands "sample" and "prompt"
    if args.len() > 1 {
        match args[1].as_str() {
            "sample" => {
                if let Err(e) = generate_sample_md() {
                    eprintln!("Error generating sample.md: {}", e);
                    process::exit(1);
                }
                return;
            }
            "prompt" => {
                if let Err(e) = generate_prompt_md() {
                    eprintln!("Error generating prompt.md: {}", e);
                    process::exit(1);
                }
                return;
            }
            _ => {
                // Fall back to default behavior: generate projects from *.md files.
                println!("Unknown command. Proceeding with project generation from markdown files.");
            }
        }
    }

    // Get current directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("Scanning folder: {:?}", current_dir);

    // Scan the directory for markdown files and parse them concurrently using rayon.
    let md_files = scanner::find_md_files(&current_dir);

    if md_files.is_empty() {
        eprintln!("No .md files found in the current directory.");
        process::exit(1);
    }

    // Process each markdown file concurrently.
    md_files.par_iter().for_each(|file_path| {
        println!("Processing file: {:?}", file_path);
        match scanner::read_file(file_path) {
            Ok(content) => {
                // Parse out file blocks (only for recognized file extensions)
                let parsed_files = parser::parse_content(&content);
                if parsed_files.is_empty() {
                    println!("No valid file blocks found in {:?}", file_path);
                } else {
                    // Derive project name from the markdown file's basename.
                    if let Some(project_name) = scanner::extract_project_name(file_path) {
                        file_gen::generate_project(&project_name, parsed_files)
                            .unwrap_or_else(|err| eprintln!("Error generating project {}: {}", project_name, err));
                        println!("Project {} generated.", project_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading file {:?}: {}", file_path, e);
            }
        }
    });
}