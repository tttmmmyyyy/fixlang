pub mod check_holes;
pub mod desugar_opaque;
pub mod name_resolution;
pub mod typecheck;
pub mod typecheckcache;

use crate::ast::program::Program;
use crate::configuration::{Configuration, OutputFileType, SubCommand};
use crate::error::Errors;
use crate::fixstd::stdlib::{make_std_mod, make_tuple_traits_mod};
use crate::parse::parser::parse_file_path;
use crate::tool::stopwatch::StopWatch;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{fs::create_dir_all, path::PathBuf};

/// Perform validations and type checking on the program, and return the updated program.
/// Changes made to the program include instantiation of symbols and setting of entry points.
// PROOF: P1, P2, P5, P6, P7 (dev-docs/proof/rc_ir/borrow-cancel)
fn elaborate(mut program: Program, config: &Configuration) -> Result<Program, Errors> {
    let _sw = StopWatch::new("check_program", config.show_build_times);

    // Add tuple definitions.
    program.add_tuple_defns();

    // Add trait implementations for tuples such as ToString or Eq.
    program.link(
        make_tuple_traits_mod(&program.used_tuple_sizes, &config)?,
        true,
    )?;

    // Validate export statements.
    program.validate_export_statements(config.output_file_type)?;

    // Identify `DEPRECATED[...]` targets and attach `DeprecationInfo` to
    // matching global values / trait members. Run before
    // `create_trait_member_symbols` so that trait-member deprecation
    // propagates naturally into the per-impl `GlobalValue`s.
    program.identify_deprecation_targets()?;

    // Calculate list of type constructors.
    program.calculate_type_env()?;

    // Validate name confliction between types, traits and global values.
    program.validate_capital_name_confliction()?;

    // Infer namespaces of traits and types that appear in declarations and associated type implementations.
    program.resolve_namespace_not_in_expr()?;

    // Resolve type aliases that appear in declarations and associated type implementations.
    program.resolve_type_aliases_not_in_expr()?;

    // Validate user-defined types.
    program.validate_type_defns()?;

    // Add struct / union methods
    program.add_methods()?;

    // Add `Std::Boxed` trait implementations.
    program.add_boxed_impls()?;

    // Validate the traits, the trait aliases and the trait implementations, structurally.
    program.validate_trait_env_structure()?;

    // Create symbols.
    program.create_trait_member_symbols()?;

    // Validate constraints of global value type.
    program.validate_global_value_type_constraints()?;

    // Check if all items referred in import statements are defined.
    // This check should be done after `add_methods` and `create_trait_member_symbols`.
    program.validate_import_statements()?;

    // Set and check kinds that appear in type signatures.
    // NOTE: kinds of type variables appearing in type annotations in expressions are set at the
    // type inference stage.
    program.set_kinds()?;

    // Check that no two implementations of one trait can apply to the same type.
    // Runs after `set_kinds`, since which types an instance head denotes depends on the kinds of
    // the type variables in it.
    program.validate_overlapping_instances()?;

    // If typechecking is not needed, return here.
    if !config.subcommand.typecheck() {
        assert!(!config.subcommand.build_binary());
        return Ok(program);
    }

    // Desugar opaque type variables before type-checking.
    program.desugar_opaque_types();

    let typechecker = program.create_typechecker(config);

    // When running diagnostics, perform type checking of target modules and return here.
    if let SubCommand::Diagnostics(diag_config) = &config.subcommand {
        let _sw = StopWatch::new("typecheck", config.show_build_times);
        let target_module_names = program.modules_from_files(&diag_config.files)?;
        let mut errors = Errors::empty();
        errors.eat_err(program.resolve_namespace_and_check_type_in_modules(
            &typechecker,
            &target_module_names,
            diag_config.target_symbols.as_deref(),
            config,
        ));
        program.deferred_errors.append(errors);
        program
            .deferred_errors
            .append(program.collect_deprecation_diagnostics(config));
        return Ok(program);
    }

    // Perform namespace resolution and type-checking for all modules upfront.
    // This ensures opaque type resolutions are available before instantiation.
    {
        let _sw = StopWatch::new("typecheck", config.show_build_times);
        let all_module_names: Vec<_> = program.modules.iter().map(|m| m.name.clone()).collect();
        program.resolve_namespace_and_check_type_in_modules(
            &typechecker,
            &all_module_names,
            None,
            config,
        )?;
    }

    // Collect deprecation diagnostics from all type-checked expressions and
    // surface them according to `Configuration.deprecation_mode`.
    program
        .deferred_errors
        .append(program.collect_deprecation_diagnostics(config));

    // Instantiate Main::main (or Test::test).
    match config.output_file_type {
        OutputFileType::Executable => {
            program.instantiate_entry_io_value(&typechecker, config.entry_point_runs_tests())?
        }
        OutputFileType::DynamicLibrary => {}
    };

    // Instantiate all exported values and values called from them.
    program.instantiate_exported_values(&typechecker)?;

    // Reject an `FFI_CALL` naming a C function it cannot name, now that every exported value carries
    // the type it is exported at, and before code generation puts one function under the name.
    program.validate_c_function_calls()?;

    // Reject a value whose type has no layout, now that the program's types are instantiated and
    // before code generation walks the fields of any of them.
    program.validate_layouts()?;

    Ok(program)
}

/// Read the whole file at `path` as a string.
/// The error carries a message naming the path and the reason it could not be read.
pub fn read_file(path: &Path) -> Result<String, String> {
    let mut file = match File::open(&path) {
        Err(why) => {
            return Err(format!(
                "Couldn't open \"{}\": {}",
                path.to_string_lossy().to_string(),
                why
            ))
        }
        Ok(file) => file,
    };
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Err(why) => {
            return Err(format!(
                "Couldn't read \"{}\": {}",
                path.to_string_lossy().to_string(),
                why
            ))
        }
        Ok(_) => (),
    }
    Ok(content)
}

/// Create the directory at `rel_path`, together with its missing ancestors, and return its path.
/// An existing directory is left as it is. Panics when the directory cannot be created.
pub fn touch_directory<P>(rel_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let dir_path = PathBuf::new().join(rel_path);
    match create_dir_all(&dir_path) {
        Err(why) => panic!(
            "Failed to create directory \"{}\": {}",
            dir_path.to_string_lossy().to_string(),
            why
        ),
        Ok(_) => {}
    };
    dir_path
}

/// Load all source files specified in the configuration, link them, and return the resulting `Program`.
fn load_source_files(config: &Configuration) -> Result<Program, Errors> {
    // Create `Std` module.
    let mut program = make_std_mod(config)?;

    // Parse all source files.
    let mut parsed_programs = vec![];
    let mut errors = Errors::empty();
    for file_path in config.source_files() {
        let parse_result = parse_file_path(file_path.clone(), config);
        errors.eat_err_or(parse_result, |parsed_program| {
            parsed_programs.push(parsed_program)
        });
    }

    if let SubCommand::Diagnostics(diag_config) = &config.subcommand {
        // If a parsing error occurs in diagnostics mode, delay the error and remove the root
        // project from modules, so that the diagnostic process that follows targets the dependent
        // projects alone. This gives the language server the information it needs for code
        // completion even when the root project has a parse error.
        if errors.has_error() {
            let mut dependency_programs = vec![];
            for parsed_program in parsed_programs {
                let target_module_names = parsed_program.modules_from_files(&diag_config.files)?;
                if target_module_names.is_empty() {
                    dependency_programs.push(parsed_program);
                }
            }
            parsed_programs = dependency_programs;
        }
        program.deferred_errors.append(errors);
    } else {
        // In usual compilation, raise the error.
        errors.to_result()?;
    }

    // Link all modules.
    for parsed_program in parsed_programs {
        program.link(parsed_program, false)?; // If an error occurs in linking, return the error.
    }

    // Resolve imports.
    program.check_imports()?;

    // Warn about an import that reaches a project the importing project does not declare.
    program
        .deferred_errors
        .append(program.collect_undeclared_dependency_diagnostics(config));

    Ok(program)
}

/// Load the program specified by the Configuration, perform validations and type checking.
pub fn elaborate_via_config(config: &Configuration) -> Result<Program, Errors> {
    let program = load_source_files(&config)?;
    let program = elaborate(program, config)?;
    Ok(program)
}
