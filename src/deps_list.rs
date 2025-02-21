use crate::{config_file::ConfigFile, error::Errors, project_file::ProjectFile};

pub fn print_all_projects(fix_config: &ConfigFile, json: bool) -> Result<(), Errors> {
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
