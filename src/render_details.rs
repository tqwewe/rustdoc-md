use crate::render_core::{ResolvedItemInfo, render_docs_with_links};
use crate::render_signatures::{format_generics, format_type};
use crate::rustdoc_json_types::Id;
use crate::rustdoc_json_types::*; // Ensure Id is in scope for Fn(&Id)

pub fn process_module_details<F>(
    output: &mut String,
    module: &Module,
    data: &Crate,
    level: usize,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    if module.is_stripped {
        output.push_str(
            "> **Note:** This module is marked as stripped. Some items may be omitted.\n\n",
        );
    }
    // The plan changes process_items to render_item_list, and its level usage.
    // Original call was `data, 3`. If level is parent's level, then `level + 1` for children.
    // The plan's render_item_list takes `level` which is the heading level for *its* sections.
    // If process_module_details is called with `level + 1` from render_item_page,
    // then render_item_list should also be called with `level` (which is parent's level + 1).
    crate::render_core::render_item_list(output, &module.items, data, level, link_resolver);
}

pub fn process_struct_details<F>(
    output: &mut String,
    struct_: &Struct,
    data: &Crate,
    level: usize,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    let heading_level = std::cmp::min(level, 6);
    match &struct_.kind {
        StructKind::Unit => {}
        StructKind::Tuple(fields) => {
            output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
            output.push_str("| Index | Type | Documentation |\n");
            output.push_str("|-------|------|---------------|\n");

            for (i, field_opt) in fields.iter().enumerate() {
                if let Some(field_id) = field_opt {
                    if let Some(field_item) = data.index.get(field_id) {
                        if let ItemEnum::StructField(field_type) = &field_item.inner {
                            let docs_str = field_item.docs.as_deref().unwrap_or("");
                            let rendered_docs = if docs_str.is_empty() {
                                "".to_string()
                            } else {
                                render_docs_with_links(
                                    docs_str,
                                    &field_item.links,
                                    data,
                                    link_resolver,
                                )
                                .replace("\n", "<br>")
                            };
                            output.push_str(&format!(
                                "| {} | `{}` | {} |\n",
                                i,
                                format_type(field_type, data),
                                rendered_docs
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
            output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
            output.push_str("| Name | Type | Documentation |\n");
            output.push_str("|------|------|---------------|\n");

            for &field_id in fields {
                if let Some(field_item) = data.index.get(&field_id) {
                    if let Some(field_name) = &field_item.name {
                        if let ItemEnum::StructField(field_type) = &field_item.inner {
                            let docs_str = field_item.docs.as_deref().unwrap_or("");
                            let rendered_docs = if docs_str.is_empty() {
                                "".to_string()
                            } else {
                                render_docs_with_links(
                                    docs_str,
                                    &field_item.links,
                                    data,
                                    link_resolver,
                                )
                                .replace("\n", "<br>")
                            };
                            output.push_str(&format!(
                                "| `{}` | `{}` | {} |\n",
                                field_name,
                                format_type(field_type, data),
                                rendered_docs
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

    if !struct_.impls.is_empty() {
        let mut non_blanket_impls_info = Vec::new();
        let mut blanket_impl_traits = Vec::new();

        for &impl_id in &struct_.impls {
            if let Some(impl_item_ref) = data.index.get(&impl_id) {
                if let ItemEnum::Impl(impl_details) = &impl_item_ref.inner {
                    if impl_details.blanket_impl.is_some() {
                        if let Some(trait_) = &impl_details.trait_ {
                            blanket_impl_traits.push(trait_.path.clone());
                        }
                    } else {
                        non_blanket_impls_info.push(ResolvedItemInfo {
                            original_item: impl_item_ref,
                            effective_name: None, // Impls are often nameless in this context
                            reexport_source_canonical_path: None,
                        });
                    }
                } else {
                    // This case implies the item is not an Impl, or data is malformed.
                    // Treat as a non-blanket impl for rendering to be safe.
                    non_blanket_impls_info.push(ResolvedItemInfo {
                        original_item: impl_item_ref,
                        effective_name: None,
                        reexport_source_canonical_path: None,
                    });
                }
            }
        }

        blanket_impl_traits.sort();
        blanket_impl_traits.dedup();

        if !non_blanket_impls_info.is_empty() || !blanket_impl_traits.is_empty() {
            output.push_str(&format!(
                "{} Implementations\n\n",
                "#".repeat(heading_level)
            ));
        }

        if !non_blanket_impls_info.is_empty() {
            for resolved_impl_info in non_blanket_impls_info {
                crate::render_core::render_item_page(
                    output,
                    &resolved_impl_info,
                    data,
                    heading_level + 1,
                    link_resolver,
                );
            }
        }

        if !blanket_impl_traits.is_empty() {
            output.push_str("<details><summary>Blanket Implementations</summary>\n\n");
            output.push_str("This type is implemented for the following traits through blanket implementations:\n\n");
            for trait_path in blanket_impl_traits {
                output.push_str(&format!("- `{}`\n", trait_path));
            }
            output.push_str("\n</details>\n\n");
        }
    }
}

pub fn process_enum_details<F>(
    output: &mut String,
    enum_: &Enum,
    data: &Crate,
    level: usize,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    let heading_level = std::cmp::min(level, 6);
    output.push_str(&format!("{} Variants\n\n", "#".repeat(heading_level)));

    for &variant_id in &enum_.variants {
        if let Some(variant_item) = data.index.get(&variant_id) {
            if let Some(variant_name) = &variant_item.name {
                let variant_heading_level = std::cmp::min(heading_level + 1, 6);
                output.push_str(&format!(
                    "{} `{}`\n\n",
                    "#".repeat(variant_heading_level),
                    variant_name
                ));

                if let Some(docs) = &variant_item.docs {
                    output.push_str(&render_docs_with_links(
                        docs,
                        &variant_item.links,
                        data,
                        link_resolver,
                    ));
                    output.push_str("\n\n");
                }

                if let ItemEnum::Variant(variant_details) = &variant_item.inner {
                    // Display variant signature (plain, tuple, or struct-like)
                    output.push_str("```rust\n");
                    // We need to reconstruct a partial signature for the variant here
                    // This is a simplified version, format_item_signature handles full items
                    let mut variant_sig = String::new();
                    variant_sig.push_str(variant_name);
                    match &variant_details.kind {
                        VariantKind::Plain => {}
                        VariantKind::Tuple(fields) => {
                            variant_sig.push('(');
                            for (i, field_opt) in fields.iter().enumerate() {
                                if let Some(field_id) = field_opt {
                                    if let Some(field_item) = data.index.get(field_id) {
                                        if let ItemEnum::StructField(field_type) = &field_item.inner
                                        {
                                            variant_sig.push_str(&format_type(field_type, data));
                                        }
                                    }
                                } else {
                                    variant_sig.push_str("/* private */");
                                }
                                if i < fields.len() - 1 {
                                    variant_sig.push_str(", ");
                                }
                            }
                            variant_sig.push(')');
                        }
                        VariantKind::Struct { .. } => {
                            // Removed unused 'fields' binding here
                            variant_sig.push_str(" { .. }"); // Simplified for now
                        }
                    }
                    if let Some(discriminant) = &variant_details.discriminant {
                        variant_sig.push_str(&format!(" = {}", discriminant.expr));
                    }
                    output.push_str(&variant_sig);
                    output.push_str("\n```\n\n");

                    // Detailed fields for Tuple and Struct variants
                    match &variant_details.kind {
                        VariantKind::Tuple(fields) => {
                            if !fields.is_empty() && fields.iter().any(|f| f.is_some()) {
                                output.push_str("Fields:\n\n");
                                output.push_str("| Index | Type | Documentation |\n");
                                output.push_str("|-------|------|---------------|\n");
                                for (i, field_opt) in fields.iter().enumerate() {
                                    if let Some(field_id) = field_opt {
                                        if let Some(field_item) = data.index.get(field_id) {
                                            if let ItemEnum::StructField(field_type) =
                                                &field_item.inner
                                            {
                                                let docs_str =
                                                    field_item.docs.as_deref().unwrap_or("");
                                                let rendered_docs = if docs_str.is_empty() {
                                                    "".to_string()
                                                } else {
                                                    render_docs_with_links(
                                                        docs_str,
                                                        &field_item.links,
                                                        data,
                                                        link_resolver,
                                                    )
                                                    .replace("\n", "<br>")
                                                };
                                                output.push_str(&format!(
                                                    "| {} | `{}` | {} |\n",
                                                    i,
                                                    format_type(field_type, data),
                                                    rendered_docs
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
                        }
                        VariantKind::Struct {
                            fields,
                            has_stripped_fields,
                        } => {
                            if !fields.is_empty() || *has_stripped_fields {
                                output.push_str("Fields:\n\n");
                                output.push_str("| Name | Type | Documentation |\n");
                                output.push_str("|------|------|---------------|\n");
                                for &field_id in fields {
                                    if let Some(field_item) = data.index.get(&field_id) {
                                        if let Some(field_name) = &field_item.name {
                                            if let ItemEnum::StructField(field_type) =
                                                &field_item.inner
                                            {
                                                let docs_str =
                                                    field_item.docs.as_deref().unwrap_or("");
                                                let rendered_docs = if docs_str.is_empty() {
                                                    "".to_string()
                                                } else {
                                                    render_docs_with_links(
                                                        docs_str,
                                                        &field_item.links,
                                                        data,
                                                        link_resolver,
                                                    )
                                                    .replace("\n", "<br>")
                                                };
                                                output.push_str(&format!(
                                                    "| `{}` | `{}` | {} |\n",
                                                    field_name,
                                                    format_type(field_type, data),
                                                    rendered_docs
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
                        VariantKind::Plain => {}
                    }
                    if let Some(discriminant) = &variant_details.discriminant {
                        output
                            .push_str(&format!("Discriminant Value: `{}`\n\n", discriminant.value));
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

    if !enum_.impls.is_empty() {
        let mut non_blanket_impls_info = Vec::new();
        let mut blanket_impl_traits = Vec::new();

        for &impl_id in &enum_.impls {
            if let Some(impl_item_ref) = data.index.get(&impl_id) {
                if let ItemEnum::Impl(impl_details) = &impl_item_ref.inner {
                    if impl_details.blanket_impl.is_some() {
                        if let Some(trait_) = &impl_details.trait_ {
                            blanket_impl_traits.push(trait_.path.clone());
                        }
                    } else {
                        non_blanket_impls_info.push(ResolvedItemInfo {
                            original_item: impl_item_ref,
                            effective_name: None,
                            reexport_source_canonical_path: None,
                        });
                    }
                } else {
                    non_blanket_impls_info.push(ResolvedItemInfo {
                        original_item: impl_item_ref,
                        effective_name: None,
                        reexport_source_canonical_path: None,
                    });
                }
            }
        }

        blanket_impl_traits.sort();
        blanket_impl_traits.dedup();

        if !non_blanket_impls_info.is_empty() || !blanket_impl_traits.is_empty() {
            output.push_str(&format!(
                "{} Implementations\n\n",
                "#".repeat(heading_level)
            ));
        }

        if !non_blanket_impls_info.is_empty() {
            for resolved_impl_info in non_blanket_impls_info {
                crate::render_core::render_item_page(
                    output,
                    &resolved_impl_info,
                    data,
                    heading_level + 1,
                    link_resolver,
                );
            }
        }

        if !blanket_impl_traits.is_empty() {
            output.push_str("<details><summary>Blanket Implementations</summary>\n\n");
            output.push_str("This type is implemented for the following traits through blanket implementations:\n\n");
            for trait_path in blanket_impl_traits {
                output.push_str(&format!("- `{}`\n", trait_path));
            }
            output.push_str("\n</details>\n\n");
        }
    }
}

pub fn process_union_details<F>(
    output: &mut String,
    union_: &Union,
    data: &Crate,
    level: usize,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    let heading_level = std::cmp::min(level, 6);
    output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
    output.push_str("| Name | Type | Documentation |\n");
    output.push_str("|------|------|---------------|\n");

    for &field_id in &union_.fields {
        if let Some(field_item) = data.index.get(&field_id) {
            if let Some(field_name) = &field_item.name {
                if let ItemEnum::StructField(field_type) = &field_item.inner {
                    let docs_str = field_item.docs.as_deref().unwrap_or("");
                    let rendered_docs = if docs_str.is_empty() {
                        "".to_string()
                    } else {
                        render_docs_with_links(docs_str, &field_item.links, data, link_resolver)
                            .replace("\n", "<br>")
                    };
                    output.push_str(&format!(
                        "| `{}` | `{}` | {} |\n",
                        field_name,
                        format_type(field_type, data),
                        rendered_docs
                    ));
                }
            }
        }
    }
    if union_.has_stripped_fields {
        output.push_str("| *private fields* | ... | *Some fields have been omitted* |\n");
    }
    output.push('\n');

    if !union_.impls.is_empty() {
        let mut non_blanket_impls_info = Vec::new();
        let mut blanket_impl_traits = Vec::new();

        for &impl_id in &union_.impls {
            if let Some(impl_item_ref) = data.index.get(&impl_id) {
                if let ItemEnum::Impl(impl_details) = &impl_item_ref.inner {
                    if impl_details.blanket_impl.is_some() {
                        if let Some(trait_) = &impl_details.trait_ {
                            blanket_impl_traits.push(trait_.path.clone());
                        }
                    } else {
                        non_blanket_impls_info.push(ResolvedItemInfo {
                            original_item: impl_item_ref,
                            effective_name: None,
                            reexport_source_canonical_path: None,
                        });
                    }
                } else {
                    non_blanket_impls_info.push(ResolvedItemInfo {
                        original_item: impl_item_ref,
                        effective_name: None,
                        reexport_source_canonical_path: None,
                    });
                }
            }
        }

        blanket_impl_traits.sort();
        blanket_impl_traits.dedup();

        if !non_blanket_impls_info.is_empty() || !blanket_impl_traits.is_empty() {
            output.push_str(&format!(
                "{} Implementations\n\n",
                "#".repeat(heading_level)
            ));
        }

        if !non_blanket_impls_info.is_empty() {
            for resolved_impl_info in non_blanket_impls_info {
                crate::render_core::render_item_page(
                    output,
                    &resolved_impl_info,
                    data,
                    heading_level + 1,
                    link_resolver,
                );
            }
        }

        if !blanket_impl_traits.is_empty() {
            output.push_str("<details><summary>Blanket Implementations</summary>\n\n");
            output.push_str("This type is implemented for the following traits through blanket implementations:\n\n");
            for trait_path in blanket_impl_traits {
                output.push_str(&format!("- `{}`\n", trait_path));
            }
            output.push_str("\n</details>\n\n");
        }
    }
}

pub fn process_trait_details<F>(
    output: &mut String,
    trait_: &Trait,
    data: &Crate,
    level: usize,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    let heading_level = std::cmp::min(level, 6);
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

    let mut required_items = Vec::new();
    let mut provided_items = Vec::new();

    for &item_id in &trait_.items {
        if let Some(item) = data.index.get(&item_id) {
            match &item.inner {
                ItemEnum::Function(f) => {
                    if f.has_body {
                        provided_items.push(item_id);
                    } else {
                        required_items.push(item_id);
                    }
                }
                ItemEnum::AssocConst { value, .. } => {
                    if value.is_some() {
                        provided_items.push(item_id);
                    } else {
                        required_items.push(item_id);
                    }
                }
                ItemEnum::AssocType { type_, .. } => {
                    if type_.is_some() {
                        provided_items.push(item_id);
                    } else {
                        required_items.push(item_id);
                    }
                }
                _ => {} // Other item kinds are not typically "required" or "provided" in the same way
            }
        }
    }

    if !required_items.is_empty() {
        output.push_str(&format!("{} Required Items\n\n", "#".repeat(heading_level)));
        render_associated_item_group(
            output,
            &required_items,
            data,
            level + 1,
            "Required",
            link_resolver,
        );
    }

    if !provided_items.is_empty() {
        output.push_str(&format!("{} Provided Items\n\n", "#".repeat(heading_level)));
        render_associated_item_group(
            output,
            &provided_items,
            data,
            level + 1,
            "Provided",
            link_resolver,
        );
    }

    if !trait_.implementations.is_empty() {
        output.push_str(&format!("{} Implementors\n\n", "#".repeat(heading_level)));
        output.push_str("This trait is implemented for the following types:\n\n");
        for &impl_id in &trait_.implementations {
            if let Some(impl_item) = data.index.get(&impl_id) {
                if let ItemEnum::Impl(impl_details) = &impl_item.inner {
                    output.push_str(&format!("- `{}`", format_type(&impl_details.for_, data)));
                    if !impl_details.generics.params.is_empty() {
                        let mut generics_str = String::new();
                        format_generics(&mut generics_str, &impl_details.generics, data);
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

pub fn process_impl_details<F>(
    output: &mut String,
    impl_: &Impl,
    data: &Crate,
    level: usize,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    let heading_level = std::cmp::min(level, 6);
    let sub_heading_level = std::cmp::min(level + 1, 6);

    if !impl_.items.is_empty() {
        let mut assoc_types = Vec::new();
        let mut assoc_consts = Vec::new();
        let mut methods = Vec::new();

        for &item_id in &impl_.items {
            if let Some(item) = data.index.get(&item_id) {
                match &item.inner {
                    ItemEnum::AssocType { .. } => assoc_types.push(item_id),
                    ItemEnum::AssocConst { .. } => assoc_consts.push(item_id),
                    ItemEnum::Function(_) => methods.push(item_id),
                    _ => {} // Other kinds of associated items?
                }
            }
        }

        if !assoc_types.is_empty() || !assoc_consts.is_empty() || !methods.is_empty() {
            // Overall heading for all associated items from the impl block itself
            // This might be redundant if the impl block itself is already a heading.
            // output.push_str(&format!("{} Associated Items from Impl\n\n", "#".repeat(heading_level)));
        }

        if !assoc_types.is_empty() {
            output.push_str(&format!(
                "{} Associated Types\n\n",
                "#".repeat(sub_heading_level)
            ));
            for item_id in assoc_types {
                if let Some(assoc_item_ref) = data.index.get(&item_id) {
                    let resolved_assoc_info = ResolvedItemInfo {
                        original_item: assoc_item_ref,
                        effective_name: assoc_item_ref.name.clone(),
                        reexport_source_canonical_path: None,
                    };
                    crate::render_core::render_item_page(
                        output,
                        &resolved_assoc_info,
                        data,
                        sub_heading_level + 1,
                        link_resolver,
                    );
                }
            }
        }

        if !assoc_consts.is_empty() {
            output.push_str(&format!(
                "{} Associated Constants\n\n",
                "#".repeat(sub_heading_level)
            ));
            for item_id in assoc_consts {
                if let Some(assoc_item_ref) = data.index.get(&item_id) {
                    let resolved_assoc_info = ResolvedItemInfo {
                        original_item: assoc_item_ref,
                        effective_name: assoc_item_ref.name.clone(),
                        reexport_source_canonical_path: None,
                    };
                    crate::render_core::render_item_page(
                        output,
                        &resolved_assoc_info,
                        data,
                        sub_heading_level + 1,
                        link_resolver,
                    );
                }
            }
        }

        if !methods.is_empty() {
            output.push_str(&format!("{} Methods\n\n", "#".repeat(sub_heading_level)));
            for item_id in methods {
                if let Some(assoc_item_ref) = data.index.get(&item_id) {
                    let resolved_assoc_info = ResolvedItemInfo {
                        original_item: assoc_item_ref,
                        effective_name: assoc_item_ref.name.clone(),
                        reexport_source_canonical_path: None,
                    };
                    crate::render_core::render_item_page(
                        output,
                        &resolved_assoc_info,
                        data,
                        sub_heading_level + 1,
                        link_resolver,
                    );
                }
            }
        }
    }

    if impl_.trait_.is_some() && !impl_.provided_trait_methods.is_empty() {
        output.push_str(&format!(
            "{} Provided Trait Methods (Not Overridden)\n\n",
            "#".repeat(heading_level)
        ));
        for provided_method_name in &impl_.provided_trait_methods {
            // Try to find the original trait method for more details if possible,
            // otherwise just list the name.
            // This part might need more sophisticated logic if full signatures are desired here.
            output.push_str(&format!("- `{}`\n", provided_method_name));
        }
        output.push('\n');
    }

    if let Some(blanket_type) = &impl_.blanket_impl {
        output.push_str(&format!(
            "This is a blanket implementation for types matching: `{}`\n\n",
            format_type(blanket_type, data)
        ));
    }
}

// Helper function to render groups of associated items for traits
fn render_associated_item_group<F>(
    output: &mut String,
    item_ids: &[Id],
    data: &Crate,
    level: usize,
    _group_prefix: &str,
    link_resolver: F,
) where
    F: Fn(&Id) -> String + Copy,
{
    let mut assoc_types = Vec::new();
    let mut assoc_consts = Vec::new();
    let mut functions = Vec::new(); // Methods or associated functions

    for &item_id in item_ids {
        if let Some(item) = data.index.get(&item_id) {
            match &item.inner {
                ItemEnum::AssocType { .. } => assoc_types.push(item_id),
                ItemEnum::AssocConst { .. } => assoc_consts.push(item_id),
                ItemEnum::Function(_) => functions.push(item_id),
                _ => {}
            }
        }
    }

    let sub_heading_level = std::cmp::min(level, 6); // level is already incremented from call site

    if !assoc_types.is_empty() {
        output.push_str(&format!(
            "{} Associated Types\n\n",
            "#".repeat(sub_heading_level)
        ));
        for item_id in assoc_types {
            if let Some(item_ref) = data.index.get(&item_id) {
                let resolved_info = ResolvedItemInfo {
                    original_item: item_ref,
                    effective_name: item_ref.name.clone(),
                    reexport_source_canonical_path: None,
                };
                crate::render_core::render_item_page(
                    output,
                    &resolved_info,
                    data,
                    sub_heading_level + 1,
                    link_resolver,
                );
            }
        }
    }

    if !assoc_consts.is_empty() {
        output.push_str(&format!(
            "{} Associated Constants\n\n",
            "#".repeat(sub_heading_level)
        ));
        for item_id in assoc_consts {
            if let Some(item_ref) = data.index.get(&item_id) {
                let resolved_info = ResolvedItemInfo {
                    original_item: item_ref,
                    effective_name: item_ref.name.clone(),
                    reexport_source_canonical_path: None,
                };
                crate::render_core::render_item_page(
                    output,
                    &resolved_info,
                    data,
                    sub_heading_level + 1,
                    link_resolver,
                );
            }
        }
    }

    if !functions.is_empty() {
        output.push_str(&format!(
            "{} Functions/Methods\n\n",
            "#".repeat(sub_heading_level)
        ));
        for item_id in functions {
            if let Some(item_ref) = data.index.get(&item_id) {
                let resolved_info = ResolvedItemInfo {
                    original_item: item_ref,
                    effective_name: item_ref.name.clone(),
                    reexport_source_canonical_path: None,
                };
                crate::render_core::render_item_page(
                    output,
                    &resolved_info,
                    data,
                    sub_heading_level + 1,
                    link_resolver,
                );
            }
        }
    }
}
