//! How the runtime's C sources are compiled into the objects every program links.
//!
//! The sources are written into a directory of each build's own and compiled from inside it, so
//! that builds running side by side neither read nor overwrite each other's copies and nothing of
//! where a build ran reaches the object it produces.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{assert_succeeded, fix_command_at_opt_level};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::TempDir;

    /// The whole of a project, small enough that building it is the runtime's compilation and
    /// little else.
    const SOURCE: &str = r#"module Main;

main : IO ();
main = println $ 1.0.to_string;
"#;

    /// A directory holding `SOURCE` as the whole of a project, ready to be built in.
    fn project_dir() -> TempDir {
        let dir = TempDir::new().expect("Failed to create temp directory");
        fs::write(dir.path().join("main.fix"), SOURCE).expect("Failed to write the source");
        fs::write(
            dir.path().join("fixproj.toml"),
            "[general]\nname = \"runtimebuild\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"main.fix\"]\n",
        )
        .expect("Failed to write the project file");
        dir
    }

    /// The name and content digest of every runtime object a build left in `dir`, sorted by name.
    fn runtime_object_digests(dir: &Path) -> Vec<(String, String)> {
        let intermediate = dir.join(".fixlang/intermediate");
        let mut digests: Vec<(String, String)> = fs::read_dir(&intermediate)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", intermediate.display(), e))
            .map(|entry| entry.expect("failed to read a directory entry").path())
            .filter(|path: &PathBuf| {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                name.starts_with("fixruntime.") && name.ends_with(".o")
            })
            .map(|path| {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let content = fs::read(&path)
                    .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
                (name, format!("{:x}", md5::compute(content)))
            })
            .collect();
        digests.sort();
        digests
    }

    /// Runs `command` in `dir`, failing the test unless it succeeds.
    fn run_in(command: &mut Command, dir: &Path, what: &str) {
        let output = command
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute {}: {}", what, e));
        assert_succeeded(&output, &format!("{} should succeed.", what));
    }

    /// Two builds, each in a directory of its own, compile the runtime into the same bytes under
    /// the same names. What an object records as the file it came from is the path the source
    /// carries in the compiler's tree, so neither the directory a build ran in nor the name it gave
    /// its own copy of the source reaches the object.
    #[test]
    fn test_two_builds_compile_the_runtime_into_the_same_objects() {
        let first = project_dir();
        run_in(
            &mut fix_command_at_opt_level("build", "none"),
            first.path(),
            "the first build",
        );

        let second = project_dir();
        run_in(
            &mut fix_command_at_opt_level("build", "none"),
            second.path(),
            "the second build",
        );

        let first = runtime_object_digests(first.path());
        assert!(
            !first.is_empty(),
            "a build leaves the runtime's object files behind"
        );
        assert_eq!(
            first,
            runtime_object_digests(second.path()),
            "two builds compile the runtime into the same objects, named the same"
        );
    }
}
