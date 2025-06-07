use crate::path_utils::get_item_fs_path; // get_item_file_name is used by get_item_fs_path
use crate::render_core::{ResolvedItemInfo, render_item_page};
use crate::rustdoc_json_types::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate(krate: &ParsedCrateDoc, output_dir: &Path) -> eyre::Result<()> {
    fs::create_dir_all(output_dir)?; // Ensure the root output directory exists
    let generator = Generator::new(krate, output_dir)?;
    generator.run()
}

struct Generator<'a> {
    krate: &'a ParsedCrateDoc,
    fs_paths: HashMap<Id, PathBuf>, // Stores the absolute path for each item's file
}

impl<'a> Generator<'a> {
    fn new(krate: &'a ParsedCrateDoc, output_dir_param: &'a Path) -> eyre::Result<Self> {
        let mut fs_paths = HashMap::new();
        // Pre-calculate filesystem paths for all items that have an ItemSummary.
        // These are typically items that can be linked to or have their own page.
        for id in krate.paths.keys() {
            // get_item_fs_path now relies only on krate.paths (via summary), not krate.index.
            let path = get_item_fs_path(krate, id, output_dir_param);
            fs_paths.insert(*id, path);
        }
        Ok(Self { krate, fs_paths })
    }

    fn run(&self) -> eyre::Result<()> {
        for id_from_paths in self.krate.paths.keys() {
            let summary = self
                .krate
                .paths
                .get(id_from_paths)
                .expect("ID from paths.keys() must exist in paths");

            // Filter 1: Only generate pages for items defined in the local crate.
            if summary.crate_id != 0 {
                continue;
            }

            // Attempt to get full item details.
            // A local item (crate_id == 0) with a path summary should have full details.
            let item = match self.krate.index.get(id_from_paths) {
                Some(i) => i,
                None => {
                    eprintln!(
                        "Warning: Local item ID {:?} (path: {:?}) has a path summary but no full item details in index. Skipping page generation.",
                        id_from_paths,
                        summary.path.join("::")
                    );
                    continue;
                }
            };

            // Filter 2: Visibility and specific item kinds that should not get their own pages.
            if item.visibility != Visibility::Public || matches!(item.inner, ItemEnum::Use(_)) {
                continue;
            }

            // Filter 3: Item kinds that get their own dedicated pages.
            // This should align with kinds handled by get_item_file_name.
            match item.inner {
                ItemEnum::Module(_)
                | ItemEnum::Struct(_)
                | ItemEnum::Enum(_)
                | ItemEnum::Union(_)
                | ItemEnum::Trait(_)
                | ItemEnum::TraitAlias(_) // Added TraitAlias
                | ItemEnum::Function(_)
                | ItemEnum::TypeAlias(_)
                | ItemEnum::Constant { .. } // Match Constant variant
                | ItemEnum::Static(_)     // Match Static variant
                | ItemEnum::Macro(_)
                | ItemEnum::ProcMacro(_) => { /* These get their own pages. */ }
                _ => continue, // Skip other items like struct fields, variants, impls, etc.
            }

            let item_file_path = self.fs_paths.get(id_from_paths).expect(
                "Path should have been pre-calculated in new() for an ID from paths.keys()",
            );
            if let Some(parent_dir) = item_file_path.parent() {
                fs::create_dir_all(parent_dir)?;
            }

            let mut content = String::new();

            // `item_file_path` is the absolute path to the current item's generated file.
            let link_resolver =
                |target_id: &Id| -> String { self.resolve_link(target_id, item_file_path) };

            let item_info = ResolvedItemInfo {
                original_item: item,
                effective_name: item.name.clone(),
                reexport_source_canonical_path: None, // This page is for the item itself, not a re-export summary
            };

            // Top-level items in multi-file mode start at heading level 1
            render_item_page(&mut content, &item_info, self.krate, 1, link_resolver);
            fs::write(item_file_path, content)?;
        }

        Ok(())
    }

    fn resolve_link(&self, target_id: &Id, from_file_path: &Path) -> String {
        let target_item_file_path = self
            .fs_paths
            .get(target_id)
            .expect(&format!("FS path not found for target ID {:?}", target_id));

        let from_dir = from_file_path
            .parent()
            .expect("Current item's file path should have a parent directory");

        let relative_path = pathdiff::diff_paths(target_item_file_path, from_dir)
            .unwrap_or_else(|| target_item_file_path.to_path_buf()); // Fallback to absolute

        let target_summary = self.krate.paths.get(target_id).unwrap();
        let default_name_owned = "unknown".to_string();
        let target_name = target_summary.path.last().unwrap_or(&default_name_owned);

        format!("[`{}`]({})", target_name, relative_path.to_string_lossy())
    }
}
