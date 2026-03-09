use crate::configuration::{BuildConfigType, Configuration, SubCommand};
use crate::elaboration::elaborate_via_config;
use crate::error::Errors;
use crate::metafiles::project_file::ProjectFile;
use crate::misc::info_msg;

pub fn check(mut config: Configuration) -> Result<(), Errors> {
    info_msg("Checking...");

    // Set up the configuration by the project file.
    let proj_file = ProjectFile::read_root_file()?;
    proj_file.set_config(&mut config, false)?;

    // Automatically generate/update lock file and install dependencies.
    // Use Test mode to include test dependencies.
    proj_file.install_dependencies(&mut config, BuildConfigType::Test)?;

    // Set all source files as diagnostics target files.
    match &mut config.subcommand {
        SubCommand::Diagnostics(diag_config) => {
            diag_config.files = config.source_files.clone();
        }
        _ => unreachable!(),
    }

    // Elaborate (parse, resolve, type-check) all entities.
    let program = elaborate_via_config(&config)?;

    // Check for deferred errors (parse errors and type errors accumulated during diagnostics).
    if program.deferred_errors.has_error() {
        return Err(program.deferred_errors);
    }

    info_msg("No errors found.");
    Ok(())
}
