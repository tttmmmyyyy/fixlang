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
        let src_files = proj_file.get_files(false);
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
            "Generating documentation for module \"{}\".",
            mod_name.to_string()
        );
        docgen_for_module(&program, &mod_name, docs_config)?;
    }
    Ok(())
}

struct MarkdownSection {
    title: String,
    paragraphs: Vec<String>,
    subsections: Vec<MarkdownSection>,
}

impl MarkdownSection {
    fn new(title: String) -> Self {
        Self {
            title,
            paragraphs: vec![],
            subsections: vec![],
        }
    }

    fn add_paragraph(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        self.paragraphs.push(text);
    }

    fn add_subsection(&mut self, section: MarkdownSection) {
        self.subsections.push(section);
    }

    fn add_subsections(&mut self, sections: Vec<MarkdownSection>) {
        self.subsections.extend(sections);
    }

    fn format(&self, section_level: usize, output: &mut String) {
        *output += &format!("{} {}", "#".repeat(section_level + 1), self.title);

        for paragraph in self.paragraphs.iter() {
            if paragraph.is_empty() {
                continue;
            }
            *output += "\n\n";
            *output += paragraph;
        }

        for subsection in self.subsections.iter() {
            *output += "\n\n";
            subsection.format(section_level + 1, output);
        }
    }
}

// Generate documentation for a Program consists of single module.
fn docgen_for_module(
    program: &Program,
    mod_name: &Name,
    config: &DocsConfig,
) -> Result<(), Errors> {
    // Check if the module exists in the program.
    if !program.modules.iter().any(|mi| mi.name == *mod_name) {
        return Err(Errors::from_msg(format!(
            "Module \"{}\" does not exist in the project.",
            mod_name
        )));
    }

    let markdown = write_module(program, mod_name, config)?;
    let mut markdown_str = String::new();
    markdown.format(0, &mut markdown_str);

    // Write `doc` into `{mod_name}.md` file.
    let doc_file = format!("{}.md", mod_name);

    // Create the output directory.
    if let Err(e) = std::fs::create_dir_all(&config.out_dir) {
        return Err(Errors::from_msg(format!(
            "Failed to create directory \"{}\": {:?}",
            config.out_dir.display(),
            e
        )));
    }
    let doc_path = config.out_dir.join(doc_file);
    std::fs::write(&doc_path, markdown_str).map_err(|e| {
        Errors::from_msg(format!(
            "Failed to write file \"{}\": {:?}",
            doc_path.display(),
            e
        ))
    })?;

    println!("Saved documentation to \"{}\".", doc_path.display());
    Ok(())
}

fn write_entries(mut entries: Vec<Entry>, doc: &mut MarkdownSection) {
    entries.sort();

    let mut last_ns = NameSpace::new(vec![]);
    let mut subsections: Vec<MarkdownSection> = vec![];
    for entry in entries {
        if entry.name.namespace != last_ns {
            last_ns = entry.name.namespace.clone();
            let title = format!("namespace {}", last_ns.to_string());
            subsections.push(MarkdownSection::new(title));
        }
        if let Some(current_section) = subsections.last_mut() {
            current_section.add_subsection(entry.doc);
        } else {
            doc.add_subsection(entry.doc);
        }
    }
    doc.add_subsections(subsections);
}

// Add the module name section to the documentation.
fn write_module(
    program: &Program,
    mod_name: &Name,
    config: &DocsConfig,
) -> Result<MarkdownSection, Errors> {
    // Add the module name section.
    let mut doc = MarkdownSection::new(format!("{}", mod_name));

    if let Some(mod_info) = program.modules.iter().find(|mi| mi.name == *mod_name) {
        let docstring = mod_info.source.get_document().ok().unwrap_or_default();
        let docstring = docstring.trim();
        doc.add_paragraph(docstring.to_string());
    }

    {
        let mut section = MarkdownSection::new("Values".to_string());
        let entries = value_entries(program, mod_name, config)?;
        write_entries(entries, &mut section);
        doc.add_subsection(section);
    }

    {
        let mut section = MarkdownSection::new("Types and aliases".to_string());
        let entries = type_entries(program, mod_name)?;
        write_entries(entries, &mut section);
        doc.add_subsection(section);
    }

    {
        let mut section = MarkdownSection::new("Traits and aliases".to_string());
        let entries = trait_entries(program, mod_name)?;
        write_entries(entries, &mut section);
        doc.add_subsection(section);
    }

    {
        let mut section = MarkdownSection::new("Trait implementations".to_string());
        let entries = trait_impl_entries(program, mod_name)?;
        write_entries(entries, &mut section);
        doc.add_subsection(section);
    }

    Ok(doc)
}

struct Entry {
    name: FullName,
    sort_key: String, // Additional key for sorting used when `name` is same.
    doc: MarkdownSection,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.sort_key == other.sort_key
    }
}

impl Eq for Entry {}

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

fn type_entries(program: &Program, mod_name: &Name) -> Result<Vec<Entry>, Errors> {
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

    let mut entries = vec![];

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

        let mut doc = MarkdownSection::new(name.name.clone());

        let defined_as = format!(
            "Defined as: `type {}{}{} = {} {}`",
            kind_constraints_with_post_space(&ty_info.tyvars),
            name.name,
            tyvars_with_pre_space(&ty_info.tyvars),
            box_or_unbox(ty_info.is_unbox),
            def_rhs,
        );
        doc.add_paragraph(defined_as);
        doc.add_paragraph(
            ty_info
                .get_document()
                .unwrap_or_default()
                .trim()
                .to_string(),
        );

        if ty_info.variant == TyConVariant::Struct {
            for field in ty_info.fields.iter() {
                let title = format!("field `{}`", field.name);
                let mut field_sec = MarkdownSection::new(title);
                field_sec.add_paragraph(format!(
                    "Type: `{}`",
                    field.syn_ty.as_ref().unwrap().to_string()
                ));
                doc.add_subsection(field_sec);
            }
        }
        if ty_info.variant == TyConVariant::Union {
            for variant in ty_info.fields.iter() {
                let title = format!("variant `{}`", variant.name);
                let mut variant_sec = MarkdownSection::new(title);
                variant_sec.add_paragraph(format!(
                    "Type: `{}`",
                    variant.syn_ty.as_ref().unwrap().to_string()
                ));
                doc.add_subsection(variant_sec);
            }
        }

        let entry = Entry {
            name: name.clone(),
            sort_key: "".to_string(),
            doc,
        };

        entries.push(entry);
    }
    for (ty_name, ty_info) in program.type_env.aliases.iter() {
        let name = ty_name.name.clone();

        if !is_entry_should_be_documented(&name, mod_name) {
            continue;
        }

        let mut doc = MarkdownSection::new(name.name.clone());
        let defined_as = format!(
            "Defined as: `type {}{}{} = {}`",
            kind_constraints_with_post_space(&ty_info.tyvars),
            name.name,
            tyvars_with_pre_space(&ty_info.tyvars),
            ty_info.value.to_string(),
        );
        doc.add_paragraph(defined_as);

        let content = &ty_info
            .source
            .as_ref()
            .map(|src| src.get_document())
            .transpose()?
            .unwrap_or_default();
        let content = content.trim();
        doc.add_paragraph(content.to_string());

        let entry = Entry {
            name: name.clone(),
            sort_key: "".to_string(),
            doc,
        };
        entries.push(entry);
    }
    Ok(entries)
}

fn trait_entries(program: &Program, mod_name: &Name) -> Result<Vec<Entry>, Errors> {
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

    let mut entries = vec![];

    for (id, info) in &program.trait_env.traits {
        let name = id.name.clone();

        if !is_entry_should_be_documented(&name, mod_name) {
            continue;
        }

        let kind_consts = kind_constraints_with_post_space(&info.kind_signs);
        let title = format!(
            "trait `{}{} : {}`",
            kind_consts, info.type_var.name, name.name
        );
        let mut doc = MarkdownSection::new(title);

        let docstring = &info
            .source
            .as_ref()
            .map(|src| src.get_document())
            .transpose()?
            .unwrap_or_default();
        let docstring = docstring.trim();
        doc.add_paragraph(docstring.to_string());

        for (assoc_ty_name, assoc_ty_defn) in &info.assoc_types {
            let mut params = vec![info.type_var.name.clone()];
            for param in assoc_ty_defn.params.iter().skip(1) {
                params.push(param.name.clone());
            }
            let title = format!("type `{}`", assoc_ty_name,);
            let mut subsection = MarkdownSection::new(title);
            let defined_as = format!(
                "Defined as: `{}{} {}{}`",
                kind_constraints_with_post_space(&assoc_ty_defn.kind_signs),
                assoc_ty_name,
                params.join(" "),
                kind_sign_with_pre_space(&assoc_ty_defn.kind_applied)
            );
            subsection.add_paragraph(defined_as);
            let docstring = assoc_ty_defn
                .src
                .as_ref()
                .map(|src| src.get_document())
                .transpose()?
                .unwrap_or_default();
            let docstring = docstring.trim();
            subsection.add_paragraph(docstring.to_string());
            doc.add_subsection(subsection);
        }
        for method in &info.methods {
            let title = format!("method `{}`", method.name);
            let mut subsection = MarkdownSection::new(title);
            subsection.add_paragraph(format!("Type: `{}`", method.qual_ty.to_string()));
            let docstring = docstring_from_opt_span(&method.source)?;
            subsection.add_paragraph(docstring);
            doc.add_subsection(subsection);
        }

        let entry = Entry {
            name: id.name.clone(),
            sort_key: "".to_string(),
            doc,
        };
        entries.push(entry);
    }

    Ok(entries)
}

fn trait_impl_entries(program: &Program, mod_name: &Name) -> Result<Vec<Entry>, Errors> {
    let mut entries = vec![];

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

            let title = format!("impl `{}`", impl_.qual_pred.to_string());
            let mut doc = MarkdownSection::new(title);

            doc.add_paragraph(docstring_from_opt_span(&impl_.source)?);

            let entry = Entry {
                name: FullName::from_strs(&[], ""),
                sort_key: impl_.qual_pred.predicate.to_string(),
                doc,
            };
            entries.push(entry);
        }
    }
    Ok(entries)
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
    config: &DocsConfig,
) -> Result<Vec<Entry>, Errors> {
    let mut entries = vec![];

    for (name, gv) in &program.global_values {
        if !is_entry_should_be_documented(&name, mod_name) {
            continue;
        }
        if gv.compiler_defined_method && !config.include_compiler_defined_methods {
            continue;
        }

        let mut doc = MarkdownSection::new(name.name.clone());

        doc.add_paragraph(format!(
            "Type: `{}`",
            gv.syn_scm.as_ref().unwrap().to_string()
        ));
        doc.add_paragraph(gv.get_document().unwrap_or_default().trim().to_string());

        let entry = Entry {
            name: name.clone(),
            sort_key: "".to_string(),
            doc,
        };
        entries.push(entry);
    }

    Ok(entries)
}

fn box_or_unbox(is_unbox: bool) -> &'static str {
    if is_unbox {
        "unbox"
    } else {
        "box"
    }
}
