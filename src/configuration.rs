use std::{env, path::PathBuf};

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
    // Requires shared library sanitizer/libfixsanitizer.so.
    pub sanitize_memory: bool,
    // LLVM optimization level.
    pub llvm_opt_level: OptimizationLevel,
    // Linked libraries
    pub linked_libraries: Vec<(String, LinkType)>,
    // Make reference counting atomic.
    pub atomic_refcnt: bool,
    // Skip optimization and create debug info.
    pub debug_mode: bool,
    // Perform uncurrying optimization.
    pub uncurry_optimization: bool,
    // Is emit llvm?
    pub emit_llvm: bool,
    // Output file name.
    pub out_file_path: Option<PathBuf>,
    // Use threads.
    // To turn on this true and link pthread library, use `set_threaded` function.
    pub threaded: bool,
}

impl Configuration {
    // Configuration for release build.
    pub fn release() -> Configuration {
        Configuration {
            source_files: vec![],
            sanitize_memory: false,
            uncurry_optimization: true, // determined by debug_mode
            llvm_opt_level: OptimizationLevel::Default,
            linked_libraries: vec![],
            atomic_refcnt: false,
            debug_mode: false,
            emit_llvm: false,
            out_file_path: None,
            threaded: false,
        }
    }

    // Usual configuration for compiler development
    #[allow(dead_code)]
    pub fn develop_compiler() -> Configuration {
        Configuration {
            source_files: vec![],
            sanitize_memory: true,
            uncurry_optimization: true,
            llvm_opt_level: OptimizationLevel::Default,
            linked_libraries: vec![],
            atomic_refcnt: false,
            debug_mode: false,
            emit_llvm: false,
            out_file_path: None,
            threaded: false,
        }
    }

    // Add dynamically linked library.
    // To link libabc.so, provide library name "abc".
    pub fn add_dyanmic_library(&mut self, name: &str) {
        self.linked_libraries
            .push((name.to_string(), LinkType::Dynamic));
    }

    pub fn get_output_llvm_ir_path(&self, pre_opt: bool) -> PathBuf {
        match &self.out_file_path {
            None => {
                if pre_opt {
                    return PathBuf::from("pre_opt.ll");
                } else {
                    return PathBuf::from("post_opt.ll");
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
                    let file_name =
                        String::from(if pre_opt { "pre_opt_" } else { "post_opt_" }) + file_name;
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
}
