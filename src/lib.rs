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
