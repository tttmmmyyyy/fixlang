use std::{
    fs::{self, File, remove_file},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::Once,
};
use crate::{
    commands::run::run, configuration::Configuration, constants::COMPILER_TEST_WORKING_PATH, error::{Errors, panic_if_err, panic_with_msg}, misc::save_temporary_source 
};

static INSTALL_FIX: Once = Once::new();

// Build fix in release mode and copy it to ~/.cargo/bin/.
// This uses incremental compilation, so it's much faster than `cargo install` when already built.
// This function is thread-safe and will only perform the installation once.
pub fn install_fix() {
    INSTALL_FIX.call_once(|| {
        // Build the fix binary in release mode (uses cache if already built)
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .output()
            .expect("Failed to run cargo build --release");
        
        if !output.status.success() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to build fix in release mode");
        }
        
        // Copy the built binary to ~/.cargo/bin/ using a temporary file to avoid "Text file busy"
        let release_binary = PathBuf::from("target/release/fix");
        let cargo_bin = dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".cargo/bin");
        let _ = fs::create_dir_all(&cargo_bin);
        let dest = cargo_bin.join("fix");
        let temp_dest = cargo_bin.join(".fix.tmp");
        
        // Copy to temporary file first
        fs::copy(&release_binary, &temp_dest)
            .expect("Failed to copy fix binary to temporary location");
        
        // Make it executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_dest)
                .expect("Failed to get metadata")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&temp_dest, perms)
                .expect("Failed to set permissions");
        }
        
        // Atomically rename to final destination (replaces even if file is in use)
        fs::rename(&temp_dest, &dest)
            .expect("Failed to move fix binary to ~/.cargo/bin/fix");
    });
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

    // Build `c_source` into an object file.
    let o_file_path = format!("{}/{}.o", COMPILER_TEST_WORKING_PATH, test_name);
    let mut com = Command::new("gcc");
    let output = com
        .arg("-c")
        .arg("-o")
        .arg(&o_file_path)
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

    // Link the object file to the Fix program.
    let mut config = Configuration::develop_mode();
    config.object_files.push(PathBuf::from(&o_file_path));

    // Run the Fix program.
    test_source(&fix_src, config);

    // Remove the object file.
    let _ = fs::remove_file(o_file_path);
}

// Copy directory recursively
pub fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}