pub mod multi_file;
pub mod path_utils;
pub mod render_core;
pub mod render_details;
pub mod render_signatures;
pub mod rustdoc_json_types;

use rustdoc_json_types::*;
use std::{fs, io, path::Path};

impl ParsedCrateDoc {
    /// Loads a `ParsedCrateDoc` from a rustdoc JSON file.
    pub fn from_file(path: &Path) -> eyre::Result<Self> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        let krate: ParsedCrateDoc = serde_json::from_reader(reader)?;
        Ok(krate)
    }

    /// Generates a single Markdown string documenting the entire crate.
    ///
    /// This is suitable for direct output or further processing.
    pub fn to_string(&self) -> String {
        let mut output = String::new();

        // Add crate header and basic info
        output.push_str("# Crate Documentation\n\n");

        if let Some(version) = &self.crate_version {
            output.push_str(&format!("**Version:** {}\n\n", version));
        }

        output.push_str(&format!("**Format Version:** {}\n\n", self.format_version));

        // Process the root module to start
        let root_id = self.root;
        if let Some(root_item) = self.index.get(&root_id) {
            crate::render_core::render_module_items_recursively(&mut output, root_item, self, 1);
        }

        output
    }

    /// Generates a single Markdown file documenting the entire crate.
    ///
    /// This function creates parent directories for `output_file` if they don't exist.
    pub fn to_single_file(&self, output_file: &Path) -> eyre::Result<()> {
        if let Some(parent_dir) = output_file.parent() {
            fs::create_dir_all(parent_dir)?;
        }
        let md = self.to_string();
        fs::write(output_file, md)?;
        Ok(())
    }

    /// Generates a hierarchical directory structure of Markdown files.
    pub fn to_multi_file(&self, output_dir: &Path) -> eyre::Result<()> {
        multi_file::generate(self, output_dir)
    }
}

/// Processes all `.json` files in a given input directory, generating multi-file
/// Markdown documentation for each successfully parsed crate.
///
/// Each crate's documentation will be placed in a subdirectory named after the crate
/// within the specified `output_dir`.
///
/// # Arguments
///
/// * `input_dir`: Path to the directory containing rustdoc JSON files (e.g., `target/doc`).
/// * `output_dir`: Path to the directory where Markdown output will be saved (e.g., `target/doc_md`).
///
/// # Errors
///
/// Returns an error if the `output_dir` cannot be created or if there are issues reading
/// the `input_dir`. Individual file processing errors (parsing, generation) are reported
/// to `stderr` but do not stop the processing of other files.
pub fn generate_markdown_for_all_json_in_dir(
    input_dir: &Path,
    output_dir: &Path,
) -> eyre::Result<()> {
    fs::create_dir_all(output_dir)?;

    let entries = match fs::read_dir(input_dir) {
        Ok(entries) => entries,
        Err(e) => {
            return Err(eyre::eyre!(
                "Failed to read input directory '{}': {}",
                input_dir.display(),
                e
            ));
        }
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to access entry in '{}': {}",
                    input_dir.display(),
                    e
                );
                continue;
            }
        };

        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            eprintln!("Processing '{}'...", path.display());
            match ParsedCrateDoc::from_file(&path) {
                Ok(krate) => {
                    // Determine crate name for the subdirectory
                    let crate_name = krate
                        .paths
                        .get(&krate.root)
                        .and_then(|summary| summary.path.first())
                        .cloned()
                        .unwrap_or_else(|| {
                            path.file_stem()
                                .map(|s| s.to_string_lossy().into_owned())
                                .unwrap_or_else(|| "unknown_crate".to_string())
                        });

                    let crate_output_dir = output_dir.join(&crate_name);
                    eprintln!(
                        "Generating Markdown for crate '{}' into '{}'...",
                        crate_name,
                        crate_output_dir.display()
                    );

                    if let Err(e) = krate.to_multi_file(&crate_output_dir) {
                        eprintln!(
                            "Warning: Failed to generate Markdown for '{}': {}",
                            path.display(),
                            e
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse JSON file '{}': {}. Skipping.",
                        path.display(),
                        e
                    );
                }
            }
        }
    }
    Ok(())
}
