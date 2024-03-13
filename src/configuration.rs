use std::{env, path::PathBuf};

use build_time::build_time_utc;
use inkwell::OptimizationLevel;

use crate::misc::error_exit;

#[derive(Clone, Copy)]
pub enum LinkType {
    Static,
    Dynamic,
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
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FixOptimizationLevel {
    None,    // For debugging; skip even tail call optimization.
    Minimum, // For fast compilation.
    Default, // For fast execution.
}

impl std::fmt::Display for FixOptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixOptimizationLevel::None => write!(f, "None"),
            FixOptimizationLevel::Minimum => write!(f, "Minimum"),
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
        let mut config = Self::default();
        config.set_sanitize_memory();
        config
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

    pub fn get_output_llvm_ir_path(&self, pre_opt: bool, unit_name: &str) -> PathBuf {
        match &self.out_file_path {
            None => {
                if pre_opt {
                    return PathBuf::from(format!("pre_opt_{}.ll", unit_name));
                } else {
                    return PathBuf::from(format!("post_opt_{}.ll", unit_name));
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
                    let file_name = String::from(if pre_opt { "pre_opt_" } else { "post_opt_" })
                        + file_name
                        + "_"
                        + unit_name;
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
            FixOptimizationLevel::Default => OptimizationLevel::Default,
        }
    }

    pub fn perform_uncurry_optimization(&self) -> bool {
        match self.fix_opt_level {
            FixOptimizationLevel::None => false,
            FixOptimizationLevel::Minimum => false,
            FixOptimizationLevel::Default => true,
        }
    }

    pub fn perform_borrowing_optimization(&self) -> bool {
        match self.fix_opt_level {
            FixOptimizationLevel::None => false,
            FixOptimizationLevel::Minimum => false,
            FixOptimizationLevel::Default => true,
        }
    }

    fn add_terminate_tasks_macro_if_needed(&mut self) {
        if self.async_task && self.sanitize_memory {
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
        data.push_str(build_time_utc!()); // Also add build time of the compiler.
        format!("{:x}", md5::compute(data))
    }

    pub fn use_compilation_cache(&self) -> bool {
        self.fix_opt_level == FixOptimizationLevel::None
            || self.fix_opt_level == FixOptimizationLevel::Minimum
    }
}
