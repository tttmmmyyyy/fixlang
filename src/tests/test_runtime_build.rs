//! How the runtime's C sources are compiled into the objects every program links.
//!
//! The sources are written into a directory of each build's own and compiled from inside it, so
//! that builds running side by side neither read nor overwrite each other's copies and nothing of
//! where a build ran reaches the object it produces.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{
        fix_command_at_opt_level, object_digests, run_in, single_source_project_dir,
    };
    use std::path::Path;
    use tempfile::TempDir;

    /// The whole of a project, small enough that building it is the runtime's compilation and
    /// little else.
    const SOURCE: &str = r#"module Main;

main : IO ();
main = println $ 1.0.to_string;
"#;

    /// A directory holding `SOURCE` as the whole of a project, ready to be built in.
    fn project_dir() -> TempDir {
        single_source_project_dir("runtimebuild", SOURCE)
    }

    /// The name and content digest of every runtime object a build left in `dir`, sorted by name.
    fn runtime_object_digests(dir: &Path) -> Vec<(String, String)> {
        object_digests(&dir.join(".fixlang/intermediate"), |name| {
            name.starts_with("fixruntime.")
        })
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
