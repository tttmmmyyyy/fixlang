//! The commands a project file's `preliminary_commands` run before compilation.

use crate::{
    configuration::Configuration,
    constants::PRELIMINARY_BUILD_LD_FLAGS,
    error::Errors,
    misc::{split_string_by_space_not_quated, to_absolute_path},
};
use std::path::PathBuf;
use std::process::Command;

/// A command a project file asks to run before compilation, one argv of its
/// `preliminary_commands`.
#[derive(Clone)]
pub struct PreliminaryCommand {
    /// Absolute path of the directory of the project that lists the command, where it runs.
    pub work_dir: PathBuf,
    /// argv of the command, never empty: the first element is the executable.
    pub command: Vec<String>,
}

impl PreliminaryCommand {
    /// Runs this command to completion, and adds the flags of each `fix.ld_flags=...` line of its
    /// standard output to `config.ld_flags`. Fails when the process does not start or exits with a
    /// nonzero status.
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
