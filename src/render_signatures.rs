use crate::rustdoc_json_types::*;

pub fn format_item_signature(output: &mut String, item: &Item, data: &ParsedCrateDoc) {
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
            } else {
                output.push(' '); // Space after "impl<...>" for inherent impls
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
                        output.push_str("_derive"); // No closing ] yet for helpers
                        if !proc_macro.helpers.is_empty() {
                            output.push_str("(");
                            for (i, helper) in proc_macro.helpers.iter().enumerate() {
                                output.push_str(helper);
                                if i < proc_macro.helpers.len() - 1 {
                                    output.push_str(", ");
                                }
                            }
                            output.push_str(")");
                        }
                        output.push(']');
                    }
                }
                output.push_str(&format!(
                    "\npub fn {}(/* ... */) -> /* ... */ {{ /* ... */ }}", // Simplified body
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
                output.push_str(&format!("extern type {};", name));
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

pub fn format_generics(output: &mut String, generics: &Generics, data: &ParsedCrateDoc) {
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

pub fn format_where_clause(
    output: &mut String,
    predicates: &[WherePredicate],
    data: &ParsedCrateDoc,
) {
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
        } else {
            output.push(' '); // Space after the last predicate before potential body
        }
    }
}

pub fn format_bounds(output: &mut String, bounds: &[GenericBound], data: &ParsedCrateDoc) {
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

pub fn format_generic_args(output: &mut String, args: &GenericArgs, data: &ParsedCrateDoc) {
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
                let mut args_str = String::new();
                format_generic_args(&mut args_str, &constraint.args, data);
                if !args_str.is_empty() && args_str != "<>" {
                    output.push_str(&args_str);
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

pub fn format_type(ty: &Type, data: &ParsedCrateDoc) -> String {
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

            let mut args_str = String::new();
            format_generic_args(&mut args_str, args, data);
            if args_str != "<>" && !args_str.is_empty() {
                output.push_str(&args_str);
            }
        }
    }

    output
}
