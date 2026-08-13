use crate::commands::build::build;
use crate::configuration::{Configuration, OutputFileType};
use crate::constants::{DOT_FIXLANG, RUN_PATH};
use crate::error::{panic_if_err, panic_with_msg, Errors};
use rand::Rng;
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{self, Output, Stdio};

/// Builds the program as an executable under `RUN_PATH` and runs it, passing it the arguments
/// `config.run_program_args` carries. The executable is then moved to the path
/// `config.out_file_path` names, and removed when the settings name no path.
///
/// # Arguments
///
/// * `inherit_streams` - Hands the program the standard streams of the `fix` process, so that it
///   reads from the terminal and writes to it as it runs. Otherwise its output is collected into
///   the returned `Output`.
///
/// # Returns
///
/// The outer result reports what went wrong while building the program, and the inner one what
/// went wrong while starting the built executable.
pub fn run(
    mut config: Configuration,
    inherit_streams: bool,
) -> Result<Result<Output, io::Error>, Errors> {
    // The kind of the output file describes what `fix build` produces, so the settings that name a
    // dynamic library reach `fix build` alone (`ProjectFile::set_config`, `set_config_from_args`).
    // A shared object put here would be handed to the operating system as a program to execute.
    assert!(
        matches!(config.output_file_type, OutputFileType::Executable),
        "a run builds an executable, which is what it then runs"
    );

    fs::create_dir_all(DOT_FIXLANG)
        .expect(format!("Failed to create \"{}\" directory.", DOT_FIXLANG).as_str());
    fs::create_dir_all(RUN_PATH)
        .expect(format!("Failed to create \"{}\" directory.", RUN_PATH).as_str());

    // For parallel execution, use different file name for each execution.
    let exec_path: String = format!("{}/a{}.out", RUN_PATH, rand::thread_rng().gen::<u64>());
    let user_specified_out_path = std::mem::replace(
        &mut config.out_file_path,
        Some(PathBuf::from(exec_path.clone())),
    );

    config.validate_run_settings()?;

    // Build executable file.
    build(&mut config)?;

    // Run the executable file.
    let mut com = config.program_run_command(&exec_path)?;
    for arg in &config.run_program_args {
        com.arg(arg);
    }
    if inherit_streams {
        com.stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit());
    }
    let output = com.output();

    // Clean up the temporary executable file.
    match user_specified_out_path {
        Some(out_path) => {
            // Move the temporary executable file to the specified output file.
            if let Err(e) = fs::rename(exec_path.clone(), out_path.clone()) {
                let _ = fs::remove_file(exec_path.clone()); // Ignore the error.
                panic_with_msg(&format!(
                    "Failed to rename \"{}\" to \"{}\": {}",
                    exec_path,
                    out_path.display(),
                    e
                ));
            }
        }
        None => {
            // If the output file is not specified, remove the temporary executable file.
            let _ = fs::remove_file(exec_path.clone()); // Ignore the error.
        }
    }

    Ok(output)
}

/// Builds the program, runs it with the terminal's streams attached, and exits the `fix` process
/// with the status the program returned. A program that a signal ends aborts `fix` instead.
pub fn run_command(config: &Configuration) {
    let output = run(config.clone(), true);
    let output = panic_if_err(output);

    if let Err(e) = output {
        panic_with_msg(&format!("Failed to run the program: {}", e));
    }
    let output = output.unwrap();

    if output.status.code().is_none() {
        #[cfg(unix)]
        {
            if let Some(signal) = output.status.signal() {
                panic_with_msg(&format!("Program terminated by signal {}", signal));
            }
        }
        panic_with_msg("Program terminated by signal");
    }
    let code = output.status.code().unwrap();

    process::exit(code);
}
