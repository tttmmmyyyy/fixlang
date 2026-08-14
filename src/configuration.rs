use crate::ast::name::FullName;
use crate::build::cpu_features::CpuFeatures;
use crate::constants::{
    CHECK_C_TYPES_PATH, C_CHAR_NAME, C_DOUBLE_NAME, C_FLOAT_NAME, C_INT_NAME, C_LONG_LONG_NAME,
    C_LONG_NAME, C_SHORT_NAME, C_SIZE_T_NAME, C_TYPES_JSON_PATH, C_UNSIGNED_CHAR_NAME,
    C_UNSIGNED_INT_NAME, C_UNSIGNED_LONG_LONG_NAME, C_UNSIGNED_LONG_NAME, C_UNSIGNED_SHORT_NAME,
    DEFAULT_COMPILATION_UNIT_MAX_SIZE, MAX_SPLIT_SCALARS, OPTIMIZATION_LEVEL_BASIC,
    OPTIMIZATION_LEVEL_EXPERIMENTAL, OPTIMIZATION_LEVEL_MAX, OPTIMIZATION_LEVEL_NONE,
};
use crate::elaboration::typecheckcache::{FileCache, TypeCheckCache};
use crate::env_vars;
use crate::error::{panic_if_err, panic_with_msg, Errors};
use crate::misc::{
    platform_thread_sanitizer_supported, platform_valgrind_supported, warn_msg, Finally, Map,
};
use crate::preliminary_command::{approve_and_run, PreliminaryCommand};
use build_time::build_time_utc;
use inkwell::module::Linkage;
use inkwell::OptimizationLevel;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::process::Command;
use std::sync::Arc;
use std::{env, path::PathBuf};

/// The pass-pipeline string for one full LLVM optimization run.
const LLVM_O3_PIPELINE: &str = "default<O3>";

/// How many times `LLVM_O3_PIPELINE` runs at the optimization levels built for speed.
///
/// One run leaves work that a second and a third still find: over `benchmark/speedtest`, the
/// second run takes 2.2% of the instructions off and the third another 0.8%, reaching 21% on
/// `nbody`. A fourth run changes no case by a single instruction.
const LLVM_O3_RUNS_FOR_SPEED: usize = 3;

/// Passes run after the `default<O3>` rounds at the optimization levels built for speed.
///
/// The three together take 0.80% off the cycle counts of the fifteen benchmark cases —
/// `get_sub` 4.4%, `fib` 4.4%, `cp_lib_dijkstra` 2.8%, `levenshtein` 2.2% — against 2.0% back on
/// `cp_lib_lsegtree` and 1.5% on `cp_lib_segtree`. **The three are one unit**: none of them earns
/// that alone, and `pseudo-probe` on its own costs 0.48%. What they change is the shape of the
/// code rather than the work it does, which is why the instruction count barely moves.
///
/// `passes_optimizer.py` searches for this list and starts from it, so `INITIAL_PASSES` there
/// must stay in sync with this and `LLVM_O3_RUNS_FOR_SPEED`.
const LLVM_TAIL_PASSES: [&str; 3] = ["speculative-execution", "loop-vectorize", "pseudo-probe"];

/// Appends a hash of `text` to `hash_source`, a hash source that concatenates several values.
///
/// The hash has the same length whatever the text is, so the value cannot run into the one appended
/// next and `"xy"` followed by `"z"` differs from `"x"` followed by `"yz"`.
fn push_text_hash(hash_source: &mut String, text: &str) {
    hash_source.push_str(&format!("{:x}", md5::compute(text)));
}

/// Appends a hash of `items` to `hash_source`, a hash source that concatenates several lists.
///
/// The count comes first so that a list's items cannot be read as the next list's.
fn push_list_hash(hash_source: &mut String, items: &[String]) {
    hash_source.push_str(&items.len().to_string());
    for item in items {
        push_text_hash(hash_source, item);
    }
}

/// How a linked library is bound to the program.
#[derive(Clone, Copy)]
pub enum LinkType {
    /// The library is copied into the output at link time.
    Static,
    /// The library is resolved when the output is loaded.
    Dynamic,
}

/// What a build produces.
#[derive(Clone, Copy)]
pub enum OutputFileType {
    /// A program that can be run on its own.
    Executable,
    /// A shared library other programs link against.
    DynamicLibrary,
}

impl OutputFileType {
    /// Reads the kind an `output_type` setting or an `--output-type` option names.
    pub fn from_str(file_type: &str) -> Result<Self, Errors> {
        match file_type {
            "exe" => Ok(OutputFileType::Executable),
            "dylib" => Ok(OutputFileType::DynamicLibrary),
            _ => Err(Errors::from_msg(format!(
                "Unknown output file type: `{}`",
                file_type
            ))),
        }
    }

    /// The name this kind is written under in a project file and on the command line.
    pub fn to_str(&self) -> &str {
        match self {
            OutputFileType::Executable => "exe",
            OutputFileType::DynamicLibrary => "dylib",
        }
    }

    /// What a build of this kind calls its output file when the settings name no path for it. The
    /// name follows what the platform's linker and loader expect of the kind.
    pub fn default_file_name(&self) -> &'static str {
        match self {
            OutputFileType::Executable => {
                if env::consts::OS == "windows" {
                    "a.exe"
                } else {
                    "a.out"
                }
            }
            OutputFileType::DynamicLibrary => {
                if env::consts::OS == "windows" {
                    "lib.dll"
                } else if env::consts::OS == "macos" {
                    "lib.dylib"
                } else {
                    "lib.so"
                }
            }
        }
    }
}

/// The valgrind tool the built program is run under in `run` mode.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ValgrindTool {
    /// Run the program directly.
    None,
    /// Run under memcheck, which reports invalid memory accesses and leaks.
    MemCheck,
    // Currently, we cannot use DRD or helgrind because valgrind does not understand atomic operations.
    // In C/C++ program, we can use `ANNOTATE_HAPPENS_BEFORE` and `ANNOTATE_HAPPENS_AFTER` to tell helgrind happens-before relations,
    // but how can we do similar things in Fix?
}

impl fmt::Display for ValgrindTool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValgrindTool::None => write!(f, "none"),
            ValgrindTool::MemCheck => write!(f, "memcheck"),
        }
    }
}

/// The sanitizer the generated program is instrumented with.
///
/// A build asks for at most one: the sanitizers that give a program shadow memory place it at
/// addresses derived from the program's own, so two of them cannot share a program.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Sanitizer {
    /// Generate the program as it is built for use.
    None,
    /// Instrument every memory access so that ThreadSanitizer can report data races.
    Thread,
}

impl fmt::Display for Sanitizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sanitizer::None => write!(f, "none"),
            Sanitizer::Thread => write!(f, "thread"),
        }
    }
}

impl Sanitizer {
    /// What instrumenting a module for this sanitizer takes: the function attribute the passes look
    /// for, and the passes themselves.
    ///
    /// The two travel together because a pass without its attribute rewrites nothing.
    pub fn instrumentation(&self) -> Option<(&'static str, &'static [&'static str])> {
        match self {
            Sanitizer::None => None,
            // The module pass registers the runtime's initializer; the function pass rewrites the
            // accesses.
            Sanitizer::Thread => Some(("sanitize_thread", &["tsan-module", "function(tsan)"])),
        }
    }

    /// Whether this platform can build and run a program instrumented with this sanitizer.
    pub fn platform_supported(&self) -> bool {
        match self {
            Sanitizer::None => true,
            Sanitizer::Thread => platform_thread_sanitizer_supported(),
        }
    }

    /// Reads the value a `sanitize` setting names, or reports the names there are.
    pub fn from_str(name: &str) -> Result<Sanitizer, Errors> {
        match name {
            "none" => Ok(Sanitizer::None),
            "thread" => Ok(Sanitizer::Thread),
            _ => Err(Errors::from_msg(format!(
                "Unknown sanitizer \"{}\". Available sanitizers are \"none\" and \"thread\".",
                name
            ))),
        }
    }
}

/// The subcommand of the `fix` command that the invocation selected, carrying the settings that
/// belong to that subcommand alone.
#[derive(Clone)]
pub enum SubCommand {
    /// Build the program and write it to the output file.
    Build,
    /// Build the program and run it.
    Run,
    /// Build the test program and run it.
    Test,
    /// Elaborate the source files and report the errors and warnings found in them, for the
    /// language server.
    Diagnostics(DiagnosticsConfig),
    /// Generate documentation for the modules.
    Docs(DocsConfig),
}

/// Which section of the project file a build reads its settings from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildConfigType {
    /// The `build` section.
    Build,
    /// The `build.test` sub-section, whose settings a test build uses in place of the ones the
    /// `build` section gives.
    Test,
}

impl Default for BuildConfigType {
    /// The settings of the `build` section.
    fn default() -> Self {
        BuildConfigType::Build
    }
}

impl SubCommand {
    /// Whether the `preliminary_commands` the project files list are run before the build.
    pub fn run_preliminary_commands(&self) -> bool {
        match self {
            SubCommand::Build => true,
            SubCommand::Run => true,
            SubCommand::Test => true,
            SubCommand::Diagnostics(_) => false,
            SubCommand::Docs(_) => false,
        }
    }

    /// Whether the build goes on to generate code and link a binary. Reporting diagnostics and
    /// generating documentation stop with the elaborated program in hand.
    pub fn build_binary(&self) -> bool {
        match self {
            SubCommand::Build => true,
            SubCommand::Run => true,
            SubCommand::Test => true,
            SubCommand::Diagnostics(_) => false,
            SubCommand::Docs(_) => false,
        }
    }

    /// Whether this subcommand produces the output file that the project file's `output` and
    /// `output_type` describe. `fix build` does; `fix run` and `fix test` build an executable in a
    /// temporary place and run it, and a `-o` given to them names where that executable is kept.
    pub fn produces_output_file(&self) -> bool {
        match self {
            SubCommand::Build => true,
            SubCommand::Run => false,
            SubCommand::Test => false,
            SubCommand::Diagnostics(_) => false,
            SubCommand::Docs(_) => false,
        }
    }

    /// Which section of the project file this subcommand's settings come from.
    pub fn build_mode(&self) -> BuildConfigType {
        match self {
            SubCommand::Build => BuildConfigType::Build,
            SubCommand::Run => BuildConfigType::Build,
            SubCommand::Test => BuildConfigType::Test,
            SubCommand::Diagnostics(_) => BuildConfigType::Test,
            SubCommand::Docs(docs_config) => docs_config.mode,
        }
    }

    /// Whether the source files are type-checked. Generating documentation reads the declarations
    /// alone, so it leaves the bodies unchecked.
    pub fn typecheck(&self) -> bool {
        match self {
            SubCommand::Build => true,
            SubCommand::Run => true,
            SubCommand::Test => true,
            SubCommand::Diagnostics(_) => true,
            SubCommand::Docs(_) => false,
        }
    }

    /// The name this subcommand is typed under on the command line.
    pub fn command_type_string(&self) -> &str {
        match self {
            SubCommand::Build => "build",
            SubCommand::Run => "run",
            SubCommand::Test => "test",
            SubCommand::Diagnostics(_) => "diagnostics",
            SubCommand::Docs(_) => "docs",
        }
    }
}

// Configuration for diagnostics subcommand.
#[derive(Clone, Default)]
pub struct DiagnosticsConfig {
    // Target source files.
    pub files: Vec<PathBuf>,
    /// In-memory overrides for source-file contents used during the LSP
    /// completion flow: when `parse_file_path` is invoked for a path
    /// present here, the supplied string is parsed instead of reading
    /// the file from disk. This lets `handle_completion` repair the
    /// live buffer (see `commands::lsp::completion::repair`) and
    /// re-elaborate via `elaborate_via_config` without touching disk.
    pub live_source_overrides: Arc<Map<PathBuf, String>>,
    /// Restrict type-checking to this specific set of global value
    /// names. `None` keeps the default (every global declared in the
    /// target files). When set, only the listed globals' bodies are
    /// typechecked, while pre-typecheck stages (parse, kind/scheme
    /// elaboration, `create_trait_member_symbols`, …) still run for
    /// every module because a checked body may reference others'
    /// schemes.
    pub target_symbols: Option<Vec<FullName>>,
    /// Type-check in error-tolerant mode: when elaboration of a
    /// sub-expression fails, the typechecker substitutes a
    /// placeholder annotated with the expected type for that node
    /// and continues elaborating its siblings, so that one type
    /// error in part of a body (e.g. an `if` condition) does not
    /// blank out inferred types elsewhere in the same body.
    pub error_tolerant: bool,
}

// Configuration for docs subcommand.
#[derive(Clone, Default)]
pub struct DocsConfig {
    // Modules to be documented.
    pub modules: Vec<String>,
    // Include compiler-defined methods in the documentation.
    pub include_compiler_defined_methods: bool,
    // Include private items in the documentation.
    pub include_private: bool,
    // Output directory.
    pub out_dir: PathBuf,
    // Dependency mode (Build or Test).
    pub mode: BuildConfigType,
}

/// Everything one invocation of the `fix` command builds with: what to compile, how to optimize and
/// link it, what to produce, and how to run it. It is assembled from the command line and the
/// project file, and then read by every stage of the build.
///
/// A field whose value changes the generated code has to be added to `object_generation_hash`,
/// which decides when a cached object file may be reused.
#[derive(Clone)]
pub struct Configuration {
    // Source files.
    pub source_files: Vec<PathBuf>,
    /// The subset of `source_files` that is user-authored: the root
    /// project's own files, files passed via `--file`, and files pushed
    /// by unit-test entry points. Excludes files contributed by
    /// dependencies. Used to scope deprecation warnings to user code,
    /// mirroring how Rust/Swift/Kotlin/etc. only flag deprecated uses in
    /// the crate or module currently being compiled.
    ///
    /// Maintain this in lockstep with `source_files` via
    /// `add_user_source_file` whenever you're adding user code.
    pub root_source_files: Vec<PathBuf>,
    // Object files to be linked.
    pub object_files: Vec<PathBuf>,
    // Fix's optimization level.
    fix_opt_level: FixOptimizationLevel,
    // Linked libraries
    pub linked_libraries: Vec<(String, LinkType)>,
    // Library search paths.
    pub library_search_paths: Vec<PathBuf>,
    // Other linker flags
    pub ld_flags: Vec<String>,
    // Create debug info.
    pub debug_info: bool,
    /// The directory the build runs in, read once when the configuration is created.
    ///
    /// `Generator::create_debug_info` writes it into the generated code as the directory of the
    /// compilation unit, and the file names the debug information carries beside it are relative,
    /// so a debugger resolves them against it. That is its one reader, which is what lets
    /// `object_generation_hash` cover it for a build with debug information alone.
    pub compilation_directory: PathBuf,
    // Whether to emit LLVM IR.
    pub emit_llvm: bool,
    // Output file name.
    pub out_file_path: Option<PathBuf>,
    // Output file type.
    pub output_file_type: OutputFileType,
    // Use threads.
    // To turn on this true and link pthread library, use `set_threaded` function.
    pub threaded: bool,
    // Macros defined in runtime.c.
    pub runtime_c_macro: Vec<String>,
    // Show times for each build steps.
    pub show_build_times: bool,
    // Verbose mode.
    pub verbose: bool,
    // Maximum size of compilation unit.
    pub max_cu_size: usize,
    // The most scalars a value is split into and carried as separate LLVM values; a type holding
    // more stays one aggregate (see `Generator::type_parts`). Lowering it brings narrower types
    // under the same treatment.
    pub max_split_scalars: usize,
    // Run program with valgrind. Effective only in `run` mode.
    pub valgrind_tool: ValgrindTool,
    /// The sanitizer the generated program is instrumented with. Instrumenting is a property of the
    /// program that is built, so the project being built decides it, as it does the optimization
    /// level.
    pub sanitizer: Sanitizer,
    // Sizes of C types.
    pub c_type_sizes: CTypeSizes,
    // Regex patterns of disabled CPU features.
    pub disable_cpu_features_regex: Vec<String>,
    // Subcommand of the `fix` command.
    pub subcommand: SubCommand,
    // Preliminary commands declared in fixproj.toml (root and dependencies).
    pub preliminary_commands: Vec<PreliminaryCommand>,
    // If true, bypass the trust-store approval prompt and treat all pending
    // preliminary_commands as one-shot approvals. Set by `--allow-preliminary-commands`.
    pub allow_preliminary_commands: bool,
    // Typecheck cache.
    pub type_check_cache: Arc<dyn TypeCheckCache + Send + Sync>,
    // Number of worker threads.
    pub num_worker_thread: usize,
    // The arguments which are passed to the program in `run` mode.
    pub run_program_args: Vec<String>,
    // LLVM passes to run in place of the ones the optimization level implies.
    // Used only for compiler development.
    pub llvm_passes_override: Option<Vec<String>>,
    // Emit symbols at each step of optimization.
    // Used only for compiler development.
    pub emit_symbols: bool,
    // Dump the RC IR of the named module's symbols (`all` = every module) to a file under
    // `.fixlang/`. `None` dumps nothing. Used only for compiler development.
    pub emit_rc_ir: Option<String>,
    /// Run the compiler's own consistency checks — the RC IR validator and the assertions in the
    /// code generator — and turn an internal error into a panic.
    pub develop_mode: bool,
    /// Enable backtrace support: keep frame pointers and link the backtrace library.
    pub backtrace: bool,
    /// Leave the run-time checks, such as the array bounds check, out of the program.
    pub no_runtime_check: bool,
    /// Compile `eval {side}; {main}` as `{main}`, so that the effect of `{side}` is left out of the
    /// program. `eval` otherwise instructs the compiler to evaluate `{side}`.
    pub skip_eval: bool,
    /// How `DEPRECATED` warnings are handled. See `DeprecationMode`.
    pub deprecation_mode: DeprecationMode,
}

/// How the compiler reacts to a use of a deprecated item.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeprecationMode {
    /// Emit a warning (default).
    Warn,
    /// Suppress the warning entirely.
    Allow,
    /// Promote the warning to an error.
    Deny,
}

impl Default for DeprecationMode {
    /// The compiler defaults to warning on deprecated uses; flags can override.
    fn default() -> Self {
        DeprecationMode::Warn
    }
}

/// How hard the compiler works to make the program fast, trading compile time for run time. The
/// variants are ordered, so a pass can turn itself on from a given level up.
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum FixOptimizationLevel {
    None,         // For debugging; skip even tail call optimization.
    Basic,        // Perform almost all of the optimizations except for LLVM-level LTO.
    Max,          // For fast execution.
    Experimental, // Performs optimizations that are still unstable.
}

impl fmt::Display for FixOptimizationLevel {
    /// Writes the level under the name `--opt-level` accepts for it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixOptimizationLevel::None => write!(f, "{}", OPTIMIZATION_LEVEL_NONE),
            FixOptimizationLevel::Basic => write!(f, "{}", OPTIMIZATION_LEVEL_BASIC),
            FixOptimizationLevel::Max => write!(f, "{}", OPTIMIZATION_LEVEL_MAX),
            FixOptimizationLevel::Experimental => write!(f, "{}", OPTIMIZATION_LEVEL_EXPERIMENTAL),
        }
    }
}

impl FixOptimizationLevel {
    /// The level `--opt-level` spells `opt_level`, the inverse of `Display`.
    pub fn from_str(opt_level: &str) -> Option<Self> {
        match opt_level {
            OPTIMIZATION_LEVEL_NONE => Some(FixOptimizationLevel::None),
            OPTIMIZATION_LEVEL_BASIC => Some(FixOptimizationLevel::Basic),
            OPTIMIZATION_LEVEL_MAX => Some(FixOptimizationLevel::Max),
            OPTIMIZATION_LEVEL_EXPERIMENTAL => Some(FixOptimizationLevel::Experimental),
            _ => None,
        }
    }
}

impl Configuration {
    /// The configuration a run of `subcommand` starts from, which the command line and the project
    /// file then override. The optimization level comes from the environment and the C type sizes
    /// from the C compiler; every other setting takes its default.
    fn new(subcommand: SubCommand) -> Result<Self, Errors> {
        Ok(Configuration {
            subcommand,
            source_files: vec![],
            root_source_files: vec![],
            object_files: vec![],
            fix_opt_level: env_vars::get_max_opt_level(),
            linked_libraries: vec![],
            ld_flags: vec![],
            debug_info: false,
            compilation_directory: env::current_dir().map_err(|e| {
                Errors::from_msg(format!("Failed to get the current directory: {}", e))
            })?,
            emit_llvm: false,
            out_file_path: None,
            output_file_type: OutputFileType::Executable,
            threaded: false,
            runtime_c_macro: vec![],
            show_build_times: false,
            verbose: false,
            max_cu_size: DEFAULT_COMPILATION_UNIT_MAX_SIZE,
            max_split_scalars: MAX_SPLIT_SCALARS,
            valgrind_tool: ValgrindTool::None,
            sanitizer: Sanitizer::None,
            library_search_paths: vec![],
            c_type_sizes: CTypeSizes::load_or_check()?,
            disable_cpu_features_regex: vec![],
            preliminary_commands: vec![],
            allow_preliminary_commands: false,
            type_check_cache: Arc::new(FileCache::new()),
            num_worker_thread: 0,
            llvm_passes_override: None,
            run_program_args: vec![],
            emit_symbols: false,
            emit_rc_ir: None,
            develop_mode: false,
            backtrace: false,
            no_runtime_check: false,
            skip_eval: false,
            deprecation_mode: DeprecationMode::default(),
        })
    }
}

impl Configuration {
    // Configuration for release build.
    pub fn release_mode(subcommand: SubCommand) -> Result<Configuration, Errors> {
        let mut config = Self::new(subcommand)?;
        config.num_worker_thread = num_cpus::get();
        Ok(config)
    }

    // Configuration for compiler development
    #[allow(dead_code)]
    pub fn develop_mode() -> Configuration {
        #[allow(unused_mut)]
        let mut config = panic_if_err(Self::new(SubCommand::Run));
        config.develop_mode = true;
        config.num_worker_thread = 0;
        config.set_valgrind(ValgrindTool::MemCheck);
        config.set_fix_opt_level(FixOptimizationLevel::Experimental);
        config.emit_llvm = true;
        config.emit_symbols = true;
        config
    }

    // Create configuration for document generation.
    pub fn docs_mode() -> Result<Configuration, Errors> {
        let mut config = Self::new(SubCommand::Docs(DocsConfig::default()))?;
        config.num_worker_thread = num_cpus::get();
        Ok(config)
    }

    // Create configuration for diagnostics subcommand.
    pub fn diagnostics_mode(
        diagnostics_config: DiagnosticsConfig,
    ) -> Result<Configuration, Errors> {
        let mut config = Self::new(SubCommand::Diagnostics(diagnostics_config))?;
        config.num_worker_thread = num_cpus::get();
        Ok(config)
    }

    /// The configuration for the `check` subcommand, which type-checks the project — test code
    /// included — and reports the diagnostics it collects. The set of files to check starts empty,
    /// and is filled in once the project file has been read.
    pub fn check_mode() -> Result<Configuration, Errors> {
        Self::diagnostics_mode(DiagnosticsConfig::default())
    }

    /// Run the built program under `tool` in `run` mode. On a platform where valgrind is
    /// unavailable the request is dropped with a warning. Any tool also disables the AVX-512
    /// features valgrind cannot interpret (#41).
    pub fn set_valgrind(&mut self, tool: ValgrindTool) -> &mut Configuration {
        if !platform_valgrind_supported() && tool != ValgrindTool::None {
            warn_msg(&format!(
                "Valgrind is not supported on this platform. Ignoring valgrind settings `{}`",
                tool
            ));
            self.valgrind_tool = ValgrindTool::None;
            return self;
        }
        self.valgrind_tool = tool;
        if tool != ValgrindTool::None {
            // Valgrind-3.22.0 does not support AVX-512 (#41).
            self.disable_cpu_features_regex.push("avx512.*".to_string());
        }
        self
    }

    // Add dynamically linked library.
    // To link libabc.so, provide library name "abc".
    pub fn add_dynamic_library(&mut self, name: &str) {
        self.linked_libraries
            .push((name.to_string(), LinkType::Dynamic));
    }

    /// Register a user-authored source file: the root project's own files,
    /// a path passed via `--file`, or a file pushed by a unit-test entry
    /// point. The file lands in `source_files` (so it is parsed alongside
    /// dependencies) and additionally in `root_source_files`, which scopes
    /// deprecation diagnostics to user code.
    ///
    /// Files contributed by *dependencies* must NOT use this — push them
    /// into `source_files` directly, leaving `root_source_files` alone.
    pub fn add_user_source_file(&mut self, path: PathBuf) {
        self.source_files.push(path.clone());
        self.root_source_files.push(path);
    }

    /// Where `--emit-llvm` writes one compilation unit's LLVM IR: a `.ll` file beside the output
    /// file, or in the working directory where the build names no output file.
    ///
    /// # Arguments
    /// * `optimized` - whether this is the IR as the LLVM pipeline left it, which takes a file of
    ///   its own alongside the IR as first emitted.
    /// * `unit_name` - the compilation unit the IR belongs to, which is what distinguishes the files
    ///   one build writes.
    pub fn get_output_llvm_ir_path(&self, optimized: bool, unit_name: &str) -> PathBuf {
        match &self.out_file_path {
            None => {
                if optimized {
                    return PathBuf::from(format!("{}_optimized.ll", unit_name));
                } else {
                    return PathBuf::from(format!("{}.ll", unit_name));
                }
            }
            Some(out_file_path) => {
                let file_name = out_file_path.file_name();
                if file_name.is_none() {
                    panic_with_msg(&format!(
                        "Invalid output file path: `{}`",
                        out_file_path.to_str().unwrap()
                    ))
                } else {
                    let file_name = file_name.unwrap().to_str().unwrap();
                    let file_name = file_name.to_string()
                        + "_"
                        + unit_name
                        + if optimized { "_optimized.ll" } else { ".ll" };
                    let mut out_file_path = out_file_path.clone();
                    out_file_path.set_file_name(file_name);
                    out_file_path
                }
            }
        }
    }

    pub fn get_output_file_path(&self) -> PathBuf {
        match &self.out_file_path {
            None => PathBuf::from(self.output_file_type.default_file_name()),
            Some(out_file_path) => out_file_path.clone(),
        }
    }

    // Set threaded = true, and add ptherad library to linked_libraries.
    pub fn set_threaded(&mut self) {
        self.threaded = true;
        self.add_dynamic_library("pthread");
    }

    pub fn set_debug_info(&mut self) {
        self.debug_info = true;
        self.set_fix_opt_level(FixOptimizationLevel::None);
    }

    pub fn set_fix_opt_level(&mut self, level: FixOptimizationLevel) {
        self.fix_opt_level = level.min(env_vars::get_max_opt_level());
    }

    pub fn fix_opt_level(&self) -> FixOptimizationLevel {
        self.fix_opt_level
    }

    /// The effort the LLVM pass pipeline is asked for. Every Fix level above `None` asks for the
    /// same default effort, so the levels above it differ in the compiler's own passes alone.
    pub fn get_llvm_opt_level(&self) -> OptimizationLevel {
        match self.fix_opt_level {
            FixOptimizationLevel::None => OptimizationLevel::None,
            FixOptimizationLevel::Basic => OptimizationLevel::Default,
            FixOptimizationLevel::Max => OptimizationLevel::Default,
            FixOptimizationLevel::Experimental => OptimizationLevel::Default,
        }
    }

    /// Whether every optimization the compiler performs itself runs regardless of the optimization
    /// level. Return `true` here to turn them all on with one edit, which is how a pass is exercised
    /// at a level that would otherwise skip it.
    ///
    /// The scope is the compiler's own passes. LLVM's pipeline follows the optimization level alone
    /// (`llvm_passes`), so that a build made to exercise a Fix pass keeps the LLVM effort its level
    /// asks for.
    pub fn force_all_optimizations(&self) -> bool {
        false
    }

    /// Split the program's symbols into several compilation units, each hashed and cached on its
    /// own, so that a rebuild regenerates only the units whose inputs changed. A function of a unit
    /// is externally visible, since another unit calls it. Runs at `Basic` and below; above that the
    /// program is one unit, which is what lets a pass see all of it at once.
    pub fn enable_separated_compilation(&self) -> bool {
        !self.force_all_optimizations() && self.fix_opt_level <= FixOptimizationLevel::Basic
    }

    /// Give each global function a version taking one, two, ... arguments at once, and send every
    /// call to the version matching the number of arguments it supplies, so that a saturated call
    /// passes them directly instead of building a closure per argument. Runs at `Basic` and above.
    pub fn enable_uncurry_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Basic
    }

    /// Defunctionalize `Std::fix` into a directly self-recursive global function. The self-call it
    /// produces is direct, so LLVM's tail-recursion elimination folds it into a loop. The loop is
    /// much stronger than the tail jumps an indirect self-call already gets from the return ABI (see
    /// `return_abi`): it also removes the closure the `fix` combinator builds on every iteration,
    /// with its heap allocation and reference-count updates. The `sum_by_fix` benchmark at `Basic`
    /// measures 47M instructions with the pass against 686M without, at equal compile time, which is
    /// why it runs from `Basic` up. Uncurrying, which flattens the produced self-call, shares that
    /// threshold.
    pub fn enable_defunctionalize_fix(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Basic
    }

    /// Removing type annotations only unwraps annotation nodes — the annotated type is already
    /// carried by the inner expression — so it is semantically neutral and every later stage and
    /// code generation accept the result. It therefore runs at all optimization levels, which lets a
    /// later pass work on a bare AST without seeing through annotations.
    pub fn enable_remove_tyanno_optimization(&self) -> bool {
        true
    }

    /// Make a value of a newtype — an unboxed struct of exactly one field — a value of that field,
    /// so that a type written in that shape, such as `Std::IO`, a state monad or an iterator,
    /// becomes the function it wraps. The gain is that change of type: only once an expression is
    /// typed as a closure can the inlining and the closure specialization below take hold of it, so
    /// it runs at `Max` and above alongside them.
    pub fn enable_unwrap_newtype_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// Substitute the definition of a global value into the places that name it, and discard a
    /// global that nothing names, repeating until the program stops changing. Runs at `Max` and
    /// above.
    pub fn enable_inline_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// Substitute a function bound to a local name into the place it is applied, turning
    /// `let f = |x| e; f(y)` into `e[x := y]`. Runs at `Max` and above.
    pub fn enable_inline_local_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// Ask the back end to inline every call of a global whose body is small enough to stand where
    /// it is called. Runs at `Max` and above, where the program is built for speed and the back end
    /// is given a pipeline to act on the request with.
    pub fn enable_request_inline_into_callers(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// Copy a function for the closure it is handed, so that the copy calls that closure's body
    /// directly instead of reaching it through a function pointer and a capture list. Runs at `Max`
    /// and above.
    pub fn enable_closure_specialization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// Replace an act-family function — `Std::Array::act(i)` and a struct's `act_{field}` — at the
    /// functors `Std::Identity`, `Std::Const` and `Std::Tuple2` by an implementation written for
    /// that functor. It rewrites the definitions of global symbols, so it is placed above the
    /// inlining that would otherwise dissolve them. Runs at `Max` and above.
    pub fn enable_act_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// Borrow-ification and cancellation of the RC IR: borrows a parameter a function only reads,
    /// then cancels the reference counting the borrow makes net-zero. Its full benefit relies on
    /// closure specialization and inlining (which are also `Max`-only), and it adds compile-time
    /// analysis, so it runs only at `Max` and above; `Basic` stays lighter for faster compilation.
    pub fn enable_borrow_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// The RC-IR term simplifier (case-of-known-constructor, case-of-case) runs at `Max` and above.
    /// It composes with the same closure specialization that borrow-ification needs — a specialized
    /// loop's body is a known function whose union it can cancel — so it shares that threshold.
    pub fn enable_simplify(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// Shorten the compiler-added suffixes of global symbol names to serial numbers, so that a
    /// symbol dump shows `Std::func#0` where the name is `Std::func#{...}#{...}`. Runs at
    /// `Experimental`.
    pub fn enable_simplify_symbol_names(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Experimental
    }

    /// Discard the global values that nothing reachable from `main` or from an exported function
    /// uses. Runs at `Max` and above.
    pub fn enable_dead_symbol_elimination(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    /// Build the program so that it prints a backtrace when it aborts: define the runtime's
    /// `BACKTRACE` macro, and on Linux link the backtrace library it then calls into.
    pub fn set_backtrace(&mut self) {
        self.backtrace = true;
        self.runtime_c_macro.push("BACKTRACE".to_string());
        if env::consts::OS == "linux" {
            self.add_dynamic_library("backtrace");
        }
    }

    /// Whether the generated code must keep its frame pointers. macOS's `backtrace()` walks them, so
    /// a build that prints a backtrace there keeps them.
    pub fn no_elim_frame_pointers(&self) -> bool {
        self.backtrace && env::consts::OS == "macos"
    }

    /// The LLVM passes to run over each generated module, in order. Each entry is a
    /// pass-pipeline string for LLVM's pass builder.
    pub fn llvm_passes(&self) -> Vec<String> {
        if let Some(passes) = &self.llvm_passes_override {
            return passes.clone();
        }
        match self.fix_opt_level {
            FixOptimizationLevel::None => vec![],
            FixOptimizationLevel::Basic => vec![LLVM_O3_PIPELINE.to_string()],
            FixOptimizationLevel::Max | FixOptimizationLevel::Experimental => {
                let mut passes = vec![LLVM_O3_PIPELINE.to_string(); LLVM_O3_RUNS_FOR_SPEED];
                passes.extend(LLVM_TAIL_PASSES.iter().map(|pass| pass.to_string()));
                passes
            }
        }
    }

    /// Get hash value of the configurations that affect the object file generation.
    ///
    /// The fields are listed by hand, so every field of `Configuration` that changes the generated
    /// code has to be hashed here: one left out makes a build reuse the object files of a build that
    /// generated different code.
    ///
    /// `test_object_generation_hash_separates_code_generation_settings` gives each setting read here
    /// a value of its own and checks that the hash follows, so its list must stay in sync with this
    /// one.
    pub fn object_generation_hash(&self) -> String {
        let mut hash_source = String::new();
        hash_source.push_str(&self.fix_opt_level.to_string());
        hash_source.push_str(&self.debug_info.to_string());
        // `Generator::create_debug_info` writes the compilation directory into the debug
        // information, which is the one way it reaches the generated code, so a build without debug
        // information takes objects generated in another directory. A second reader of the field
        // would make this condition wrong.
        if self.debug_info {
            push_text_hash(
                &mut hash_source,
                &self.compilation_directory.to_string_lossy(),
            );
        }
        hash_source.push_str(&self.threaded.to_string());
        // The instrumentation is part of the code that is generated, so an object built without it
        // cannot stand in for one built with it. Leaving this out would let a build reuse
        // uninstrumented objects and report a clean run of a program nothing was checking.
        hash_source.push_str(&self.sanitizer.to_string());
        hash_source.push_str(&self.backtrace.to_string());
        hash_source.push_str(&self.no_runtime_check.to_string());
        hash_source.push_str(&self.skip_eval.to_string());
        hash_source.push_str(&self.c_type_sizes.to_string());
        hash_source.push_str(&self.max_split_scalars.to_string());
        // The kind of the output file reaches the code in two ways: a dynamic library is generated
        // with position-independent relocations (`get_target_machine`), and an executable is the
        // only kind that carries the entry point (`elaborate_via_config`). An object built for one
        // kind therefore fails to link into the other.
        hash_source.push_str(self.output_file_type.to_str());
        push_list_hash(&mut hash_source, &self.disable_cpu_features_regex);

        // The LLVM passes. `--llvm-passes-file` replaces the passes the optimization level
        // implies, so the pipeline is hashed in full: were it left out, objects generated under
        // one pipeline would be reused under another, and a comparison of two pipelines would
        // measure whichever one compiled first.
        push_list_hash(&mut hash_source, &self.llvm_passes());

        // Command type.
        // The implementation of the entry point function differs depending on the command type.
        hash_source.push_str(self.subcommand.command_type_string());

        // Build time of the compiler.
        hash_source.push_str(build_time_utc!());

        format!("{:x}", md5::compute(hash_source))
    }

    /// Apply this configuration's `disable_cpu_features_regex` to `features`, turning off every
    /// feature a pattern matches.
    pub fn edit_cpu_features(&self, features: &mut CpuFeatures) {
        features.disable_by_regexes(&self.disable_cpu_features_regex);
    }

    /// The `valgrind` invocation to run a built program under, set up for the tool selected in this
    /// configuration and for the errors the Fix runtime's memory management can produce.
    fn valgrind_command(&self) -> Result<Command, Errors> {
        // Check if valgrind is installed
        let which_output = Command::new("which").arg("valgrind").output();
        if which_output.is_err() || !which_output.unwrap().status.success() {
            return Err(Errors::from_msg(
                "valgrind is not installed on this system. Please install valgrind to use this feature.".to_string()
            ));
        }

        let mut command = Command::new("valgrind");
        command.arg("--error-exitcode=1"); // This option makes valgrind return 1 if an error is detected.

        // Add suppressions file if it exists
        if PathBuf::from("valgrind.supp").exists() {
            command.arg("--suppressions=valgrind.supp");
        }

        match self.valgrind_tool {
            ValgrindTool::None => {
                return Err(Errors::from_msg(
                    "Valgrind tool is not specified.".to_string(),
                ));
            }
            ValgrindTool::MemCheck => {
                // Check memory leaks.
                command.arg("--tool=memcheck");
                command.arg("--leak-check=yes"); // This option turns memory leak into error.

                // An array large enough to have its elements aligned sits above the base of its
                // allocation, so the only pointer to that block is an interior one, which the leak
                // checker calls possibly lost for as long as the array is alive. Take as errors the
                // kinds a reference counting mistake produces: a block nothing points to is
                // definitely lost, and one held only by such a block is indirectly lost.
                command.arg("--errors-for-leak-kinds=definite,indirect");
            }
        }
        Ok(command)
    }

    /// The linkage to give a symbol that other compilation units may call: external where each unit
    /// is compiled on its own, internal where the program is optimized as a whole.
    pub fn external_if_separated(&self) -> Linkage {
        if self.enable_separated_compilation() {
            Linkage::External
        } else {
            Linkage::Internal
        }
    }

    /// Instrument the generated program with `sanitizer`.
    ///
    /// A sanitizer this platform cannot provide is an error. Everything else here works to keep a
    /// build from calling itself sanitized while carrying no instrumentation, and quietly dropping
    /// the setting is that same failure arriving through the front door. A test that wants the
    /// instrumentation asks `platform_thread_sanitizer_supported` first and says that it skipped.
    pub fn set_sanitizer(&mut self, sanitizer: Sanitizer) -> Result<&mut Configuration, Errors> {
        if !sanitizer.platform_supported() {
            return Err(Errors::from_msg(format!(
                "The `{}` sanitizer is not available on this platform.",
                sanitizer
            )));
        }
        self.sanitizer = sanitizer;
        Ok(self)
    }

    /// Whether the settings this configuration carries can be met together.
    ///
    /// An instrumented program brings its own runtime, which lays out memory the way the sanitizer
    /// needs. Valgrind gives the program a machine of its own instead, and the two arrangements do
    /// not survive each other: the instrumented program dies at startup with a message that names
    /// neither setting.
    pub fn validate_run_settings(&self) -> Result<(), Errors> {
        if self.sanitizer != Sanitizer::None && self.valgrind_tool != ValgrindTool::None {
            return Err(Errors::from_msg(format!(
                "A program instrumented with the `{}` sanitizer cannot also be run under {}. \
                 Choose one of the two.",
                self.sanitizer, self.valgrind_tool
            )));
        }
        Ok(())
    }

    /// The command that runs the built program at `exec_path`, under whatever the run settings ask
    /// to run it under.
    pub fn program_run_command(&self, exec_path: &str) -> Result<Command, Errors> {
        assert!(
            self.validate_run_settings().is_ok(),
            "the run settings pick at most one of valgrind and a sanitizer"
        );
        if self.valgrind_tool != ValgrindTool::None {
            let mut com = self.valgrind_command()?;
            com.arg(exec_path);
            return Ok(com);
        }
        if self.sanitizer != Sanitizer::None {
            // A sanitizer maps its shadow memory to addresses it derives from the program's own, so
            // it needs the program where it expects to find it. `setarch -R` lays the address space
            // out the same way on every run, which is what lets the sanitizer start. A program built
            // by `fix build` is run by hand, so the same wrapper is what its user writes.
            let mut com = Command::new("setarch");
            com.arg(env::consts::ARCH).arg("-R").arg(exec_path);
            return Ok(com);
        }
        Ok(Command::new(exec_path))
    }

    pub fn run_preliminary_commands(&mut self) -> Result<(), Errors> {
        approve_and_run(self)
    }

    /// Whether the generated program keeps the checks that abort it on a violation, such as array
    /// bounds checks and the union variant checks of the `as_` functions.
    pub fn runtime_check(&self) -> bool {
        !self.no_runtime_check
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CTypeSizes {
    pub char: usize,
    pub short: usize,
    pub int: usize,
    pub long: usize,
    pub long_long: usize,
    pub size_t: usize,
    pub float: usize,
    pub double: usize,
}

impl CTypeSizes {
    // The C numeric types, each paired with the sign and the bit width of the Fix type it is an
    // alias of. The name built from those two must be one of `C_SCALAR_NAMES`, which is the set
    // `TyCon::get_c_type` can map.
    pub fn get_c_types(&self) -> Vec<(&str, &str, usize)> {
        vec![
            (C_CHAR_NAME, "I", self.char),
            (C_UNSIGNED_CHAR_NAME, "U", self.char),
            (C_SHORT_NAME, "I", self.short),
            (C_UNSIGNED_SHORT_NAME, "U", self.short),
            (C_INT_NAME, "I", self.int),
            (C_UNSIGNED_INT_NAME, "U", self.int),
            (C_LONG_NAME, "I", self.long),
            (C_UNSIGNED_LONG_NAME, "U", self.long),
            (C_LONG_LONG_NAME, "I", self.long_long),
            (C_UNSIGNED_LONG_LONG_NAME, "U", self.long_long),
            (C_SIZE_T_NAME, "U", self.size_t),
            (C_FLOAT_NAME, "F", self.float),
            (C_DOUBLE_NAME, "F", self.double),
        ]
    }

    fn to_string(&self) -> String {
        vec![
            format!("char: {}", self.char),
            format!("short: {}", self.short),
            format!("int: {}", self.int),
            format!("long: {}", self.long),
            format!("long long: {}", self.long_long),
            format!("size_t: {}", self.size_t),
            format!("float: {}", self.float),
            format!("double: {}", self.double),
        ]
        .join(", ")
    }

    // Get the size of each C types by compiling and running a C program.
    fn from_gcc() -> Result<Self, Errors> {
        // First, create a C source file to check the size of each C types.
        let c_source = r#"
#include <stdio.h>
#include <stddef.h>
#include <limits.h>
int main() {
    printf("%lu\n", sizeof(char) * CHAR_BIT);
    printf("%lu\n", sizeof(short) * CHAR_BIT);
    printf("%lu\n", sizeof(int) * CHAR_BIT);
    printf("%lu\n", sizeof(long) * CHAR_BIT);
    printf("%lu\n", sizeof(long long) * CHAR_BIT);
    printf("%lu\n", sizeof(size_t) * CHAR_BIT);
    printf("%lu\n", sizeof(float) * CHAR_BIT);
    printf("%lu\n", sizeof(double) * CHAR_BIT);
    return 0;
}
        "#;
        let mut finally = Finally::new();

        // Then save it to a temporary file ".fixlang/check_c_types.{random_number}.c".
        let check_c_types_path =
            CHECK_C_TYPES_PATH.to_string() + &format!(".{}.c", rand::random::<u32>());
        {
            // Create parent folders
            let check_c_types_path = PathBuf::from(check_c_types_path.clone());
            let parent = check_c_types_path.parent().unwrap();
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(Errors::from_msg(format!(
                    "Failed to create directory \"{}\": {}",
                    parent.to_string_lossy().to_string(),
                    e
                )));
            }

            let check_c_types_path_clone = check_c_types_path.clone();
            finally.defer(move || {
                let _ = fs::remove_file(&check_c_types_path_clone);
            });

            // Write the C source to the file.
            if let Err(e) = fs::write(&check_c_types_path, c_source) {
                return Err(Errors::from_msg(format!(
                    "Failed to write file \"{}\": {}",
                    check_c_types_path.to_string_lossy().to_string(),
                    e
                )));
            }
        }

        // Build the program to an executable file ".fixlang/check_c_types.out.{random_number}".
        let check_c_types_exec_path =
            CHECK_C_TYPES_PATH.to_string() + &format!(".{}.out", rand::random::<u32>());

        let check_c_types_exec_path_clone = check_c_types_exec_path.clone();
        finally.defer(move || {
            let _ = fs::remove_file(&check_c_types_exec_path_clone);
        });

        let output = Command::new("gcc")
            .arg(check_c_types_path.clone())
            .arg("-o")
            .arg(check_c_types_exec_path.clone())
            .output();
        if let Err(e) = output {
            return Err(Errors::from_msg(format!(
                "Failed to compile \"{}\": {}.",
                check_c_types_path, e
            )));
        }
        let output = output.unwrap();

        // Run the program and parse the result to create CTypeSizes.
        if !output.status.success() {
            return Err(Errors::from_msg(format!(
                "Failed to compile \"{}\": \"{}\".",
                check_c_types_path,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        let output = Command::new(check_c_types_exec_path.clone()).output();
        if let Err(e) = output {
            return Err(Errors::from_msg(format!(
                "Failed to run \"{}\": {}.",
                check_c_types_exec_path, e
            )));
        }
        let output = output.unwrap();
        if !output.status.success() {
            return Err(Errors::from_msg(format!(
                "Failed to run \"{}\": \"{}\".",
                check_c_types_exec_path,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        let output = String::from_utf8_lossy(&output.stdout);
        let mut lines = output.lines();
        // The program prints one size per line, in the order the fields are read here.
        let mut next_size = || -> usize { lines.next().unwrap().parse().unwrap() };
        let char = next_size();
        let short = next_size();
        let int = next_size();
        let long = next_size();
        let long_long = next_size();
        let size_t = next_size();
        let float = next_size();
        let double = next_size();
        let sizes = CTypeSizes {
            char,
            short,
            int,
            long,
            long_long,
            size_t,
            float,
            double,
        };
        Ok(sizes)
    }

    /// Write these sizes as JSON to `C_TYPES_JSON_PATH`, from where a later compiler run reads them
    /// back.
    fn save_to_file(&self) -> Result<(), Errors> {
        // Open json file.
        let path = C_TYPES_JSON_PATH;
        let file = File::create(path);
        if let Err(e) = file {
            return Err(Errors::from_msg(format!(
                "Failed to create \"{}\": {}",
                path, e
            )));
        }
        let file = file.unwrap();

        // Serialize and write to the file.
        if let Err(e) = serde_json::to_writer_pretty(file, self) {
            return Err(Errors::from_msg(format!(
                "Failed to write \"{}\": {}",
                path, e
            )));
        }
        Ok(())
    }

    fn load_file() -> Option<Self> {
        let path = PathBuf::from(C_TYPES_JSON_PATH);
        if !path.exists() {
            return None;
        }
        let file = File::open(path);
        if file.is_err() {
            warn_msg(&format!("Failed to open \"{}\".", C_TYPES_JSON_PATH));
            return None;
        }
        let file = file.unwrap();
        let sizes = serde_json::from_reader(file);
        if sizes.is_err() {
            warn_msg(&format!(
                "Failed to parse the content of \"{}\".",
                C_TYPES_JSON_PATH
            ));
            return None;
        }
        Some(sizes.unwrap())
    }

    fn load_or_check() -> Result<Self, Errors> {
        match Self::load_file() {
            Some(sizes) => Ok(sizes),
            None => {
                let sizes = Self::from_gcc()?;
                sizes.save_to_file()?;
                Ok(sizes)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The object generation hash of a build configuration to which `edit` has been applied.
    fn hash_after(edit: impl FnOnce(&mut Configuration)) -> String {
        let mut config = Configuration::release_mode(SubCommand::Build)
            .unwrap_or_else(|errs| panic!("Failed to create a configuration: {}", errs));
        edit(&mut config);
        config.object_generation_hash()
    }

    /// Two builds reuse each other's object files exactly when they agree on this hash, so each
    /// setting that reaches code generation gives the hash a value of its own.
    ///
    /// Each setting is written to its field, so that the list names what the hash reads, and the
    /// list must stay in sync with `object_generation_hash`.
    #[test]
    fn test_object_generation_hash_separates_code_generation_settings() {
        let baseline = hash_after(|_| {});

        let settings: Vec<(&str, Box<dyn FnOnce(&mut Configuration)>)> = vec![
            (
                "output_file_type",
                Box::new(|config: &mut Configuration| {
                    config.output_file_type = OutputFileType::DynamicLibrary
                }),
            ),
            (
                "fix_opt_level",
                Box::new(|config: &mut Configuration| {
                    // `FIX_MAX_OPT_LEVEL` decides the level the configuration starts at, so the
                    // level here is chosen to differ from the one that is there.
                    config.fix_opt_level = if config.fix_opt_level == FixOptimizationLevel::None {
                        FixOptimizationLevel::Max
                    } else {
                        FixOptimizationLevel::None
                    }
                }),
            ),
            (
                "llvm_passes_override",
                Box::new(|config: &mut Configuration| {
                    config.llvm_passes_override = Some(vec!["default<O0>".to_string()])
                }),
            ),
            (
                "debug_info",
                Box::new(|config: &mut Configuration| config.debug_info = true),
            ),
            (
                "compilation_directory",
                Box::new(|config: &mut Configuration| {
                    // The directory reaches the generated code as the directory the debug
                    // information records, so the two are one setting here.
                    config.debug_info = true;
                    config.compilation_directory = config.compilation_directory.join("elsewhere");
                }),
            ),
            (
                "threaded",
                Box::new(|config: &mut Configuration| config.threaded = true),
            ),
            (
                "sanitizer",
                Box::new(|config: &mut Configuration| config.sanitizer = Sanitizer::Thread),
            ),
            (
                "backtrace",
                Box::new(|config: &mut Configuration| config.backtrace = true),
            ),
            (
                "no_runtime_check",
                Box::new(|config: &mut Configuration| config.no_runtime_check = true),
            ),
            (
                "skip_eval",
                Box::new(|config: &mut Configuration| config.skip_eval = true),
            ),
            (
                "max_split_scalars",
                Box::new(|config: &mut Configuration| config.max_split_scalars += 1),
            ),
            (
                "c_type_sizes",
                Box::new(|config: &mut Configuration| config.c_type_sizes.long += 1),
            ),
            (
                "disable_cpu_features_regex",
                Box::new(|config: &mut Configuration| {
                    config.disable_cpu_features_regex.push("avx.*".to_string())
                }),
            ),
            (
                "subcommand",
                Box::new(|config: &mut Configuration| config.subcommand = SubCommand::Run),
            ),
        ];

        let mut settings_by_hash: Map<String, &str> = Map::default();
        for (name, edit) in settings {
            let hash = hash_after(edit);
            assert_ne!(
                baseline, hash,
                "`{}` reaches code generation, so it belongs in the object generation hash.",
                name
            );
            // Two settings landing on one hash would share each other's object files, so the hash
            // separates the settings from one another as well as from the baseline.
            if let Some(other_setting) = settings_by_hash.insert(hash, name) {
                panic!(
                    "`{}` and `{}` generate different code, so the object generation hash has to \
                     tell them apart.",
                    name, other_setting
                );
            }
        }
    }
}
