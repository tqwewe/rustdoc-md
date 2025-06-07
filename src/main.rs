use std::{fs, io};

use clap::Parser;
use rustdoc_md::{rustdoc_json_to_fs, rustdoc_json_to_markdown, rustdoc_json_types::Crate};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to a rust docs json file
    #[arg(short, long)]
    path: std::path::PathBuf,

    /// The path for the output. A single file for single-file mode, or a directory for multi-file mode.
    #[arg(short, long)]
    output: std::path::PathBuf,

    /// Generate a directory of markdown files instead of a single file.
    #[arg(long)]
    multi_file: bool,
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    let file = fs::File::open(cli.path)?;
    let reader = io::BufReader::new(file);

    let data: Crate = serde_json::from_reader(reader)?;

    if cli.multi_file {
        if cli.output.exists() && !cli.output.is_dir() {
            return Err(eyre::eyre!(
                "For multi-file output, the output path must be a directory."
            ));
        }
        fs::create_dir_all(&cli.output)?;
        rustdoc_json_to_fs(&data, &cli.output)?;
        println!(
            "Generated multi-file documentation in: {}",
            cli.output.display()
        );
    } else {
        let md = rustdoc_json_to_markdown(data);
        fs::write(cli.output, md)?;
    }

    Ok(())
}
