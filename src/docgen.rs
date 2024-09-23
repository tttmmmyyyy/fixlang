use std::{path::PathBuf, sync::Arc};

use crate::{
    error::Errors, kind_star, make_std_mod, parse_file_path, Configuration, FullName, Kind,
    KindSignature, Name, NameSpace, Program, TyCon, TyConVariant, TyVar, TypeDeclValue,
};

pub fn generate_docs_for_files(files: &[PathBuf]) -> Result<(), Errors> {
    let config = Configuration::release();
    for file in files {
        let mut program = if file == &PathBuf::from("std.fix") {
            make_std_mod(&config)
        } else {
            parse_file_path(file.clone(), &config)
        }?;
        program.calculate_type_env()?;
        generate_doc(&program)?;
    }
    Ok(())
}

// Generate documentation for a Program consists of single module.
fn generate_doc(program: &Program) -> Result<(), Errors> {
    let mut doc = String::new();

    // The module name section.
    assert!(program.module_to_files.len() == 1);
    let mod_name = mod_name_section(program, &mut doc);

    let mut entries = vec![];

    type_entries(program, &mut entries)?;
    trait_entries(program, &mut entries)?;

    write_entries(entries, &mut doc, mod_name.clone());

    // Write `doc` into `{mod_name}.md` file.
    let doc_file = format!("{}.md", mod_name);
    std::fs::write(&doc_file, doc)
        .map_err(|e| Errors::from_msg(format!("Failed to write file \"{}\": {:?}", doc_file, e)))?;
    Ok(())
}

fn write_entries(mut entries: Vec<Entry>, doc: &mut String, mod_name: Name) {
    entries.sort();
    let mut last_ns = NameSpace::new(vec![]);

    for entry in entries {
        if entry.namespace != last_ns {
            last_ns = entry.namespace.clone();
            *doc += format!("\n## `namespace {}`\n", last_ns.to_string()).as_str();
        }
        *doc += format!("\n### {}\n\n", entry.title).as_str();
        *doc += format!("{}", entry.doc).as_str();
    }
}

// Add the module name section to the documentation.
// Return the module name.
fn mod_name_section(program: &Program, doc: &mut String) -> String {
    assert!(program.module_to_files.len() == 1);
    let (mod_name, _src) = program.module_to_files.iter().next().unwrap();
    *doc += format!("# `module {}`\n", mod_name).as_str();
    mod_name.clone()
}

#[derive(PartialEq, Eq)]
struct Entry {
    namespace: NameSpace,
    kind: EntryKind,
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
        if self.namespace < other.namespace {
            return std::cmp::Ordering::Less;
        } else if self.namespace > other.namespace {
            return std::cmp::Ordering::Greater;
        } else {
            return self.kind.cmp(&other.kind);
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum EntryKind {
    Type,
    Trait,
    Value,
}

fn to_markdown_link(header: &str) -> String {
    let mut link = header.to_lowercase();
    link = link.replace(" ", "-");
    link.retain(|c| c.is_ascii_alphanumeric() || c == '-');
    link
}

// Add types, type aliases and associated namespaces sections to the documentation.
fn type_entries(program: &Program, entries: &mut Vec<Entry>) -> Result<(), Errors> {
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
        // Skip types contains with "#".
        if name.name.contains("#") {
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
            // kind_specification_with_pre_space(&ty_info.kind)
        );

        let mut doc = String::new();
        doc += &format!(
            "[See related values](#{})\n\n",
            to_markdown_link(&format!("namespace `{}`", name.to_namespace().to_string()))
        );

        doc += &ty_info
            .source
            .as_ref()
            .map(|src| src.get_document())
            .transpose()?
            .unwrap_or_default();

        let entry = Entry {
            namespace: name.namespace.clone(),
            kind: EntryKind::Type,
            title,
            doc,
        };

        entries.push(entry);
    }
    Ok(())
}

fn trait_entries(program: &Program, entries: &mut Vec<Entry>) -> Result<(), Errors> {
    fn kind_constraints_with_space(kind_signs: &Vec<KindSignature>) -> String {
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
        let kind_consts = kind_constraints_with_space(&info.kind_signs);
        let title = format!("`trait {}{}`", kind_consts, name.name);

        let mut doc = String::new();
        doc += &info
            .source
            .as_ref()
            .map(|src| src.get_document())
            .transpose()?
            .unwrap_or_default();
        for method in &info.methods {
            doc += &format!(
                "\n\n#### `{} : {}`\n\n{}",
                method.name,
                method.qual_ty.to_string(),
                method
                    .source
                    .as_ref()
                    .map(|src| src.get_document())
                    .transpose()?
                    .unwrap_or_default()
            );
        }

        let entry = Entry {
            namespace: id.name.namespace.clone(),
            kind: EntryKind::Trait,
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
