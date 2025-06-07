use crate::rustdoc_json_types::*;
use regex::Regex;
use std::collections::HashMap;

// Helper struct to carry context about how an item is being presented
#[derive(Clone, Debug)]
pub struct ResolvedItemInfo<'a> {
    pub original_item: &'a Item,
    pub effective_name: Option<String>, // Name in the current scope (alias or original)
    pub reexport_source_canonical_path: Option<String>, // Canonical path of the original item, if re-exported
}

pub fn get_item_kind_string(item_enum: &ItemEnum) -> &'static str {
    match item_enum {
        ItemEnum::Module(_) => "Module",
        ItemEnum::Struct(_) => "Struct",
        ItemEnum::Enum(_) => "Enum",
        ItemEnum::Union(_) => "Union",
        ItemEnum::Trait(_) => "Trait",
        ItemEnum::TraitAlias(_) => "Trait Alias",
        ItemEnum::Function(_) => "Function",
        ItemEnum::TypeAlias(_) => "Type Alias",
        ItemEnum::Constant { .. } => "Constant",
        ItemEnum::Static(_) => "Static",
        ItemEnum::Macro(_) => "Macro",
        ItemEnum::ProcMacro(_) => "Procedural Macro",
        ItemEnum::ExternCrate { .. } => "Extern Crate",
        ItemEnum::Use(_) => "Use Statement",
        ItemEnum::StructField(_) => "Struct Field",
        ItemEnum::Variant(_) => "Variant",
        ItemEnum::Impl(_) => "Implementation",
        ItemEnum::Primitive(_) => "Primitive",
        ItemEnum::ExternType => "Extern Type",
        ItemEnum::AssocConst { .. } => "Associated Constant",
        ItemEnum::AssocType { .. } => "Associated Type",
    }
}

pub fn render_item_list<F>(
    output: &mut String,
    item_ids: &[Id],
    data: &ParsedCrateDoc,
    level: usize,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    // Cap heading level at 6 (maximum valid Markdown heading level)
    let heading_level = std::cmp::min(level, 6);

    let mut all_resolved_items: Vec<ResolvedItemInfo<'_>> = Vec::new();

    // --- PASS 1: Collect and Resolve All Items (Direct and Re-exported) ---
    for &id in item_ids {
        if let Some(item) = data.index.get(&id) {
            if let ItemEnum::Use(use_item) = &item.inner {
                // If the 'use' statement is public, resolve its contents.
                if let Visibility::Public = item.visibility {
                    if use_item.is_glob {
                        if let Some(target_module_id) = use_item.id {
                            if let Some(target_module_item) = data.index.get(&target_module_id) {
                                if let ItemEnum::Module(target_module_details) =
                                    &target_module_item.inner
                                {
                                    for &glob_item_id in &target_module_details.items {
                                        if let Some(glob_item) = data.index.get(&glob_item_id) {
                                            if glob_item.visibility == Visibility::Public {
                                                let canonical_path = data
                                                    .paths
                                                    .get(&glob_item.id)
                                                    .map(|s| s.path.join("::"))
                                                    .unwrap_or_default();
                                                all_resolved_items.push(ResolvedItemInfo {
                                                    original_item: glob_item,
                                                    effective_name: glob_item.name.clone(),
                                                    reexport_source_canonical_path: Some(
                                                        canonical_path,
                                                    ),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Named re-export
                        if let Some(target_id) = use_item.id {
                            if let Some(target_item) = data.index.get(&target_id) {
                                let canonical_path = data
                                    .paths
                                    .get(&target_item.id)
                                    .map(|s| s.path.join("::"))
                                    .unwrap_or_default();
                                all_resolved_items.push(ResolvedItemInfo {
                                    original_item: target_item,
                                    effective_name: Some(use_item.name.clone()),
                                    reexport_source_canonical_path: Some(canonical_path),
                                });
                            }
                        }
                    }
                }
            } else {
                // Direct item definition
                all_resolved_items.push(ResolvedItemInfo {
                    original_item: item,
                    effective_name: item.name.clone(),
                    reexport_source_canonical_path: None,
                });
            }
        }
    }

    // --- PASS 2: Group and Render ---
    let mut modules: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut types: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut traits: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut functions: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut constants: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut macros_vec: Vec<&ResolvedItemInfo<'_>> = Vec::new(); // Renamed to avoid conflict

    // The existing categorization logic is good.
    for resolved_info in &all_resolved_items {
        match &resolved_info.original_item.inner {
            ItemEnum::Module(_) => modules.push(resolved_info),
            ItemEnum::Struct(_)
            | ItemEnum::Enum(_)
            | ItemEnum::Union(_)
            | ItemEnum::TypeAlias(_) => types.push(resolved_info),
            ItemEnum::Trait(_) | ItemEnum::TraitAlias(_) => traits.push(resolved_info),
            ItemEnum::Function(_) => functions.push(resolved_info),
            ItemEnum::Constant { .. } | ItemEnum::Static(_) => constants.push(resolved_info),
            ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => macros_vec.push(resolved_info), // Use renamed vec
            _ => {} // Ignore 'Use' and other item kinds not meant for direct documentation here.
        }
    }

    // --- PASS 3: Render in a Standardized Order ---
    if !modules.is_empty() {
        output.push_str(&format!("{} Modules\n\n", "#".repeat(heading_level)));
        for resolved_info in modules {
            if let Some(_name) = &resolved_info.effective_name {
                // Changed name to _name as it's not used directly
                let link = link_resolver(&resolved_info.original_item.id);
                // For modules, just show the link. Their docs are at the top of their page.
                output.push_str(&format!("- {}\n", link));
            }
        }
        output.push_str("\n");
    }

    // Helper to render a list of items with their one-line doc summary
    let mut render_list_with_docs = |title: &str, items_list: &Vec<&ResolvedItemInfo<'_>>| {
        if !items_list.is_empty() {
            output.push_str(&format!("{} {}\n\n", "#".repeat(heading_level), title));
            for resolved_info in items_list {
                if let Some(_name) = &resolved_info.effective_name {
                    // Changed name to _name
                    let link = link_resolver(&resolved_info.original_item.id);
                    let item_kind_str = get_item_kind_string(&resolved_info.original_item.inner);
                    output.push_str(&format!("- **{}**: {}", item_kind_str, link));
                    if let Some(docs) = &resolved_info.original_item.docs {
                        // Render one-line summary
                        let first_line = docs.lines().next().unwrap_or("").trim();
                        if !first_line.is_empty() {
                            output.push_str(&format!(
                                " - {}\n",
                                render_docs_with_links(
                                    first_line,
                                    &resolved_info.original_item.links,
                                    data,
                                    link_resolver
                                )
                            ));
                        } else {
                            output.push('\n');
                        }
                    } else {
                        output.push('\n');
                    }
                }
            }
            output.push_str("\n");
        }
    };

    render_list_with_docs(
        "Structs",
        &types
            .iter()
            .filter(|i| matches!(i.original_item.inner, ItemEnum::Struct(_)))
            .copied() // Dereference &&ResolvedItemInfo to &ResolvedItemInfo
            .collect(),
    );
    render_list_with_docs(
        "Enums",
        &types
            .iter()
            .filter(|i| matches!(i.original_item.inner, ItemEnum::Enum(_)))
            .copied() // Dereference &&ResolvedItemInfo to &ResolvedItemInfo
            .collect(),
    );
    render_list_with_docs(
        "Unions",
        &types
            .iter()
            .filter(|i| matches!(i.original_item.inner, ItemEnum::Union(_)))
            .copied() // Dereference &&ResolvedItemInfo to &ResolvedItemInfo
            .collect(),
    );
    render_list_with_docs(
        "Type Aliases",
        &types
            .iter()
            .filter(|i| matches!(i.original_item.inner, ItemEnum::TypeAlias(_)))
            .copied() // Dereference &&ResolvedItemInfo to &ResolvedItemInfo
            .collect(),
    );
    render_list_with_docs("Traits", &traits);
    render_list_with_docs("Functions", &functions);
    render_list_with_docs("Constants and Statics", &constants);
    render_list_with_docs("Macros", &macros_vec); // Use renamed vec
}

pub fn render_item_page<F>(
    output: &mut String,
    resolved_info: &ResolvedItemInfo,
    data: &ParsedCrateDoc,
    level: usize,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    let item = resolved_info.original_item;
    let heading_level = std::cmp::min(level, 6);
    let heading_prefix = "#".repeat(heading_level);

    let display_name_opt = resolved_info.effective_name.as_ref().or(item.name.as_ref());
    let item_kind_str = get_item_kind_string(&item.inner);

    // For single file mode, add an anchor
    // The link_resolver for single file mode will generate "#anchor_name"
    // The link_resolver for multi file mode will generate "path/to/file.md"
    // So, the anchor is only strictly needed for single file.
    // We can determine this by checking if the link_resolver output starts with #
    // However, the plan suggests adding it always and relying on the link_resolver.
    // Let's follow the plan's specific instruction for adding the anchor.
    if let Some(summary) = data.paths.get(&item.id) {
        output.push_str(&format!(
            "<a name=\"{}\"></a>\n",
            crate::path_utils::get_item_anchor(item, summary)
        ));
    }

    // Standardized Heading Logic
    if let Some(display_name) = display_name_opt {
        if let Some(canonical_path) = &resolved_info.reexport_source_canonical_path {
            output.push_str(&format!(
                "{} Re-exported {} `{}` (from `{}`)\n\n",
                heading_prefix, item_kind_str, display_name, canonical_path
            ));
        } else {
            // Treat all items equally for headings
            output.push_str(&format!(
                "{} {} `{}`\n\n",
                heading_prefix, item_kind_str, display_name
            ));
        }
    } else {
        // Handle nameless items like impls
        match &item.inner {
            ItemEnum::Impl(impl_details) => {
                // The special handling for blanket_impl.is_some() has been removed from here.
                // It's now handled by the calling functions in render_details.rs to consolidate them.

                if let Some(trait_) = &impl_details.trait_ {
                    output.push_str(&format!(
                        "{} Implementation of `{}` for `{}`\n\n",
                        heading_prefix,
                        trait_.path,
                        crate::render_signatures::format_type(&impl_details.for_, data)
                    ));
                } else {
                    output.push_str(&format!(
                        "{} Implementation for `{}`\n\n",
                        heading_prefix,
                        crate::render_signatures::format_type(&impl_details.for_, data)
                    ));
                }
            }
            _ => {
                output.push_str(&format!("{} {}\n\n", heading_prefix, item_kind_str));
            }
        }
    }

    if !item.attrs.is_empty() {
        output.push_str("**Attributes:**\n\n");
        for attr in &item.attrs {
            output.push_str(&format!("- `{}`\n", attr));
        }
        output.push('\n');
    }

    if let Some(deprecation) = &item.deprecation {
        output.push_str("**⚠️ Deprecated");
        if let Some(since) = &deprecation.since {
            output.push_str(&format!(" since {}", since));
        }
        output.push_str("**");

        if let Some(note) = &deprecation.note {
            output.push_str(&format!(": {}", note));
        }
        output.push_str("\n\n");
    }

    if let Some(docs) = &item.docs {
        output.push_str(&render_docs_with_links(
            docs,
            &item.links,
            data,
            link_resolver,
        ));
        output.push_str("\n\n");
    }

    output.push_str("```rust\n");
    crate::render_signatures::format_item_signature(output, item, data);
    output.push_str("\n```\n\n");

    match &item.inner {
        ItemEnum::Module(module) => crate::render_details::process_module_details(
            output,
            module,
            data,
            level + 1,
            link_resolver,
        ),
        ItemEnum::Struct(struct_) => crate::render_details::process_struct_details(
            output,
            struct_,
            data,
            level + 1,
            link_resolver,
        ),
        ItemEnum::Enum(enum_) => crate::render_details::process_enum_details(
            output,
            enum_,
            data,
            level + 1,
            link_resolver,
        ),
        ItemEnum::Union(union_) => crate::render_details::process_union_details(
            output,
            union_,
            data,
            level + 1,
            link_resolver,
        ),
        ItemEnum::Trait(trait_) => crate::render_details::process_trait_details(
            output,
            trait_,
            data,
            level + 1,
            link_resolver,
        ),
        ItemEnum::Impl(impl_) => crate::render_details::process_impl_details(
            output,
            impl_,
            data,
            level + 1,
            link_resolver,
        ),
        _ => {}
    }
}

// As per fix_plan.md Step 3 for Intra-Doc Link Resolution
pub fn render_docs_with_links<F>(
    docs: &str,
    links: &HashMap<String, Id>,
    _data: &ParsedCrateDoc,
    link_resolver: F,
) -> String
where
    F: Fn(&Id) -> String,
{
    let re = Regex::new(r"\[`([^`]+)`\]\[?([^\]]*)\]?").unwrap(); // Matches [`Thing`] and [`Thing`][label]

    let result = re.replace_all(docs, |caps: &regex::Captures| {
        let link_text = &caps[1];
        // The key for the `links` HashMap is the text that was resolved by rustdoc.
        // This is usually the text inside the brackets, but can be more complex.
        // The regex `\[`([^`]+)`\]` is a good first approximation for simple links.

        // For links like `[`MyType`]` or `[`my_function()`]`, the link text inside the backticks
        // is usually what's in the `links` map.
        if let Some(target_id) = links.get(link_text) {
            link_resolver(target_id)
        } else {
            // If not found, it might be a more complex link rustdoc didn't resolve for us,
            // or just regular text that happens to use this markdown pattern.
            // Render as-is to be safe.
            caps.get(0).unwrap().as_str().to_string()
        }
    });

    result.into_owned()
}

/// Recursively renders a module and its public contents for single-file output.
/// This function is primarily used by `Crate::to_string()` for generating
/// a single, self-contained Markdown document.
pub fn render_module_items_recursively(
    output: &mut String,
    module_item: &Item,
    krate: &ParsedCrateDoc,
    level: usize,
) {
    // The link resolver for single-file mode generates anchor links.
    let link_resolver = |target_id: &Id| -> String {
        let summary = krate
            .paths
            .get(target_id)
            .expect("Link target must have a path");
        let target_item = krate.index.get(target_id).unwrap();
        let anchor = crate::path_utils::get_item_anchor(target_item, summary);
        let name = summary.path.last().unwrap();
        format!("[`{}`](#{})", name, anchor)
    };

    let module_info = ResolvedItemInfo {
        original_item: module_item,
        effective_name: module_item.name.clone(),
        reexport_source_canonical_path: None,
    };

    // Render the current module's page content (title, docs, item list).
    render_item_page(output, &module_info, krate, level, link_resolver);

    // Now, recursively render the full content of any public submodules.
    if let ItemEnum::Module(module_details) = &module_item.inner {
        for &item_id in &module_details.items {
            if let Some(item) = krate.index.get(&item_id) {
                if item.visibility == Visibility::Public
                    && matches!(item.inner, ItemEnum::Module(_))
                {
                    render_module_items_recursively(output, item, krate, level + 1);
                }
            }
        }
    }
}
