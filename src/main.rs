extern crate pest;
#[macro_use]
extern crate pest_derive;
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
mod build;
mod commands;
mod configuration;
mod constants;
mod dependency;
mod edit;
mod elaboration;
mod env_vars;
mod error;
mod fixstd;
mod generator;
mod graph;
mod metafiles;
mod misc;
mod object;
mod optimization;
mod parse;
mod preliminary_command;
mod printer;
mod rc_ir;
mod return_abi;
#[cfg(test)]
mod tests;
mod tool;
mod type_size;

use crate::error::Errors;
use crate::misc::{disable_colored_no_tty, spawn_compiler_thread};
use clap::ArgAction;
use clap::ArgMatches;
use clap::PossibleValue;
use clap::{value_parser, App, AppSettings, Arg};
use commands::lsp::server::launch_language_server;
use commands::{check, clean, deps, docs, run};
use configuration::{
    BuildConfigType, Configuration, DeprecationMode, FixOptimizationLevel, LinkType,
    OutputFileType, Sanitizer, SubCommand,
};
use constants::{
    DEFAULT_COMPILATION_UNIT_MAX_SIZE_STR, DEFAULT_REGISTRY, OPTIMIZATION_LEVEL_BASIC,
    OPTIMIZATION_LEVEL_EXPERIMENTAL, OPTIMIZATION_LEVEL_MAX, OPTIMIZATION_LEVEL_NONE,
    PROJECT_FILE_PATH,
};
use edit::edit_explict_import;
use error::panic_if_err;
use git_version::git_version;
use metafiles::config_file::ConfigFile;
use metafiles::project_file::ProjectFile;
use mimalloc::MiMalloc;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::vec::Vec;

/// The allocator the compiler process itself runs on. A program the compiler builds allocates
/// through the Fix runtime instead.
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// The version of the compiler: the released version, followed in parentheses by the revision it
/// was built from. The revision is the `git describe` output for that source, carrying a `-dirty`
/// suffix when the working tree held uncommitted changes, and it is what tells two builds of the
/// same released version apart.
///
/// `concat!` expands `env!` and leaves a procedural macro call unexpanded, so the released version
/// joins the revision from inside `git_version!`, as its prefix.
const VERSION: &str = git_version!(
    args = ["--abbrev=7", "--always", "--dirty", "--broken"],
    prefix = concat!(env!("CARGO_PKG_VERSION"), " ("),
    suffix = ")"
);

/// Run the `fix` command, exiting with status 1 when it fails.
fn main() {
    // The compiler recurses over the user program's expression tree, whose nesting depth is
    // unbounded, so it runs on a thread with a stack sized for that recursion — the size its
    // type-checking and code-generation workers already use — instead of the smaller default
    // main-thread stack. `run_cli` reports any failure through the panic hook before unwinding, so
    // a panicked join only needs to become a non-zero exit here.
    if spawn_compiler_thread(run_cli).join().is_err() {
        process::exit(1);
    }
}

/// Build the command-line interface, parse the invocation, and run the selected subcommand.
fn run_cli() {
    disable_colored_no_tty();

    // Options
    let source_file = Arg::new("source-files")
        .long("file")
        .short('f')
        .action(ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Source files to be compiled and linked.");
    let object_file = Arg::new("object-files")
        .long("object")
        .short('b')
        .action(ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Object files to be linked.");
    let static_link_library = Arg::new("static-link-library")
        .long("static-link")
        .short('s')
        .action(ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Add statically linked library. For example, give \"abc\" to link \"libabc.a\".");
    let dynamic_link_library = Arg::new("dynamic-link-library")
        .long("dynamic-link")
        .short('d')
        .action(ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Add dynamically linked library. For example, give \"abc\" to link \"libabc.so\".");
    let library_paths = Arg::new("library-paths")
        .long("library-paths")
        .short('L')
        .action(ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Add library search paths.");
    let ld_flags = Arg::new("ld-flags")
        .long("ld-flags")
        .action(ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help("Other linker flags.");
    let debug_info = Arg::new("debug-info")
        .long("debug")
        .short('g')
        .takes_value(false)
        .help("Generate debugging information. \n\
              This option automatically turns on `-O none`. You can override this by explicitly specifying another optimization level.");
    let backtrace = Arg::new("backtrace")
        .long("backtrace")
        .takes_value(false)
        .help("Displays a backtrace when the program aborts abnormally. Requires libbacktrace to be installed. Compiling with \"-g\" to add debug information yields better results.");
    let opt_level = Arg::new("opt-level")
        .long("opt-level")
        .short('O')
        .takes_value(true)
        .possible_value(PossibleValue::new(OPTIMIZATION_LEVEL_NONE).help("No optimizations; the shortest compile time. Suitable for debugging."))
        .possible_value(PossibleValue::new(OPTIMIZATION_LEVEL_BASIC).help("Enables basic optimizations, providing a good balance between performance and compilation time."))
        .possible_value(PossibleValue::new(OPTIMIZATION_LEVEL_MAX).help("Enables all optimizations for maximum performance. This is the default optimization level."))
        .possible_value(PossibleValue::new(OPTIMIZATION_LEVEL_EXPERIMENTAL).help("Enables all optimizations, including experimental ones (intended for compiler development)."))
        // The option carries no default value, so that an invocation that gives it explicitly is
        // told apart from one that leaves the level to the project file or to `--debug`.
        .help("Optimization level.");
    let disable_cpu_feature = Arg::new("disable-cpu-feature")
        .long("disable-cpu-feature")
        .action(ArgAction::Append)
        .multiple_values(true)
        .takes_value(true)
        .help(
            "Disable specific CPU features (e.g., \"sse4.2\", \"avx\", \"avx2\").\n\
            This option takes regex patterns, e.g., \"avx.*\" disables all AVX-related features.\n\
            By default, Fix enables all CPU features supported by the host CPU.\n",
        );
    let emit_llvm = Arg::new("emit-llvm")
        .long("emit-llvm")
        .takes_value(false)
        .help("Emit LLVM-IR file.");
    let threaded = Arg::new("threaded")
        .long("threaded")
        .takes_value(false)
        .help("Enable multi-threading. Turning this option ON increases overhead, it is recommended keeping this option OFF for single-threaded programs.");
    let sanitize = Arg::new("sanitize")
        .long("sanitize")
        .takes_value(true)
        .possible_value(
            PossibleValue::new("none").help("Build the program as it is built for use."),
        )
        .possible_value(PossibleValue::new("thread").help(
            "Instrument the program with ThreadSanitizer, which reports data races at run time. \
             The instrumented program runs several times slower and uses much more memory.",
        ))
        .help("Sanitizer to instrument the built program with.");
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
        // The option carries no default value, so that an invocation that gives it explicitly is
        // told apart from one that leaves the kind to the project file.
        .help("The kind of file the build produces. An executable file, unless this option or the project file asks for a dynamic library.");
    let verbose = Arg::new("verbose")
        .long("verbose")
        .short('v')
        .takes_value(false)
        .help("Show verbose messages.");
    let max_cu_size = Arg::new("max-cu-size")
        .long("max-cu-size")
        .takes_value(true)
        .default_value(DEFAULT_COMPILATION_UNIT_MAX_SIZE_STR)
        .value_parser(value_parser!(usize))
        .help(
            "Maximum size of compilation units created by separate compilation.\n\
            Decreasing this value improves parallelism of compilation, but increases time for linking.\n\
            NOTE: Separate compilation is disabled under the default optimization level.\n",
        );
    let llvm_passes_file = Arg::new("llvm-passes-file")
        .long("llvm-passes-file")
        .takes_value(true)
        .help(
            "Path to a file listing LLVM passes, one pass-pipeline string per line, to run in place of the ones the optimization level implies (intended for compiler development).\n",
        );
    let emit_symbols = Arg::new("emit-symbols")
        .long("emit-symbols")
        .help("Output symbols of the Fix program (intended for compiler development).");
    let emit_rc_ir = Arg::new("emit-rc-ir")
        .long("emit-rc-ir")
        .takes_value(true)
        .value_name("MODULE")
        .help("Write the RC IR of a module's symbols to `.fixlang/rc_ir.<module>.txt`, or `all` for every module to `.fixlang/rc_ir.txt` (intended for compiler development).");
    let program_args = Arg::new("program-args")
        .last(true)
        .takes_value(true)
        .allow_hyphen_values(true)
        .help(
            "Arguments passed to the Fix program.\n\
            Use '--' to separate Fix compiler options from program arguments.\n\
            Example: fix run -- arg1 arg2\n\
            These arguments can be accessed using the `get_args` function in your Fix program.",
        );
    let project_name = Arg::new("project-name")
        .index(1)
        .takes_value(true)
        .help("Name of this Fix project.");
    let no_runtime_check = Arg::new("no-runtime-check")
        .long("no-runtime-check")
        .help(
            "Disable runtime checks that would abort the program.\n\
            This includes disabling array bounds checks, union variant checks in `as_` functions, and `Std::undefined`, etc."
        );
    let skip_eval = Arg::new("skip-eval")
        .long("skip-eval")
        .takes_value(false)
        .help(
            "Skip the evaluation instructed by the `eval` syntax: build `eval {expr0}; {expr1}` as `{expr1}`.\n\
            Use it to drop a debugging effect such as `Debug::debug_println` from a built program."
        );
    let allow_preliminary_commands = Arg::new("allow-preliminary-commands")
        .long("allow-preliminary-commands")
        .help(
            "Approve all preliminary_commands for this invocation without prompting.\n\
            Intended for CI and other non-interactive runs.",
        );
    let allow_deprecated = Arg::new("allow-deprecated")
        .long("allow-deprecated")
        .takes_value(false)
        .help("Suppress warnings about uses of `DEPRECATED` items.");
    let deny_deprecated = Arg::new("deny-deprecated")
        .long("deny-deprecated")
        .takes_value(false)
        .help("Treat warnings about uses of `DEPRECATED` items as errors.");

    // "fix version" subcommand
    let version_subc = App::new("version").about("Prints the version of the Fix compiler.");

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
        .arg(ld_flags.clone())
        .arg(debug_info.clone())
        .arg(opt_level.clone())
        .arg(disable_cpu_feature.clone())
        .arg(emit_llvm.clone())
        .arg(threaded.clone())
        .arg(sanitize.clone())
        .arg(verbose.clone())
        .arg(max_cu_size.clone())
        .arg(llvm_passes_file.clone())
        .arg(emit_symbols.clone())
        .arg(emit_rc_ir.clone())
        .arg(backtrace.clone())
        .arg(no_runtime_check.clone())
        .arg(skip_eval.clone())
        .arg(allow_preliminary_commands.clone())
        .arg(allow_deprecated.clone())
        .arg(deny_deprecated.clone());

    // The options of a subcommand that builds a Fix program and then executes it. They are listed
    // in the order `--help` shows them.
    let add_run_and_test_options = |app: App<'static>| {
        app.arg(source_file.clone())
            .arg(object_file.clone())
            .arg(output_file.clone())
            .arg(static_link_library.clone())
            .arg(dynamic_link_library.clone())
            .arg(library_paths.clone())
            .arg(ld_flags.clone())
            .arg(debug_info.clone())
            .arg(opt_level.clone())
            .arg(disable_cpu_feature.clone())
            .arg(emit_llvm.clone())
            .arg(threaded.clone())
            .arg(sanitize.clone())
            .arg(verbose.clone())
            .arg(max_cu_size.clone())
            .arg(llvm_passes_file.clone())
            .arg(emit_symbols.clone())
            .arg(emit_rc_ir.clone())
            .arg(program_args.clone())
            .arg(backtrace.clone())
            .arg(no_runtime_check.clone())
            .arg(skip_eval.clone())
            .arg(allow_preliminary_commands.clone())
            .arg(allow_deprecated.clone())
            .arg(deny_deprecated.clone())
    };

    // "fix run" subcommand
    let run_subc = add_run_and_test_options(
        App::new("run")
            .trailing_var_arg(true)
            .about("Runs a Fix program. Executes `Main::main : IO ()`."),
    );

    // "fix test" subcommand
    let test_subc = add_run_and_test_options(
        App::new("test")
            .trailing_var_arg(true)
            .about("Tests a Fix program. Executes `Test::test : IO ()`."),
    );

    // "fix deps" subcommand
    let deps = App::new("deps").about("Manage dependencies.");
    let test_flag = Arg::new("test")
        .long("test")
        .takes_value(false)
        .help("Operate on test dependencies instead of build dependencies.");
    let deps_install = App::new("install")
        .about("Install dependencies specified in the lock file. By default, build dependencies are installed. Use --test to install test dependencies.")
        .arg(test_flag.clone());
    let deps_update = App::new("update")
        .about("Update the lock file so that it satisfies the dependencies specified in the project file, and install the dependencies. By default, build lock file is updated. Use --test to update test lock file.")
        .arg(test_flag.clone());
    let deps_add_about = format!("Update the project file by adding `[[dependencies]]` tables which describe dependencies to specified Fix projects.\n\
    Repositories for a Fix project is searched in the registry files listed in the configuration file (\"~/.fixconfig.toml\") and the default registry \"{}\".", DEFAULT_REGISTRY);
    let deps_add_about: &'static str = deps_add_about.leak();
    let deps_add = App::new("add")
        .about(deps_add_about)
        .arg(
            Arg::new("projects")
                .multiple_values(true)
                .takes_value(true)
                .help("Projects to be added. \nEach entry should be in the form \"proj-name\" or \"proj-name@ver_req\" (e.g.,\"hashmap@0.1.0\")."),
        )
        .arg(test_flag.clone());
    let deps_list = App::new("list")
        .about("List all available projects in the registry.")
        .arg(
            Arg::new("json")
                .long("json")
                .takes_value(false)
                .help("Output the result in JSON format. NOTE: this option is experimental and may be removed in the future."),
        );

    let deps_subc = deps
        .subcommand(deps_install)
        .subcommand(deps_update)
        .subcommand(deps_add)
        .subcommand(deps_list);

    // "fix clean" subcommand
    let clean_subc = App::new("clean").about("Removes intermediate files or cache files.");

    // "fix language-server" subcommand
    let lsp_subc = App::new("language-server").about("Launch language server for Fix.");

    // "fix docs" subcommand
    let docs_subc = App::new("docs")
        .about("Generate documentations (Markdown files).")
        .long_about("Generate documentations (Markdown files).\n\n\
This command generates documentation for the Fix project located in the current directory.\n\n\
The target Fix project must be free of errors.\n\n\
Consecutive line comments immediately preceding an entity declaration in the source files are treated as documentation for that entity.")
        .arg(
            Arg::new("modules")
                .long("mods")
                .short('m')
                .action(ArgAction::Append)
                .multiple_values(true)
                .takes_value(true)
                .help("Modules for which documents should be generated. If not specified, documents are generated for all modules. To specify modules that are only included during testing, the --test option must be added."),
        ).arg(
            Arg::new("include-compiler-defined-methods").long("with-compiler-defined-methods").help("Include compiler-defined methods such as `@{field_name}` or `as_{variant_name}` in the documentation."),
        ).arg(
            Arg::new("out-dir").long("out-dir").short('o').takes_value(true).help("Output directory for generated documents.").default_value("docs"),
        ).arg(
            Arg::new("private").long("with-private").help("Include private values (i.e., values whose name starts with underscore) in the documentation."),
        ).arg(
            Arg::new("test").long("test").help("Include test modules in the documentation."));

    // "fix init" subcommand
    let init_subc = App::new("init")
        .about("Generates a project file \"fixproj.toml\" in the current directory.")
        .arg(project_name.clone());

    // "fix edit" subcommand
    let edit_explicit_import = App::new("explicit-import").about(
        "(Experimental)\n\
         Rewrite import statements to import only the necessary entities explicitly.\n\
         This command checks if the project has errors, and for each source file,\n\
         collects all referenced names and rewrites import statements.",
    );
    let edit_subc = App::new("edit")
        .about("Edit source code.")
        .subcommand(edit_explicit_import);

    // "fix check" subcommand
    let check_subc = App::new("check")
        .about("Checks whether a Fix project compiles without errors. Type-checks all entities including test code.");

    let mut app = App::new("fix")
        .bin_name("fix")
        .version(VERSION)
        .propagate_version(true)
        .about("The toolchain for Fix, a fast, simple, purely functional language.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(version_subc)
        .subcommand(build_subc)
        .subcommand(run_subc)
        .subcommand(test_subc)
        .subcommand(clean_subc)
        .subcommand(lsp_subc)
        .subcommand(deps_subc)
        .subcommand(docs_subc)
        .subcommand(init_subc)
        .subcommand(edit_subc)
        .subcommand(check_subc);

    /// Every path the option `opt_id` collects, across all of its occurrences.
    fn read_path_list_option(args: &ArgMatches, opt_id: &str) -> Vec<PathBuf> {
        let Some(paths) = args.get_many::<String>(opt_id) else {
            return vec![];
        };
        paths.map(PathBuf::from).collect()
    }

    /// The kind of file the `--output-type` option asks the build to produce, if the invocation
    /// gives that option.
    fn read_output_file_type_option(args: &ArgMatches) -> Result<Option<OutputFileType>, Errors> {
        match args.get_one::<String>("output-file-type") {
            None => return Ok(None),
            Some(file_type) => Ok(Some(OutputFileType::from_str(file_type)?)),
        }
    }

    /// Apply the options of one `fix docs` invocation to the documentation settings `config`
    /// carries.
    fn read_docs_options(args: &ArgMatches, config: &mut Configuration) -> Result<(), Errors> {
        let docs_config = match &mut config.subcommand {
            SubCommand::Docs(docs_config) => docs_config,
            subcommand => unreachable!(
                "the options of `fix docs` were read into the configuration of `fix {}`",
                subcommand.command_type_string()
            ),
        };

        // `modules` option
        docs_config.modules = read_string_list_option(args, "modules");

        // `with-compiler-defined-methods` option
        docs_config.include_compiler_defined_methods =
            args.contains_id("include-compiler-defined-methods");

        // `private` option
        docs_config.include_private = args.contains_id("private");

        // `out-dir` option
        let dir = args
            .get_one::<String>("out-dir")
            .expect("the `--out-dir` option carries a default value");
        docs_config.out_dir = PathBuf::from(dir);

        // `test` option
        docs_config.mode = get_build_mode(args);

        Ok(())
    }

    /// The path the `--output` option names for the built file, if the invocation gives that
    /// option.
    fn read_output_file_option(args: &ArgMatches) -> Option<PathBuf> {
        args.get_one::<String>("output-file").map(PathBuf::from)
    }

    /// Every value the option `opt_id` collects, across all of its occurrences. A subcommand that
    /// has no such option yields an empty list.
    fn read_string_list_option(args: &ArgMatches, opt_id: &str) -> Vec<String> {
        args.try_get_many::<String>(opt_id)
            .unwrap_or_default()
            .unwrap_or_default()
            .cloned()
            .collect()
    }

    /// Every library the invocation links, each paired with how it is bound: `--static-link` names
    /// the libraries copied into the output, `--dynamic-link` the ones resolved at load time.
    fn read_library_options(args: &ArgMatches) -> Vec<(String, LinkType)> {
        let mut options = vec![];
        for (opt_id, link_type) in [
            ("static-link-library", LinkType::Static),
            ("dynamic-link-library", LinkType::Dynamic),
        ] {
            options.extend(
                read_string_list_option(args, opt_id)
                    .into_iter()
                    .map(|name| (name, link_type)),
            );
        }
        options
    }

    /// The directories the `--library-paths` option adds to the linker's search path for
    /// libraries.
    fn read_library_paths_option(args: &ArgMatches) -> Vec<PathBuf> {
        read_string_list_option(args, "library-paths")
            .into_iter()
            .map(PathBuf::from)
            .collect()
    }

    /// The CPU features the `--disable-cpu-feature` option turns off, as regex patterns matched
    /// against the host's feature names, checked here for valid regex syntax.
    fn read_disable_cpu_feature_option(args: &ArgMatches) -> Result<Vec<String>, Errors> {
        let features = read_string_list_option(args, "disable-cpu-feature");
        ProjectFile::validate_disable_cpu_features(&features)?;
        Ok(features)
    }

    /// The LLVM passes listed in the file given by `--llvm-passes-file`, one pass-pipeline string
    /// per line.
    fn read_llvm_passes_file_option(args: &ArgMatches) -> Result<Option<Vec<String>>, Errors> {
        let Some(path) = args.get_one::<String>("llvm-passes-file") else {
            return Ok(None);
        };
        let content = fs::read_to_string(path).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to read the LLVM passes file \"{}\": {}.",
                path, e
            ))
        })?;
        Ok(Some(
            content
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect(),
        ))
    }

    /// The set of project-file declarations the invocation applies to: `--test` selects the test
    /// dependencies and test source files, and its absence the build ones.
    fn get_build_mode(args: &ArgMatches) -> BuildConfigType {
        if args.contains_id("test") {
            BuildConfigType::Test
        } else {
            BuildConfigType::Build
        }
    }

    /// Apply the options of one invocation on top of `config`, which already carries what the
    /// project file declares.
    fn set_config_from_args(config: &mut Configuration, args: &ArgMatches) -> Result<(), Errors> {
        // Files passed via `--file` are user code — append to both
        // `source_files` and `root_source_files`. Note that this runs
        // *after* the root `set_config` in `create_config`, so inside a
        // project directory `--file foo.fix` adds `foo.fix` on top of
        // whatever `fixproj.toml` already declared.
        for file in read_path_list_option(args, "source-files") {
            config.add_user_source_file(file);
        }

        // Set `object_files`.
        config
            .object_files
            .append(&mut read_path_list_option(args, "object-files"));

        // Set `linked_libraries`.
        config
            .linked_libraries
            .append(&mut read_library_options(args));

        // Set `library_search_paths`.
        config
            .library_search_paths
            .append(&mut read_library_paths_option(args));

        // Set `ld_flags`.
        config
            .ld_flags
            .append(&mut read_string_list_option(args, "ld-flags"));

        // Set `emit_llvm`.
        config.emit_llvm = args.contains_id("emit-llvm");

        // Set `threaded`.
        if args.contains_id("threaded") {
            config.set_threaded();
        }

        // Set `sanitizer`.
        if let Some(sanitizer) = args.value_of("sanitize") {
            panic_if_err(config.set_sanitizer(panic_if_err(Sanitizer::from_str(sanitizer))));
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
                OPTIMIZATION_LEVEL_BASIC => config.set_fix_opt_level(FixOptimizationLevel::Basic),
                OPTIMIZATION_LEVEL_MAX => config.set_fix_opt_level(FixOptimizationLevel::Max),
                OPTIMIZATION_LEVEL_EXPERIMENTAL => {
                    config.set_fix_opt_level(FixOptimizationLevel::Experimental)
                }
                _ => unreachable!(
                    "the `--opt-level` option accepted the value `{}`, which names no optimization level",
                    opt_level
                ),
            }
        }

        // Set `output_file_path`.
        config.out_file_path = read_output_file_option(args).or(config.out_file_path.clone());

        // Set `output_file_type`. The `--output-type` argument exists only on the subcommand that
        // produces the output file, so reading it is asked for only there.
        if config.subcommand.produces_output_file() {
            if let Some(output_file_type) = read_output_file_type_option(args)? {
                config.output_file_type = output_file_type;
            }
        }

        // Set `disable_cpu_features_regex`.
        if args.contains_id("disable-cpu-feature") {
            config
                .disable_cpu_features_regex
                .append(&mut read_disable_cpu_feature_option(args)?);
        }

        // Set `verbose`.
        if args.contains_id("verbose") {
            config.verbose = true;
        }

        // Set `max_cu_size`.
        config.max_cu_size = *args
            .get_one::<usize>("max-cu-size")
            .expect("the `--max-cu-size` option carries a default value");

        // Set `llvm_passes_override`.
        // Reading the file here puts the passes into `Configuration::object_generation_hash`, so
        // that a change to them invalidates the objects compiled under the previous ones.
        if let Some(passes) = read_llvm_passes_file_option(args)? {
            config.llvm_passes_override = Some(passes);
        }

        // Set `emit_symbols`.
        if args.contains_id("emit-symbols") {
            config.emit_symbols = true;
        }

        // Set `emit_rc_ir`.
        config.emit_rc_ir = args.get_one::<String>("emit-rc-ir").cloned();

        // Set `backtrace`.
        if args.contains_id("backtrace") {
            config.set_backtrace();
        }

        // Set `no_runtime_check`.
        if args.contains_id("no-runtime-check") {
            config.no_runtime_check = true;
        }

        // Set `skip_eval`.
        if args.contains_id("skip-eval") {
            config.skip_eval = true;
        }

        // Set `allow_preliminary_commands`.
        if args.contains_id("allow-preliminary-commands") {
            config.allow_preliminary_commands = true;
        }

        // Set deprecation handling mode.
        let allow_deprecated = args.contains_id("allow-deprecated");
        let deny_deprecated = args.contains_id("deny-deprecated");
        if allow_deprecated && deny_deprecated {
            return Err(Errors::from_msg(
                "`--allow-deprecated` and `--deny-deprecated` cannot be used together.".to_string(),
            ));
        }
        if allow_deprecated {
            config.deprecation_mode = DeprecationMode::Allow;
        } else if deny_deprecated {
            config.deprecation_mode = DeprecationMode::Deny;
        }

        // Set `run_program_args`.
        match config.subcommand {
            SubCommand::Run | SubCommand::Test => {
                let mut program_args = args
                    .get_many::<String>("program-args")
                    .unwrap_or_default()
                    .cloned()
                    .collect::<Vec<_>>();
                config.run_program_args.append(&mut program_args);
            }
            _ => {}
        }

        Ok(())
    }

    /// Create configuration from the command line arguments and the project file. The project
    /// file's settings are laid down first, so an option on the command line overrides them.
    fn create_config(subcommand: SubCommand, args: &ArgMatches) -> Configuration {
        let mode = subcommand.build_mode();
        let mut config = panic_if_err(Configuration::release_mode(subcommand));

        // Set up configuration from the project file if it exists.
        if Path::new(PROJECT_FILE_PATH).exists() {
            let proj_file = panic_if_err(ProjectFile::read_root_file());
            panic_if_err(proj_file.set_config(&mut config));
            panic_if_err(proj_file.install_dependencies(&mut config, mode));
        }

        // Set up configuration from the command line arguments, to overwrite the configuration described in the project file.
        panic_if_err(set_config_from_args(&mut config, args));
        config
    }

    /// Print the help of the subcommand `name`, as `fix <name> --help` prints it. The subcommand
    /// comes from the built `app`, so the help carries the version and names the command by the
    /// path the user types.
    fn print_subcommand_help(app: &mut App, name: &str) {
        app.find_subcommand_mut(name)
            .unwrap_or_else(|| {
                panic!(
                    "the command line reached the subcommand `{}`, which the command does not define",
                    name
                )
            })
            .print_help()
            .unwrap();
    }

    let fix_config = panic_if_err(ConfigFile::load());

    let matches = app.get_matches_mut();
    match matches.subcommand() {
        Some(("version", _args)) => {
            print!("{}", app.render_version());
            process::exit(0);
        }
        Some(("build", args)) => {
            panic_if_err(commands::build::build(&mut create_config(
                SubCommand::Build,
                args,
            )));
        }
        Some(("run", args)) => {
            run::run_command(&create_config(SubCommand::Run, args));
        }
        Some(("test", args)) => {
            run::run_command(&create_config(SubCommand::Test, args));
        }
        Some(("deps", args)) => match args.subcommand() {
            Some(("install", args)) => {
                deps::deps_install_command(args);
            }
            Some(("update", args)) => {
                deps::deps_update_command(args);
            }
            Some(("add", args)) => {
                deps::deps_add_command(args, &fix_config);
            }
            Some(("list", args)) => {
                deps::deps_list_command(args, &fix_config);
            }
            _ => print_subcommand_help(&mut app, "deps"),
        },
        Some(("language-server", _args)) => {
            launch_language_server();
        }
        Some(("clean", _args)) => {
            clean::clean_command();
        }
        Some(("docs", args)) => {
            // Create the configuration.
            let mut config = panic_if_err(Configuration::docs_mode());
            panic_if_err(read_docs_options(args, &mut config));
            panic_if_err(docs::generate_docs_for_files(config));
        }
        Some(("init", args)) => {
            let project_name = args
                .value_of("project-name")
                .unwrap_or("myproject")
                .to_string();
            panic_if_err(ProjectFile::validate_project_name(&project_name, None));
            panic_if_err(ProjectFile::create_example_file(project_name));
        }
        Some(("check", _args)) => {
            let config = panic_if_err(Configuration::check_mode());
            panic_if_err(check::check(config));
        }
        Some(("edit", args)) => match args.subcommand() {
            Some(("explicit-import", _args)) => {
                panic_if_err(edit_explict_import::run_explicit_import_command());
            }
            _ => print_subcommand_help(&mut app, "edit"),
        },
        // A command line naming no subcommand is answered by the help, and one naming a subcommand
        // the command does not define by an error, both before the match is reached.
        subcommand => unreachable!(
            "the command line reached the subcommand {:?}, which has no handler",
            subcommand.map(|(name, _args)| name)
        ),
    }
}
