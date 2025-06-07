pub mod multi_file;
pub mod path_utils;
pub mod render_core;
pub mod render_details;
pub mod render_signatures;
pub mod rustdoc_json_types;

use render_core::{ResolvedItemInfo, render_item_page}; // This import is fine now
use rustdoc_json_types::*; // process_items is now render_item_list

pub use multi_file::generate as rustdoc_json_to_fs;

pub fn rustdoc_json_to_markdown(data: Crate) -> String {
    let mut output = String::new();

    // Add crate header and basic info
    output.push_str("# Crate Documentation\n\n");

    if let Some(version) = &data.crate_version {
        output.push_str(&format!("**Version:** {}\n\n", version));
    }

    output.push_str(&format!("**Format Version:** {}\n\n", data.format_version));

    // Process the root module to start
    let root_id = data.root;
    if let Some(root_item) = data.index.get(&root_id) {
        render_module_items_recursively(&mut output, root_item, &data, 1);
    }

    output
}

/// Recursively renders a module and its public contents for single-file output.
fn render_module_items_recursively(
    output: &mut String,
    module_item: &Item,
    data: &Crate,
    level: usize,
) {
    // The link resolver for single-file mode generates anchor links.
    let link_resolver = |target_id: &Id| -> String {
        let summary = data
            .paths
            .get(target_id)
            .expect("Link target must have a path");
        let target_item = data.index.get(target_id).unwrap();
        let anchor = path_utils::get_item_anchor(target_item, summary);
        let name = summary.path.last().unwrap();
        format!("[`{}`](#{})", name, anchor)
    };

    let module_info = ResolvedItemInfo {
        original_item: module_item,
        effective_name: module_item.name.clone(),
        reexport_source_canonical_path: None,
    };

    // Render the current module's page content (title, docs, item list).
    render_item_page(output, &module_info, data, level, link_resolver);

    // Now, recursively render the full content of any public submodules.
    if let ItemEnum::Module(module_details) = &module_item.inner {
        for &item_id in &module_details.items {
            if let Some(item) = data.index.get(&item_id) {
                if item.visibility == Visibility::Public
                    && matches!(item.inner, ItemEnum::Module(_))
                {
                    render_module_items_recursively(output, item, data, level + 1);
                }
            }
        }
    }
}
