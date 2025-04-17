mod parser;
mod scanner;
mod file_gen;
mod extra;
mod execute;

use std::env;
use std::path::Path;
use std::process;
use execute::execute_project_if_needed;
use extra::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Optional subcommand: sample or prompt.
    #[arg(value_enum, default_value = "none")]
    command: Option<CommandChoice>,

    /// Optional output directory for the generated project.
    #[arg(long, short, default_value = "output")]
    output_dir: String,

    #[arg(short, long)]
    execute: bool,

    /// Optional force MD pattern type (choices: code_tag, hash, delimiter, raw, file_code).
    #[arg(long, short, value_enum)]
    pattern: Option<MdPatternCli>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CommandChoice {
    Sample,
    Prompt,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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

    // Handle sample and prompt subcommands.
    if let Some(cmd) = cli.command {
        match cmd {
            CommandChoice::Sample => {
                if let Err(e) = generate_sample_md() {
                    eprintln!("Error generating sample.md: {}", e);
                    process::exit(1);
                }
                return;
            }
            CommandChoice::Prompt => {
                if let Err(e) = generate_prompt_md() {
                    eprintln!("Error generating prompt.md: {}", e);
                    process::exit(1);
                }
                return;
            }
            CommandChoice::None => {} // Continue.
        }
    }

    // Get current directory.
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("Scanning folder: {:?}", current_dir);

    // Find Markdown files.
    let md_files = scanner::find_md_files(&current_dir);
    if md_files.is_empty() {
        eprintln!("No .md files found in the current directory.");
        process::exit(1);
    }

    // Process each Markdown file concurrently.
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
                        let output = &Path::new(output_dir.as_str());
                        if let Err(err) = execute_project_if_needed(&output, &output) {
                            eprintln!("Execution failed for {}: {}", project_name, err);
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error reading file {:?}: {}", file_path, e),
        }
    });
}
