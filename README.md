# rustdoc-md

Convert Rust documentation JSON into clean, organized Markdown.

[![Crates.io](https://img.shields.io/crates/v/rustdoc-md.svg)](https://crates.io/crates/rustdoc-md)
[![Documentation](https://docs.rs/rustdoc-md/badge.svg)](https://docs.rs/rustdoc-md)

`rustdoc-md` transforms the JSON output from `rustdoc` into a comprehensive, well-structured Markdown document. This makes your Rust API documentation easily shareable in contexts where HTML documentation isn't ideal, such as GitHub wikis, embedding in other documents, or sharing with AI assistants.

## Features

- Converts complete rustdoc JSON output to a single navigable Markdown file
- Preserves full method signatures with parameters and return types
- Maintains proper hierarchical organization of modules, types, traits, etc.
- Formats code examples, documentation, and API details for optimal readability
- Handles re-exports, implementations, and other specialized items


## Usage

### Step 1: Generate JSON documentation

The JSON output format is currently a nightly-only feature, but you can use it on stable Rust with the `RUSTC_BOOTSTRAP=1` environment variable:

```bash
# On nightly Rust:
RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo doc --no-deps

# On stable Rust (using bootstrap):
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo doc --no-deps
```

This will generate JSON documentation in your `target/doc` directory.

### Step 2: Convert to Markdown using CLI

```bash
rustdoc-md --path target/doc/your_crate.json --output api_docs.md
```

### API Usage

You can also use `rustdoc-md` as a library in your Rust projects:

```rust
use rustdoc_md::rustdoc_json_types::ParsedCrateDoc;
use std::{fs, path::Path};

fn main() -> eyre::Result<()> {
    let json_path = Path::new("target/doc/your_crate.json");

    // Load the rustdoc JSON data
    let krate = ParsedCrateDoc::from_file(json_path)?;
    
    // Generate a single Markdown string
    let markdown_string = krate.to_string();
    fs::write("api_docs_string.md", markdown_string)?;
    println!("Single string Markdown generated to api_docs_string.md");

    // Or, generate and save to a single file directly
    krate.to_single_file(Path::new("api_docs_single_file.md"))?;
    println!("Single file Markdown generated to api_docs_single_file.md");

    // Or, generate a multi-file documentation structure
    krate.to_multi_file(Path::new("docs_output_directory"))?;
    println!("Multi-file Markdown generated in docs_output_directory/");
    
    Ok(())
}
```

## Compatibility

This crate is compatible with rustdoc JSON format version 43 (as per `FORMAT_VERSION` in `rustdoc_json_types.rs`). The format may change in future Rust releases as it's still considered unstable.

For tracking the latest rustdoc JSON schema changes, see the [rustdoc-json-types repository](https://github.com/rust-lang/rust/blob/master/src/rustdoc-json-types/lib.rs).

## Why Use rustdoc-md?

- Creating comprehensive API documentation for AI-assisted development
- Generating documentation for offline use or embedding in other tools
- Making your API documentation available in contexts where HTML isn't suitable
- Simplifying documentation review in pull requests and code discussions

## License

MIT or Apache-2.0, at your option.
