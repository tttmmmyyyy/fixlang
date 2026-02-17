use crate::error::Errors;
use crate::make_std_mod;
use crate::make_tuple_traits_mod;
use crate::parse_file_path;
use crate::stopwatch::StopWatch;
use crate::Configuration;
use crate::OutputFileType;
use crate::Program;
use crate::SubCommand;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{fs::create_dir_all, path::PathBuf};

// Perform validations and type checking on the program, and return the updated program.
// Changes made to the program include instantiation of symbols and setting of entry points.
fn check_program(mut program: Program, config: &Configuration) -> Result<Program, Errors> {
    let _sw = StopWatch::new("check_program", config.show_build_times);

    // Add tuple definitions.
    program.add_tuple_defns();

    // Add trait implementations for tuples such as ToString or Eq.
    program.link(
        make_tuple_traits_mod(&program.used_tuple_sizes, &config)?,
        true,
    )?;

    // Validate export statements.
    program.validate_export_statements()?;

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

    // Validate trait env.
    program.validate_trait_env()?;

    // Create symbols.
    program.create_trait_member_symbols();

    // Validate constraints of global value type.
    program.validate_global_value_type_constraints()?;

    // Check if all items referred in import statements are defined.
    // This check should be done after `add_methods` and `create_trait_method_symbols`.
    program.validate_import_statements()?;

    // Set and check kinds that appear in type signatures.
    // NOTE: kinds of type variables appearing in type annotations in expressions are set not at this stage but at the type inference stage.
    program.set_kinds()?;

    // If typechecking is not needed, return here.
    if !config.subcommand.typecheck() {
        assert!(!config.subcommand.build_binary());
        return Ok(program);
    }

    let typechecker = program.create_typechecker(config);

    // When running diagnostics, perform type checking of target modules and return here.
    if let SubCommand::Diagnostics(diag_config) = &config.subcommand {
        let _sw = StopWatch::new("typecheck", config.show_build_times);
        let modules = program.modules_from_files(&diag_config.files)?;
        let mut errors = Errors::empty();
        errors.eat_err(program.resolve_namespace_and_check_type_in_modules(&typechecker, &modules));
        program.deferred_errors.append(errors);
        return Ok(program);
    }

    // Instantiate Main::main (or Test::test).
    match config.output_file_type {
        OutputFileType::Executable => program.instantiate_entry_io_value(
            &typechecker,
            matches!(config.subcommand, SubCommand::Test),
        )?,
        OutputFileType::DynamicLibrary => {}
    };

    // Instantiate all exported values and values called from them.
    program.instantiate_exported_values(&typechecker)?;

    Ok(program)
}

// Return file content and last modified.
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
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => {
            return Err(format!(
                "Couldn't read \"{}\": {}",
                path.to_string_lossy().to_string(),
                why
            ))
        }
        Ok(_) => (),
    }
    Ok(s)
}

// Create a directory if it doesn't exist, and return its path.
pub fn touch_directory<P>(rel_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let res = PathBuf::new().join(rel_path);
    match create_dir_all(&res) {
        Err(why) => panic!(
            "Failed to create directory \"{}\": {}",
            res.to_string_lossy().to_string(),
            why
        ),
        Ok(_) => {}
    };
    res
}

// Load all source files specified in the configuration, link them, and return the resulting `Program`.
fn load_source_files(config: &Configuration) -> Result<Program, Errors> {
    // Create `Std` module.
    let mut program = make_std_mod(config)?;

    // Parse all source files.
    let mut modules = vec![];
    let mut errors = Errors::empty();
    for file_path in &config.source_files {
        let res = parse_file_path(file_path.clone(), config);
        errors.eat_err_or(res, |mod_| modules.push(mod_));
    }

    // If an error occurres in parsing,
    if let SubCommand::Diagnostics(diag_config) = &config.subcommand {
        // In eny parsing error occurres in diagnostics mode, delay the error and remove the root project from modules.
        // In other words, in the following diagnostic process, only the dependent projects are targeted.
        // This allows us to give the language server the information it needs for code completion, even if there is a parse error in the root project.
        if errors.has_error() {
            let mut dependent_projects = vec![];
            for mod_ in modules {
                let mods = mod_.modules_from_files(&diag_config.files)?;
                if mods.is_empty() {
                    dependent_projects.push(mod_);
                }
            }
            modules = dependent_projects;
        }
        program.deferred_errors.append(errors);
    } else {
        // In usual compilation, raise the error.
        errors.to_result()?;
    }

    // Link all modules.
    for mod_ in modules {
        program.link(mod_, false)?; // If an error occurres in linking, return the error.
    }

    // Resolve imports.
    program.check_imports()?;

    Ok(program)
}

// Load the program specified by the Configuration, perform validations and type checking.
pub fn check_program_via_config(config: &Configuration) -> Result<Program, Errors> {
    let program = load_source_files(&config)?;
    let program = check_program(program, config)?;
    Ok(program)
}
