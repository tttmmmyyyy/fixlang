use crate::config_file::ConfigFile;
use crate::configuration::BuildConfigType;
use crate::dependency_lockfile::{DependecyLockFile, LockFileType};
use crate::deps_list;
use crate::error::panic_if_err;
use crate::project_file::ProjectFile;
use clap::ArgMatches;

fn get_build_mode(args: &ArgMatches) -> BuildConfigType {
    if args.contains_id("test") {
        BuildConfigType::Test
    } else {
        BuildConfigType::Build
    }
}

fn read_projects_option(m: &ArgMatches) -> Vec<String> {
    m.try_get_many::<String>("projects")
        .unwrap_or_default()
        .unwrap_or_default()
        .cloned()
        .collect::<Vec<_>>()
}

pub fn deps_install_command(args: &ArgMatches) {
    let mode = get_build_mode(args);
    let proj_file = panic_if_err(ProjectFile::read_root_file());
    panic_if_err(
        proj_file
            .open_lock_file(LockFileType::from_build_config_type(mode))
            .and_then(|lf| lf.install()),
    );
}

pub fn deps_update_command(args: &ArgMatches) {
    let mode = get_build_mode(args);
    panic_if_err(DependecyLockFile::update_and_install(mode));
}

pub fn deps_add_command(args: &ArgMatches, fix_config: &ConfigFile) {
    let mode = get_build_mode(args);
    let projects = read_projects_option(args);
    let proj_file = panic_if_err(ProjectFile::read_root_file());
    panic_if_err(proj_file.add_dependencies(&projects, fix_config, mode));

    // After adding, update the appropriate lock file
    panic_if_err(DependecyLockFile::update_and_install(mode));
}

pub fn deps_list_command(args: &ArgMatches, fix_config: &ConfigFile) {
    let json = args.contains_id("json");
    panic_if_err(deps_list::print_all_projects(fix_config, json));
}
