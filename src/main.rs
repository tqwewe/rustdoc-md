use std::{fs, io, path::PathBuf};

use clap::Parser;
use rustdoc_md::{rustdoc_json_to_markdown, rustdoc_json_types::Crate};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to a rust docs json file
    #[arg(short, long)]
    path: PathBuf,

    /// The path to a rust docs json file
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    let file = fs::File::open(cli.path)?;
    let reader = io::BufReader::new(file);

    let data: Crate = serde_json::from_reader(reader)?;

    let md = rustdoc_json_to_markdown(data);

    fs::write(cli.output, md)?;

    Ok(())
}
