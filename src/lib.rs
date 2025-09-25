use rustdoc_types::{
    Abi, AssocItemConstraintKind, Crate, Enum, GenericArg, GenericArgs, GenericBound,
    GenericParamDefKind, Generics, Id, Impl, Item, ItemEnum, MacroKind, Module,
    PreciseCapturingArg, Struct, StructKind, Term, Trait, TraitBoundModifier, Type, Union,
    VariantKind, Visibility, WherePredicate,
};

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

fn process_items(output: &mut String, item_ids: &[Id], data: &Crate, level: usize) {
    // Cap heading level at 6 (maximum valid Markdown heading level)
    let heading_level = std::cmp::min(level, 6);

    // Group items by kind for better organization
    let mut modules = Vec::new();
    let mut types = Vec::new();
    let mut traits = Vec::new();
    let mut functions = Vec::new();
    let mut constants = Vec::new();
    let mut macros = Vec::new();
    let mut reexports = Vec::new(); // New category for re-exports
    let mut other_items = Vec::new();

    for &id in item_ids {
        if let Some(item) = data.index.get(&id) {
            match &item.inner {
                ItemEnum::Module(_) => modules.push(id),
                ItemEnum::Struct(_)
                | ItemEnum::Enum(_)
                | ItemEnum::Union(_)
                | ItemEnum::TypeAlias(_) => types.push(id),
                ItemEnum::Trait(_) | ItemEnum::TraitAlias(_) => traits.push(id),
                ItemEnum::Function(_) => functions.push(id),
                ItemEnum::Constant { .. } | ItemEnum::Static(_) => constants.push(id),
                ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => macros.push(id),
                ItemEnum::Use(_) => reexports.push(id), // Categorize re-exports
                _ => other_items.push(id),
            }
        }
    }

    // Process each group in order
    if !modules.is_empty() {
        output.push_str(&format!("{} Modules\n\n", "#".repeat(heading_level)));
        for id in modules {
            process_item(output, data.index.get(&id).unwrap(), data, level + 1);
        }
    }

    if !types.is_empty() {
        output.push_str(&format!("{} Types\n\n", "#".repeat(heading_level)));
        for id in types {
            process_item(output, data.index.get(&id).unwrap(), data, level + 1);
        }
    }

    if !traits.is_empty() {
        output.push_str(&format!("{} Traits\n\n", "#".repeat(heading_level)));
        for id in traits {
            process_item(output, data.index.get(&id).unwrap(), data, level + 1);
        }
    }

    if !functions.is_empty() {
        output.push_str(&format!("{} Functions\n\n", "#".repeat(heading_level)));
        for id in functions {
            process_item(output, data.index.get(&id).unwrap(), data, level + 1);
        }
    }

    if !constants.is_empty() {
        output.push_str(&format!(
            "{} Constants and Statics\n\n",
            "#".repeat(heading_level)
        ));
        for id in constants {
            process_item(output, data.index.get(&id).unwrap(), data, level + 1);
        }
    }

    if !macros.is_empty() {
        output.push_str(&format!("{} Macros\n\n", "#".repeat(heading_level)));
        for id in macros {
            process_item(output, data.index.get(&id).unwrap(), data, level + 1);
        }
    }

    if !reexports.is_empty() {
        output.push_str(&format!("{} Re-exports\n\n", "#".repeat(heading_level)));
        for id in reexports {
            process_item(output, data.index.get(&id).unwrap(), data, level + 1);
        }
    }

    if !other_items.is_empty() {
        output.push_str(&format!("{} Other Items\n\n", "#".repeat(heading_level)));
        for id in other_items {
            process_item(output, data.index.get(&id).unwrap(), data, level + 1);
        }
    }
}

fn process_item(output: &mut String, item: &Item, data: &Crate, level: usize) {
    // Cap heading level at 6 (maximum valid Markdown heading level)
    let heading_level = std::cmp::min(level, 6);
    let heading = "#".repeat(heading_level);

    // Add item heading with name and kind
    match &item.inner {
        // Check for re-exports first, regardless of whether they have a name
        ItemEnum::Use(use_item) => {
            // Extract the meaningful part of the source path
            let source_name = use_item
                .source
                .split("::")
                .last()
                .unwrap_or(&use_item.source);

            // Format the heading based on the type of re-export
            if use_item.is_glob {
                output.push_str(&format!(
                    "{} Re-export `{}::*`\n\n",
                    heading, use_item.source
                ));
            } else if let Some(name) = &item.name {
                if name != source_name {
                    output.push_str(&format!(
                        "{} Re-export `{}` as `{}`\n\n",
                        heading, source_name, name
                    ));
                } else {
                    output.push_str(&format!("{} Re-export `{}`\n\n", heading, name));
                }
            } else {
                output.push_str(&format!("{} Re-export `{}`\n\n", heading, source_name));
            }
        }
        _ => {
            // Handle all other items as before
            if let Some(name) = &item.name {
                match &item.inner {
                    // For modules, always use a consistent level (level 2) to ensure they stand out
                    ItemEnum::Module(_) => output.push_str(&format!("## Module `{}`\n\n", name)),
                    ItemEnum::Struct(_) => {
                        output.push_str(&format!("{} Struct `{}`\n\n", heading, name))
                    }
                    ItemEnum::Enum(_) => {
                        output.push_str(&format!("{} Enum `{}`\n\n", heading, name))
                    }
                    ItemEnum::Union(_) => {
                        output.push_str(&format!("{} Union `{}`\n\n", heading, name))
                    }
                    ItemEnum::Trait(_) => {
                        output.push_str(&format!("{} Trait `{}`\n\n", heading, name))
                    }
                    ItemEnum::TraitAlias(_) => {
                        output.push_str(&format!("{} Trait Alias `{}`\n\n", heading, name))
                    }
                    ItemEnum::Function(_) => {
                        output.push_str(&format!("{} Function `{}`\n\n", heading, name))
                    }
                    ItemEnum::TypeAlias(_) => {
                        output.push_str(&format!("{} Type Alias `{}`\n\n", heading, name))
                    }
                    ItemEnum::Constant { .. } => {
                        output.push_str(&format!("{} Constant `{}`\n\n", heading, name))
                    }
                    ItemEnum::Static(_) => {
                        output.push_str(&format!("{} Static `{}`\n\n", heading, name))
                    }
                    ItemEnum::Macro(_) => {
                        output.push_str(&format!("{} Macro `{}`\n\n", heading, name))
                    }
                    ItemEnum::ProcMacro(_) => {
                        output.push_str(&format!("{} Procedural Macro `{}`\n\n", heading, name))
                    }
                    ItemEnum::ExternCrate {
                        name: crate_name, ..
                    } => output.push_str(&format!("{} Extern Crate `{}`\n\n", heading, crate_name)),
                    _ => output.push_str(&format!("{} `{}`\n\n", heading, name)),
                }
            } else {
                // Special case for impl blocks and other nameless items
                match &item.inner {
                    ItemEnum::Impl(impl_) => {
                        if let Some(trait_) = &impl_.trait_ {
                            // For trait impls, show "Implementation of TraitName for Type"
                            output.push_str(&format!(
                                "{} Implementation of `{}` for `{}`\n\n",
                                heading,
                                trait_.path,
                                format_type(&impl_.for_, data)
                            ));
                        } else {
                            // For inherent impls, show "Implementation for Type"
                            output.push_str(&format!(
                                "{} Implementation for `{}`\n\n",
                                heading,
                                format_type(&impl_.for_, data)
                            ));
                        }
                    }
                    _ => {
                        // For other items without names
                        output.push_str(&format!("{} Unnamed Item\n\n", heading));
                    }
                }
            }
        }
    }

    // Add item attributes if present
    if !item.attrs.is_empty() {
        output.push_str("**Attributes:**\n\n");
        for attr in &item.attrs {
            output.push_str(&format!("- `{:?}`\n", attr));
        }
        output.push('\n');
    }

    // Add deprecation info if present
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

    // Add documentation if available
    if let Some(docs) = &item.docs {
        output.push_str(&format!("{}\n\n", docs));
    }

    // Add code block with item signature
    output.push_str("```rust\n");
    format_item_signature(output, item, data);
    output.push_str("\n```\n\n");

    // Process additional details based on item kind
    match &item.inner {
        ItemEnum::Module(module) => process_module_details(output, module, data, level + 1),
        ItemEnum::Struct(struct_) => process_struct_details(output, struct_, data, level + 1),
        ItemEnum::Enum(enum_) => process_enum_details(output, enum_, data, level + 1),
        ItemEnum::Union(union_) => process_union_details(output, union_, data, level + 1),
        ItemEnum::Trait(trait_) => process_trait_details(output, trait_, data, level + 1),
        ItemEnum::Impl(impl_) => process_impl_details(output, impl_, data, level + 1),
        _ => {}
    }
}

fn format_item_signature(output: &mut String, item: &Item, data: &Crate) {
    // Format visibility
    match &item.visibility {
        Visibility::Public => output.push_str("pub "),
        Visibility::Crate => output.push_str("pub(crate) "),
        Visibility::Restricted { path, .. } => output.push_str(&format!("pub(in {}) ", path)),
        Visibility::Default => {}
    }

    // Format item based on its kind
    match &item.inner {
        ItemEnum::Module(_) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("mod {} {{ /* ... */ }}", name));
            }
        }
        ItemEnum::Struct(struct_) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("struct {}", name));
                format_generics(output, &struct_.generics, data);

                match &struct_.kind {
                    StructKind::Unit => output.push(';'),
                    StructKind::Tuple(fields) => {
                        output.push('(');
                        for (i, field_opt) in fields.iter().enumerate() {
                            if let Some(field_id) = field_opt {
                                if let Some(field_item) = data.index.get(field_id) {
                                    if let ItemEnum::StructField(field_type) = &field_item.inner {
                                        // Field visibility if needed
                                        match &field_item.visibility {
                                            Visibility::Public => output.push_str("pub "),
                                            Visibility::Crate => output.push_str("pub(crate) "),
                                            Visibility::Restricted { path, .. } => {
                                                output.push_str(&format!("pub(in {}) ", path))
                                            }
                                            Visibility::Default => {}
                                        }
                                        output.push_str(&format_type(field_type, data));
                                    }
                                }
                                if i < fields.len() - 1 {
                                    output.push_str(", ");
                                }
                            } else {
                                // For stripped fields
                                output.push_str("/* private field */");
                                if i < fields.len() - 1 {
                                    output.push_str(", ");
                                }
                            }
                        }
                        output.push_str(");");
                    }
                    StructKind::Plain {
                        fields,
                        has_stripped_fields,
                    } => {
                        output.push_str(" {\n");
                        for &field_id in fields {
                            if let Some(field_item) = data.index.get(&field_id) {
                                if let Some(field_name) = &field_item.name {
                                    if let ItemEnum::StructField(field_type) = &field_item.inner {
                                        // Field visibility
                                        match &field_item.visibility {
                                            Visibility::Public => output.push_str("    pub "),
                                            Visibility::Crate => output.push_str("    pub(crate) "),
                                            Visibility::Restricted { path, .. } => {
                                                output.push_str(&format!("    pub(in {}) ", path))
                                            }
                                            Visibility::Default => output.push_str("    "),
                                        }
                                        output.push_str(&format!(
                                            "{}: {},\n",
                                            field_name,
                                            format_type(field_type, data)
                                        ));
                                    }
                                }
                            }
                        }
                        if *has_stripped_fields {
                            output.push_str("    // Some fields omitted\n");
                        }
                        output.push('}');
                    }
                }
            }
        }
        ItemEnum::Enum(enum_) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("enum {}", name));
                format_generics(output, &enum_.generics, data);
                output.push_str(" {\n");

                for &variant_id in &enum_.variants {
                    if let Some(variant_item) = data.index.get(&variant_id) {
                        if let Some(variant_name) = &variant_item.name {
                            output.push_str(&format!("    {}", variant_name));

                            if let ItemEnum::Variant(variant) = &variant_item.inner {
                                match &variant.kind {
                                    VariantKind::Plain => {}
                                    VariantKind::Tuple(fields) => {
                                        output.push('(');
                                        for (i, field_opt) in fields.iter().enumerate() {
                                            if let Some(field_id) = field_opt {
                                                if let Some(field_item) = data.index.get(field_id) {
                                                    if let ItemEnum::StructField(field_type) =
                                                        &field_item.inner
                                                    {
                                                        output.push_str(&format_type(
                                                            field_type, data,
                                                        ));
                                                    }
                                                }
                                                if i < fields.len() - 1 {
                                                    output.push_str(", ");
                                                }
                                            } else {
                                                // For stripped fields
                                                output.push_str("/* private field */");
                                                if i < fields.len() - 1 {
                                                    output.push_str(", ");
                                                }
                                            }
                                        }
                                        output.push(')');
                                    }
                                    VariantKind::Struct {
                                        fields,
                                        has_stripped_fields,
                                    } => {
                                        output.push_str(" {\n");
                                        for &field_id in fields {
                                            if let Some(field_item) = data.index.get(&field_id) {
                                                if let Some(field_name) = &field_item.name {
                                                    if let ItemEnum::StructField(field_type) =
                                                        &field_item.inner
                                                    {
                                                        output.push_str(&format!(
                                                            "        {}: {},\n",
                                                            field_name,
                                                            format_type(field_type, data)
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                        if *has_stripped_fields {
                                            output.push_str("        // Some fields omitted\n");
                                        }
                                        output.push_str("    }");
                                    }
                                }

                                if let Some(discriminant) = &variant.discriminant {
                                    output.push_str(&format!(" = {}", discriminant.expr));
                                }
                            }

                            output.push_str(",\n");
                        }
                    }
                }

                if enum_.has_stripped_variants {
                    output.push_str("    // Some variants omitted\n");
                }

                output.push('}');
            }
        }
        ItemEnum::Union(union_) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("union {}", name));
                format_generics(output, &union_.generics, data);
                output.push_str(" {\n");

                for &field_id in &union_.fields {
                    if let Some(field_item) = data.index.get(&field_id) {
                        if let Some(field_name) = &field_item.name {
                            if let ItemEnum::StructField(field_type) = &field_item.inner {
                                match &field_item.visibility {
                                    Visibility::Public => output.push_str("    pub "),
                                    Visibility::Crate => output.push_str("    pub(crate) "),
                                    Visibility::Restricted { path, .. } => {
                                        output.push_str(&format!("    pub(in {}) ", path))
                                    }
                                    Visibility::Default => output.push_str("    "),
                                }
                                output.push_str(&format!(
                                    "{}: {},\n",
                                    field_name,
                                    format_type(field_type, data)
                                ));
                            }
                        }
                    }
                }

                if union_.has_stripped_fields {
                    output.push_str("    // Some fields omitted\n");
                }

                output.push('}');
            }
        }
        ItemEnum::Function(function) => {
            // Function header
            if function.header.is_const {
                output.push_str("const ");
            }
            if function.header.is_unsafe {
                output.push_str("unsafe ");
            }
            if function.header.is_async {
                output.push_str("async ");
            }

            // ABI
            match &function.header.abi {
                Abi::Rust => {}
                Abi::C { unwind } => {
                    if *unwind {
                        output.push_str("extern \"C-unwind\" ");
                    } else {
                        output.push_str("extern \"C\" ");
                    }
                }
                Abi::Cdecl { unwind } => {
                    if *unwind {
                        output.push_str("extern \"cdecl-unwind\" ");
                    } else {
                        output.push_str("extern \"cdecl\" ");
                    }
                }
                Abi::Stdcall { unwind } => {
                    if *unwind {
                        output.push_str("extern \"stdcall-unwind\" ");
                    } else {
                        output.push_str("extern \"stdcall\" ");
                    }
                }
                Abi::Fastcall { unwind } => {
                    if *unwind {
                        output.push_str("extern \"fastcall-unwind\" ");
                    } else {
                        output.push_str("extern \"fastcall\" ");
                    }
                }
                Abi::Aapcs { unwind } => {
                    if *unwind {
                        output.push_str("extern \"aapcs-unwind\" ");
                    } else {
                        output.push_str("extern \"aapcs\" ");
                    }
                }
                Abi::Win64 { unwind } => {
                    if *unwind {
                        output.push_str("extern \"win64-unwind\" ");
                    } else {
                        output.push_str("extern \"win64\" ");
                    }
                }
                Abi::SysV64 { unwind } => {
                    if *unwind {
                        output.push_str("extern \"sysv64-unwind\" ");
                    } else {
                        output.push_str("extern \"sysv64\" ");
                    }
                }
                Abi::System { unwind } => {
                    if *unwind {
                        output.push_str("extern \"system-unwind\" ");
                    } else {
                        output.push_str("extern \"system\" ");
                    }
                }
                Abi::Other(abi) => {
                    output.push_str(&format!("extern \"{}\" ", abi));
                }
            }

            // Function name
            if let Some(name) = &item.name {
                output.push_str(&format!("fn {}", name));

                // Generic parameters
                format_generics(output, &function.generics, data);

                // Parameters
                output.push('(');
                for (i, (param_name, param_type)) in function.sig.inputs.iter().enumerate() {
                    output.push_str(&format!(
                        "{}: {}",
                        param_name,
                        format_type(param_type, data)
                    ));
                    if i < function.sig.inputs.len() - 1 || function.sig.is_c_variadic {
                        output.push_str(", ");
                    }
                }

                // Variadic
                if function.sig.is_c_variadic {
                    output.push_str("...");
                }

                output.push(')');

                // Return type
                if let Some(return_type) = &function.sig.output {
                    output.push_str(&format!(" -> {}", format_type(return_type, data)));
                }

                // Where clause
                format_where_clause(output, &function.generics.where_predicates, data);

                // Function body indication
                if function.has_body {
                    output.push_str(" { /* ... */ }");
                } else {
                    output.push(';');
                }
            }
        }
        ItemEnum::Trait(trait_) => {
            // Trait modifiers
            if trait_.is_auto {
                output.push_str("auto ");
            }
            if trait_.is_unsafe {
                output.push_str("unsafe ");
            }

            // Trait definition
            if let Some(name) = &item.name {
                output.push_str(&format!("trait {}", name));
                format_generics(output, &trait_.generics, data);

                // Trait bounds
                if !trait_.bounds.is_empty() {
                    output.push_str(": ");
                    format_bounds(output, &trait_.bounds, data);
                }

                // Where clause
                format_where_clause(output, &trait_.generics.where_predicates, data);

                output.push_str(" {\n    /* Associated items */\n}");
            }
        }
        ItemEnum::TraitAlias(trait_alias) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("trait {}", name));
                format_generics(output, &trait_alias.generics, data);
                output.push_str(" = ");
                format_bounds(output, &trait_alias.params, data);
                format_where_clause(output, &trait_alias.generics.where_predicates, data);
                output.push(';');
            }
        }
        ItemEnum::Impl(impl_) => {
            // Impl modifiers
            if impl_.is_unsafe {
                output.push_str("unsafe ");
            }

            output.push_str("impl");

            // Generics
            format_generics(output, &impl_.generics, data);

            // Trait reference if this is a trait impl
            if let Some(trait_) = &impl_.trait_ {
                if impl_.is_negative {
                    output.push_str(" !");
                } else {
                    output.push(' ');
                }

                output.push_str(&trait_.path);
                if let Some(args) = &trait_.args {
                    let mut args_str = String::new();
                    format_generic_args(&mut args_str, args, data);
                    output.push_str(&args_str);
                }

                output.push_str(" for ");
            }

            // For type
            output.push_str(&format_type(&impl_.for_, data));

            // Where clause
            format_where_clause(output, &impl_.generics.where_predicates, data);

            output.push_str(" {\n    /* Associated items */\n}");

            // Add note if this is a compiler-generated impl
            if impl_.is_synthetic {
                output.push_str("\n// Note: This impl is compiler-generated");
            }
        }
        ItemEnum::TypeAlias(type_alias) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("type {}", name));
                format_generics(output, &type_alias.generics, data);
                format_where_clause(output, &type_alias.generics.where_predicates, data);
                output.push_str(&format!(" = {};", format_type(&type_alias.type_, data)));
            }
        }
        ItemEnum::Constant { type_, const_ } => {
            if let Some(name) = &item.name {
                output.push_str(&format!(
                    "const {}: {} = {};",
                    name,
                    format_type(type_, data),
                    const_.expr
                ));
            }
        }
        ItemEnum::Static(static_) => {
            if let Some(name) = &item.name {
                output.push_str("static ");
                if static_.is_mutable {
                    output.push_str("mut ");
                }
                if static_.is_unsafe {
                    output.push_str("/* unsafe */ ");
                }
                output.push_str(&format!(
                    "{}: {} = {};",
                    name,
                    format_type(&static_.type_, data),
                    static_.expr
                ));
            }
        }
        ItemEnum::Macro(macro_body) => {
            if let Some(name) = &item.name {
                output.push_str(&format!(
                    "macro_rules! {} {{\n    /* {} */\n}}",
                    name, macro_body
                ));
            }
        }
        ItemEnum::ProcMacro(proc_macro) => {
            if let Some(name) = &item.name {
                output.push_str("#[proc_macro");
                match proc_macro.kind {
                    MacroKind::Bang => output.push(']'),

                    MacroKind::Attr => output.push_str("_attribute]"),
                    MacroKind::Derive => {
                        output.push_str("_derive]");
                        if !proc_macro.helpers.is_empty() {
                            output.push_str("\n// Helpers: ");
                            for (i, helper) in proc_macro.helpers.iter().enumerate() {
                                output.push_str(&format!("#[{}]", helper));
                                if i < proc_macro.helpers.len() - 1 {
                                    output.push_str(", ");
                                }
                            }
                        }
                    }
                }
                output.push_str(&format!(
                    "\npub fn {}(/* ... */) -> /* ... */ {{\n    /* ... */\n}}",
                    name
                ));
            }
        }
        ItemEnum::ExternCrate { name, rename } => {
            output.push_str(&format!("extern crate {}", name));
            if let Some(rename_val) = rename {
                output.push_str(&format!(" as {}", rename_val));
            }
            output.push(';');
        }
        ItemEnum::Use(use_item) => {
            output.push_str(&format!("use {}", use_item.source));
            if use_item.is_glob {
                output.push_str("::*");
            } else if use_item.name
                != use_item
                    .source
                    .split("::")
                    .last()
                    .unwrap_or(&use_item.source)
            {
                output.push_str(&format!(" as {}", use_item.name));
            }
            output.push(';');
        }
        ItemEnum::StructField(field_type) => {
            // For struct fields, just output the type
            if let Some(name) = &item.name {
                match &item.visibility {
                    Visibility::Public => output.push_str("pub "),
                    Visibility::Crate => output.push_str("pub(crate) "),
                    Visibility::Restricted { path, .. } => {
                        output.push_str(&format!("pub(in {}) ", path))
                    }
                    Visibility::Default => {}
                }
                output.push_str(&format!("{}: {}", name, format_type(field_type, data)));
            } else {
                output.push_str(&format_type(field_type, data));
            }
        }
        ItemEnum::Variant(variant) => {
            // For enum variants
            if let Some(name) = &item.name {
                output.push_str(name);

                match &variant.kind {
                    VariantKind::Plain => {}
                    VariantKind::Tuple(fields) => {
                        output.push('(');
                        for (i, field_opt) in fields.iter().enumerate() {
                            if let Some(field_id) = field_opt {
                                if let Some(field_item) = data.index.get(field_id) {
                                    if let ItemEnum::StructField(field_type) = &field_item.inner {
                                        output.push_str(&format_type(field_type, data));
                                    }
                                }
                                if i < fields.len() - 1 {
                                    output.push_str(", ");
                                }
                            } else {
                                // For stripped fields
                                output.push_str("/* private field */");
                                if i < fields.len() - 1 {
                                    output.push_str(", ");
                                }
                            }
                        }
                        output.push(')');
                    }
                    VariantKind::Struct {
                        fields,
                        has_stripped_fields,
                    } => {
                        output.push_str(" {\n");
                        for &field_id in fields {
                            if let Some(field_item) = data.index.get(&field_id) {
                                if let Some(field_name) = &field_item.name {
                                    if let ItemEnum::StructField(field_type) = &field_item.inner {
                                        output.push_str(&format!(
                                            "    {}: {},\n",
                                            field_name,
                                            format_type(field_type, data)
                                        ));
                                    }
                                }
                            }
                        }
                        if *has_stripped_fields {
                            output.push_str("    // Some fields omitted\n");
                        }
                        output.push('}');
                    }
                }

                if let Some(discriminant) = &variant.discriminant {
                    output.push_str(&format!(" = {}", discriminant.expr));
                }
            }
        }
        ItemEnum::Primitive(primitive) => {
            output.push_str(&format!("// Primitive type: {}", primitive.name));
        }
        ItemEnum::ExternType => {
            if let Some(name) = &item.name {
                output.push_str(&format!("extern {{ type {}; }}", name));
            }
        }
        ItemEnum::AssocConst { type_, value } => {
            if let Some(name) = &item.name {
                output.push_str(&format!("const {}: {}", name, format_type(type_, data)));
                if let Some(val) = value {
                    output.push_str(&format!(" = {}", val));
                }
                output.push(';');
            }
        }
        ItemEnum::AssocType {
            generics,
            bounds,
            type_,
        } => {
            if let Some(name) = &item.name {
                output.push_str(&format!("type {}", name));
                format_generics(output, generics, data);

                if !bounds.is_empty() {
                    output.push_str(": ");
                    format_bounds(output, bounds, data);
                }

                if let Some(ty) = type_ {
                    output.push_str(&format!(" = {}", format_type(ty, data)));
                }

                format_where_clause(output, &generics.where_predicates, data);
                output.push(';');
            }
        }
    }
}

fn format_generics(output: &mut String, generics: &Generics, data: &Crate) {
    if generics.params.is_empty() {
        return;
    }

    output.push('<');
    for (i, param) in generics.params.iter().enumerate() {
        match &param.kind {
            GenericParamDefKind::Lifetime { outlives } => {
                output.push_str(&format!("'{}", param.name));
                if !outlives.is_empty() {
                    output.push_str(": ");
                    for (j, lifetime) in outlives.iter().enumerate() {
                        output.push_str(&format!("'{}", lifetime));
                        if j < outlives.len() - 1 {
                            output.push_str(" + ");
                        }
                    }
                }
            }
            GenericParamDefKind::Type {
                bounds,
                default,
                is_synthetic,
            } => {
                // If synthetic, add a note
                if *is_synthetic {
                    output.push_str("/* synthetic */ ");
                }

                output.push_str(&param.name);
                if !bounds.is_empty() {
                    output.push_str(": ");
                    format_bounds(output, bounds, data);
                }
                if let Some(default_type) = default {
                    output.push_str(&format!(" = {}", format_type(default_type, data)));
                }
            }
            GenericParamDefKind::Const { type_, default } => {
                output.push_str(&format!(
                    "const {}: {}",
                    param.name,
                    format_type(type_, data)
                ));
                if let Some(default_value) = default {
                    output.push_str(&format!(" = {}", default_value));
                }
            }
        }

        if i < generics.params.len() - 1 {
            output.push_str(", ");
        }
    }
    output.push('>');
}

fn format_where_clause(output: &mut String, predicates: &[WherePredicate], data: &Crate) {
    if predicates.is_empty() {
        return;
    }

    output.push_str("\nwhere\n    ");
    for (i, predicate) in predicates.iter().enumerate() {
        match predicate {
            WherePredicate::BoundPredicate {
                type_,
                bounds,
                generic_params,
            } => {
                if !generic_params.is_empty() {
                    output.push_str("for<");
                    for (j, param) in generic_params.iter().enumerate() {
                        match &param.kind {
                            GenericParamDefKind::Lifetime { .. } => {
                                output.push_str(&format!("'{}", param.name));
                            }
                            _ => output.push_str(&param.name),
                        }

                        if j < generic_params.len() - 1 {
                            output.push_str(", ");
                        }
                    }
                    output.push_str("> ");
                }

                output.push_str(&format_type(type_, data));

                if !bounds.is_empty() {
                    output.push_str(": ");
                    format_bounds(output, bounds, data);
                }
            }
            WherePredicate::LifetimePredicate { lifetime, outlives } => {
                output.push_str(&format!("'{}", lifetime));
                if !outlives.is_empty() {
                    output.push_str(": ");
                    for (j, outlive) in outlives.iter().enumerate() {
                        output.push_str(&format!("'{}", outlive));
                        if j < outlives.len() - 1 {
                            output.push_str(" + ");
                        }
                    }
                }
            }
            WherePredicate::EqPredicate { lhs, rhs } => {
                output.push_str(&format_type(lhs, data));
                output.push_str(" = ");
                match rhs {
                    Term::Type(type_) => output.push_str(&format_type(type_, data)),
                    Term::Constant(constant) => output.push_str(&constant.expr),
                }
            }
        }

        if i < predicates.len() - 1 {
            output.push_str(",\n    ");
        }
    }
}

fn format_bounds(output: &mut String, bounds: &[GenericBound], data: &Crate) {
    for (i, bound) in bounds.iter().enumerate() {
        match bound {
            GenericBound::TraitBound {
                trait_,
                generic_params,
                modifier,
            } => {
                match modifier {
                    TraitBoundModifier::None => {}
                    TraitBoundModifier::Maybe => output.push('?'),
                    TraitBoundModifier::MaybeConst => output.push_str("~const "),
                }

                if !generic_params.is_empty() {
                    output.push_str("for<");
                    for (j, param) in generic_params.iter().enumerate() {
                        match &param.kind {
                            GenericParamDefKind::Lifetime { .. } => {
                                output.push_str(&format!("'{}", param.name));
                            }
                            _ => output.push_str(&param.name),
                        }

                        if j < generic_params.len() - 1 {
                            output.push_str(", ");
                        }
                    }
                    output.push_str("> ");
                }

                output.push_str(&trait_.path);
                if let Some(args) = &trait_.args {
                    let mut args_str = String::new();
                    format_generic_args(&mut args_str, args, data);
                    output.push_str(&args_str);
                }
            }
            GenericBound::Outlives(lifetime) => {
                output.push_str(&format!("'{}", lifetime));
            }
            GenericBound::Use(args) => {
                output.push_str("use<");
                for (i, arg) in args.iter().enumerate() {
                    match arg {
                        PreciseCapturingArg::Lifetime(lifetime) => {
                            output.push_str(&format!("'{}", lifetime))
                        }
                        PreciseCapturingArg::Param(param) => output.push_str(param),
                    }

                    if i < args.len() - 1 {
                        output.push_str(", ");
                    }
                }
                output.push('>');
            }
        }

        if i < bounds.len() - 1 {
            output.push_str(" + ");
        }
    }
}

fn format_generic_args(output: &mut String, args: &GenericArgs, data: &Crate) {
    match args {
        GenericArgs::AngleBracketed { args, constraints } => {
            if args.is_empty() && constraints.is_empty() {
                return;
            }

            output.push('<');

            // Format args
            for (i, arg) in args.iter().enumerate() {
                match arg {
                    GenericArg::Lifetime(lifetime) => output.push_str(&format!("'{}", lifetime)),
                    GenericArg::Type(type_) => output.push_str(&format_type(type_, data)),
                    GenericArg::Const(constant) => output.push_str(&constant.expr),
                    GenericArg::Infer => output.push('_'),
                }

                if i < args.len() - 1 || !constraints.is_empty() {
                    output.push_str(", ");
                }
            }

            // Format constraints
            for (i, constraint) in constraints.iter().enumerate() {
                output.push_str(&constraint.name.to_string());

                // Format constraint args if present
                if let Some(args) = &constraint.args {
                    let mut args_str = String::new();
                    format_generic_args(&mut args_str, &args, data);
                    if !args_str.is_empty() && args_str != "<>" {
                        output.push_str(&args_str);
                    }
                }

                match &constraint.binding {
                    AssocItemConstraintKind::Equality(term) => {
                        output.push_str(" = ");
                        match term {
                            Term::Type(type_) => output.push_str(&format_type(type_, data)),
                            Term::Constant(constant) => output.push_str(&constant.expr),
                        }
                    }
                    AssocItemConstraintKind::Constraint(bounds) => {
                        output.push_str(": ");
                        format_bounds(output, bounds, data);
                    }
                }

                if i < constraints.len() - 1 {
                    output.push_str(", ");
                }
            }

            output.push('>');
        }
        GenericArgs::Parenthesized {
            inputs,
            output: output_type,
        } => {
            output.push('(');

            for (i, input) in inputs.iter().enumerate() {
                output.push_str(&format_type(input, data));
                if i < inputs.len() - 1 {
                    output.push_str(", ");
                }
            }

            output.push(')');

            if let Some(output_ty) = output_type {
                output.push_str(&format!(" -> {}", format_type(output_ty, data)));
            }
        }
        GenericArgs::ReturnTypeNotation => {
            output.push_str("::method(..)");
        }
    }
}

fn format_type(ty: &Type, data: &Crate) -> String {
    let mut output = String::new();

    match ty {
        Type::ResolvedPath(path) => {
            output.push_str(&path.path);
            if let Some(args) = &path.args {
                let mut args_str = String::new();
                format_generic_args(&mut args_str, args, data);
                output.push_str(&args_str);
            }
        }
        Type::DynTrait(dyn_trait) => {
            output.push_str("dyn ");

            for (i, trait_) in dyn_trait.traits.iter().enumerate() {
                // Higher-rank bounds if necessary
                if !trait_.generic_params.is_empty() {
                    output.push_str("for<");
                    for (j, param) in trait_.generic_params.iter().enumerate() {
                        match &param.kind {
                            GenericParamDefKind::Lifetime { .. } => {
                                output.push_str(&format!("'{}", param.name));
                            }
                            _ => output.push_str(&param.name),
                        }

                        if j < trait_.generic_params.len() - 1 {
                            output.push_str(", ");
                        }
                    }
                    output.push_str("> ");
                }

                output.push_str(&trait_.trait_.path);
                if let Some(args) = &trait_.trait_.args {
                    let mut args_str = String::new();
                    format_generic_args(&mut args_str, args, data);
                    output.push_str(&args_str);
                }

                if i < dyn_trait.traits.len() - 1 {
                    output.push_str(" + ");
                }
            }

            // Lifetime bound if present
            if let Some(lifetime) = &dyn_trait.lifetime {
                output.push_str(&format!(" + '{}", lifetime));
            }
        }
        Type::Generic(name) => {
            output.push_str(name);
        }
        Type::Primitive(name) => {
            output.push_str(name);
        }
        Type::FunctionPointer(fn_ptr) => {
            // For clarity about the parameters
            if !fn_ptr.generic_params.is_empty() {
                output.push_str("for<");
                for (j, param) in fn_ptr.generic_params.iter().enumerate() {
                    match &param.kind {
                        GenericParamDefKind::Lifetime { .. } => {
                            output.push_str(&format!("'{}", param.name));
                        }
                        _ => output.push_str(&param.name),
                    }

                    if j < fn_ptr.generic_params.len() - 1 {
                        output.push_str(", ");
                    }
                }
                output.push_str("> ");
            }

            // Function header (const, unsafe, extern, etc.)
            if fn_ptr.header.is_const {
                output.push_str("const ");
            }
            if fn_ptr.header.is_unsafe {
                output.push_str("unsafe ");
            }

            // ABI
            match &fn_ptr.header.abi {
                Abi::Rust => {}
                Abi::C { unwind } => {
                    if *unwind {
                        output.push_str("extern \"C-unwind\" ");
                    } else {
                        output.push_str("extern \"C\" ");
                    }
                }
                Abi::Cdecl { unwind } => {
                    if *unwind {
                        output.push_str("extern \"cdecl-unwind\" ");
                    } else {
                        output.push_str("extern \"cdecl\" ");
                    }
                }
                Abi::Stdcall { unwind } => {
                    if *unwind {
                        output.push_str("extern \"stdcall-unwind\" ");
                    } else {
                        output.push_str("extern \"stdcall\" ");
                    }
                }
                Abi::Fastcall { unwind } => {
                    if *unwind {
                        output.push_str("extern \"fastcall-unwind\" ");
                    } else {
                        output.push_str("extern \"fastcall\" ");
                    }
                }
                Abi::Aapcs { unwind } => {
                    if *unwind {
                        output.push_str("extern \"aapcs-unwind\" ");
                    } else {
                        output.push_str("extern \"aapcs\" ");
                    }
                }
                Abi::Win64 { unwind } => {
                    if *unwind {
                        output.push_str("extern \"win64-unwind\" ");
                    } else {
                        output.push_str("extern \"win64\" ");
                    }
                }
                Abi::SysV64 { unwind } => {
                    if *unwind {
                        output.push_str("extern \"sysv64-unwind\" ");
                    } else {
                        output.push_str("extern \"sysv64\" ");
                    }
                }
                Abi::System { unwind } => {
                    if *unwind {
                        output.push_str("extern \"system-unwind\" ");
                    } else {
                        output.push_str("extern \"system\" ");
                    }
                }
                Abi::Other(abi) => {
                    output.push_str(&format!("extern \"{}\" ", abi));
                }
            }

            output.push_str("fn(");

            // Parameters
            for (i, (_, param_type)) in fn_ptr.sig.inputs.iter().enumerate() {
                output.push_str(&format_type(param_type, data));
                if i < fn_ptr.sig.inputs.len() - 1 || fn_ptr.sig.is_c_variadic {
                    output.push_str(", ");
                }
            }

            // Variadic
            if fn_ptr.sig.is_c_variadic {
                output.push_str("...");
            }

            output.push(')');

            // Return type
            if let Some(return_type) = &fn_ptr.sig.output {
                output.push_str(&format!(" -> {}", format_type(return_type, data)));
            }
        }
        Type::Tuple(types) => {
            if types.is_empty() {
                output.push_str("()");
            } else {
                output.push('(');
                for (i, ty) in types.iter().enumerate() {
                    output.push_str(&format_type(ty, data));
                    if i < types.len() - 1 {
                        output.push_str(", ");
                    }
                }
                output.push(')');
            }
        }
        Type::Slice(ty) => {
            output.push_str(&format!("[{}]", format_type(ty, data)));
        }
        Type::Array { type_, len } => {
            output.push_str(&format!("[{}; {}]", format_type(type_, data), len));
        }
        Type::Pat {
            type_,
            __pat_unstable_do_not_use,
        } => {
            output.push_str(&format!(
                "{} is {}",
                format_type(type_, data),
                __pat_unstable_do_not_use
            ));
        }
        Type::ImplTrait(bounds) => {
            output.push_str("impl ");

            let mut bounds_str = String::new();
            format_bounds(&mut bounds_str, bounds, data);
            output.push_str(&bounds_str);
        }
        Type::Infer => {
            output.push('_');
        }
        Type::RawPointer { is_mutable, type_ } => {
            if *is_mutable {
                output.push_str("*mut ");
            } else {
                output.push_str("*const ");
            }
            output.push_str(&format_type(type_, data));
        }
        Type::BorrowedRef {
            lifetime,
            is_mutable,
            type_,
        } => {
            output.push('&');
            if let Some(lt) = lifetime {
                output.push_str(&format!("'{} ", lt));
            }
            if *is_mutable {
                output.push_str("mut ");
            }
            output.push_str(&format_type(type_, data));
        }
        Type::QualifiedPath {
            name,
            args,
            self_type,
            trait_,
        } => {
            output.push('<');
            output.push_str(&format_type(self_type, data));

            if let Some(trait_path) = trait_ {
                output.push_str(&format!(" as {}", trait_path.path));
                if let Some(trait_args) = &trait_path.args {
                    let mut args_str = String::new();
                    format_generic_args(&mut args_str, trait_args, data);
                    output.push_str(&args_str);
                }
            }

            output.push_str(&format!(">::{}", name));

            if let Some(args) = args {
                let mut args_str = String::new();
                format_generic_args(&mut args_str, args, data);
                if args_str != "<>" && !args_str.is_empty() {
                    output.push_str(&args_str);
                }
            }
        }
    }

    output
}

fn process_module_details(output: &mut String, module: &Module, data: &Crate, _level: usize) {
    if module.is_stripped {
        output.push_str(
            "> **Note:** This module is marked as stripped. Some items may be omitted.\n\n",
        );
    }

    // Reset level when entering a module to avoid excessive nesting
    // This ensures that module contents are always at a reasonable heading level
    process_items(output, &module.items, data, 3);
}

fn process_struct_details(output: &mut String, struct_: &Struct, data: &Crate, level: usize) {
    // Cap heading level at 6 (maximum valid Markdown heading level)
    let heading_level = std::cmp::min(level, 6);
    // Detail fields based on struct kind
    match &struct_.kind {
        StructKind::Unit => {
            // Nothing to detail for unit structs
        }
        StructKind::Tuple(fields) => {
            // Use heading_level for Fields section (since level is already incremented in process_item)
            output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
            output.push_str("| Index | Type | Documentation |\n");
            output.push_str("|-------|------|---------------|\n");

            for (i, field_opt) in fields.iter().enumerate() {
                if let Some(field_id) = field_opt {
                    if let Some(field_item) = data.index.get(field_id) {
                        if let ItemEnum::StructField(field_type) = &field_item.inner {
                            let docs = field_item
                                .docs
                                .as_deref()
                                .unwrap_or("")
                                .replace("\n", "<br>");
                            output.push_str(&format!(
                                "| {} | `{}` | {} |\n",
                                i,
                                format_type(field_type, data),
                                docs
                            ));
                        }
                    }
                } else {
                    output.push_str(&format!("| {} | `private` | *Private field* |\n", i));
                }
            }
            output.push('\n');
        }
        StructKind::Plain {
            fields,
            has_stripped_fields,
        } => {
            // Use heading_level for Fields section
            output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
            output.push_str("| Name | Type | Documentation |\n");
            output.push_str("|------|------|---------------|\n");

            for &field_id in fields {
                if let Some(field_item) = data.index.get(&field_id) {
                    if let Some(field_name) = &field_item.name {
                        if let ItemEnum::StructField(field_type) = &field_item.inner {
                            let docs = field_item
                                .docs
                                .as_deref()
                                .unwrap_or("")
                                .replace("\n", "<br>");
                            output.push_str(&format!(
                                "| `{}` | `{}` | {} |\n",
                                field_name,
                                format_type(field_type, data),
                                docs
                            ));
                        }
                    }
                }
            }

            if *has_stripped_fields {
                output.push_str("| *private fields* | ... | *Some fields have been omitted* |\n");
            }

            output.push('\n');
        }
    }

    // Process impls
    if !struct_.impls.is_empty() {
        // Use heading_level for Implementations section
        output.push_str(&format!(
            "{} Implementations\n\n",
            "#".repeat(heading_level)
        ));

        // Group impls by trait
        let mut trait_impls: std::collections::HashMap<String, Vec<Id>> =
            std::collections::HashMap::new();
        let mut inherent_impls: Vec<Id> = Vec::new();

        for &impl_id in &struct_.impls {
            if let Some(impl_item) = data.index.get(&impl_id) {
                if let ItemEnum::Impl(impl_) = &impl_item.inner {
                    if let Some(trait_) = &impl_.trait_ {
                        let trait_name = trait_.path.clone();
                        trait_impls.entry(trait_name).or_default().push(impl_id);
                    } else {
                        // Inherent impl
                        inherent_impls.push(impl_id);
                    }
                }
            }
        }

        // First list inherent impls
        if !inherent_impls.is_empty() {
            // Use level+1 for Methods (one level deeper than Implementations)
            output.push_str(&format!(
                "{} Methods\n\n",
                "#".repeat(std::cmp::min(heading_level + 1, 6))
            ));
            for &impl_id in &inherent_impls {
                if let Some(impl_item) = data.index.get(&impl_id) {
                    if let ItemEnum::Impl(impl_) = &impl_item.inner {
                        for &item_id in &impl_.items {
                            if let Some(method_item) = data.index.get(&item_id) {
                                if let ItemEnum::Function(_) = &method_item.inner {
                                    // Format method signature
                                    let mut method_signature = String::new();
                                    format_item_signature(&mut method_signature, method_item, data);

                                    // Output with proper code block formatting
                                    output.push_str("- ```rust\n  ");
                                    output.push_str(&method_signature.trim());
                                    output.push_str("\n  ```");

                                    // Add documentation if available
                                    if let Some(docs) = &method_item.docs {
                                        if let Some(first_line) = docs.lines().next() {
                                            if !first_line.trim().is_empty() {
                                                output.push_str(&format!("\n  {}", first_line));
                                            }
                                        }
                                    }
                                    output.push_str("\n\n");
                                }
                            }
                        }
                    }
                }
            }
        }

        // Then list trait impls
        if !trait_impls.is_empty() {
            // Use level+1 for Trait Implementations (one level deeper than Implementations)
            output.push_str(&format!(
                "{} Trait Implementations\n\n",
                "#".repeat(std::cmp::min(heading_level + 1, 6))
            ));
            // Sort trait implementations alphabetically for deterministic output
            let mut sorted_trait_impls: Vec<_> = trait_impls.into_iter().collect();
            sorted_trait_impls.sort_by(|a, b| a.0.cmp(&b.0));
            for (trait_name, impls) in sorted_trait_impls {
                output.push_str(&format!("- **{}**\n", trait_name));
                for &impl_id in &impls {
                    if let Some(impl_item) = data.index.get(&impl_id) {
                        if let ItemEnum::Impl(impl_) = &impl_item.inner {
                            for &item_id in &impl_.items {
                                if let Some(method_item) = data.index.get(&item_id) {
                                    if let ItemEnum::Function(_) = &method_item.inner {
                                        // Format method signature
                                        let mut method_signature = String::new();
                                        format_item_signature(
                                            &mut method_signature,
                                            method_item,
                                            data,
                                        );

                                        // Output with proper code block formatting
                                        output.push_str("  - ```rust\n    ");
                                        output.push_str(&method_signature.trim());
                                        output.push_str("\n    ```");

                                        // Add documentation if available
                                        if let Some(docs) = &method_item.docs {
                                            if let Some(first_line) = docs.lines().next() {
                                                if !first_line.trim().is_empty() {
                                                    output
                                                        .push_str(&format!("\n    {}", first_line));
                                                }
                                            }
                                        }
                                        output.push_str("\n\n");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn process_enum_details(output: &mut String, enum_: &Enum, data: &Crate, level: usize) {
    // Cap heading level at 6 (maximum valid Markdown heading level)
    let heading_level = std::cmp::min(level, 6);
    // Detail variants with proper nesting
    output.push_str(&format!("{} Variants\n\n", "#".repeat(heading_level)));

    for &variant_id in &enum_.variants {
        if let Some(variant_item) = data.index.get(&variant_id) {
            if let Some(variant_name) = &variant_item.name {
                // Use heading_level + 1 for individual variants (capped at 6)
                let variant_heading_level = std::cmp::min(heading_level + 1, 6);
                output.push_str(&format!(
                    "{} `{}`\n\n",
                    "#".repeat(variant_heading_level),
                    variant_name
                ));

                // Add variant docs if available
                if let Some(docs) = &variant_item.docs {
                    output.push_str(&format!("{}\n\n", docs));
                }

                if let ItemEnum::Variant(variant) = &variant_item.inner {
                    match &variant.kind {
                        VariantKind::Plain => {
                            // Nothing additional to display for plain variants
                            if let Some(discriminant) = &variant.discriminant {
                                output.push_str(&format!(
                                    "Discriminant: `{}`\n\n",
                                    discriminant.expr
                                ));
                            }
                        }
                        VariantKind::Tuple(fields) => {
                            output.push_str("Fields:\n\n");
                            output.push_str("| Index | Type | Documentation |\n");
                            output.push_str("|-------|------|---------------|\n");

                            for (i, field_opt) in fields.iter().enumerate() {
                                if let Some(field_id) = field_opt {
                                    if let Some(field_item) = data.index.get(field_id) {
                                        if let ItemEnum::StructField(field_type) = &field_item.inner
                                        {
                                            let docs = field_item
                                                .docs
                                                .as_deref()
                                                .unwrap_or("")
                                                .replace("\n", "<br>");
                                            output.push_str(&format!(
                                                "| {} | `{}` | {} |\n",
                                                i,
                                                format_type(field_type, data),
                                                docs
                                            ));
                                        }
                                    }
                                } else {
                                    output.push_str(&format!(
                                        "| {} | `private` | *Private field* |\n",
                                        i
                                    ));
                                }
                            }
                            output.push('\n');
                        }
                        VariantKind::Struct {
                            fields,
                            has_stripped_fields,
                        } => {
                            output.push_str("Fields:\n\n");
                            output.push_str("| Name | Type | Documentation |\n");
                            output.push_str("|------|------|---------------|\n");

                            for &field_id in fields {
                                if let Some(field_item) = data.index.get(&field_id) {
                                    if let Some(field_name) = &field_item.name {
                                        if let ItemEnum::StructField(field_type) = &field_item.inner
                                        {
                                            let docs = field_item
                                                .docs
                                                .as_deref()
                                                .unwrap_or("")
                                                .replace("\n", "<br>");
                                            output.push_str(&format!(
                                                "| `{}` | `{}` | {} |\n",
                                                field_name,
                                                format_type(field_type, data),
                                                docs
                                            ));
                                        }
                                    }
                                }
                            }

                            if *has_stripped_fields {
                                output.push_str("| *private fields* | ... | *Some fields have been omitted* |\n");
                            }

                            output.push('\n');
                        }
                    }

                    if let Some(discriminant) = &variant.discriminant {
                        output
                            .push_str(&format!("Discriminant value: `{}`\n\n", discriminant.value));
                    }
                }
            }
        }
    }

    if enum_.has_stripped_variants {
        output.push_str(
            "*Note: Some variants have been omitted because they are private or hidden.*\n\n",
        );
    }

    // Process impls (same as for struct)
    if !enum_.impls.is_empty() {
        output.push_str(&format!(
            "{} Implementations\n\n",
            "#".repeat(heading_level)
        ));

        // Group impls by trait
        let mut trait_impls: std::collections::HashMap<String, Vec<Id>> =
            std::collections::HashMap::new();
        let mut inherent_impls: Vec<Id> = Vec::new();

        for &impl_id in &enum_.impls {
            if let Some(impl_item) = data.index.get(&impl_id) {
                if let ItemEnum::Impl(impl_) = &impl_item.inner {
                    if let Some(trait_) = &impl_.trait_ {
                        let trait_name = trait_.path.clone();
                        trait_impls.entry(trait_name).or_default().push(impl_id);
                    } else {
                        // Inherent impl
                        inherent_impls.push(impl_id);
                    }
                }
            }
        }

        // First list inherent impls
        if !inherent_impls.is_empty() {
            let methods_level = std::cmp::min(heading_level + 1, 6);
            output.push_str(&format!("{} Methods\n\n", "#".repeat(methods_level)));
            for &impl_id in &inherent_impls {
                if let Some(impl_item) = data.index.get(&impl_id) {
                    if let ItemEnum::Impl(impl_) = &impl_item.inner {
                        for &item_id in &impl_.items {
                            if let Some(method_item) = data.index.get(&item_id) {
                                if let ItemEnum::Function(_) = &method_item.inner {
                                    // Format method signature
                                    let mut method_signature = String::new();
                                    format_item_signature(&mut method_signature, method_item, data);

                                    // Output with proper code block formatting
                                    output.push_str("- ```rust\n  ");
                                    output.push_str(&method_signature.trim());
                                    output.push_str("\n  ```");

                                    // Add documentation if available
                                    if let Some(docs) = &method_item.docs {
                                        if let Some(first_line) = docs.lines().next() {
                                            if !first_line.trim().is_empty() {
                                                output.push_str(&format!("\n  {}", first_line));
                                            }
                                        }
                                    }
                                    output.push_str("\n\n");
                                }
                            }
                        }
                    }
                }
            }
        }

        // Then list trait impls
        if !trait_impls.is_empty() {
            let trait_impl_level = std::cmp::min(heading_level + 1, 6);
            output.push_str(&format!(
                "{} Trait Implementations\n\n",
                "#".repeat(trait_impl_level)
            ));
            // Sort trait implementations alphabetically for deterministic output
            let mut sorted_trait_impls: Vec<_> = trait_impls.into_iter().collect();
            sorted_trait_impls.sort_by(|a, b| a.0.cmp(&b.0));
            for (trait_name, impls) in sorted_trait_impls {
                output.push_str(&format!("- **{}**\n", trait_name));
                for &impl_id in &impls {
                    if let Some(impl_item) = data.index.get(&impl_id) {
                        if let ItemEnum::Impl(impl_) = &impl_item.inner {
                            for &item_id in &impl_.items {
                                if let Some(method_item) = data.index.get(&item_id) {
                                    if let ItemEnum::Function(_) = &method_item.inner {
                                        // Format method signature
                                        let mut method_signature = String::new();
                                        format_item_signature(
                                            &mut method_signature,
                                            method_item,
                                            data,
                                        );

                                        // Output with proper code block formatting
                                        output.push_str("  - ```rust\n    ");
                                        output.push_str(&method_signature.trim());
                                        output.push_str("\n    ```");

                                        // Add documentation if available
                                        if let Some(docs) = &method_item.docs {
                                            if let Some(first_line) = docs.lines().next() {
                                                if !first_line.trim().is_empty() {
                                                    output
                                                        .push_str(&format!("\n    {}", first_line));
                                                }
                                            }
                                        }
                                        output.push_str("\n\n");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn process_union_details(output: &mut String, union_: &Union, data: &Crate, level: usize) {
    // Cap heading level at 6 (maximum valid Markdown heading level)
    let heading_level = std::cmp::min(level, 6);
    // Detail fields
    output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
    output.push_str("| Name | Type | Documentation |\n");
    output.push_str("|------|------|---------------|\n");

    for &field_id in &union_.fields {
        if let Some(field_item) = data.index.get(&field_id) {
            if let Some(field_name) = &field_item.name {
                if let ItemEnum::StructField(field_type) = &field_item.inner {
                    let docs = field_item
                        .docs
                        .as_deref()
                        .unwrap_or("")
                        .replace("\n", "<br>");
                    output.push_str(&format!(
                        "| `{}` | `{}` | {} |\n",
                        field_name,
                        format_type(field_type, data),
                        docs
                    ));
                }
            }
        }
    }

    if union_.has_stripped_fields {
        output.push_str("| *private fields* | ... | *Some fields have been omitted* |\n");
    }

    output.push('\n');

    // Process impls
    if !union_.impls.is_empty() {
        output.push_str(&format!(
            "{} Implementations\n\n",
            "#".repeat(heading_level)
        ));

        // Group impls by trait
        let mut trait_impls: std::collections::HashMap<String, Vec<Id>> =
            std::collections::HashMap::new();
        let mut inherent_impls: Vec<Id> = Vec::new();

        for &impl_id in &union_.impls {
            if let Some(impl_item) = data.index.get(&impl_id) {
                if let ItemEnum::Impl(impl_) = &impl_item.inner {
                    if let Some(trait_) = &impl_.trait_ {
                        let trait_name = trait_.path.clone();
                        trait_impls.entry(trait_name).or_default().push(impl_id);
                    } else {
                        // Inherent impl
                        inherent_impls.push(impl_id);
                    }
                }
            }
        }

        // First list inherent impls
        if !inherent_impls.is_empty() {
            let methods_level = std::cmp::min(heading_level + 1, 6);
            output.push_str(&format!("{} Methods\n\n", "#".repeat(methods_level)));
            for &impl_id in &inherent_impls {
                if let Some(impl_item) = data.index.get(&impl_id) {
                    if let ItemEnum::Impl(impl_) = &impl_item.inner {
                        for &item_id in &impl_.items {
                            if let Some(method_item) = data.index.get(&item_id) {
                                if let ItemEnum::Function(_) = &method_item.inner {
                                    if let Some(name) = &method_item.name {
                                        output.push_str(&format!("- `{}`: ", name));
                                        if let Some(docs) = &method_item.docs {
                                            let first_line = docs.lines().next().unwrap_or("");
                                            output.push_str(first_line);
                                        }
                                        output.push('\n');
                                    }
                                }
                            }
                        }
                    }
                }
            }
            output.push('\n');
        }

        // Then list trait impls
        if !trait_impls.is_empty() {
            let trait_impl_level = std::cmp::min(heading_level + 1, 6);
            output.push_str(&format!(
                "{} Trait Implementations\n\n",
                "#".repeat(trait_impl_level)
            ));
            // Sort trait implementations alphabetically for deterministic output
            let mut sorted_trait_impls: Vec<_> = trait_impls.into_iter().collect();
            sorted_trait_impls.sort_by(|a, b| a.0.cmp(&b.0));
            for (trait_name, impls) in sorted_trait_impls {
                output.push_str(&format!("- **{}**\n", trait_name));
                for &impl_id in &impls {
                    if let Some(impl_item) = data.index.get(&impl_id) {
                        if let ItemEnum::Impl(impl_) = &impl_item.inner {
                            for &item_id in &impl_.items {
                                if let Some(method_item) = data.index.get(&item_id) {
                                    if let Some(name) = &method_item.name {
                                        output.push_str(&format!("  - `{}`: ", name));
                                        if let Some(docs) = &method_item.docs {
                                            let first_line = docs.lines().next().unwrap_or("");
                                            output.push_str(first_line);
                                        }
                                        output.push('\n');
                                    }
                                }
                            }
                        }
                    }
                }
            }
            output.push('\n');
        }
    }
}

fn process_trait_details(output: &mut String, trait_: &Trait, data: &Crate, level: usize) {
    // Cap heading level at 6 (maximum valid Markdown heading level)
    let heading_level = std::cmp::min(level, 6);
    // Special traits info
    if trait_.is_auto {
        output.push_str("> This is an auto trait.\n\n");
    }
    if trait_.is_unsafe {
        output.push_str("> This trait is unsafe to implement.\n\n");
    }
    if !trait_.is_dyn_compatible {
        output.push_str(
            "> This trait is not object-safe and cannot be used in dynamic trait objects.\n\n",
        );
    }

    // Associated items
    if !trait_.items.is_empty() {
        // Group items by kind
        let mut required_methods = Vec::new();
        let mut provided_methods = Vec::new();
        let mut assoc_types = Vec::new();
        let mut assoc_consts = Vec::new();

        for &item_id in &trait_.items {
            if let Some(item) = data.index.get(&item_id) {
                match &item.inner {
                    ItemEnum::Function(function) => {
                        if function.has_body {
                            provided_methods.push(item_id);
                        } else {
                            required_methods.push(item_id);
                        }
                    }
                    ItemEnum::AssocType { .. } => assoc_types.push(item_id),
                    ItemEnum::AssocConst { value, .. } => {
                        if value.is_some() {
                            // Has a default value
                            provided_methods.push(item_id);
                        } else {
                            assoc_consts.push(item_id);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Required items
        if !required_methods.is_empty() || !assoc_types.is_empty() || !assoc_consts.is_empty() {
            output.push_str(&format!("{} Required Items\n\n", "#".repeat(heading_level)));

            if !assoc_types.is_empty() {
                output.push_str(&format!(
                    "{} Associated Types\n\n",
                    "#".repeat(heading_level + 1)
                ));
                for &type_id in &assoc_types {
                    if let Some(type_item) = data.index.get(&type_id) {
                        if let Some(name) = &type_item.name {
                            output.push_str(&format!("- `{}`", name));
                            if let Some(docs) = &type_item.docs {
                                if let Some(first_line) = docs.lines().next() {
                                    if !first_line.trim().is_empty() {
                                        output.push_str(&format!(": {}", first_line));
                                    }
                                }
                            }
                            output.push('\n');
                        }
                    }
                }
                output.push('\n');
            }

            if !assoc_consts.is_empty() {
                output.push_str(&format!(
                    "{} Associated Constants\n\n",
                    "#".repeat(heading_level + 1)
                ));
                for &const_id in &assoc_consts {
                    if let Some(const_item) = data.index.get(&const_id) {
                        if let Some(name) = &const_item.name {
                            output.push_str(&format!("- `{}`", name));
                            if let Some(docs) = &const_item.docs {
                                if let Some(first_line) = docs.lines().next() {
                                    if !first_line.trim().is_empty() {
                                        output.push_str(&format!(": {}", first_line));
                                    }
                                }
                            }
                            output.push('\n');
                        }
                    }
                }
                output.push('\n');
            }

            if !required_methods.is_empty() {
                output.push_str(&format!(
                    "{} Required Methods\n\n",
                    "#".repeat(heading_level + 1)
                ));
                for &method_id in &required_methods {
                    if let Some(method_item) = data.index.get(&method_id) {
                        if let Some(name) = &method_item.name {
                            output.push_str(&format!("- `{}`", name));
                            if let Some(docs) = &method_item.docs {
                                if let Some(first_line) = docs.lines().next() {
                                    if !first_line.trim().is_empty() {
                                        output.push_str(&format!(": {}", first_line));
                                    }
                                }
                            }
                            output.push('\n');
                        }
                    }
                }
                output.push('\n');
            }
        }

        // Provided items
        if !provided_methods.is_empty() {
            output.push_str(&format!(
                "{} Provided Methods\n\n",
                "#".repeat(heading_level)
            ));
            for &method_id in &provided_methods {
                if let Some(method_item) = data.index.get(&method_id) {
                    if let ItemEnum::Function(_) = &method_item.inner {
                        // Format method signature
                        let mut method_signature = String::new();
                        format_item_signature(&mut method_signature, method_item, data);

                        // Output with proper code block formatting
                        output.push_str("- ```rust\n  ");
                        output.push_str(&method_signature.trim());
                        output.push_str("\n  ```");

                        // Add documentation if available
                        if let Some(docs) = &method_item.docs {
                            if let Some(first_line) = docs.lines().next() {
                                if !first_line.trim().is_empty() {
                                    output.push_str(&format!("\n  {}", first_line));
                                }
                            }
                        }
                        output.push_str("\n\n");
                    }
                }
            }
        }
    }

    // Implementations
    if !trait_.implementations.is_empty() {
        output.push_str(&format!(
            "{} Implementations\n\n",
            "#".repeat(heading_level)
        ));
        output.push_str("This trait is implemented for the following types:\n\n");

        for &impl_id in &trait_.implementations {
            if let Some(impl_item) = data.index.get(&impl_id) {
                if let ItemEnum::Impl(impl_) = &impl_item.inner {
                    output.push_str(&format!("- `{}`", format_type(&impl_.for_, data)));
                    // Add generics if present
                    if !impl_.generics.params.is_empty() {
                        let mut generics_str = String::new();
                        format_generics(&mut generics_str, &impl_.generics, data);
                        if generics_str != "<>" {
                            output.push_str(" with ");
                            output.push_str(&generics_str);
                        }
                    }
                    output.push('\n');
                }
            }
        }
        output.push('\n');
    }
}

fn process_impl_details(output: &mut String, impl_: &Impl, data: &Crate, level: usize) {
    // Cap heading level at 6 (maximum valid Markdown heading level)
    let heading_level = std::cmp::min(level, 6);
    // List all items in the impl
    if !impl_.items.is_empty() {
        output.push_str(&format!(
            "{} Associated Items\n\n",
            "#".repeat(heading_level)
        ));

        // Group by kind
        let mut methods = Vec::new();
        let mut assoc_types = Vec::new();
        let mut assoc_consts = Vec::new();

        for &item_id in &impl_.items {
            if let Some(item) = data.index.get(&item_id) {
                match &item.inner {
                    ItemEnum::Function(_) => methods.push(item_id),
                    ItemEnum::AssocType { .. } => assoc_types.push(item_id),
                    ItemEnum::AssocConst { .. } => assoc_consts.push(item_id),
                    _ => {}
                }
            }
        }

        if !assoc_types.is_empty() {
            output.push_str(&format!(
                "{} Associated Types\n\n",
                "#".repeat(heading_level + 1)
            ));
            for &type_id in &assoc_types {
                process_item(output, data.index.get(&type_id).unwrap(), data, level + 1);
            }
        }

        if !assoc_consts.is_empty() {
            output.push_str(&format!(
                "{} Associated Constants\n\n",
                "#".repeat(heading_level + 1)
            ));
            for &const_id in &assoc_consts {
                process_item(output, data.index.get(&const_id).unwrap(), data, level + 1);
            }
        }

        if !methods.is_empty() {
            output.push_str(&format!("{} Methods\n\n", "#".repeat(heading_level + 1)));
            for &method_id in &methods {
                process_item(output, data.index.get(&method_id).unwrap(), data, level + 1);
            }
        }
    }

    // If this is a trait impl, list the provided trait methods that aren't overridden
    if impl_.trait_.is_some() && !impl_.provided_trait_methods.is_empty() {
        output.push_str(&format!(
            "{} Provided Trait Methods\n\n",
            "#".repeat(heading_level)
        ));
        output.push_str("The following methods are available through the trait but not explicitly implemented:\n\n");

        for provided_method in &impl_.provided_trait_methods {
            output.push_str(&format!("- `{}`\n", provided_method));
        }

        output.push('\n');
    }

    // If this is a blanket impl, mention it
    if let Some(blanket_type) = &impl_.blanket_impl {
        output.push_str(&format!(
            "This is a blanket implementation for all types that match: `{}`\n\n",
            format_type(blanket_type, data)
        ));
    }
}
