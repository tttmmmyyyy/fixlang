use crate::commands::build::build;
use crate::configuration::{Configuration, OutputFileType};
use crate::constants::{DOT_FIXLANG, RUN_PATH};
use crate::error::{panic_if_err, panic_with_msg, Errors};
use crate::project_fs::remove_inside_dot_fixlang;
use rand::{thread_rng, Rng};
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
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
    let user_specified_out_path = config.out_file_path.take();
    let executable = TemporaryExecutable::build(config)?;
    let config = executable.config();
    let mut com = config.program_run_command(executable.path())?;
    for arg in &config.run_program_args {
        com.arg(arg);
    }
    if inherit_streams {
        com.stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit());
    }
    let output = com.output();
    if let Some(out_path) = user_specified_out_path {
        executable.keep_at(&out_path);
    }
    Ok(output)
}

/// An executable built under `RUN_PATH`, which is removed when the value is dropped.
pub struct TemporaryExecutable {
    /// The settings the executable was built with, whose output path is `path`.
    config: Configuration,
    /// The path of the executable.
    path: PathBuf,
    /// Whether the executable was moved away from `path` to stay, which leaves nothing to remove.
    kept: bool,
}

impl TemporaryExecutable {
    /// Builds the program `config` describes as an executable under `RUN_PATH`. The output path
    /// `config` names is replaced by a path of its own, so that builds running at the same time
    /// write different files.
    pub fn build(mut config: Configuration) -> Result<Self, Errors> {
        // The kind of the output file describes what `fix build` produces, so the settings that name
        // a dynamic library reach `fix build` alone (`ProjectFile::set_config`,
        // `set_config_from_args`). A shared object put here would be handed to the operating system
        // as a program to execute.
        assert!(
            matches!(config.output_file_type, OutputFileType::Executable),
            "a run builds an executable, which is what it then runs"
        );

        fs::create_dir_all(DOT_FIXLANG)
            .expect(format!("Failed to create \"{}\" directory.", DOT_FIXLANG).as_str());
        fs::create_dir_all(RUN_PATH)
            .expect(format!("Failed to create \"{}\" directory.", RUN_PATH).as_str());

        let path = PathBuf::from(RUN_PATH).join(format!("a{}.out", thread_rng().gen::<u64>()));
        config.out_file_path = Some(path.clone());
        config.validate_run_settings()?;
        // The value is made before the build, so that a build failing after it wrote the file
        // removes it.
        let executable = TemporaryExecutable {
            config,
            path,
            kept: false,
        };
        build(&executable.config)?;
        Ok(executable)
    }

    /// The settings the executable was built with.
    pub fn config(&self) -> &Configuration {
        &self.config
    }

    /// The path of the executable.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Moves the executable to `out_path`, where it stays.
    pub fn keep_at(mut self, out_path: &Path) {
        if let Err(e) = fs::rename(&self.path, out_path) {
            let _ = remove_inside_dot_fixlang(&self.path); // Ignore the error.
            panic_with_msg(&format!(
                "Failed to rename \"{}\" to \"{}\": {}",
                self.path.display(),
                out_path.display(),
                e
            ));
        }
        self.kept = true;
    }
}

impl Drop for TemporaryExecutable {
    fn drop(&mut self) {
        if !self.kept {
            let _ = remove_inside_dot_fixlang(&self.path); // Ignore the error.
        }
    }
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
