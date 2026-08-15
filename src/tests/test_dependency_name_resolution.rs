//! Name resolution across a project boundary: what a dependency's modules contribute to the
//! candidate table, and how a name written in the root project reaches them.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::fix_command;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Output;
    use tempfile::TempDir;

    /// Writes `content` to `dir/name`, creating `dir` if it does not exist.
    fn write_file(dir: &Path, name: &str, content: &str) {
        fs::create_dir_all(dir).expect("Failed to create a project directory");
        fs::write(dir.join(name), content).expect("Failed to write a project file");
    }

    /// Lays out a root project that depends on one library project, both under a fresh temporary
    /// directory, and returns the guard along with the root project's directory.
    ///
    /// The dependency is reached by a relative path, so the pair moves with the temporary directory
    /// and several tests can run at once.
    fn setup_two_projects(dep_source: &str, main_source: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dep_dir = temp_dir.path().join("dep");
        let main_dir = temp_dir.path().join("main");

        write_file(
            &dep_dir,
            "fixproj.toml",
            "[general]\nname = \"dep\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"lib.fix\"]\n",
        );
        write_file(&dep_dir, "lib.fix", dep_source);

        write_file(
            &main_dir,
            "fixproj.toml",
            "[general]\nname = \"main-proj\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"main.fix\"]\n\n[[dependencies]]\nname = \"dep\"\npath = \"../dep\"\n",
        );
        write_file(&main_dir, "main.fix", main_source);

        (temp_dir, main_dir)
    }

    /// Runs `fix run` in `project_dir` and returns its captured output.
    fn run_project(project_dir: &Path) -> Output {
        fix_command()
            .arg("run")
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix run")
    }

    /// A module name may carry `.`, and a name written in the root project reaches a dependency's
    /// module `Dep.Sub` by its last component alone, `Sub::Widget`, once `import Dep.Sub;` makes it
    /// accessible.
    #[test]
    fn dotted_dependency_module_is_reached_by_its_last_component() {
        let (_temp_dir, project_dir) = setup_two_projects(
            "module Dep.Sub;\n\
             \n\
             type Widget = unbox struct { v : I64 };\n\
             \n\
             make : I64 -> Widget;\n\
             make = |i| Widget { v : i };\n",
            "module Main;\n\
             \n\
             import Dep.Sub;\n\
             \n\
             w : Sub::Widget;\n\
             w = Sub::make(3);\n\
             \n\
             main : IO ();\n\
             main = println $ w.@v.to_string;\n",
        );

        let output = run_project(&project_dir);
        assert!(
            output.status.success(),
            "`Sub::Widget` and `Sub::make` should reach the dependency's module `Dep.Sub`.\n\
             stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "3");
    }

    /// `fix edit explicit-import` derives the import statements it writes from the names resolution
    /// recorded as required. Every kind of entity a dependency exports — a type, a type alias, a
    /// trait, a trait alias, an associated type, a trait member and a value in a nested namespace —
    /// has to be recorded, or the rewritten project stops building.
    #[test]
    fn explicit_import_rewrite_of_a_dependency_still_builds() {
        let (_temp_dir, project_dir) = setup_two_projects(
            "module Dep;\n\
             \n\
             type Box2 = unbox struct { v : I64 };\n\
             \n\
             type Alias = Box2;\n\
             \n\
             trait a : Container {\n\
             \x20   type Held a;\n\
             \x20   get_held : a -> Held a;\n\
             }\n\
             \n\
             impl Box2 : Container {\n\
             \x20   type Held Box2 = I64;\n\
             \x20   get_held = |b| b.@v;\n\
             }\n\
             \n\
             trait Showable = Container + ToString;\n\
             \n\
             impl Box2 : ToString {\n\
             \x20   to_string = |b| \"Box2(\" + b.@v.to_string + \")\";\n\
             }\n\
             \n\
             namespace Inner {\n\
             \x20   make : I64 -> Box2;\n\
             \x20   make = |i| Box2 { v : i };\n\
             }\n",
            "module Main;\n\
             \n\
             import Dep;\n\
             \n\
             use_alias : Alias -> I64;\n\
             use_alias = |a| a.@v;\n\
             \n\
             use_assoc : [c : Container, Held c = I64] c -> I64;\n\
             use_assoc = |c| c.get_held;\n\
             \n\
             use_trait_alias : [c : Showable] c -> String;\n\
             use_trait_alias = |c| c.to_string;\n\
             \n\
             main : IO ();\n\
             main = (\n\
             \x20   let b = Inner::make(7);\n\
             \x20   println $ b.use_alias.to_string + \" \" + b.use_assoc.to_string + \" \" + b.use_trait_alias\n\
             );\n",
        );

        let edit = fix_command()
            .args(["edit", "explicit-import"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix edit explicit-import");
        assert!(
            edit.status.success(),
            "`fix edit explicit-import` should succeed.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&edit.stdout),
            String::from_utf8_lossy(&edit.stderr),
        );

        let rewritten =
            fs::read_to_string(project_dir.join("main.fix")).expect("Failed to read main.fix");
        assert!(
            !rewritten.contains("import Dep;"),
            "the wholesale `import Dep;` should have been replaced by explicit items. Got:\n{}",
            rewritten,
        );

        let output = run_project(&project_dir);
        assert!(
            output.status.success(),
            "the rewritten project should still build. main.fix is now:\n{}\nstdout: {}\nstderr: {}",
            rewritten,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "7 7 Box2(7)"
        );
    }
}
