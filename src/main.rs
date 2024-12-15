extern crate pest;
#[macro_use]
extern crate pest_derive;
#[allow(unused_imports)]
#[macro_use]
extern crate serial_test;
extern crate build_time;
extern crate chrono;
extern crate colored;
extern crate difference;
extern crate fxhash;
extern crate git2;
extern crate lsp_types;
extern crate num_bigint;
extern crate num_cpus;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate serde_pickle;
extern crate tempfile;
extern crate toml;
extern crate urlencoding;

mod ast;
mod borrowing_optimization;
mod builtin;
mod compile_unit;
mod config_file;
mod configuration;
mod constants;
mod cpu_features;
mod dependency_lockfile;
mod dependency_resolver;
mod docgen;
mod error;
mod generator;
mod graph;
mod llvm_passes;
mod lsp;
mod misc;
mod object;
mod parser;
mod printer;
mod project_file;
mod registry_file;
mod runner;
mod runtime;
mod sourcefile;
mod stdlib;
mod stopwatch;
#[cfg(test)]
mod tests;
mod typecheck;
mod typecheckcache;
mod uncurry_optimization;

use crate::error::Errors;
use ast::expr::*;
use ast::inline_llvm::*;
use ast::name::Name;
use ast::pattern::*;
use ast::program::*;
use ast::traits::*;
use ast::typedecl::*;
use ast::types::*;
use borrowing_optimization::*;
use builtin::*;
use clap::ArgMatches;
use clap::PossibleValue;
use clap::{App, AppSettings, Arg};
use config_file::ConfigFile;
use configuration::*;
use constants::*;
use dependency_lockfile::DependecyLockFile;
use error::panic_if_err;
use generator::*;
use graph::*;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, FunctionType, IntType, PointerType, StructType};
use inkwell::values::{
    BasicValue, BasicValueEnum, CallableValue, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate};
use lsp::language_server::launch_language_server;
use object::*;
use parser::*;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use project_file::ProjectFile;
use runner::*;
use runtime::*;
use sourcefile::*;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::vec::Vec;
use stdlib::*;
use typecheck::*;
use uncurry_optimization::*;

fn main() {
    // Options
    let source_file = Arg::new("source-files")
        .long("file")
        .short('f')
        .action(clap::ArgAction::Append)
        .takes_value(true)
        .help("Source files to be compiled and linked.");
    let object_file = Arg::new("object-files")
        .long("object")
        .short('b')
        .action(clap::ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Object files to be linked.");
    let static_link_library = Arg::new("static-link-library")
        .long("static-link")
        .short('s')
        .action(clap::ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Add statically linked library. For example, give \"abc\" to link \"libabc.a\".");
    let dynamic_link_library = Arg::new("dynamic-link-library")
        .long("dynamic-link")
        .short('d')
        .action(clap::ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Add dynamically linked library. For example, give \"abc\" to link \"libabc.so\".");
    let library_paths = Arg::new("library-paths")
        .long("library-paths")
        .short('L')
        .action(clap::ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Add library search paths.");
    let debug_info = Arg::new("debug-info")
        .long("debug")
        .short('g')
        .takes_value(false)
        .help("Generate debugging information. \n\
              This option automatically turns on `-O none`. You can override this by explicitly specifying another optimization level.");
    let opt_level = Arg::new("opt-level")
        .long("opt-level")
        .short('O')
        .takes_value(true)
        .possible_value(PossibleValue::new("none").help("Perform no optimizations. Good for debugging, but tail call recursion is not optimized and may cause stack overflow."))
        .possible_value(PossibleValue::new("minimum").help("Perform only few optimizations for fast compilation. Tail call recursion is optimized."))
        .possible_value(PossibleValue::new("separated").help("Perform optimizations which can be done under separate compilation."))
        .possible_value(PossibleValue::new("default").help("Perform all optimizations to minimize runtime. Separate compilation is disabled."))
        // .default_value("default") // we do not set default value because we want to check if this option is specified by user.
        .help("Optimization level.");
    let emit_llvm = Arg::new("emit-llvm")
        .long("emit-llvm")
        .takes_value(false)
        .help("Emit LLVM-IR file.");
    let threaded = Arg::new("threaded")
        .long("threaded")
        .takes_value(false)
        .help("Enable multi-threading. Turning this option ON increases overhead, it is recommended keeping this option OFF for single-threaded programs.");
    let output_file = Arg::new("output-file")
        .long("output")
        .short('o')
        .takes_value(true)
        .help("Path to output file.");
    let output_type = Arg::new("output-file-type")
        .long("output-type")
        .takes_value(true)
        .possible_value(PossibleValue::new("exe").help("Builds an executable file."))
        .possible_value(PossibleValue::new("dylib").help("Builds a dynamic library."))
        .default_value("exe");
    let verbose = Arg::new("verbose")
        .long("verbose")
        .short('v')
        .takes_value(false)
        .help("Show verbose messages.");
    let max_cu_size = Arg::new("max-cu-size")
        .long("max-cu-size")
        .takes_value(true)
        .default_value(DEFAULT_COMPILATION_UNIT_MAX_SIZE_STR)
        .value_parser(clap::value_parser!(usize))
        .help(
            "Maximum size of compilation units created by separate compilation.\n\
            Decreasing this value improves parallelism of compilation, but increases time for linking.\n\
            NOTE: Separate compilation is disabled under the default optimization level.\n",
        );
    let llvm_passes_file = Arg::new("llvm-passes-file")
        .long("llvm-passes-file")
        .takes_value(true)
        .help(
            "Path to a file which contains a list of LLVM passes. This option is used for compiler development, and normal users do not need to use this.",
        );
    let output_symbols = Arg::new("output-symbols")
        .long("output-symbols")
        .help("Output symbols of the Fix program. This option is used for compiler development, and normal users do not need to use this.");
    let program_args = Arg::new("program-args")
        .last(true)
        .takes_value(true)
        .allow_hyphen_values(true)
        .help("Arguments passed to the Fix program.");
    let project_name = Arg::new("project-name")
        .index(1)
        .takes_value(true)
        .help("Name of this Fix project.");

    // "fix build" subcommand
    let build_subc = App::new("build")
        .about("Builds the binary of a Fix program.")
        .arg(source_file.clone())
        .arg(object_file.clone())
        .arg(output_file.clone())
        .arg(output_type.clone())
        .arg(static_link_library.clone())
        .arg(dynamic_link_library.clone())
        .arg(library_paths.clone())
        .arg(debug_info.clone())
        .arg(opt_level.clone())
        .arg(emit_llvm.clone())
        .arg(threaded.clone())
        .arg(verbose.clone())
        .arg(max_cu_size.clone())
        .arg(llvm_passes_file.clone())
        .arg(output_symbols.clone());

    // "fix run" subcommand
    let run_subc = App::new("run")
        .trailing_var_arg(true)
        .about("Runs a Fix program. Executes \"Main::main\" of type `IO ()`.")
        .arg(source_file.clone())
        .arg(object_file.clone())
        .arg(output_file.clone())
        .arg(static_link_library.clone())
        .arg(dynamic_link_library.clone())
        .arg(library_paths.clone())
        .arg(debug_info.clone())
        .arg(opt_level.clone())
        .arg(emit_llvm.clone())
        .arg(threaded.clone())
        .arg(verbose.clone())
        .arg(max_cu_size.clone())
        .arg(llvm_passes_file.clone())
        .arg(output_symbols.clone())
        .arg(program_args.clone());

    // "fix test" subcommand
    let test_subc = App::new("test")
        .trailing_var_arg(true)
        .about("Tests a Fix program. Executes \"Test::test\" of type `IO ()`.")
        .arg(source_file.clone())
        .arg(object_file.clone())
        .arg(output_file.clone())
        .arg(static_link_library.clone())
        .arg(dynamic_link_library.clone())
        .arg(library_paths.clone())
        .arg(debug_info.clone())
        .arg(opt_level.clone())
        .arg(emit_llvm.clone())
        .arg(threaded.clone())
        .arg(verbose.clone())
        .arg(max_cu_size.clone())
        .arg(llvm_passes_file.clone())
        .arg(output_symbols.clone())
        .arg(program_args.clone());

    // "fix deps" subcommand
    let deps = App::new("deps").about("Manage dependencies.");
    let deps_install =
        App::new("install").about("Install dependencies specified in the lock file.");
    let deps_update = App::new("update").about(
        "Update the lock file so that it satisfies the dependencies specified in the project file, and install the dependencies."
    );
    let add_about_str = format!("Update the project file by adding `[[dependencies]]` tables which describe dependencies to specified Fix projects.\n\
    Repositories for a Fix project is searched in the registry files listed in the configuration file (\"~/.fixconfig.toml\") and the default registry \"{}\".", DEFAULT_REGISTRY);
    let add_about_str: &'static str = add_about_str.leak();
    let deps_add = App::new("add")
        .about(add_about_str)
        .arg(
            Arg::new("projects")
                .multiple_values(true)
                .takes_value(true)
                .help("Projects to be added. \nEach entry should be in the form \"proj-name\" or \"proj-name@ver_req\" (e.g.,\"hashmap@0.1.0\")."),
        );

    let mut deps_subc = deps
        .subcommand(deps_install)
        .subcommand(deps_update)
        .subcommand(deps_add);

    // "fix clean" subcommand
    let clean_subc = App::new("clean").about("Removes intermediate files or cache files.");

    // "fix language-server" subcommand
    let lsp_subc = App::new("language-server").about("Launch language server for Fix.");

    // "fix docs" subcommand
    let docs_subc = App::new("docs")
        .about(
            "Generate documentations (markdown files) for specified Fix modules.\n\
            This command requires the project file to be present in the current directory, and the project should be in the state that it can be built successfully.\n\
            Consecutive line comments above declarations are recognized as documentations.",
        )
        .arg(
            Arg::new("modules")
                .long("mods")
                .short('m')
                .action(clap::ArgAction::Append)
                .multiple_values(true)
                .takes_value(true)
                .help("Modules for which documents should be generated. If not specified, documents are generated for all modules."),
        ).arg(
            Arg::new("include-compiler-defined-methods").long("with-compiler-defined-methods").help("Include compiler-defined methods such as `@{field_name}` or `as_{variant_name}` in the documentation."),
        ).arg(
            Arg::new("out-dir").long("out-dir").short('o').takes_value(true).help("Output directory for generated documents."),
        );

    // "fix init" subcommand
    let init_subc = App::new("init")
        .about("Generates a project file \"fixproj.toml\" in the current directory.")
        .arg(project_name.clone());

    let app = App::new("Fix-lang")
        .bin_name("fix")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(build_subc)
        .subcommand(run_subc)
        .subcommand(test_subc)
        .subcommand(clean_subc)
        .subcommand(lsp_subc)
        .subcommand(deps_subc.clone())
        .subcommand(docs_subc)
        .subcommand(init_subc);

    fn read_source_files_options(m: &ArgMatches) -> Result<Vec<PathBuf>, Errors> {
        let files = m.get_many::<String>("source-files");
        if files.is_none() {
            return Ok(vec![]);
        }
        let files = files.unwrap();
        let mut pathbufs = vec![];
        for file in files {
            pathbufs.push(PathBuf::from(file));
        }
        Ok(pathbufs)
    }

    fn read_object_files_options(m: &ArgMatches) -> Result<Vec<PathBuf>, Errors> {
        let files = m.get_many::<String>("object-files");
        if files.is_none() {
            return Ok(vec![]);
        }
        let files = files.unwrap();
        let mut pathbufs = vec![];
        for file in files {
            pathbufs.push(PathBuf::from(file));
        }
        Ok(pathbufs)
    }

    fn read_output_file_type_option(m: &ArgMatches) -> Result<Option<OutputFileType>, Errors> {
        match m.get_one::<String>("output-file-type") {
            None => return Ok(None),
            Some(file_type) => Ok(Some(OutputFileType::from_str(file_type)?)),
        }
    }

    fn read_docs_options(m: &ArgMatches, config: &mut Configuration) -> Result<(), Errors> {
        let docs_config = match &mut config.subcommand {
            SubCommand::Docs(docs_config) => docs_config,
            _ => panic!("Invalid subcommand."),
        };

        // `modules` option
        let modules = m
            .get_many::<String>("modules")
            .unwrap_or_default()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();
        docs_config.modules = modules;

        // `with-compiler-defined-methods` option
        docs_config.include_compiler_defined_methods =
            m.contains_id("include-compiler-defined-methods");

        // `out-dir` option
        let dir = m
            .get_one::<String>("out-dir")
            .map(|s| s.to_string())
            .unwrap_or_default();
        docs_config.out_dir = PathBuf::from(dir);
        Ok(())
    }

    fn read_output_file_option(m: &ArgMatches) -> Option<PathBuf> {
        m.get_one::<String>("output-file").map(|s| PathBuf::from(s))
    }

    fn read_library_options(m: &ArgMatches) -> Vec<(String, LinkType)> {
        let mut options = vec![];
        for (opt_id, link_type) in [
            ("static-link-library", LinkType::Static),
            ("dynamic-link-library", LinkType::Dynamic),
        ] {
            options.append(
                &mut m
                    .try_get_many::<String>(opt_id)
                    .unwrap_or_default()
                    .unwrap_or_default()
                    .map(|v| (v.clone(), link_type))
                    .collect::<Vec<_>>(),
            );
        }
        options
    }

    fn read_library_paths_option(m: &ArgMatches) -> Vec<PathBuf> {
        m.try_get_many::<String>("library-paths")
            .unwrap_or_default()
            .unwrap_or_default()
            .map(|v| PathBuf::from(v))
            .collect::<Vec<_>>()
    }

    fn read_projects_option(m: &ArgMatches) -> Vec<String> {
        m.try_get_many::<String>("projects")
            .unwrap_or_default()
            .unwrap_or_default()
            .cloned()
            .collect::<Vec<_>>()
    }

    fn set_config_from_args(config: &mut Configuration, args: &ArgMatches) -> Result<(), Errors> {
        // Set `source_files`.
        config
            .source_files
            .append(&mut read_source_files_options(args)?);

        // Set `object_files`.
        config
            .object_files
            .append(&mut read_object_files_options(args)?);

        // Set `output_file_type`.
        if let Some(type_) = read_output_file_type_option(args)? {
            config.output_file_type = type_;
        }

        // Set `output_file_path`.
        config.out_file_path = read_output_file_option(args).or(config.out_file_path.clone());

        // Set `linked_libraries`.
        config
            .linked_libraries
            .append(&mut read_library_options(args));

        // Set `library_search_paths`.
        config
            .library_search_paths
            .append(&mut read_library_paths_option(args));

        // Set `emit_llvm`.
        config.emit_llvm = args.contains_id("emit-llvm");

        // Set `threaded`.
        if args.contains_id("threaded") {
            config.set_threaded();
        }

        // Set `debug_info`.
        if args.contains_id("debug-info") {
            config.set_debug_info();
        }

        // Set `opt_level`.
        if args.contains_id("opt-level") {
            // These lines should be after calling `set_debug_info`; otherwise, user cannot specify the optimization level while generating debug information.
            let opt_level = args.get_one::<String>("opt-level").unwrap();
            match opt_level.as_str() {
                OPTIMIZATION_LEVEL_NONE => config.set_fix_opt_level(FixOptimizationLevel::None),
                OPTIMIZATION_LEVEL_MINIMUM => {
                    config.set_fix_opt_level(FixOptimizationLevel::Minimum)
                }
                OPTIMIZATION_LEVEL_SEPARATED => {
                    config.set_fix_opt_level(FixOptimizationLevel::Separated)
                }
                OPTIMIZATION_LEVEL_DEFAULT => {
                    config.set_fix_opt_level(FixOptimizationLevel::Default)
                }
                _ => panic!("Unknown optimization level: {}", opt_level),
            }
        }

        // Set `verbose`.
        if args.contains_id("verbose") {
            config.verbose = true;
        }

        // Set `max_cu_size`.
        config.max_cu_size = *args
            .get_one::<usize>("max-cu-size")
            .unwrap_or(&DEFAULT_COMPILATION_UNIT_MAX_SIZE);

        // Set `llvm_passes_file`.
        if let Some(llvm_passes_file) = args.get_one::<String>("llvm-passes-file") {
            config.llvm_passes_file = Some(PathBuf::from(llvm_passes_file));
        }

        // Set `output_symbols`.
        if args.contains_id("output-symbols") {
            config.output_symbols = true;
        }

        // Set `run_program_args`.
        match config.subcommand {
            SubCommand::Run | SubCommand::Test => {
                let mut args = args
                    .get_many::<String>("program-args")
                    .unwrap_or_default()
                    .cloned()
                    .collect::<Vec<_>>();
                config.run_program_args.append(&mut args);
            }
            _ => {}
        }

        Ok(())
    }

    // Create configuration from the command line arguments and the project file.
    fn create_config(subcommand: SubCommand, args: &ArgMatches) -> Configuration {
        let mut config = Configuration::release_mode(subcommand);

        // Set up configuration from the project file if it exists.
        if Path::new(PROJECT_FILE_PATH).exists() {
            let proj_file = panic_if_err(ProjectFile::read_root_file());
            panic_if_err(proj_file.set_config(&mut config, false));
            panic_if_err(proj_file.install_dependencies(&mut config));
        }

        // Set up configuration from the command line arguments, to overwrite the configuration described in the project file.
        panic_if_err(set_config_from_args(&mut config, args));
        config
    }

    let fix_config = panic_if_err(ConfigFile::load());

    match app.get_matches().subcommand() {
        Some(("build", args)) => {
            panic_if_err(build_file(&mut create_config(SubCommand::Build, args)));
        }
        Some(("run", args)) => {
            process::exit(run_file(create_config(SubCommand::Run, args)));
        }
        Some(("test", args)) => {
            process::exit(run_file(create_config(SubCommand::Test, args)));
        }
        Some(("deps", args)) => match args.subcommand() {
            Some(("install", _args)) => {
                let proj_file = panic_if_err(ProjectFile::read_root_file());
                panic_if_err(proj_file.open_lock_file().and_then(|lf| lf.install()));
            }
            Some(("update", _args)) => {
                panic_if_err(DependecyLockFile::update_and_install());
            }
            Some(("add", args)) => {
                let projects = read_projects_option(args);
                let proj_file = panic_if_err(ProjectFile::read_root_file());
                panic_if_err(proj_file.add_dependencies(&projects, &fix_config));
                panic_if_err(DependecyLockFile::update_and_install());
            }
            _ => deps_subc.print_help().unwrap(),
        },
        Some(("language-server", _args)) => {
            launch_language_server();
        }
        Some(("clean", _args)) => {
            clean_command();
        }
        Some(("docs", args)) => {
            // Create the configuration.
            let mut config = panic_if_err(Configuration::docs_mode());
            panic_if_err(read_docs_options(args, &mut config));
            panic_if_err(docgen::generate_docs_for_files(config));
        }
        Some(("init", args)) => {
            let prj_name = args
                .value_of("project-name")
                .unwrap_or("myproject")
                .to_string();
            panic_if_err(ProjectFile::validate_project_name(&prj_name, None));
            panic_if_err(ProjectFile::create_example_file(prj_name));
        }
        _ => eprintln!("Unknown command. To show list of available commands, run `fix --help`."),
    }
}
