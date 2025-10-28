use std::{fs, io, path::PathBuf};

use clap::{ArgGroup, Parser};
use eyre::bail;
use rustdoc_md::rustdoc_json_to_markdown;
use rustdoc_types::Crate;

use ureq::http::StatusCode;
use zstd::decode_all;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(&["path", "crate_name"]),
))]
struct Cli {
    /// The path to a local rust docs json file.
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// The name of the crate to fetch from docs.rs.
    #[arg(long)]
    crate_name: Option<String>,

    /// The version of the crate to fetch (defaults to latest). Requires --crate-name.
    #[arg(long, default_value = "latest", requires = "crate_name")]
    crate_version: String,

    /// The target triple to fetch documentation for. Requires --crate-name.
    #[arg(
        long,
        default_value = "x86_64-unknown-linux-gnu",
        requires = "crate_name"
    )]
    target: String,

    /// The path to the output markdown file.
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    let data: Crate = if let Some(path) = cli.path {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        serde_json::from_reader(reader)?
    } else if let Some(crate_name) = cli.crate_name {
        let url = format!(
            "https://docs.rs/crate/{crate_name}/{}/{}/json",
            cli.crate_version, cli.target
        );

        let resp = ureq::get(&url)
            .header(
                "user-agent",
                concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            )
            .call()?;
        let status = resp.status();
        if !status.is_success() {
            match status {
                StatusCode::NOT_FOUND => {
                    bail!("crate or version not found, or doesn't provide rustdocs as json");
                }
                _ => {
                    bail!("failed to fetch crate json: {status}");
                }
            }
        }

        let reader = resp.into_body().into_reader();
        let body = decode_all(reader)?;
        serde_json::from_reader(body.as_slice())?
    } else {
        unreachable!("neither --path nor --crate-name set");
    };

    let md = rustdoc_json_to_markdown(data);
    fs::write(&cli.output, md)?;

    println!("successfully wrote to file {}", cli.output.display());

    Ok(())
}
