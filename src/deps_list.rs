use crate::{config_file::ConfigFile, error::Errors, project_file::ProjectFile};

pub fn print_all_projects(fix_config: &ConfigFile) -> Result<(), Errors> {
    for (i, url) in fix_config.registries.iter().enumerate() {
        let reg_file = ProjectFile::download_registry_file(url)?;
        if i > 0 {
            println!("");
        }
        println!("# {}", url);
        println!("");
        reg_file.print_projects();
    }
    Ok(())
}
