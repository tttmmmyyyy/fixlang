use crate::metafiles::config_file::ConfigFile;
use crate::configuration::BuildConfigType;
use crate::dependency::lockfile::{DependecyLockFile, LockFileType};
use crate::error::{panic_if_err, Errors};
use crate::metafiles::project_file::ProjectFile;
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
    panic_if_err(print_all_projects(fix_config, json));
}

fn print_all_projects(fix_config: &ConfigFile, json: bool) -> Result<(), Errors> {
    if json {
        let mut projects = Vec::new();
        for loc in &fix_config.registries {
            let reg_file = ProjectFile::retrieve_registry_file(loc)?;
            projects.extend(reg_file.projects.clone());
        }
        projects.sort_by(|a, b| a.name.cmp(&b.name));

        let json = serde_json::to_string_pretty(&projects);
        match json {
            Ok(json) => {
                println!("{}", json);
            }
            Err(e) => {
                return Err(Errors::from_msg(e.to_string()));
            }
        };
    } else {
        for (i, loc) in fix_config.registries.iter().enumerate() {
            let reg_file = ProjectFile::retrieve_registry_file(loc)?;
            if i > 0 {
                println!("");
            }
            println!("# {}", loc);
            println!("");
            reg_file.print_projects();
        }
    }

    Ok(())
}
