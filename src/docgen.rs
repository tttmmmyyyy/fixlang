use std::sync::Arc;

use crate::{
    ast::name::{FullName, Name, NameSpace},
    build_file,
    error::Errors,
    kind_star,
    misc::to_absolute_path,
    project_file::ProjectFile,
    Configuration, DocsConfig, Kind, KindSignature, Program, Span, TyConVariant, TyVar,
};

pub fn generate_docs_for_files(mut config: Configuration) -> Result<(), Errors> {
    println!("Loading source files...");

    // Set up the configuration by the project file.
    let proj_file = ProjectFile::read_root_file()?;
    proj_file.set_config(&mut config, false)?;

    // Set up the configuration by the lock file.
    proj_file.open_lock_file()?.set_config(&mut config)?;

    // Build the file and get the errors.
    let program = build_file(&mut config)?.program.unwrap();
    println!("Generating documentation...");

    let docs_config = match &config.subcommand {
        crate::SubCommand::Docs(docs_config) => docs_config,
        _ => unreachable!(),
    };

    // Determine modules to generate documentation.
    let mod_names = if docs_config.modules.len() > 0 {
        // In case modules are given in the command line arguments, use them.
        docs_config.modules.clone()
    } else {
        let mut mod_names = vec![];
        // Use all modules defined in the root project file.
        let src_files = proj_file.get_files(true);
        let abs_src_paths = src_files
            .iter()
            .map(|f| to_absolute_path(f))
            .collect::<Vec<_>>();
        for mi in program.modules.iter() {
            let src_file = to_absolute_path(&mi.source.input.file_path);
            if abs_src_paths.iter().any(|f| f == &src_file) {
                mod_names.push(mi.name.clone());
            }
        }
        mod_names
    };

    for mod_name in mod_names {
        println!(
            "Generating documentation for module `{}`.",
            mod_name.to_string()
        );
        generate_doc(&program, &mod_name, docs_config)?;
    }
    Ok(())
}

// Generate documentation for a Program consists of single module.
fn generate_doc(program: &Program, mod_name: &Name, config: &DocsConfig) -> Result<(), Errors> {
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
    value_entries(program, mod_name, &mut entries, config)?;
    write_entries(&mut entries, &mut doc);

    // Write `doc` into `{mod_name}.md` file.
    let doc_file = format!("{}.md", mod_name);
    let doc_path = config.out_dir.join(doc_file);
    std::fs::write(&doc_path, doc).map_err(|e| {
        Errors::from_msg(format!(
            "Failed to write file \"{}\": {:?}",
            doc_path.display(),
            e
        ))
    })?;

    println!("Saved documentation to \"{}\".", doc_path.display());
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

// Creates string of kind signature with pre-space, e.e, " : * -> *".
// If the kind is `*`, returns empty string.
fn kind_sign_with_pre_space(kind: &Arc<Kind>) -> String {
    if kind == &kind_star() {
        return String::new();
    }
    format!(" : {}", kind.to_string())
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
            TyConVariant::Primitive => "{ built-in }",
            TyConVariant::Array => "{ built-in }",
            TyConVariant::Arrow => "{ built-in }",
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

        let docstring = ty_info.get_document().unwrap_or_default();
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
        for (assoc_ty_name, assoc_ty_defn) in &info.assoc_types {
            let mut params = vec![info.type_var.name.clone()];
            for param in assoc_ty_defn.params.iter().skip(1) {
                params.push(param.name.clone());
            }
            doc += &format!(
                "\n\n#### associated type `{}{} {}{}`",
                kind_constraints_with_post_space(&assoc_ty_defn.kind_signs),
                assoc_ty_name,
                params.join(" "),
                kind_sign_with_pre_space(&assoc_ty_defn.kind_applied)
            );
            let docstring = assoc_ty_defn
                .src
                .as_ref()
                .map(|src| src.get_document())
                .transpose()?
                .unwrap_or_default();
            let docstring = docstring.trim();
            if !docstring.is_empty() {
                doc += &format!("\n\n{}", docstring);
            }
        }
        for method in &info.methods {
            doc += &format!(
                "\n\n#### method `{} : {}`",
                method.name,
                method.qual_ty.to_string(),
            );
            let docstring = docstring_from_opt_span(&method.source)?;
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
            // Skip impls for compiler-generated types / traits.
            let impl_ty_str = impl_.impl_type().to_string_normalize();
            if impl_ty_str.contains("#") {
                continue;
            }

            let title = format!("`impl {}`", impl_.qual_pred.to_string());

            let mut doc = String::new();
            doc += &docstring_from_opt_span(&impl_.source)?;

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

fn docstring_from_opt_span(src: &Option<Span>) -> Result<String, Errors> {
    Ok(src
        .as_ref()
        .map(|src| src.get_document())
        .transpose()?
        .unwrap_or_default()
        .trim()
        .to_string())
}

fn value_entries(
    program: &Program,
    mod_name: &Name,
    entries: &mut Vec<Entry>,
    config: &DocsConfig,
) -> Result<(), Errors> {
    for (name, gv) in &program.global_values {
        if !is_entry_should_be_documented(&name, mod_name) {
            continue;
        }
        if gv.compiler_defined_method && !config.include_compiler_defined_methods {
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
