// Module for the "fix edit explicit-import" command.
// This command rewrites import statements in a Fix project to import only the necessary entities explicitly.

use crate::ast::import::ImportStatement;
use crate::ast::name::{FullName, Name};
use crate::ast::program::{Program, SymbolExpr};
use crate::edit::edit_util::apply_text_edits;
use crate::error::Errors;
use crate::lsp::language_server::{
    create_text_edit_to_insert_imports, create_text_edits_to_erase_imports, run_diagnostics,
};
use crate::misc::to_absolute_path;
use crate::misc::Set;
use crate::project_file::ProjectFile;
use crate::typecheckcache::MemoryCache;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

// Run the "fix edit explicit-import" command.
// This command:
// 1. Checks if the project has any errors (exits if errors exist)
// 2. For each source file in the project (excluding dependencies):
//    - Collects all referenced type names, trait names, and value names
//    - Removes all existing import statements
//    - Writes explicit import statements for all collected names
pub fn run_explicit_import_command() -> Result<(), Errors> {
    println!("Running diagnostics on the program...");

    // Read the project file to get the list of source files.
    let proj_file = ProjectFile::read_root_file()?;

    // Run diagnostics to check if the project has errors and get the Program.
    let typecheck_cache = Arc::new(MemoryCache::new());
    let result = run_diagnostics(typecheck_cache)?;
    let program = result.program;
    if program.deferred_errors.has_error() {
        return Err(program.deferred_errors);
    }

    // Get the list of source files in the project (excluding dependencies).
    let user_files = get_user_source_files(&proj_file)?;

    // For each source file, collect referenced names and rewrite import statements.
    for file_path in user_files {
        rewrite_imports_for_file(&file_path, &program)?;
    }

    Ok(())
}

// Get the list of source files that belong to the project (not dependencies).
fn get_user_source_files(proj_file: &ProjectFile) -> Result<Vec<PathBuf>, Errors> {
    // Use get_files(true) to get the root project's source files (excluding dependencies).
    // The parameter true includes test files.
    let files = proj_file.get_files(true);

    // Convert to absolute paths
    let mut abs_files = Vec::new();
    for file_path in files {
        let abs_path = to_absolute_path(&file_path).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to get absolute path for {}: {}",
                file_path.display(),
                e
            ))
        })?;
        abs_files.push(abs_path);
    }

    Ok(abs_files)
}

// Rewrite import statements for a single file.
fn rewrite_imports_for_file(file_path: &PathBuf, program: &Program) -> Result<(), Errors> {
    // Find the module info for this file.
    let module_info = program
        .modules
        .iter()
        .find(|mi| {
            let mi_path = to_absolute_path(&mi.source.input.file_path).ok();
            mi_path.as_ref() == Some(file_path)
        })
        .ok_or_else(|| {
            Errors::from_msg(format!(
                "Could not find module info for {}",
                file_path.display()
            ))
        })?;

    let mod_name = &module_info.name;

    // Collect all names referenced in this module.
    let referenced_names = collect_referenced_names(program, mod_name);

    // Filter out names that are defined in the same module.
    let mut names_to_import: Vec<FullName> = referenced_names
        .into_iter()
        .filter(|name| &name.module() != mod_name && !name.is_absolute())
        .collect();
    names_to_import.sort();
    names_to_import.dedup();

    // Generate new import statements.
    let new_import_stmts = generate_import_statements(mod_name.clone(), names_to_import);

    // Read the file content.
    let content = fs::read_to_string(file_path).map_err(|e| {
        Errors::from_msg(format!(
            "Failed to read file {}: {}",
            file_path.display(),
            e
        ))
    })?;

    // Rewrite the file with new import statements.
    let new_content = rewrite_file_content(&content, &module_info, &new_import_stmts, program)?;

    // Write the new content back to the file.
    fs::write(file_path, new_content).map_err(|e| {
        Errors::from_msg(format!(
            "Failed to write file {}: {}",
            file_path.display(),
            e
        ))
    })?;

    println!("Rewrote imports for: {}", file_path.display());

    Ok(())
}

// Collect all names (types, traits, values) referenced in a module.
fn collect_referenced_names(program: &Program, mod_name: &Name) -> Set<FullName> {
    let mut names = Set::default();

    // Collect names referenced by global values in this module.
    for (full_name, gv) in &program.global_values {
        // Check if this global value is defined in the module we're processing.
        if &full_name.module() != mod_name {
            continue;
        }

        match &gv.expr {
            SymbolExpr::Simple(typed_expr) => {
                // Collect all referenced names (values, types, traits) from the expression
                typed_expr.expr.collect_referenced_names(&mut names);
            }
            SymbolExpr::Method(_) => {
                // Trait member implementations are handled separately
            }
        }

        // Collect referenced type names from the type of the global value.
        // This also includes types of members defined in trait definitions.
        gv.scm.collect_referenced_names(&mut names);
    }

    // Collect names referenced by trait implementations defined in this module.
    for impls in program.trait_env.impls.values() {
        for impl_ in impls {
            if &impl_.define_module != mod_name {
                continue;
            }
            // Collect names from the qualified predicate (including constraints).
            impl_.qual_pred.collect_referenced_names(&mut names);

            // Collect names from method implementations.
            for (_method_name, method_expr) in &impl_.members {
                method_expr.collect_referenced_names(&mut names);
            }

            // Collect types from method signatures.
            for (_method_name, qual_type) in &impl_.member_sigs {
                qual_type.collect_referenced_names(&mut names);
            }
        }
    }

    // Collect names from trait alias definitions in this module.
    for (trait_id, trait_alias) in &program.trait_env.aliases.data {
        // Check if this trait alias is defined in the module.
        if &trait_id.name.module() != mod_name {
            continue;
        }
        // Collect the aliased trait names.
        for (aliased_trait_id, _span) in &trait_alias.value {
            names.insert(aliased_trait_id.name.clone());
        }
    }

    // Collect names from type definitions (struct/union fields) in this module.
    for (tycon, tycon_info) in program.type_env.tycons.iter() {
        // Check if this type is defined in the module.
        if &tycon.name.module() != mod_name {
            continue;
        }
        // Collect type names from field types.
        for field in &tycon_info.fields {
            // Collect from the syntactic type.
            if let Some(syn_ty) = &field.syn_ty {
                syn_ty.collect_referenced_names(&mut names);
            }
        }
    }

    // Collect names from type alias definitions in this module.
    for (tycon, alias_info) in program.type_env.aliases.iter() {
        // Check if this type alias is defined in the module.
        if &tycon.name.module() != mod_name {
            continue;
        }
        // Collect type names from the alias value.
        alias_info.value.collect_referenced_names(&mut names);
    }

    names.into_iter().filter(|name| name.is_global()).collect()
}

// Generate import statements for the given names.
pub fn generate_import_statements(
    current_module: Name,
    names: Vec<FullName>,
) -> Vec<ImportStatement> {
    let mut import_stmts = Vec::new();

    for name in names {
        ImportStatement::add_import(&mut import_stmts, current_module.clone(), name);
    }

    ImportStatement::sort(&mut import_stmts);
    import_stmts
}

// Rewrite the file content by removing old import statements and inserting new ones.
fn rewrite_file_content(
    content: &str,
    module_info: &crate::ast::program::ModuleInfo,
    new_import_stmts: &[ImportStatement],
    program: &Program,
) -> Result<String, Errors> {
    // Find the range of existing import statements.
    let empty_vec = vec![];
    let import_stmts = program
        .mod_to_import_stmts
        .get(&module_info.name)
        .unwrap_or(&empty_vec);

    let mut text_edits = vec![];

    // Erase all existing import statements.
    text_edits.extend(create_text_edits_to_erase_imports(content, import_stmts));

    // Insert the import statement at the end of the module definition.
    text_edits.push(create_text_edit_to_insert_imports(
        module_info,
        new_import_stmts,
    ));

    // Apply the text edits to the content.
    let result = apply_text_edits(content, &text_edits);

    Ok(result)
}
