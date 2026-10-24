/// Tests that a symbolic link a project directory holds, such as one committed to a repository,
/// cannot lead the subcommands that build nothing to write outside the project.
#[cfg(all(test, unix))]
mod integration_tests {
    use crate::constants::TYPE_CHECK_CACHE_PATH;
    use crate::tests::test_lsp::lsp_client::LspClient;
    use crate::tests::test_util::{assert_failed_with, assert_succeeded, create_git_repo, run_fix};
    use std::os::unix::fs::symlink;
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

    /// The report of a write that a symbolic link would lead outside the project.
    const REFUSED: &str = "is a symbolic link in the project leading outside it";

    /// A temporary directory holding a project `proj`, whose `Main` module documents its `main`,
    /// and a directory `victim` beside it holding the file `data.txt`.
    fn setup_test_env() -> (TempDir, PathBuf) {
        let root = TempDir::new().expect("Failed to create temp directory");
        let proj = root.path().join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::write(
            proj.join("fixproj.toml"),
            "[general]\nname = \"proj\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"main.fix\"]\n",
        )
        .unwrap();
        fs::write(
            proj.join("main.fix"),
            "module Main;\n\n/// Prints a greeting.\nmain : IO ();\nmain = println(\"hello\");\n",
        )
        .unwrap();
        let victim = root.path().join("victim");
        fs::create_dir_all(&victim).unwrap();
        fs::write(victim.join("data.txt"), "important").unwrap();
        (root, proj)
    }

    /// The directory `victim` that `setup_test_env` places beside the project.
    fn victim(root: &TempDir) -> PathBuf {
        root.path().join("victim")
    }

    /// Asserts that `victim` holds the file `data.txt` with the content `setup_test_env` gives it,
    /// and nothing else.
    fn assert_victim_untouched(root: &TempDir) {
        let entries: Vec<_> = fs::read_dir(victim(root))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, vec!["data.txt"], "the victim directory changed");
        assert_eq!(
            fs::read_to_string(victim(root).join("data.txt")).unwrap(),
            "important",
            "the victim file changed"
        );
    }

    /// Asserts that `path` is a regular file, which no symbolic link stands in for.
    fn assert_regular_file(path: &Path) {
        assert!(
            fs::symlink_metadata(path).unwrap().file_type().is_file(),
            "\"{}\" is not a regular file",
            path.to_string_lossy()
        );
    }

    /// `fix check` writes no type-check cache through a `.fixlang/cache` that links outside the
    /// project, and reports the refusal without failing.
    #[test]
    fn check_writes_no_cache_through_a_cache_directory_linking_outside() {
        let (root, proj) = setup_test_env();
        fs::create_dir_all(proj.join(".fixlang")).unwrap();
        symlink(victim(&root), proj.join(".fixlang/cache")).unwrap();
        let output = run_fix(&proj, &["check"]);
        assert_succeeded(&output, "`fix check` should check the program without its cache.");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(REFUSED),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_victim_untouched(&root);
    }

    /// `fix check` replaces a type-check cache file that is a symbolic link by a file of its own,
    /// leaving what the link pointed to.
    #[test]
    fn check_replaces_a_cache_file_that_is_a_link() {
        let (root, proj) = setup_test_env();
        assert_succeeded(&run_fix(&proj, &["check"]), "`fix check` should write its cache.");
        let cache_dir = proj.join(TYPE_CHECK_CACHE_PATH);
        let cache_files: Vec<PathBuf> = fs::read_dir(&cache_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert!(!cache_files.is_empty(), "`fix check` wrote no cache file");
        for file in &cache_files {
            fs::remove_file(file).unwrap();
            symlink(victim(&root).join("data.txt"), file).unwrap();
        }
        assert_succeeded(
            &run_fix(&proj, &["check"]),
            "`fix check` should check the program again.",
        );
        assert_victim_untouched(&root);
        for file in &cache_files {
            assert_regular_file(file);
        }
    }

    /// `fix check` refuses to save its sources under a `.fixlang/tmp` that links outside the
    /// project.
    #[test]
    fn check_refuses_a_temporary_directory_linking_outside() {
        let (root, proj) = setup_test_env();
        fs::create_dir_all(proj.join(".fixlang")).unwrap();
        symlink(victim(&root), proj.join(".fixlang/tmp")).unwrap();
        assert_failed_with(
            &run_fix(&proj, &["check"]),
            REFUSED,
            "`fix check` should refuse to write under the link.",
        );
        assert_victim_untouched(&root);
    }

    /// `fix docs` replaces an output file that is a symbolic link by the documentation, leaving
    /// what the link pointed to.
    #[test]
    fn docs_replaces_an_output_file_that_is_a_link() {
        let (root, proj) = setup_test_env();
        fs::create_dir_all(proj.join("docs")).unwrap();
        symlink(victim(&root).join("data.txt"), proj.join("docs/Main.md")).unwrap();
        assert_succeeded(
            &run_fix(&proj, &["docs"]),
            "`fix docs` should write the documentation.",
        );
        assert_victim_untouched(&root);
        assert_regular_file(&proj.join("docs/Main.md"));
        assert!(fs::read_to_string(proj.join("docs/Main.md"))
            .unwrap()
            .contains("Prints a greeting."));
    }

    /// `fix docs` refuses an output directory that links outside the project.
    #[test]
    fn docs_refuses_an_output_directory_linking_outside() {
        let (root, proj) = setup_test_env();
        symlink(victim(&root), proj.join("docs")).unwrap();
        assert_failed_with(
            &run_fix(&proj, &["docs"]),
            REFUSED,
            "`fix docs` should refuse to write through the link.",
        );
        assert_victim_untouched(&root);
    }

    /// `fix docs` writes through an output directory that links to another directory of the
    /// project.
    #[test]
    fn docs_writes_through_a_link_leading_inside_the_project() {
        let (_root, proj) = setup_test_env();
        fs::create_dir_all(proj.join("site")).unwrap();
        symlink(proj.join("site"), proj.join("docs")).unwrap();
        assert_succeeded(
            &run_fix(&proj, &["docs"]),
            "`fix docs` should write through the link.",
        );
        assert_regular_file(&proj.join("site/Main.md"));
    }

    /// `fix docs` writes to an output directory the command line names outside the project, along
    /// a symbolic link that lies outside the project too.
    #[test]
    fn docs_writes_where_the_command_line_names_outside_the_project() {
        let (root, proj) = setup_test_env();
        let out = root.path().join("out");
        fs::create_dir_all(&out).unwrap();
        symlink(&out, root.path().join("out-link")).unwrap();
        assert_succeeded(
            &run_fix(&proj, &["docs", "--out-dir", "../out-link"]),
            "`fix docs` should write where the command line names.",
        );
        assert_regular_file(&out.join("Main.md"));
    }

    /// `fix deps install` refuses to clone a git dependency into a `.fixlang/deps` that links
    /// outside the project.
    #[test]
    fn deps_install_refuses_a_deps_directory_linking_outside() {
        let (root, proj) = setup_test_env();
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
        fs::create_dir_all(proj.join(".fixlang")).unwrap();
        symlink(victim(&root), proj.join(".fixlang/deps")).unwrap();
        assert_failed_with(
            &run_fix(&proj, &["deps", "update"]),
            REFUSED,
            "`fix deps update` should refuse to clone under the link.",
        );
        assert_victim_untouched(&root);
    }

    /// The language server, serving a project whose `.fixlang/fix.log` and `.fixlang/cache` link
    /// outside it, writes nothing through either link.
    #[test]
    fn language_server_writes_nothing_through_links_in_dot_fixlang() {
        let (root, proj) = setup_test_env();
        fs::create_dir_all(proj.join(".fixlang")).unwrap();
        symlink(victim(&root).join("data.txt"), proj.join(".fixlang/fix.log")).unwrap();
        symlink(victim(&root), proj.join(".fixlang/cache")).unwrap();

        let mut client = LspClient::new(&proj).expect("Failed to start the language server");
        client
            .initialize(&proj, LspClient::RESPONSE_TIMEOUT)
            .expect("Failed to initialize the language server");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open the document");
        client
            .wait_for_ended_passes(1, LspClient::PASS_TIMEOUT)
            .expect("the diagnostics pass is expected to end");
        client.shutdown().expect("Failed to shut the language server down");

        assert_victim_untouched(&root);
    }
}
