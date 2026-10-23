/// Tests that what a project directory holds — its lock file, its `.fixlang` — cannot lead `fix` to
/// remove anything outside the project's `.fixlang` directory.
#[cfg(test)]
mod integration_tests {
    use crate::constants::LOCK_FILE_PATH;
    use crate::tests::test_util::{assert_failed_with, assert_succeeded, create_git_repo, run_fix};
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

    /// The directory `fix` installs the git dependency of these tests into.
    const INSTALL_PATH: &str = ".fixlang/deps/dep_0.1.0";

    /// A temporary directory holding a git repository `dep-repo` of a project `dep` at version
    /// 0.1.0, a project `proj` depending on it by its commit, and a directory `victim` beside them
    /// holding the file `data.txt`. The lock file of `proj` is written and its dependency
    /// installed.
    fn setup_test_env() -> (TempDir, PathBuf) {
        let root = TempDir::new().expect("Failed to create temp directory");

        let repo_dir = root.path().join("dep-repo");
        fs::create_dir_all(&repo_dir).unwrap();
        let commit = create_git_repo(
            &repo_dir,
            &[
                (
                    "fixproj.toml",
                    "[general]\nname = \"dep\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"lib.fix\"]\n",
                ),
                ("lib.fix", "module Dep;\n\nvalue : I64;\nvalue = 42;\n"),
            ],
        );

        let proj = root.path().join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::write(
            proj.join("fixproj.toml"),
            format!(
                "[general]\nname = \"proj\"\nversion = \"0.1.0\"\n\n\
                 [build]\nfiles = [\"main.fix\"]\n\n\
                 [[dependencies]]\nname = \"dep\"\n\
                 git = {{ url = \"file://{}\", rev = \"{}\" }}\n",
                repo_dir.to_string_lossy(),
                commit
            ),
        )
        .unwrap();
        fs::write(
            proj.join("main.fix"),
            "module Main;\n\nimport Dep;\n\nmain : IO ();\nmain = println(Dep::value.to_string);\n",
        )
        .unwrap();
        assert_succeeded(
            &run_fix(&proj, &["deps", "update"]),
            "`fix deps update` should write the lock file and install the dependency.",
        );

        let victim = root.path().join("victim");
        fs::create_dir_all(&victim).unwrap();
        fs::write(victim.join("data.txt"), "important").unwrap();

        (root, proj)
    }

    /// Replaces `from` with `to` in the lock file of `proj`, which has to hold `from`.
    fn edit_lock_file(proj: &Path, from: &str, to: &str) {
        let path = proj.join(LOCK_FILE_PATH);
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(from), "lock file: {}", content);
        fs::write(&path, content.replace(from, to)).unwrap();
    }

    /// Asserts that `dir` still holds the file `data.txt` with the content `setup` gives it.
    fn assert_data_kept(dir: &Path) {
        assert_eq!(
            fs::read_to_string(dir.join("data.txt")).ok(),
            Some("important".to_string()),
            "\"{}\" lost its file",
            dir.to_string_lossy()
        );
    }

    /// Asserts that `victim/data.txt` under `root` is still there.
    fn assert_victim_kept(root: &TempDir) {
        assert_data_kept(&root.path().join("victim"));
    }

    /// Asserts that the lock file of `proj` places the dependency at `INSTALL_PATH`.
    fn assert_lock_file_places_dependency_at_install_path(proj: &Path) {
        let lock_file = fs::read_to_string(proj.join(LOCK_FILE_PATH)).unwrap();
        assert!(
            lock_file.contains(&format!("path = \"{}\"", INSTALL_PATH)),
            "lock file: {}",
            lock_file
        );
    }

    /// A lock file placing a git dependency at a path of its own is rejected by `fix deps install`,
    /// before anything there is removed.
    #[test]
    fn test_install_rejects_git_dependency_path_off_the_install_path() {
        let (root, proj) = setup_test_env();
        edit_lock_file(
            &proj,
            &format!("path = \"{}\"", INSTALL_PATH),
            "path = \"../victim\"",
        );
        assert_failed_with(
            &run_fix(&proj, &["deps", "install"]),
            "the git dependency \"dep\" is at \"../victim\", where it belongs at \".fixlang/deps/dep_0.1.0\"",
            "`fix deps install` should reject the path the lock file gives.",
        );
        assert_victim_kept(&root);
    }

    /// A lock file whose dependency name climbs out of the install directory is rejected by `fix
    /// deps install`, even with the path the name and the version make.
    #[test]
    fn test_install_rejects_dependency_name_with_path_separators() {
        let (root, proj) = setup_test_env();
        let name = "../../../victim";
        edit_lock_file(&proj, "name = \"dep\"", &format!("name = \"{}\"", name));
        edit_lock_file(
            &proj,
            &format!("path = \"{}\"", INSTALL_PATH),
            &format!("path = \".fixlang/deps/{}_0.1.0\"", name),
        );
        fs::rename(root.path().join("victim"), root.path().join("victim_0.1.0")).unwrap();
        // The path the lock file gives reaches the directory renamed above, so a `fix` that
        // removes what the lock file points at harms nothing outside the temporary directory.
        assert_eq!(
            proj.join(".fixlang/deps")
                .join(format!("{}_0.1.0", name))
                .canonicalize()
                .unwrap(),
            root.path().join("victim_0.1.0").canonicalize().unwrap()
        );
        assert_failed_with(
            &run_fix(&proj, &["deps", "install"]),
            "\"../../../victim\" is not a project name",
            "`fix deps install` should reject the name the lock file gives.",
        );
        assert_data_kept(&root.path().join("victim_0.1.0"));
    }

    /// `fix build` writes a broken lock file afresh, as it does one that is out of date, and builds
    /// with the dependency installed where the new lock file places it.
    #[test]
    fn test_build_rewrites_lock_file_with_git_dependency_path_off_the_install_path() {
        let (root, proj) = setup_test_env();
        edit_lock_file(
            &proj,
            &format!("path = \"{}\"", INSTALL_PATH),
            "path = \"../victim\"",
        );
        assert_succeeded(
            &run_fix(&proj, &["build"]),
            "`fix build` should write the lock file afresh and build.",
        );
        assert_victim_kept(&root);
        assert_lock_file_places_dependency_at_install_path(&proj);
    }

    /// Each command that writes or removes files under `.fixlang` fails on a `.fixlang` that is a
    /// symbolic link, before it writes or removes anything where the link points.
    #[cfg(unix)]
    #[test]
    fn test_commands_reject_symlinked_dot_fixlang() {
        let (root, proj) = setup_test_env();
        let target = root.path().join("victim");
        fs::remove_dir_all(proj.join(".fixlang")).unwrap();
        symlink(&target, proj.join(".fixlang")).unwrap();

        for args in [
            &["build"][..],
            &["run"][..],
            &["test"][..],
            &["check"][..],
            &["deps", "install"][..],
            &["deps", "update"][..],
        ] {
            assert_failed_with(
                &run_fix(&proj, args),
                "\".fixlang\" is a symbolic link",
                &format!("`fix {}` should reject the link.", args.join(" ")),
            );
            let entries: Vec<_> = fs::read_dir(&target)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            assert_eq!(
                entries,
                ["data.txt"],
                "`fix {}` changed what the link points to",
                args.join(" ")
            );
            assert_victim_kept(&root);
        }
    }

    /// `fix deps install` refuses to remove anything under a directory inside `.fixlang` that is a
    /// symbolic link, so what the link points to is kept.
    #[cfg(unix)]
    #[test]
    fn test_install_refuses_removal_under_symlink_inside_dot_fixlang() {
        let (root, proj) = setup_test_env();
        let target = root.path().join("victim");
        fs::create_dir_all(target.join("dep_0.1.0")).unwrap();
        fs::write(target.join("dep_0.1.0/data.txt"), "important").unwrap();
        fs::remove_dir_all(proj.join(".fixlang/deps")).unwrap();
        symlink(&target, proj.join(".fixlang/deps")).unwrap();

        assert_failed_with(
            &run_fix(&proj, &["deps", "install"]),
            "Refused to remove \".fixlang/deps/dep_0.1.0\"",
            "`fix deps install` should refuse to remove what lies under the link.",
        );
        assert_victim_kept(&root);
        assert_data_kept(&target.join("dep_0.1.0"));
    }

    /// `fix clean` removes a `.fixlang` that is a symbolic link as a link, leaving what it points
    /// to.
    #[cfg(unix)]
    #[test]
    fn test_clean_removes_symlinked_dot_fixlang_as_link() {
        let (root, proj) = setup_test_env();
        fs::remove_dir_all(proj.join(".fixlang")).unwrap();
        symlink(root.path().join("victim"), proj.join(".fixlang")).unwrap();

        assert_succeeded(&run_fix(&proj, &["clean"]), "`fix clean` should succeed.");
        assert!(
            fs::symlink_metadata(proj.join(".fixlang")).is_err(),
            "`fix clean` should remove the link"
        );
        assert_victim_kept(&root);
    }

    /// `fix build` writes afresh a lock file whose version climbs out of the install directory,
    /// even when the path the name and the version make is the one the lock file gives.
    #[test]
    fn test_build_rewrites_lock_file_with_version_climbing_out_of_the_install_directory() {
        let (root, proj) = setup_test_env();
        let version = "0.1.0/../../../../victim";
        let path = format!(".fixlang/deps/dep_{}", version);
        // The path the lock file gives reaches `victim`, so a `fix` that removes what the lock
        // file points at harms nothing outside the temporary directory.
        assert_eq!(
            proj.join(&path).canonicalize().unwrap(),
            root.path().join("victim").canonicalize().unwrap()
        );
        edit_lock_file(
            &proj,
            "version = \"0.1.0\"",
            &format!("version = \"{}\"", version),
        );
        edit_lock_file(
            &proj,
            &format!("path = \"{}\"", INSTALL_PATH),
            &format!("path = \"{}\"", path),
        );
        assert_succeeded(
            &run_fix(&proj, &["build"]),
            "`fix build` should write the lock file afresh and build.",
        );
        assert_victim_kept(&root);
        assert_lock_file_places_dependency_at_install_path(&proj);
    }

    /// A symbolic link at the install directory of a git dependency is removed as a link when the
    /// dependency is reinstalled, leaving what it points to.
    #[cfg(unix)]
    #[test]
    fn test_install_replaces_symlink_at_install_path_leaving_its_target() {
        let (root, proj) = setup_test_env();
        fs::remove_dir_all(proj.join(INSTALL_PATH)).unwrap();
        symlink(root.path().join("victim"), proj.join(INSTALL_PATH)).unwrap();

        assert_succeeded(
            &run_fix(&proj, &["deps", "install"]),
            "`fix deps install` should replace the link with the dependency.",
        );
        assert_victim_kept(&root);
        let installed = fs::symlink_metadata(proj.join(INSTALL_PATH)).unwrap();
        assert!(
            installed.is_dir(),
            "the link should give way to a directory"
        );
        assert!(proj.join(INSTALL_PATH).join("fixproj.toml").is_file());
    }

    /// `fix run` removes the executable it builds under `.fixlang/run`.
    #[test]
    fn test_run_removes_its_temporary_executable() {
        let (_root, proj) = setup_test_env();
        let output = run_fix(&proj, &["run"]);
        assert_succeeded(&output, "`fix run` should build and run the program.");
        assert_eq!(String::from_utf8_lossy(&output.stdout), "42\n");
        let left: Vec<_> = fs::read_dir(proj.join(".fixlang/run"))
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert!(left.is_empty(), "left in .fixlang/run: {:?}", left);
    }

    /// A lock file whose git dependency path ends in `/` or `/.`, where a symbolic link stands at the
    /// install directory, leaves what the link points to as it is.
    #[cfg(unix)]
    #[test]
    fn test_install_keeps_symlink_target_when_lock_file_path_ends_in_separator() {
        for suffix in ["/", "/."] {
            let (root, proj) = setup_test_env();
            fs::remove_dir_all(proj.join(INSTALL_PATH)).unwrap();
            symlink(root.path().join("victim"), proj.join(INSTALL_PATH)).unwrap();
            edit_lock_file(
                &proj,
                &format!("path = \"{}\"", INSTALL_PATH),
                &format!("path = \"{}{}\"", INSTALL_PATH, suffix),
            );
            let _ = run_fix(&proj, &["deps", "install"]);
            assert_victim_kept(&root);
        }
    }

    /// A lock file that is a symbolic link to a file outside the project is replaced by the lock file
    /// `fix` writes, leaving the file it pointed to as it is, both when a build writes the lock file
    /// afresh and when `fix deps update` does.
    #[cfg(unix)]
    #[test]
    fn test_writing_lock_file_replaces_symlink_leaving_its_target() {
        for args in [&["build"][..], &["deps", "update"][..]] {
            let (root, proj) = setup_test_env();
            fs::remove_file(proj.join(LOCK_FILE_PATH)).unwrap();
            symlink(
                root.path().join("victim/data.txt"),
                proj.join(LOCK_FILE_PATH),
            )
            .unwrap();
            assert_succeeded(
                &run_fix(&proj, args),
                &format!("`fix {}` should write the lock file.", args.join(" ")),
            );
            assert_victim_kept(&root);
            assert!(
                fs::symlink_metadata(proj.join(LOCK_FILE_PATH))
                    .unwrap()
                    .is_file(),
                "the lock file should be a file of its own"
            );
        }
    }
}
