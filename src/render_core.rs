use crate::rustdoc_json_types::*;

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
    let mut use_item_ids_for_reexport_section: Vec<Id> = Vec::new();

    for &id in item_ids {
        if let Some(item) = data.index.get(&id) {
            match &item.inner {
                ItemEnum::Use(use_item) => {
                    use_item_ids_for_reexport_section.push(id); // Always add Use item for "Re-exports" section

                    if use_item.is_glob {
                        // Handle glob re-exports
                        if let Some(target_module_id) = use_item.id {
                            if let Some(target_module_item) = data.index.get(&target_module_id) {
                                if let ItemEnum::Module(target_module_details) = &target_module_item.inner {
                                    for &glob_item_id in &target_module_details.items {
                                        if let Some(glob_item) = data.index.get(&glob_item_id) {
                                            // Only re-export public items from the target module
                                            if glob_item.visibility == Visibility::Public {
                                                let canonical_path = data.paths.get(&glob_item.id)
                                                    .map(|summary| summary.path.join("::"))
                                                    .unwrap_or_else(|| glob_item.name.clone().unwrap_or_default());

                                                all_resolved_items.push(ResolvedItemInfo {
                                                    original_item: glob_item,
                                                    effective_name: glob_item.name.clone(), // Use original name
                                                    reexport_source_canonical_path: Some(canonical_path),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Process named re-exports for inlining
                        if let Some(target_id) = use_item.id {
                            if let Some(target_item) = data.index.get(&target_id) {
                                // For Phase 1 & 2, let's focus on re-exporting most item kinds
                                // that can be reasonably inlined.
                                match &target_item.inner {
                                    ItemEnum::Struct(_) | ItemEnum::Enum(_) | ItemEnum::Union(_) |
                                    ItemEnum::Trait(_) | ItemEnum::Function(_) | ItemEnum::TypeAlias(_) |
                                    ItemEnum::Constant{..} | ItemEnum::Static(_) | ItemEnum::Macro(_) | ItemEnum::ProcMacro(_)=> {
                                        let canonical_path = data.paths.get(&target_item.id)
                                            .map(|summary| summary.path.join("::"))
                                            .unwrap_or_else(|| target_item.name.clone().unwrap_or_default());

                                        all_resolved_items.push(ResolvedItemInfo {
                                            original_item: target_item,
                                            effective_name: Some(use_item.name.clone()), // Alias name
                                            reexport_source_canonical_path: Some(canonical_path),
                                        });
                                    }
                                    _ => {
                                        // Other re-exported types (like Modules via `pub use other_mod;`)
                                        // will just be listed in the "Re-exports" section for now.
                                        // Or, if it's a module, it might be handled if it's directly in item_ids.
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Direct items
                    all_resolved_items.push(ResolvedItemInfo {
                        original_item: item,
                        effective_name: item.name.clone(), // Use its own name
                        reexport_source_canonical_path: None,
                    });
                }
            }
        }
    }

    // Group resolved items by kind for better organization
    let mut modules: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut types: Vec<&ResolvedItemInfo<'_>> = Vec::new(); // Structs, Enums, Unions, TypeAliases
    let mut traits: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut functions: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut constants: Vec<&ResolvedItemInfo<'_>> = Vec::new(); // Constants, Statics
    let mut macros: Vec<&ResolvedItemInfo<'_>> = Vec::new();
    let mut other_items: Vec<&ResolvedItemInfo<'_>> = Vec::new();


    for resolved_info in &all_resolved_items {
        match &resolved_info.original_item.inner {
            ItemEnum::Module(_) => modules.push(resolved_info),
            ItemEnum::Struct(_) | ItemEnum::Enum(_) | ItemEnum::Union(_) | ItemEnum::TypeAlias(_) => {
                types.push(resolved_info)
            }
            ItemEnum::Trait(_) | ItemEnum::TraitAlias(_) => traits.push(resolved_info),
            ItemEnum::Function(_) => functions.push(resolved_info),
            ItemEnum::Constant { .. } | ItemEnum::Static(_) => constants.push(resolved_info),
            ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => macros.push(resolved_info),
            ItemEnum::Use(_) => {} // These are handled by use_item_ids_for_reexport_section or resolved above
            _ => other_items.push(resolved_info),
        }
    }

    // Process each group in order
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

    // Render the "Re-exports" section for transparency, listing the use statements themselves
    if !use_item_ids_for_reexport_section.is_empty() {
        output.push_str(&format!("{} Re-exports\n\n", "#".repeat(heading_level)));
        for id in use_item_ids_for_reexport_section {
            if let Some(use_item_obj) = data.index.get(&id) {
                 format_use_statement_for_listing(output, use_item_obj, data, level + 1);
            }
        }
    }

    if !other_items.is_empty() {
        output.push_str(&format!("{} Other Items\n\n", "#".repeat(heading_level)));
        for resolved_info in other_items {
            process_item(output, resolved_info, data, level + 1);
        }
    }
}

// New function to format just the 'use' statement for the "Re-exports" section
pub fn format_use_statement_for_listing(output: &mut String, item: &Item, _data: &Crate, level: usize) {
    let heading_level = std::cmp::min(level, 6); 
    let heading = "#".repeat(heading_level);

    if let ItemEnum::Use(use_item) = &item.inner {
        let source_name_segment = use_item.source.split("::").last().unwrap_or(&use_item.source);

        // Heading for the use statement
        if use_item.is_glob {
            output.push_str(&format!("{} `use {}::*`\n\n", heading, use_item.source));
        } else {
            let display_name = item.name.as_ref().unwrap_or(&use_item.name);
            if display_name != source_name_segment && item.name.is_some() { 
                 output.push_str(&format!("{} `use {} as {}`\n\n", heading, use_item.source, display_name));
            } else { 
                 output.push_str(&format!("{} `use {}`\n\n", heading, use_item.source));
            }
        }

        if let Some(docs) = &item.docs {
            output.push_str(&format!("{}\n\n", docs));
        }

        output.push_str("```rust\n");
        let mut use_signature = String::new();
        match &item.visibility {
            Visibility::Public => use_signature.push_str("pub "),
            Visibility::Crate => use_signature.push_str("pub(crate) "),
            Visibility::Restricted { path, .. } => use_signature.push_str(&format!("pub(in {}) ", path)),
            Visibility::Default => {}
        }
        use_signature.push_str(&format!("use {}", use_item.source));
        if use_item.is_glob {
            use_signature.push_str("::*");
        } else if let Some(name_attr) = &item.name { 
            if name_attr != source_name_segment {
                 use_signature.push_str(&format!(" as {}", name_attr));
            }
        }
        use_signature.push(';');
        output.push_str(&use_signature);
        output.push_str("\n```\n\n");
    }
}


pub fn process_item(output: &mut String, resolved_info: &ResolvedItemInfo, data: &Crate, level: usize) {
    let item = resolved_info.original_item;
    let heading_level = std::cmp::min(level, 6);
    let heading_prefix = "#".repeat(heading_level);

    let display_name_opt = resolved_info.effective_name.as_ref().or(item.name.as_ref());
    let item_kind_str = get_item_kind_string(&item.inner);

    if let Some(display_name) = display_name_opt {
        if let Some(canonical_path) = &resolved_info.reexport_source_canonical_path {
            output.push_str(&format!(
                "{} Re-exported {} `{}` (from `{}`)\n\n",
                heading_prefix, item_kind_str, display_name, canonical_path
            ));
        } else {
            if let ItemEnum::Module(_) = &item.inner {
                 output.push_str(&format!("## Module `{}`\n\n", display_name));
            } else {
                 output.push_str(&format!("{} {} `{}`\n\n", heading_prefix, item_kind_str, display_name));
            }
        }
    } else {
        match &item.inner {
            ItemEnum::Impl(impl_details) => {
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
        output.push_str(&format!("{}\n\n", docs));
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
