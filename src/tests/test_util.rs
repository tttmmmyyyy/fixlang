use std::{
    fs::{self, File, remove_file},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};
use crate::{
    configuration::Configuration, constants::COMPILER_TEST_WORKING_PATH, error::{Errors, panic_if_err, panic_with_msg}, misc::save_temporary_source, runner::run
};

// Run `cargo install --locked --path .`.
pub fn install_fix() {
    let _ = Command::new("cargo")
        .arg("install")
        .arg("--locked")
        .arg("--path")
        .arg(".")
        .output()
        .expect("Failed to run cargo install.");
}

fn run_source(
    source: &str,
    mut config: Configuration,
) -> Result<Result<Output, io::Error>, Errors> {
    const MAIN_RUN: &str = "main_run";
    let src = save_temporary_source(source, MAIN_RUN)?;
    config.source_files.push(src.file_path);
    run(config, false)
}

pub fn test_source(source: &str, config: Configuration) {
    let res = run_source(source, config);
    let res = panic_if_err(res);
    let output = res.unwrap();
    let code = match output.status.code() {
        Some(code) => code,
        None => {
            eprintln!(
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            panic_with_msg("The process was terminated by signal.")
        },
    };
    if code != 0 {
        eprintln!(
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic_with_msg(&format!("The program exited with non-zero code: {}", code));
    }
}

pub fn test_source_fail(source: &str, config: Configuration, included_errmsg: &str) {
    let res = run_source(source, config);
    let errmsg = match res {
        Err(errs) => errs.to_string(),
        Ok(run_output) => match run_output {
            Err(e) => e.to_string(),
            Ok(output) => {
                let code = output.status.code();
                if let Some(code) = code {
                    if code == 0 {
                        panic_with_msg("The source code was expected to fail, but succeeded.");
                    }
                }
                String::from_utf8_lossy(&output.stderr).to_string()
            }
        },
    };
    assert!(errmsg.contains(included_errmsg), 
        "Error message did not contain expected text.\nExpected to include:\n{}\n\nActual message:\n{}", included_errmsg, errmsg);
}

// Run all "*.fix" files in the specified directory.
// If the directory contains subdirectories, run Fix program consists of all "*.fix" files in each subdirectory.
pub fn test_files_in_directory(path: &Path) {
    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let mut config = Configuration::develop_mode();
        if path.is_dir() {
            // Skip hidden directories.
            if path.file_name().unwrap().to_str().unwrap().starts_with(".") {
                continue;
            }

            // For each directory in "tests" directory, run Fix program which consists of "*.fix" files in the directory.
            let files = fs::read_dir(&path).unwrap();
            for file in files {
                let file = file.unwrap().path();
                if file.extension().is_none() || file.extension().unwrap() != "fix" {
                    continue;
                }
                config.source_files.push(file);
            }
        } else {
            // For each file which has extention "fix" in "tests" directory, run it as Fix program.
            if path.extension().is_none() || path.extension().unwrap() != "fix" {
                continue;
            }
            config.source_files.push(path.clone());
        }
        println!("[{}]:", path.to_string_lossy().to_string());
        let res = run(config, false);
        let res = panic_if_err(res);
        let output = res.unwrap();
        let code = output.status.code().unwrap();
        assert_eq!(code, 0);
        remove_file("test_process_text_file.txt").unwrap_or(());
    }
}

pub fn test_source_with_c(fix_src: &str, c_src: &str, test_name: &str) {
    // Create a working directory.
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);

    // Save `c_source` to a file.
    let c_file = format!("{}/{}.c", COMPILER_TEST_WORKING_PATH, test_name);
    let mut file = File::create(&c_file).unwrap();
    file.write_all(c_src.as_bytes()).unwrap();

    // Build `c_source` into a shared library.
    let lib_name = test_name;
    let so_file_path = format!("lib{}.so", lib_name);
    let mut com = Command::new("gcc");
    let output = com
        .arg("-shared")
        .arg("-fPIC")
        .arg("-o")
        .arg(so_file_path.clone())
        .arg(&c_file)
        .output()
        .expect("Failed to run gcc.");
    if output.stderr.len() > 0 {
        eprintln!(
            "{}",
            String::from_utf8(output.stderr)
                .unwrap_or("(failed to parse stderr from gcc as UTF8.)".to_string())
        );
    }

    // Link the shared library to the Fix program.
    let mut config = Configuration::develop_mode();
    config.add_dynamic_library(lib_name);
    // Add the library search path.
    config.library_search_paths.push(PathBuf::from("."));

    // Run the Fix program.
    test_source(&fix_src, config);

    // Remove the shared library.
    let _ = fs::remove_file(so_file_path);
}