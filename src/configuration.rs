use std::process::Command;
use std::{env, path::PathBuf};

use build_time::build_time_utc;
use inkwell::module::Linkage;
use inkwell::OptimizationLevel;
use serde::{Deserialize, Serialize};

use crate::cpu_features::CpuFeatures;

use crate::constants::{CHECK_C_TYPES_EXEC_PATH, CHECK_C_TYPES_PATH, C_TYPES_JSON_PATH};
use crate::{error::error_exit, DEFAULT_COMPILATION_UNIT_MAX_SIZE};
use crate::{
    C_CHAR_NAME, C_DOUBLE_NAME, C_FLOAT_NAME, C_INT_NAME, C_LONG_LONG_NAME, C_LONG_NAME,
    C_SHORT_NAME, C_SIZE_T_NAME, C_UNSIGNED_CHAR_NAME, C_UNSIGNED_INT_NAME,
    C_UNSIGNED_LONG_LONG_NAME, C_UNSIGNED_LONG_NAME, C_UNSIGNED_SHORT_NAME,
};

#[derive(Clone, Copy)]
pub enum LinkType {
    Static,
    Dynamic,
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

#[derive(Clone)]
pub struct Configuration {
    // Source files.
    pub source_files: Vec<PathBuf>,
    // Runs memory sanitizer to detect memory leak and invalid memory reference at early time.
    // Requires shared library ,/sanitizer/libfixsanitizer.so.
    pub sanitize_memory: bool,
    // Fix's optimization level.
    pub fix_opt_level: FixOptimizationLevel,
    // Linked libraries
    pub linked_libraries: Vec<(String, LinkType)>,
    // Library search paths.
    pub library_search_paths: Vec<PathBuf>,
    // Create debug info.
    pub debug_info: bool,
    // Is emit llvm?
    pub emit_llvm: bool,
    // Output file name.
    pub out_file_path: Option<PathBuf>,
    // Use threads.
    // To turn on this true and link pthread library, use `set_threaded` function.
    pub threaded: bool,
    // Use AsyncTask module.
    pub async_task: bool,
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
    // Is this configuration for language server?
    pub for_language_server: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FixOptimizationLevel {
    None,      // For debugging; skip even tail call optimization.
    Minimum,   // For fast compilation.
    Separated, // Perform almost all of the optimizations except for LLVM-level LTO.
    Default,   // For fast execution.
}

impl std::fmt::Display for FixOptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixOptimizationLevel::None => write!(f, "None"),
            FixOptimizationLevel::Minimum => write!(f, "Minimum"),
            FixOptimizationLevel::Separated => write!(f, "Separated"),
            FixOptimizationLevel::Default => write!(f, "Default"),
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            source_files: vec![],
            sanitize_memory: false,
            fix_opt_level: FixOptimizationLevel::Default, // Fix's optimization level.
            linked_libraries: vec![],
            debug_info: false,
            emit_llvm: false,
            out_file_path: None,
            threaded: false,
            async_task: false,
            runtime_c_macro: vec![],
            show_build_times: false,
            verbose: false,
            max_cu_size: DEFAULT_COMPILATION_UNIT_MAX_SIZE,
            valgrind_tool: ValgrindTool::None,
            library_search_paths: vec![],
            c_type_sizes: CTypeSizes::load_or_check(),
            for_language_server: false,
        }
    }
}

impl Configuration {
    // Configuration for release build.
    pub fn release() -> Configuration {
        Self::default()
    }

    // Usual configuration for compiler development
    #[allow(dead_code)]
    pub fn develop_compiler() -> Configuration {
        #[allow(unused_mut)]
        let mut config = Self::default();
        config.set_valgrind(ValgrindTool::MemCheck);
        // config.fix_opt_level = FixOptimizationLevel::Separated;
        // config.set_sanitize_memory();
        // config.emit_llvm = true;
        // config.debug_info = true;
        config
    }

    // Create configuration for language server.
    pub fn for_language_server() -> Configuration {
        let mut config = Self::default();
        config.for_language_server = true;
        config
    }

    pub fn set_valgrind(&mut self, tool: ValgrindTool) -> &mut Configuration {
        self.valgrind_tool = tool;
        self
    }

    // Add dynamically linked library.
    // To link libabc.so, provide library name "abc".
    pub fn add_dyanmic_library(&mut self, name: &str) {
        self.linked_libraries
            .push((name.to_string(), LinkType::Dynamic));
    }

    // Add `libm.so` to dynamically linked libraries.
    pub fn add_libm(&mut self) {
        self.add_dyanmic_library("m");
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
                    error_exit(&format!(
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

    pub fn get_output_executable_file_path(&self) -> PathBuf {
        match &self.out_file_path {
            None => PathBuf::from(if env::consts::OS != "windows" {
                "a.out"
            } else {
                "a.exe"
            }),
            Some(out_file_path) => out_file_path.clone(),
        }
    }

    // Set threaded = true, and add ptherad library to linked_libraries.
    pub fn set_threaded(&mut self) {
        self.threaded = true;
        self.add_dyanmic_library("pthread");
    }

    pub fn set_async_task(&mut self) {
        self.async_task = true;
        self.set_threaded();
        self.runtime_c_macro.push("THREAD".to_string());
        self.add_terminate_tasks_macro_if_needed();
    }

    #[allow(dead_code)]
    pub fn set_sanitize_memory(&mut self) {
        self.sanitize_memory = true;
        self.add_terminate_tasks_macro_if_needed();
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
            FixOptimizationLevel::Minimum => OptimizationLevel::Less,
            FixOptimizationLevel::Separated => OptimizationLevel::Default,
            FixOptimizationLevel::Default => OptimizationLevel::Default,
        }
    }

    pub fn perform_uncurry_optimization(&self) -> bool {
        match self.fix_opt_level {
            FixOptimizationLevel::None => false,
            FixOptimizationLevel::Minimum => false,
            FixOptimizationLevel::Separated => true,
            FixOptimizationLevel::Default => true,
        }
    }

    pub fn perform_borrowing_optimization(&self) -> bool {
        match self.fix_opt_level {
            FixOptimizationLevel::None => false,
            FixOptimizationLevel::Minimum => false,
            FixOptimizationLevel::Separated => true,
            FixOptimizationLevel::Default => true,
        }
    }

    pub fn should_terminate_tasks(&self) -> bool {
        // Sanitizer and valgrind may detect detached thread as a memory leak.
        // To avoid this, wait for termination of detached threads before the program exits.
        self.async_task && self.sanitize_memory

        // Leak checking by valgrind has a similar problem that it may detect detached thread as a memory leak.
        // This is not resolved by waiting for the termination of detached threads.
        // We handle this problem just by ignoring `possibly lost` leaks.
    }

    fn add_terminate_tasks_macro_if_needed(&mut self) {
        if self.should_terminate_tasks() {
            self.runtime_c_macro.push("TERMINATE_TASKS".to_string());
        }
    }

    // Get hash value of the configurations that affect the object file generation.
    pub fn object_generation_hash(&self) -> String {
        let mut data = String::new();
        data.push_str(&self.sanitize_memory.to_string());
        data.push_str(&self.fix_opt_level.to_string());
        data.push_str(&self.debug_info.to_string());
        data.push_str(&self.threaded.to_string());
        data.push_str(&self.async_task.to_string());
        data.push_str(&self.c_type_sizes.to_string());
        data.push_str(build_time_utc!()); // Also add build time of the compiler.
        format!("{:x}", md5::compute(data))
    }

    pub fn separate_compilation(&self) -> bool {
        self.fix_opt_level != FixOptimizationLevel::Default
    }

    pub fn edit_features(&self, features: &mut CpuFeatures) {
        if self.valgrind_tool != ValgrindTool::None {
            features.disable_avx512(); // Valgrind-3.22.0 does not support AVX-512 (#41).
        }
    }

    pub fn valgrind_command(&self) -> Command {
        let mut com = Command::new("valgrind");
        com.arg("--error-exitcode=1"); // This option makes valgrind return 1 if an error is detected.
        com.arg("--suppressions=valgrind.supp");
        match self.valgrind_tool {
            ValgrindTool::None => {
                error_exit("Valgrind tool is not specified.");
            }
            ValgrindTool::MemCheck => {
                // Check memory leaks.
                com.arg("--tool=memcheck");
                com.arg("--leak-check=yes"); // This option turns memory leak into error.
                if self.async_task {
                    com.arg("--errors-for-leak-kinds=definite"); // Ignore `possibly lost` leaks, which are caused by detached threads.
                }
            }
        }
        com
    }

    pub fn external_if_separated(&self) -> Linkage {
        if self.separate_compilation() {
            Linkage::External
        } else {
            Linkage::Internal
        }
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
    fn from_gcc() -> Self {
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
        // Then save it to a temporary file ".fixlang/check_c_types.c".
        let check_c_types_path = PathBuf::from(CHECK_C_TYPES_PATH);
        // Create parent folders.
        let parent = check_c_types_path.parent().unwrap();
        if std::fs::create_dir_all(parent).is_err() {
            error_exit(&format!("Failed to create directory \"{:?}\".", parent));
        }
        if std::fs::write(&check_c_types_path, c_source).is_err() {
            error_exit(&format!(
                "Failed to write file \"{:?}\".",
                check_c_types_path
            ));
        }
        // Run it by gcc.
        let output = Command::new("gcc")
            .arg(CHECK_C_TYPES_PATH)
            .arg("-o")
            .arg(CHECK_C_TYPES_EXEC_PATH)
            .output();
        if output.is_err() {
            error_exit(&format!("Failed to compile \"{}\".", CHECK_C_TYPES_PATH));
        }
        let output = output.unwrap();
        // Run the program and parse the result to create CTypeSizes.
        if !output.status.success() {
            error_exit(&format!(
                "Failed to compile \"{}\": \"{}\".",
                CHECK_C_TYPES_PATH,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let output = Command::new(CHECK_C_TYPES_EXEC_PATH).output();
        if output.is_err() {
            error_exit(&format!("Failed to run \"{}\".", CHECK_C_TYPES_EXEC_PATH));
        }
        let output = output.unwrap();
        if !output.status.success() {
            error_exit(&format!(
                "Failed to run \"{}\": \"{}\".",
                CHECK_C_TYPES_EXEC_PATH,
                String::from_utf8_lossy(&output.stderr)
            ));
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
        res
    }

    fn save_to_file(&self) {
        // Open json file.
        let path = C_TYPES_JSON_PATH;
        let file = std::fs::File::create(path);
        if file.is_err() {
            error_exit(&format!("Failed to create \"{}\".", path));
        }
        let file = file.unwrap();
        // Serialize and write to the file.
        serde_json::to_writer_pretty(file, self)
            .expect(format!("Failed to write \"{}\".", path).as_str());
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

    fn load_or_check() -> Self {
        match Self::load_file() {
            Some(sizes) => sizes,
            None => {
                let sizes = Self::from_gcc();
                sizes.save_to_file();
                sizes
            }
        }
    }
}
