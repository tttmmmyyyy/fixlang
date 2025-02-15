use crate::{config_file::ConfigFile, error::Errors, project_file::ProjectFile};

pub fn print_all_projects(fix_config: &ConfigFile, locs_only: bool) -> Result<(), Errors> {
    for (i, loc) in fix_config.registries.iter().enumerate() {
        let reg_file = ProjectFile::retrieve_registry_file(loc)?;
        if locs_only {
            for project in &reg_file.projects {
                println!("{}", project.git);
            }
        } else {
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
