pub mod rustdoc_json_types;
pub mod render_core;
pub mod render_details;
pub mod render_signatures;

use rustdoc_json_types::*;
use render_core::process_items;

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
        if let ItemEnum::Module(module) = &root_item.inner {
            if let Some(name) = &root_item.name {
                output.push_str(&format!("# Module `{}`\n\n", name));
            } else if module.is_crate {
                output.push_str("# Crate Root\n\n");
            }

            // Add root documentation if available
            if let Some(docs) = &root_item.docs {
                output.push_str(&format!("{}\n\n", docs));
            }

            // Process all items in the module with consistent heading levels
            // starting at level 2 for top-level categories
            process_items(&mut output, &module.items, &data, 2);
        }
    }

    output
}
