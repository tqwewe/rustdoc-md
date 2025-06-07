use std::path::PathBuf;

use clap::Parser;
use rustdoc_md::rustdoc_json_types::ParsedCrateDoc;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to a rust docs json file
    #[arg(short, long, value_name = "JSON_PATH")]
    input_json: PathBuf,

    /// The output path.
    /// If not specified, Markdown is printed to stdout (single-document mode).
    /// If specified and --multi-file is not used, output is a single Markdown file.
    /// If specified and --multi-file is used, output is a directory for multiple Markdown files.
    #[arg(short, long, value_name = "OUTPUT_PATH")]
    output: Option<PathBuf>,

    /// Generate a directory of markdown files instead of a single document.
    /// Requires --output to be specified.
    #[arg(long)]
    multi_file: bool,
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    let krate = ParsedCrateDoc::from_file(&cli.input_json)?;

    match cli.output {
        None => {
            // Mode 1: Output to stdout
            if cli.multi_file {
                return Err(eyre::eyre!(
                    "--multi-file mode requires an --output path to be specified."
                ));
            }
            let md = krate.to_string();
            println!("{}", md);
            eprintln!("Generated single-document Markdown to stdout.");
        }
        Some(output_path) => {
            if cli.multi_file {
                // Mode 3: Multi-file output to directory
                if output_path.exists() && !output_path.is_dir() {
                    return Err(eyre::eyre!(
                        "For multi-file output, the output path '{}' must be a directory, but it's a file.",
                        output_path.display()
                    ));
                }
                krate.to_multi_file(&output_path)?;
                println!(
                    "Generated multi-file documentation in: {}",
                    output_path.display()
                );
            } else {
                // Mode 2: Single-file output to file
                if output_path.is_dir() {
                    return Err(eyre::eyre!(
                        "For single-file output, the output path '{}' must be a file, but it's a directory.",
                        output_path.display()
                    ));
                }
                krate.to_single_file(&output_path)?;
                println!(
                    "Generated single-file documentation to: {}",
                    output_path.display()
                );
            }
        }
    }

    Ok(())
}
