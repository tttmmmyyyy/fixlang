//! The commands a project file's `preliminary_commands` run before compilation.

use crate::{
    constants::PRELIMINARY_BUILD_LD_FLAGS,
    error::Errors,
    misc::split_string_by_space_not_quated,
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
    /// Runs this command to completion, and returns the linker flags its standard output gives on
    /// `fix.ld_flags=...` lines. Fails when the process does not start or exits with a nonzero
    /// status.
    ///
    /// # Examples
    /// A command printing `fix.ld_flags=-L/opt/lib -lfoo` returns `["-L/opt/lib", "-lfoo"]`.
    pub fn run(&self) -> Result<Vec<String>, Errors> {
        let mut com = Command::new(&self.command[0]);
        for arg in &self.command[1..] {
            com.arg(arg);
        }
        com.current_dir(&self.work_dir);
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

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter_map(|line| line.strip_prefix(PRELIMINARY_BUILD_LD_FLAGS))
            .flat_map(|flags| split_string_by_space_not_quated(flags.trim()))
            .collect())
    }
}
