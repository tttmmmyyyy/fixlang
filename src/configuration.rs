use crate::constants::{CHECK_C_TYPES_PATH, C_TYPES_JSON_PATH};
use crate::cpu_features::CpuFeatures;
use crate::error::{panic_if_err, Errors};
use crate::misc::{split_string_by_space_not_quated, to_absolute_path, warn_msg, Finally};
use crate::typecheckcache::{self, TypeCheckCache};
use crate::{error::panic_with_msg, DEFAULT_COMPILATION_UNIT_MAX_SIZE};
use crate::{
    C_CHAR_NAME, C_DOUBLE_NAME, C_FLOAT_NAME, C_INT_NAME, C_LONG_LONG_NAME, C_LONG_NAME,
    C_SHORT_NAME, C_SIZE_T_NAME, C_UNSIGNED_CHAR_NAME, C_UNSIGNED_INT_NAME,
    C_UNSIGNED_LONG_LONG_NAME, C_UNSIGNED_LONG_NAME, C_UNSIGNED_SHORT_NAME,
    OPTIMIZATION_LEVEL_BASIC, OPTIMIZATION_LEVEL_EXPERIMENTAL, OPTIMIZATION_LEVEL_MAX,
    OPTIMIZATION_LEVEL_NONE, PRELIMINARY_BUILD_LD_FLAGS,
};
use build_time::build_time_utc;
use inkwell::module::Linkage;
use inkwell::OptimizationLevel;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use std::{env, path::PathBuf};

#[derive(Clone, Copy)]
pub enum LinkType {
    Static,
    Dynamic,
}

#[derive(Clone, Copy)]
pub enum OutputFileType {
    Executable,
    DynamicLibrary,
}

impl OutputFileType {
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

    pub fn to_str(&self) -> &str {
        match self {
            OutputFileType::Executable => "exe",
            OutputFileType::DynamicLibrary => "dylib",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ValgrindTool {
    None,
    MemCheck,
    // Currently, we cannot use DRD or helgrind because valgrind does not understand atomic operations.
    // In C/C++ program, we can use `ANNOTATE_HAPPENS_BEFORE` and `ANNOTATE_HAPPENS_AFTER` to tell helgrind happens-before relations,
    // but how can we do similar things in Fix?
    // DataRaceDetection,
}

impl std::fmt::Display for ValgrindTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValgrindTool::None => write!(f, "none"),
            ValgrindTool::MemCheck => write!(f, "memcheck"),
        }
    }
}

// Subcommands of the `fix` command.
#[derive(Clone)]
pub enum SubCommand {
    Build,
    Run,
    Test,
    Diagnostics(DiagnosticsConfig),
    Docs(DocsConfig),
}

impl SubCommand {
    // Should we run preliminary commands before building the program?
    pub fn run_preliminary_commands(&self) -> bool {
        match self {
            SubCommand::Build => true,
            SubCommand::Run => true,
            SubCommand::Test => true,
            SubCommand::Diagnostics(_) => false,
            SubCommand::Docs(_) => false,
        }
    }

    // Should we build program binary?
    pub fn build_binary(&self) -> bool {
        match self {
            SubCommand::Build => true,
            SubCommand::Run => true,
            SubCommand::Test => true,
            SubCommand::Diagnostics(_) => false,
            SubCommand::Docs(_) => false,
        }
    }

    // Should we use test files?
    pub fn use_test_files(&self) -> bool {
        match self {
            SubCommand::Build => false,
            SubCommand::Run => false,
            SubCommand::Test => true,
            SubCommand::Diagnostics(_) => true,
            SubCommand::Docs(_) => true,
        }
    }

    // Should we typecheck the program?
    pub fn typecheck(&self) -> bool {
        match self {
            SubCommand::Build => true,
            SubCommand::Run => true,
            SubCommand::Test => true,
            SubCommand::Diagnostics(_) => true,
            SubCommand::Docs(_) => false,
        }
    }

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
}

#[derive(Clone)]
pub struct Configuration {
    // Source files.
    pub source_files: Vec<PathBuf>,
    // Object files to be linked.
    pub object_files: Vec<PathBuf>,
    // Fix's optimization level.
    pub fix_opt_level: FixOptimizationLevel,
    // Linked libraries
    pub linked_libraries: Vec<(String, LinkType)>,
    // Library search paths.
    pub library_search_paths: Vec<PathBuf>,
    // Other linker flags
    pub ld_flags: Vec<String>,
    // Create debug info.
    pub debug_info: bool,
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
    // Run program with valgrind. Effective only in `run` mode.
    pub valgrind_tool: ValgrindTool,
    // Sizes of C types.
    pub c_type_sizes: CTypeSizes,
    // Regex patterns of disabled CPU features.
    pub disable_cpu_features_regex: Vec<String>,
    // Subcommand of the `fix` command.
    pub subcommand: SubCommand,
    // Extra build commands.
    pub extra_commands: Vec<ExtraCommand>,
    // Typecheck cache.
    pub type_check_cache: Arc<dyn TypeCheckCache + Send + Sync>,
    // Number of worker threads.
    pub num_worker_thread: usize,
    // The arguments which are passed to the program in `run` mode.
    pub run_program_args: Vec<String>,
    // File containing LLVM passes.
    // Used only for compiler development.
    pub llvm_passes_file: Option<PathBuf>,
    // Emit symbols at each step of optimization.
    // Used only for compiler development.
    pub emit_symbols: bool,
    // Is in compiler development mode?
    pub develop_mode: bool,
    // Enable backtrace support (keep frame pointers and add backtrace library).
    pub backtrace: bool,
    // Disable runtime checks such as array bounds check.
    pub no_runtime_check: bool,
}

#[derive(Clone)]
pub struct ExtraCommand {
    pub work_dir: PathBuf,
    pub command: Vec<String>,
}

impl ExtraCommand {
    pub fn run(&self, config: &mut Configuration) -> Result<(), Errors> {
        let mut com = Command::new(&self.command[0]);
        for arg in &self.command[1..] {
            com.arg(arg);
        }
        let work_dir = to_absolute_path(&self.work_dir)?;
        com.current_dir(&work_dir);
        let status = com.status().map_err(|e| {
            Errors::from_msg(format!(
                "Failed to run command \"{}\": {:?}",
                self.command.join(" "),
                e
            ))
        })?;
        if !status.success() {
            return Err(Errors::from_msg(format!(
                "Command \"{}\" failed with exit code {}.",
                self.command.join(" "),
                status.code().unwrap_or(-1)
            )));
        }

        // Get stdout as String.
        let output = com.output().map_err(|e| {
            Errors::from_msg(format!(
                "Failed to run command \"{}\": {:?}",
                self.command.join(" "),
                e
            ))
        })?;

        // If the command outputs build flags in the designated format, add them to the configuration.
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout_lines: Vec<&str> = stdout.lines().collect();
        for stdout_line in stdout_lines {
            if stdout_line.starts_with(PRELIMINARY_BUILD_LD_FLAGS) {
                let ld_flags = stdout_line[PRELIMINARY_BUILD_LD_FLAGS.len()..].trim();
                let mut ld_flags = split_string_by_space_not_quated(ld_flags);
                config.ld_flags.append(&mut ld_flags);
            }
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum FixOptimizationLevel {
    None,         // For debugging; skip even tail call optimization.
    Basic,        // Perform almost all of the optimizations except for LLVM-level LTO.
    Max,          // For fast execution.
    Experimental, // Performs optimizations that are still unstable.
}

impl std::fmt::Display for FixOptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixOptimizationLevel::None => write!(f, "{}", OPTIMIZATION_LEVEL_NONE),
            FixOptimizationLevel::Basic => write!(f, "{}", OPTIMIZATION_LEVEL_BASIC),
            FixOptimizationLevel::Max => write!(f, "{}", OPTIMIZATION_LEVEL_MAX),
            FixOptimizationLevel::Experimental => write!(f, "{}", OPTIMIZATION_LEVEL_EXPERIMENTAL),
        }
    }
}

impl FixOptimizationLevel {
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
    fn new(subcommand: SubCommand) -> Result<Self, Errors> {
        Ok(Configuration {
            subcommand,
            source_files: vec![],
            object_files: vec![],
            fix_opt_level: FixOptimizationLevel::Max, // Fix's optimization level.
            linked_libraries: vec![],
            ld_flags: vec![],
            debug_info: false,
            emit_llvm: false,
            out_file_path: None,
            output_file_type: OutputFileType::Executable,
            threaded: false,
            runtime_c_macro: vec![],
            show_build_times: false,
            verbose: false,
            max_cu_size: DEFAULT_COMPILATION_UNIT_MAX_SIZE,
            valgrind_tool: ValgrindTool::None,
            library_search_paths: vec![],
            c_type_sizes: CTypeSizes::load_or_check()?,
            disable_cpu_features_regex: vec![],
            extra_commands: vec![],
            type_check_cache: Arc::new(typecheckcache::FileCache::new()),
            num_worker_thread: 0,
            llvm_passes_file: None,
            run_program_args: vec![],
            emit_symbols: false,
            develop_mode: false,
            backtrace: false,
            no_runtime_check: false,
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
        config.fix_opt_level = FixOptimizationLevel::Experimental;
        // config.set_sanitize_memory();
        config.emit_llvm = true;
        // config.debug_info = true;
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
    pub fn diagnostics_mode(config: DiagnosticsConfig) -> Result<Configuration, Errors> {
        let mut config = Self::new(SubCommand::Diagnostics(config))?;
        config.num_worker_thread = num_cpus::get();
        Ok(config)
    }

    pub fn set_valgrind(&mut self, tool: ValgrindTool) -> &mut Configuration {
        if env::consts::OS != "linux" && tool != ValgrindTool::None {
            warn_msg(&format!(
                "Valgrind is only supported on Linux. Ignoring valgrind settings `{}`",
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
            None => {
                let path = match self.output_file_type {
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
                };
                PathBuf::from(path)
            }
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
        self.fix_opt_level = level;
    }

    pub fn get_llvm_opt_level(&self) -> OptimizationLevel {
        match self.fix_opt_level {
            FixOptimizationLevel::None => OptimizationLevel::None,
            FixOptimizationLevel::Basic => OptimizationLevel::Default,
            FixOptimizationLevel::Max => OptimizationLevel::Default,
            FixOptimizationLevel::Experimental => OptimizationLevel::Default,
        }
    }

    pub fn force_all_optimizations(&self) -> bool {
        false
    }

    pub fn enable_separated_compilation(&self) -> bool {
        !self.force_all_optimizations() && self.fix_opt_level <= FixOptimizationLevel::Basic
    }

    pub fn enable_uncurry_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Basic
    }

    pub fn enable_remove_tyanno_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    pub fn enable_remove_hktvs_transformation(&self) -> bool {
        self.force_all_optimizations() || self.enable_unwrap_newtype_optimization()
    }

    pub fn enable_unwrap_newtype_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    pub fn enable_inline_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    pub fn enable_inline_local_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    pub fn enable_decapturing_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    pub fn enable_act_optimization(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    pub fn enable_simplify_symbol_names(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Experimental
    }

    pub fn enable_dead_symbol_elimination(&self) -> bool {
        self.force_all_optimizations() || self.fix_opt_level >= FixOptimizationLevel::Max
    }

    pub fn set_backtrace(&mut self) {
        self.backtrace = true;
        self.runtime_c_macro.push("BACKTRACE".to_string());
        if env::consts::OS == "linux" {
            self.add_dynamic_library("backtrace");
        }
    }

    // Check if frame pointers should not be eliminated.
    // This is necessary on macOS when backtrace is enabled, as backtrace() relies on frame pointers.
    pub fn no_elim_frame_pointers(&self) -> bool {
        self.backtrace && env::consts::OS == "macos"
    }

    // Get hash value of the configurations that affect the object file generation.
    pub fn object_generation_hash(&self) -> String {
        let mut data = String::new();
        data.push_str(&self.fix_opt_level.to_string());
        data.push_str(&self.debug_info.to_string());
        data.push_str(&self.threaded.to_string());
        data.push_str(&self.backtrace.to_string());
        data.push_str(&self.c_type_sizes.to_string());
        for disabled_cpu_feature in &self.disable_cpu_features_regex {
            // To ensure that the arrays ["xy", "x"] and ["x", "xy"] produce different hash values, we hash each element before concatenation instead of simply joining them.
            data.push_str(&format!("{:x}", md5::compute(disabled_cpu_feature)));
        }

        // Command type.
        // The implementation of the entry point function differs depending on the command type.
        data.push_str(self.subcommand.command_type_string());

        // Build time of the compiler.
        data.push_str(build_time_utc!());

        format!("{:x}", md5::compute(data))
    }

    // Edit CPU features according to the configuration.
    pub fn edit_cpu_features(&self, features: &mut CpuFeatures) {
        features.disable_by_regexes(&self.disable_cpu_features_regex);
    }

    pub fn valgrind_command(&self) -> Command {
        let mut com = Command::new("valgrind");
        com.arg("--error-exitcode=1"); // This option makes valgrind return 1 if an error is detected.
        com.arg("--suppressions=valgrind.supp");
        match self.valgrind_tool {
            ValgrindTool::None => {
                panic_with_msg("Valgrind tool is not specified.");
            }
            ValgrindTool::MemCheck => {
                // Check memory leaks.
                com.arg("--tool=memcheck");
                com.arg("--leak-check=yes"); // This option turns memory leak into error.
            }
        }
        com
    }

    pub fn external_if_separated(&self) -> Linkage {
        if self.enable_separated_compilation() {
            Linkage::External
        } else {
            Linkage::Internal
        }
    }

    pub fn run_extra_commands(&mut self) -> Result<(), Errors> {
        for com in &self.extra_commands.clone() {
            com.run(self)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_diagnostics_mode(&self) -> bool {
        match &self.subcommand {
            SubCommand::Diagnostics(_) => true,
            _ => false,
        }
    }

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
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(Errors::from_msg(format!(
                    "Failed to create directory \"{}\": {}",
                    parent.to_string_lossy().to_string(),
                    e
                )));
            }

            let check_c_types_path_clone = check_c_types_path.clone();
            finally.defer(move || {
                let _ = std::fs::remove_file(&check_c_types_path_clone);
            });

            // Write the C source to the file.
            if let Err(e) = std::fs::write(&check_c_types_path, c_source) {
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
            let _ = std::fs::remove_file(&check_c_types_exec_path_clone);
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
        let char = lines.next().unwrap().parse().unwrap();
        let short = lines.next().unwrap().parse().unwrap();
        let int = lines.next().unwrap().parse().unwrap();
        let long = lines.next().unwrap().parse().unwrap();
        let long_long = lines.next().unwrap().parse().unwrap();
        let size_t = lines.next().unwrap().parse().unwrap();
        let float = lines.next().unwrap().parse().unwrap();
        let double = lines.next().unwrap().parse().unwrap();
        let res = CTypeSizes {
            char,
            short,
            int,
            long,
            long_long,
            size_t,
            float,
            double,
        };
        Ok(res)
    }

    fn save_to_file(&self) -> Result<(), Errors> {
        // Open json file.
        let path = C_TYPES_JSON_PATH;
        let file = std::fs::File::create(path);
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
        let file = std::fs::File::open(path);
        if file.is_err() {
            eprintln!("Failed to open \"{}\".", C_TYPES_JSON_PATH);
            return None;
        }
        let file = file.unwrap();
        let sizes = serde_json::from_reader(file);
        if sizes.is_err() {
            eprintln!("Failed to parse the content of \"{}\".", C_TYPES_JSON_PATH);
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
