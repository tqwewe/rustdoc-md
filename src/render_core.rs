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

pub fn get_item_kind_string(item_enum: &ItemEnum) -> &str {
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

pub fn process_items(output: &mut String, item_ids: &[Id], data: &Crate, level: usize) {
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
                                if let ItemEnum::Module(target_module_details) = &target_module_item.inner {
                                    for &glob_item_id in &target_module_details.items {
                                        if let Some(glob_item) = data.index.get(&glob_item_id) {
                                            if glob_item.visibility == Visibility::Public {
                                                let canonical_path = data.paths.get(&glob_item.id).map(|s| s.path.join("::")).unwrap_or_default();
                                                all_resolved_items.push(ResolvedItemInfo {
                                                    original_item: glob_item,
                                                    effective_name: glob_item.name.clone(),
                                                    reexport_source_canonical_path: Some(canonical_path),
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
                                let canonical_path = data.paths.get(&target_item.id).map(|s| s.path.join("::")).unwrap_or_default();
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
    let mut macros: Vec<&ResolvedItemInfo<'_>> = Vec::new();

    // The existing categorization logic is good.
    for resolved_info in &all_resolved_items {
        match &resolved_info.original_item.inner {
            ItemEnum::Module(_) => modules.push(resolved_info),
            ItemEnum::Struct(_) | ItemEnum::Enum(_) | ItemEnum::Union(_) | ItemEnum::TypeAlias(_) => types.push(resolved_info),
            ItemEnum::Trait(_) | ItemEnum::TraitAlias(_) => traits.push(resolved_info),
            ItemEnum::Function(_) => functions.push(resolved_info),
            ItemEnum::Constant { .. } | ItemEnum::Static(_) => constants.push(resolved_info),
            ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => macros.push(resolved_info),
            _ => {} // Ignore 'Use' and other item kinds not meant for direct documentation here.
        }
    }
    
    // --- PASS 3: Render in a Standardized Order ---
    if !modules.is_empty() {
        output.push_str(&format!("{} Modules\n\n", "#".repeat(heading_level)));
        for resolved_info in modules {
            process_item(output, resolved_info, data, level + 1);
        }
    }

    if !types.is_empty() {
        output.push_str(&format!("{} Types\n\n", "#".repeat(heading_level)));
        for resolved_info in types {
            process_item(output, resolved_info, data, level + 1);
        }
    }

    if !traits.is_empty() {
        output.push_str(&format!("{} Traits\n\n", "#".repeat(heading_level)));
        for resolved_info in traits {
            process_item(output, resolved_info, data, level + 1);
        }
    }

    if !functions.is_empty() {
        output.push_str(&format!("{} Functions\n\n", "#".repeat(heading_level)));
        for resolved_info in functions {
            process_item(output, resolved_info, data, level + 1);
        }
    }

    if !constants.is_empty() {
        output.push_str(&format!(
            "{} Constants and Statics\n\n",
            "#".repeat(heading_level)
        ));
        for resolved_info in constants {
            process_item(output, resolved_info, data, level + 1);
        }
    }

    if !macros.is_empty() {
        output.push_str(&format!("{} Macros\n\n", "#".repeat(heading_level)));
        for resolved_info in macros {
            process_item(output, resolved_info, data, level + 1);
        }
    }
}

pub fn process_item(output: &mut String, resolved_info: &ResolvedItemInfo, data: &Crate, level: usize) {
    let item = resolved_info.original_item;
    let heading_level = std::cmp::min(level, 6);
    let heading_prefix = "#".repeat(heading_level);

    let display_name_opt = resolved_info.effective_name.as_ref().or(item.name.as_ref());
    let item_kind_str = get_item_kind_string(&item.inner);

    // Standardized Heading Logic
    if let Some(display_name) = display_name_opt {
        if let Some(canonical_path) = &resolved_info.reexport_source_canonical_path {
            output.push_str(&format!(
                "{} Re-exported {} `{}` (from `{}`)\n\n",
                heading_prefix, item_kind_str, display_name, canonical_path
            ));
        } else {
            // Treat all items equally for headings
            output.push_str(&format!("{} {} `{}`\n\n", heading_prefix, item_kind_str, display_name));
        }
    } else {
        // Handle nameless items like impls
        match &item.inner {
            ItemEnum::Impl(impl_details) => {
                // *** START CHANGE (Blanket Impl Handling from fix_plan.md) ***
                // Check for blanket impls and render a collapsed summary instead of the full block
                if impl_details.blanket_impl.is_some() {
                    output.push_str("<details><summary>Blanket Implementations</summary>\n\n");
                    output.push_str("This type is implemented for the following traits through blanket implementations:\n\n");
                    if let Some(trait_) = &impl_details.trait_ {
                         output.push_str(&format!("- `{}`\n", trait_.path));
                    }
                    output.push_str("\n</details>\n\n");
                    return; // Stop processing this item further
                }
                // *** END CHANGE ***

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
        output.push_str(&render_docs_with_links(docs, &item.links, data));
        output.push_str("\n\n");
    }

    output.push_str("```rust\n");
    crate::render_signatures::format_item_signature(output, item, data); 
    output.push_str("\n```\n\n");

    match &item.inner {
        ItemEnum::Module(module) => crate::render_details::process_module_details(output, module, data, level + 1),
        ItemEnum::Struct(struct_) => crate::render_details::process_struct_details(output, struct_, data, level + 1),
        ItemEnum::Enum(enum_) => crate::render_details::process_enum_details(output, enum_, data, level + 1),
        ItemEnum::Union(union_) => crate::render_details::process_union_details(output, union_, data, level + 1),
        ItemEnum::Trait(trait_) => crate::render_details::process_trait_details(output, trait_, data, level + 1),
        ItemEnum::Impl(impl_) => crate::render_details::process_impl_details(output, impl_, data, level + 1),
        _ => {}
    }
}

// As per fix_plan.md Step 3 for Intra-Doc Link Resolution
pub fn render_docs_with_links(docs: &str, links: &HashMap<String, Id>, data: &Crate) -> String {
    let re = Regex::new(r"\[`([^`]+)`\]\[?([^\]]*)\]?").unwrap(); // Matches [`Thing`] and [`Thing`][label]

    let result = re.replace_all(docs, |caps: &regex::Captures| {
        let link_text = &caps[1];
        if let Some(target_id) = links.get(link_text) {
            if let Some(_summary) = data.paths.get(target_id) {
                // Placeholder from fix_plan.md: bold it to show it was resolved.
                format!("**`{}`**", link_text)
            } else {
                // Link target not found in paths, render as code
                format!("`{}`", link_text)
            }
        } else {
            // Not a rustdoc link, render as code
            format!("`{}`", link_text)
        }
    });

    result.into_owned()
}
