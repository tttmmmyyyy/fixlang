use std::sync::Arc;

use crate::{
    error::Errors, kind_star, lsp::language_server::run_diagnostics, FullName, Kind, KindSignature,
    Name, NameSpace, Program, TyConVariant, TyVar,
};

pub fn generate_docs_for_files(mod_names: &[Name]) -> Result<(), Errors> {
    println!("Running diagnostics for this Fix project.");
    let program = run_diagnostics(None)?.prgoram;
    println!("Diagnostics completed.");
    for mod_name in mod_names {
        println!(
            "Generating documentation for module `{}`",
            mod_name.to_string()
        );
        generate_doc(&program, mod_name)?;
    }
    Ok(())
}

// Generate documentation for a Program consists of single module.
fn generate_doc(program: &Program, mod_name: &Name) -> Result<(), Errors> {
    // Check if the module exists in the program.
    if !program.modules.iter().any(|mi| mi.name == *mod_name) {
        return Err(Errors::from_msg(format!(
            "Module `{}` does not exist in the program.",
            mod_name
        )));
    }

    let mut doc = String::new();

    // The module name section.
    write_module_section(program, mod_name, &mut doc);

    let mut entries = vec![];

    doc += "\n\n# Types and aliases";
    type_entries(program, mod_name, &mut entries)?;
    write_entries(&mut entries, &mut doc);

    doc += "\n\n# Traits and aliases";
    trait_entries(program, mod_name, &mut entries)?;
    write_entries(&mut entries, &mut doc);

    doc += "\n\n# Trait implementations";
    trait_impl_entries(program, mod_name, &mut entries)?;
    write_entries(&mut entries, &mut doc);

    doc += "\n\n# Values";
    value_entries(program, mod_name, &mut entries)?;
    write_entries(&mut entries, &mut doc);

    // Write `doc` into `{mod_name}.md` file.
    let doc_file = format!("{}.md", mod_name);
    std::fs::write(&doc_file, doc)
        .map_err(|e| Errors::from_msg(format!("Failed to write file \"{}\": {:?}", doc_file, e)))?;

    println!("Saved documentation to \"{}\"", doc_file);
    Ok(())
}

fn write_entries(entries: &mut Vec<Entry>, doc: &mut String) {
    entries.sort();
    let mut last_ns = NameSpace::new(vec![]);

    for entry in entries.iter() {
        if entry.name.namespace != last_ns {
            last_ns = entry.name.namespace.clone();
            *doc += format!("\n\n## `namespace {}`", last_ns.to_string()).as_str();
        }
        *doc += format!("\n\n### {}", entry.title).as_str();
        let doc_trim = entry.doc.trim();
        if !doc_trim.is_empty() {
            *doc += "\n\n";
            *doc += doc_trim;
        }
    }

    entries.clear();
}

// Add the module name section to the documentation.
fn write_module_section(program: &Program, mod_name: &Name, doc: &mut String) {
    *doc += format!("# `module {}`", mod_name).as_str();
    if let Some(mod_info) = program.modules.iter().find(|mi| mi.name == *mod_name) {
        let docstring = mod_info.source.get_document().ok().unwrap_or_default();
        let docstring = docstring.trim();
        if !docstring.is_empty() {
            *doc += "\n\n";
            *doc += docstring;
        }
    }
}

#[derive(PartialEq, Eq)]
struct Entry {
    name: FullName,
    sort_key: String, // Additional key for sorting used when `name` is same.
    title: String,
    doc: String,
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.name.namespace != other.name.namespace {
            return self.name.namespace.cmp(&other.name.namespace);
        }
        if self.name != other.name {
            return self.name.cmp(&other.name);
        }
        self.sort_key.cmp(&other.sort_key)
    }
}

#[allow(dead_code)]
fn to_markdown_link(header: &str) -> String {
    let mut link = header.to_lowercase();
    link = link.replace(" ", "-");
    link.retain(|c| c.is_ascii_alphanumeric() || c == '-');
    link
}

fn is_entry_should_be_documented(name: &FullName, mod_name: &Name) -> bool {
    if &name.module() != mod_name {
        return false;
    }
    if name.to_string().contains("#") {
        return false;
    }
    true
}

fn type_entries(
    program: &Program,
    mod_name: &Name,
    entries: &mut Vec<Entry>,
) -> Result<(), Errors> {
    fn kind_constraints_with_post_space(tyvars: &Vec<Arc<TyVar>>) -> String {
        if tyvars.is_empty() {
            return String::new();
        }
        let mut consts = vec![];
        for tyvar in tyvars.iter() {
            if tyvar.kind == kind_star() {
                continue;
            }
            consts.push(format!("{} : {}", tyvar.name, tyvar.kind.to_string()));
        }
        if consts.is_empty() {
            return String::new();
        }
        format!("[{}] ", consts.join(", "))
    }
    #[allow(dead_code)]
    fn kind_specification_with_pre_space(kind: &Arc<Kind>) -> String {
        if kind == &kind_star() {
            return String::new();
        }
        format!(" : {}", kind.to_string())
    }
    fn tyvars_with_pre_space(tyvars: &Vec<Arc<TyVar>>) -> String {
        if tyvars.is_empty() {
            return String::new();
        }
        format!(
            " {}",
            tyvars
                .iter()
                .map(|tyvar| tyvar.name.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }

    for (ty_name, ty_info) in program.type_env.tycons.iter() {
        let name = ty_name.name.clone();

        if !is_entry_should_be_documented(&name, mod_name) {
            continue;
        }

        // Skip dynamic object type
        if ty_info.variant == TyConVariant::DynamicObject {
            continue;
        }

        let def_rhs: &str = match &ty_info.variant {
            TyConVariant::Primitive => "{ primitive }",
            TyConVariant::Array => "{ primitive }",
            TyConVariant::Struct => "struct { ...fields... }",
            TyConVariant::Union => "union { ...variants... }",
            TyConVariant::DynamicObject => {
                unreachable!()
            }
        };
        let title = format!(
            "`type {}{}{} = {} {}`",
            kind_constraints_with_post_space(&ty_info.tyvars),
            name.name,
            tyvars_with_pre_space(&ty_info.tyvars),
            box_or_unbox(ty_info.is_unbox),
            def_rhs,
        );

        let mut doc = String::new();

        let docstring = &ty_info
            .source
            .as_ref()
            .map(|src| src.get_document())
            .transpose()?
            .unwrap_or_default();
        let docstring = docstring.trim();
        if !docstring.is_empty() {
            doc += &format!("\n\n{}", docstring);
        }

        if ty_info.variant == TyConVariant::Struct {
            for field in ty_info.fields.iter() {
                doc += &format!("\n\n#### field `{} : {}`", field.name, field.ty.to_string(),);
            }
        }
        if ty_info.variant == TyConVariant::Union {
            for variant in ty_info.fields.iter() {
                doc += &format!(
                    "\n\n#### variant `{} : {}`",
                    variant.name,
                    variant.ty.to_string(),
                );
            }
        }

        let entry = Entry {
            name: name.clone(),
            sort_key: "".to_string(),
            title,
            doc,
        };

        entries.push(entry);
    }
    for (ty_name, ty_info) in program.type_env.aliases.iter() {
        let name = ty_name.name.clone();

        if !is_entry_should_be_documented(&name, mod_name) {
            continue;
        }

        let title = format!(
            "`type {}{} = {}`",
            kind_constraints_with_post_space(&ty_info.tyvars),
            name.name,
            ty_info.value.to_string(),
        );

        let mut doc = String::new();
        let docstring = &ty_info
            .source
            .as_ref()
            .map(|src| src.get_document())
            .transpose()?
            .unwrap_or_default();
        let docstring = docstring.trim();
        doc += docstring;

        let entry = Entry {
            name: name.clone(),
            sort_key: "".to_string(),
            title,
            doc,
        };
        entries.push(entry);
    }
    Ok(())
}

fn trait_entries(
    program: &Program,
    mod_name: &Name,
    entries: &mut Vec<Entry>,
) -> Result<(), Errors> {
    fn kind_constraints_with_post_space(kind_signs: &Vec<KindSignature>) -> String {
        if kind_signs.is_empty() {
            return String::new();
        }
        let mut consts = vec![];
        for kind_sign in kind_signs.iter() {
            if kind_sign.kind == kind_star() {
                continue;
            }
            consts.push(kind_sign.to_string());
        }
        if consts.is_empty() {
            return String::new();
        }
        format!("[{}] ", consts.join(", "))
    }

    for (id, info) in &program.trait_env.traits {
        let name = id.name.clone();

        if !is_entry_should_be_documented(&name, mod_name) {
            continue;
        }

        let kind_consts = kind_constraints_with_post_space(&info.kind_signs);
        let title = format!(
            "`trait {}{} : {}`",
            kind_consts, info.type_var.name, name.name
        );

        let mut doc = String::new();
        let docstring = &info
            .source
            .as_ref()
            .map(|src| src.get_document())
            .transpose()?
            .unwrap_or_default();
        let docstring = docstring.trim();
        doc += docstring;
        for method in &info.methods {
            doc += &format!(
                "\n\n#### method `{} : {}`",
                method.name,
                method.qual_ty.to_string(),
            );
            let docstring = method
                .source
                .as_ref()
                .map(|src| src.get_document())
                .transpose()?
                .unwrap_or_default();
            let docstring = docstring.trim();
            if !docstring.is_empty() {
                doc += &format!("\n\n{}", docstring);
            }
        }

        let entry = Entry {
            name: id.name.clone(),
            sort_key: "".to_string(),
            title,
            doc,
        };
        entries.push(entry);
    }
    Ok(())
}

fn trait_impl_entries(
    program: &Program,
    mod_name: &Name,
    entries: &mut Vec<Entry>,
) -> Result<(), Errors> {
    for (_id, impls) in &program.trait_env.instances {
        for impl_ in impls {
            if &impl_.define_module != mod_name {
                continue;
            }

            let title = format!("`impl {}`", impl_.qual_pred.to_string());

            let mut doc = String::new();
            let docstring = impl_
                .source
                .as_ref()
                .map(|src| src.get_document())
                .transpose()?
                .unwrap_or_default();
            let docstring = docstring.trim();
            doc += docstring;

            let entry = Entry {
                name: FullName::from_strs(&[], ""),
                sort_key: impl_.qual_pred.predicate.to_string(),
                title,
                doc,
            };
            entries.push(entry);
        }
    }
    Ok(())
}

fn value_entries(
    program: &Program,
    mod_name: &Name,
    entries: &mut Vec<Entry>,
) -> Result<(), Errors> {
    for (name, gv) in &program.global_values {
        if !is_entry_should_be_documented(&name, mod_name) {
            continue;
        }

        let title = format!("`{} : {}`", name.name, gv.scm.to_string());

        let mut doc = String::new();
        doc += gv.get_document().unwrap_or_default().trim();

        let entry = Entry {
            name: name.clone(),
            sort_key: "".to_string(),
            title,
            doc,
        };
        entries.push(entry);
    }
    Ok(())
}

fn box_or_unbox(is_unbox: bool) -> &'static str {
    if is_unbox {
        "unbox"
    } else {
        "box"
    }
}
