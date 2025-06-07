use crate::render_core::get_item_kind_string;
use crate::rustdoc_json_types::*;
use std::path::{Path, PathBuf};

// Helper to map ItemKind to a filename prefix.
fn item_kind_to_prefix(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Struct => "struct",
        ItemKind::Enum => "enum",
        ItemKind::Union => "union",
        ItemKind::Trait => "trait",
        ItemKind::TraitAlias => "trait_alias",
        ItemKind::Function => "fn",
        ItemKind::TypeAlias => "type",
        ItemKind::Constant => "const",
        ItemKind::Static => "static",
        ItemKind::Macro => "macro", // Covers ItemEnum::Macro and ProcMacro(Bang)
        ItemKind::ProcAttribute => "proc_attribute", // Covers ProcMacro(Attr)
        ItemKind::ProcDerive => "proc_derive", // Covers ProcMacro(Derive)
        // ItemKind::Module is special-cased to "index.md" in get_item_file_name.
        // Other kinds like StructField, Variant, Impl, etc., typically don't get their own top-level files.
        _ => "item", // Generic fallback for other kinds if they were to have files.
    }
}

/// Generates a filename for an item (e.g., `struct.MyStruct.md`) based on its ItemSummary.
pub fn get_item_file_name(summary: &ItemSummary) -> String {
    if summary.kind == ItemKind::Module {
        return "index.md".to_string(); // Modules are index files in their directory
    }

    let kind_prefix = item_kind_to_prefix(&summary.kind);
    // Use the last path segment from ItemSummary as the name.
    let name = summary
        .path
        .last()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    format!("{}.{}.md", kind_prefix, name)
}

/// Generates a stable anchor ID for an item, used in single-file mode.
pub fn get_item_anchor(item: &Item, summary: &ItemSummary) -> String {
    let kind_prefix = get_item_kind_string(&item.inner)
        .to_lowercase()
        .replace(' ', "-");
    // Use the full path for the anchor to ensure uniqueness, as names can collide across modules.
    let path_slug = summary.path.join("-").to_lowercase().replace("::", "-");
    format!("{}-{}", kind_prefix, path_slug)
}

/// Constructs the full filesystem path for an item in multi-file mode.
pub fn get_item_fs_path(krate: &ParsedCrateDoc, id: &Id, base_dir: &Path) -> PathBuf {
    let summary = krate
        .paths
        .get(id)
        .expect("ItemSummary not found for ID in get_item_fs_path");
    // No longer access krate.index here; rely only on summary.

    let mut fs_path = base_dir.to_path_buf();

    // `summary.path` is the full path from the crate root, e.g., ["my_crate", "module", "MyStruct"].
    // The file for "MyStruct" should be at "base_dir/my_crate/module/struct.MyStruct.md".
    // If the item is a module, its path will be like ["my_crate", "module"], and its file should be
    // "base_dir/my_crate/module/index.md".
    if summary.kind == ItemKind::Module {
        fs_path.extend(&summary.path);
    } else {
        // For non-module items, the last path segment is the item name (part of filename).
        // The directory path is all segments except the last.
        // Ensure path is not empty before slicing.
        if !summary.path.is_empty() {
            if summary.path.len() > 1 {
                fs_path.extend(&summary.path[..summary.path.len() - 1]);
            } else {
                // Path has only one segment (e.g. crate name for a root-level item not a module)
                // This case implies the item is directly under base_dir/crate_name_dir
                // However, our convention is that non-module items are within module directories.
                // If summary.path is ["my_crate_struct"], it should be base_dir/my_crate_struct_dir/
                // This needs to align with how crate root dir is handled.
                // For now, assume items are always in a module, even if it's the root module.
                // If path is ["MyItemName"], it means it's in the crate root.
                // The file would be base_dir/crate_name/type.MyItemName.md
                // The first segment of summary.path is the crate name.
                fs_path.push(
                    summary.path.first().expect(
                        "Summary path should not be empty if not a module and path len is 1",
                    ),
                );
            }
        }
        // If summary.path is empty, fs_path remains base_dir. This seems unlikely for valid items.
    }

    let filename = get_item_file_name(summary); // Uses the modified get_item_file_name
    fs_path.push(filename);
    fs_path
}
