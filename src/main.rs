mod parser;
mod scanner;
mod file_gen;
mod extra;
mod execute;
mod extract;

use std::env;
use std::fs;
use std::path::Path;
use std::process;
use clap::{Parser, ValueEnum};
use execute::execute_project_if_needed;
use extract::{ExtractConfig, extract_to_markdown};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Choose an operation: sample, prompt, extract, or none (default).
    #[arg(value_enum, default_value = "none")]
    command: CommandChoice,

    /// Output directory for generated projects or extracted markdown.
    #[arg(short, long, default_value = "output")]
    output_dir: String,

    /// Execute generated projects (cargo run for main.rs, cargo test for lib.rs).
    #[arg(short, long)]
    execute: bool,

    /// Force a specific Markdown pattern for parsing (e.g. code-tag, hash, delimiter, raw, file-code, file-fence).
    #[arg(short, long, value_enum)]
    pattern: Option<MdPatternCli>,

    /// Optional project type hint for extraction (e.g. "rust", "flutter", "node").
    #[arg(long)]
    project_type: Option<String>,

    /// Comma‑separated list of file or folder names to skip during extraction.
    #[arg(long, value_delimiter = ',')]
    skip: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CommandChoice {
    Sample,
    Prompt,
    Extract,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum MdPatternCli {
    CodeTag,
    Hash,
    Delimiter,
    Raw,
    FileCode,
    FileFence,
}

impl From<MdPatternCli> for parser::MdPatternType {
    fn from(item: MdPatternCli) -> Self {
        match item {
            MdPatternCli::CodeTag => parser::MdPatternType::CodeTag,
            MdPatternCli::Hash => parser::MdPatternType::HashMarker,
            MdPatternCli::Delimiter => parser::MdPatternType::Delimiter,
            MdPatternCli::Raw => parser::MdPatternType::Raw,
            MdPatternCli::FileCode => parser::MdPatternType::FileCode,
            MdPatternCli::FileFence => parser::MdPatternType::FileFence,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // Handle sample, prompt, and extract subcommands.
    match cli.command {
        CommandChoice::Sample => {
            if let Err(e) = extra::generate_sample_md() {
                eprintln!("Error generating sample.md: {}", e);
                process::exit(1);
            }
            return;
        }
        CommandChoice::Prompt => {
            if let Err(e) = extra::generate_prompt_md() {
                eprintln!("Error generating prompt.md: {}", e);
                process::exit(1);
            }
            return;
        }
        CommandChoice::Extract => {
            let current_dir = env::current_dir().expect("Failed to get current directory");
            let ignore_file = current_dir.join(".gitignore");
            let config = ExtractConfig {
                root: current_dir.clone(),
                ignore_file: if ignore_file.exists() { Some(ignore_file) } else { None },
                extra_ignores: cli.skip.clone(),
                project_type: cli.project_type.clone(),
                pattern: cli.pattern.clone(),
            };
            match extract_to_markdown(config) {
                Ok(md) => {
                    let out_md = Path::new(&cli.output_dir).join("codebase.md");
                    fs::create_dir_all(&cli.output_dir).unwrap();
                    fs::write(&out_md, md).expect("Failed to write codebase.md");
                    println!("Extracted markdown to {:?}", out_md);
                }
                Err(e) => {
                    eprintln!("Extraction failed: {}", e);
                    process::exit(1);
                }
            }
            return;
        }
        CommandChoice::None => {}
    }

    // Default: generate Rust projects from Markdown files.
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("Scanning folder: {:?}", current_dir);

    let md_files = scanner::find_md_files(&current_dir);
    if md_files.is_empty() {
        eprintln!("No .md files found in the current directory.");
        process::exit(1);
    }

    md_files.par_iter().for_each(|file_path| {
        println!("Processing file: {:?}", file_path);
        match scanner::read_file(file_path) {
            Ok(content) => {
                let forced = cli.pattern.map(|pt| pt.into());
                let parsed_files = parser::parse_content(&content, forced);
                if parsed_files.is_empty() {
                    println!("No valid file blocks found in {:?}", file_path);
                } else if let Some(project_name) = scanner::extract_project_name(file_path) {
                    let output_dir = format!("{}/{}", cli.output_dir, project_name);
                    if let Err(err) = file_gen::generate_project_with_dir(&output_dir, parsed_files, file_path) {
                        eprintln!("Error generating project {}: {}", project_name, err);
                    } else {
                        println!("Project {} generated in {}", project_name, output_dir);
                        if cli.execute {
                            let project_path = Path::new(&output_dir);
                            if let Err(err) = execute_project_if_needed(project_path, project_path) {
                                eprintln!("Execution failed for {}: {}", project_name, err);
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error reading file {:?}: {}", file_path, e),
        }
    });
}