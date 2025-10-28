use std::{fs, io, path::PathBuf};

use clap::{ArgGroup, Parser};
use rustdoc_md::rustdoc_json_to_markdown;
use rustdoc_types::Crate;

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
    #[arg(long, requires = "crate_name")]
    target: Option<String>,

    /// The path to the output markdown file.
    #[arg(short, long)]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    let data: Crate = if let Some(path) = cli.path {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        serde_json::from_reader(reader)?
    } else if let Some(crate_name) = cli.crate_name {
        let mut url = format!("https://docs.rs/crate/{}/{}", crate_name, cli.crate_version);
        if let Some(target) = cli.target {
            url.push_str(&format!("/{target}"));
        }
        url.push_str("/json");

        let client = reqwest::Client::builder().build()?;
        let resp = client.get(&url).send().await?;
        let bytes = resp.bytes().await?;
        let body = decode_all(bytes.as_ref())?;
        serde_json::from_reader(body.as_slice())?
    } else {
        unreachable!();
    };

    let md = rustdoc_json_to_markdown(data);

    fs::write(cli.output, md)?;

    Ok(())
}
