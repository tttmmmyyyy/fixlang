/// Tests that a symbolic link in a project, such as one committed to a repository, cannot make the
/// subcommands that build nothing write outside the project.
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

    /// The part of the error message that reports a write a symbolic link would lead outside the
    /// project.
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
        let victim = victim(&root);
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

    /// Asserts that the entry at `path` is a regular file. A symbolic link to a regular file fails
    /// the assertion.
    fn assert_regular_file(path: &Path) {
        assert!(
            fs::symlink_metadata(path).unwrap().file_type().is_file(),
            "\"{}\" is not a regular file",
            path.to_string_lossy()
        );
    }

    /// `fix check` writes no type-check cache through a `.fixlang/cache` that links outside the
    /// project, and reports the refusal once, however many values it checks, without failing.
    #[test]
    fn check_writes_no_cache_through_a_cache_directory_linking_outside() {
        let (root, proj) = setup_test_env();
        fs::write(
            proj.join("main.fix"),
            "module Main;\n\n\
             num_a : I64;\nnum_a = 1;\n\n\
             num_b : I64;\nnum_b = num_a + num_a;\n\n\
             num_c : I64;\nnum_c = num_b + num_a;\n\n\
             main : IO ();\nmain = println(num_c.to_string);\n",
        )
        .unwrap();
        fs::create_dir_all(proj.join(".fixlang")).unwrap();
        symlink(victim(&root), proj.join(".fixlang/cache")).unwrap();
        let output = run_fix(&proj, &["check"]);
        assert_succeeded(
            &output,
            "`fix check` should check the program without its cache.",
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(stderr.matches(REFUSED).count(), 1, "stderr: {}", stderr);
        assert_victim_untouched(&root);
    }

    /// `fix check` replaces a type-check cache file that is a symbolic link by a file of its own,
    /// leaving what the link pointed to.
    #[test]
    fn check_replaces_a_cache_file_that_is_a_link() {
        let (root, proj) = setup_test_env();
        assert_succeeded(
            &run_fix(&proj, &["check"]),
            "`fix check` should write its cache.",
        );
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

    /// `fix check` saves the cache entry of a value whose name is longer than the longest file name
    /// the file system allows.
    #[test]
    fn check_saves_the_cache_of_a_value_with_a_long_name() {
        let (_root, proj) = setup_test_env();
        let name = "a".repeat(300);
        fs::write(
            proj.join("main.fix"),
            format!(
                "module Main;\n\n{n} : I64;\n{n} = 1;\n\nmain : IO ();\nmain = println({n}.to_string);\n",
                n = name
            ),
        )
        .unwrap();
        let output = run_fix(&proj, &["check"]);
        assert_succeeded(&output, "`fix check` should check the program.");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("Failed to write"), "stderr: {}", stderr);
        assert!(fs::read_dir(proj.join(TYPE_CHECK_CACHE_PATH))
            .unwrap()
            .any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("aaaa")));
    }

    /// `fix docs` writes the documentation of a module whose name is close to the longest file name
    /// the file system allows.
    #[test]
    fn docs_writes_the_documentation_of_a_module_with_a_long_name() {
        let (_root, proj) = setup_test_env();
        let module = format!("M{}", "a".repeat(239));
        fs::write(
            proj.join("main.fix"),
            format!(
                "module {};\n\n/// A value.\nvalue : I64;\nvalue = 1;\n",
                module
            ),
        )
        .unwrap();
        assert_succeeded(
            &run_fix(&proj, &["docs"]),
            "`fix docs` should write the documentation.",
        );
        assert_regular_file(&proj.join("docs").join(format!("{}.md", module)));
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

    /// `fix deps update` refuses to clone a git dependency into a `.fixlang/deps` that links
    /// outside the project.
    #[test]
    fn deps_update_refuses_a_deps_directory_linking_outside() {
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

    /// `fix deps add` refuses to append to a project file that links outside the project, before it
    /// looks the project up in any registry.
    #[test]
    fn deps_add_refuses_a_project_file_linking_outside() {
        let (root, proj) = setup_test_env();
        let outside_project_file = root.path().join("fixproj.toml");
        fs::rename(proj.join("fixproj.toml"), &outside_project_file).unwrap();
        symlink(&outside_project_file, proj.join("fixproj.toml")).unwrap();
        let content = fs::read_to_string(&outside_project_file).unwrap();
        assert_failed_with(
            &run_fix(&proj, &["deps", "add", "dep"]),
            REFUSED,
            "`fix deps add` should refuse to write through the link.",
        );
        assert_eq!(
            fs::read_to_string(&outside_project_file).unwrap(),
            content,
            "the project file the link points to changed"
        );
    }

    /// The language server, serving a project whose `.fixlang/fix.log` and `.fixlang/tmp` link
    /// outside it, writes nothing through either link.
    #[test]
    fn language_server_writes_nothing_through_links_in_dot_fixlang() {
        let (root, proj) = setup_test_env();
        fs::create_dir_all(proj.join(".fixlang")).unwrap();
        symlink(
            victim(&root).join("data.txt"),
            proj.join(".fixlang/fix.log"),
        )
        .unwrap();
        symlink(victim(&root), proj.join(".fixlang/tmp")).unwrap();

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
        client
            .shutdown()
            .expect("Failed to shut the language server down");

        assert_victim_untouched(&root);
    }

    /// `fix docs` refuses an output directory that links to another link of the project, which in
    /// turn links outside it.
    #[test]
    fn docs_refuses_a_chain_of_links_leading_outside() {
        let (root, proj) = setup_test_env();
        symlink(victim(&root), proj.join("site")).unwrap();
        symlink(proj.join("site"), proj.join("docs")).unwrap();
        assert_failed_with(
            &run_fix(&proj, &["docs"]),
            REFUSED,
            "`fix docs` should refuse to write through the links.",
        );
        assert_victim_untouched(&root);
    }

    /// `fix docs` refuses an output directory the command line names by an absolute path, when a
    /// symbolic link in the project along that path leads outside it.
    #[test]
    fn docs_refuses_an_absolute_output_directory_through_a_link_in_the_project() {
        let (root, proj) = setup_test_env();
        symlink(victim(&root), proj.join("docs")).unwrap();
        let out_dir = proj.canonicalize().unwrap().join("docs");
        assert_failed_with(
            &run_fix(&proj, &["docs", "--out-dir", &out_dir.to_string_lossy()]),
            REFUSED,
            "`fix docs` should refuse to write through the link.",
        );
        assert_victim_untouched(&root);
    }

    /// The library source `setup_explicit_import_env` places, which `fix edit explicit-import`
    /// rewrites by giving it an import statement for `I64`.
    const LIB_SOURCE: &str = "module Lib;\n\ntwice : I64 -> I64;\ntwice = |x| x + x;\n";

    /// A project `proj` whose `main.fix` imports `Lib`, with its project file listing `main.fix`
    /// and `lib_entry` as its source files. `lib_entry` names the source of `Lib`, which the test
    /// places.
    fn setup_explicit_import_env(lib_entry: &str) -> (TempDir, PathBuf) {
        let (root, proj) = setup_test_env();
        fs::write(
            proj.join("fixproj.toml"),
            format!(
                "[general]\nname = \"proj\"\nversion = \"0.1.0\"\n\n\
                 [build]\nfiles = [\"main.fix\", \"{}\"]\n",
                lib_entry
            ),
        )
        .unwrap();
        fs::write(
            proj.join("main.fix"),
            "module Main;\nimport Lib;\n\nmain : IO ();\nmain = println(twice(3).to_string);\n",
        )
        .unwrap();
        (root, proj)
    }

    /// `fix edit explicit-import` leaves a source file outside the project, which the project file
    /// lists, as it is, and still rewrites the files inside.
    #[test]
    fn edit_leaves_a_source_file_outside_the_project_as_it_is() {
        let (root, proj) = setup_explicit_import_env("../victim/lib.fix");
        fs::write(victim(&root).join("lib.fix"), LIB_SOURCE).unwrap();
        let output = run_fix(&proj, &["edit", "explicit-import"]);
        assert_succeeded(
            &output,
            "`fix edit explicit-import` should rewrite the project.",
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("it lies outside the project"),
            "stderr: {}",
            stderr
        );
        assert_eq!(
            fs::read_to_string(victim(&root).join("lib.fix")).unwrap(),
            LIB_SOURCE
        );
        assert!(fs::read_to_string(proj.join("main.fix"))
            .unwrap()
            .contains("import Lib::twice;"));
    }

    /// `fix edit explicit-import` leaves a source file that is a symbolic link leading outside the
    /// project as it is, together with what the link points to.
    #[test]
    fn edit_leaves_a_source_file_linking_outside_as_it_is() {
        let (root, proj) = setup_explicit_import_env("lib.fix");
        fs::write(victim(&root).join("lib.fix"), LIB_SOURCE).unwrap();
        symlink(victim(&root).join("lib.fix"), proj.join("lib.fix")).unwrap();
        let output = run_fix(&proj, &["edit", "explicit-import"]);
        assert_succeeded(
            &output,
            "`fix edit explicit-import` should rewrite the project.",
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Left \"lib.fix\" as it is: it lies outside the project."),
            "stderr: {}",
            stderr
        );
        assert_eq!(
            fs::read_to_string(victim(&root).join("lib.fix")).unwrap(),
            LIB_SOURCE
        );
    }

    /// `fix edit explicit-import` rewrites a source file that is a symbolic link to another file
    /// of the project, and keeps the link.
    #[test]
    fn edit_rewrites_a_source_file_linking_inside_the_project() {
        let (_root, proj) = setup_explicit_import_env("lib.fix");
        fs::create_dir_all(proj.join("src")).unwrap();
        fs::write(proj.join("src/lib.fix"), LIB_SOURCE).unwrap();
        symlink(proj.join("src/lib.fix"), proj.join("lib.fix")).unwrap();
        assert_succeeded(
            &run_fix(&proj, &["edit", "explicit-import"]),
            "`fix edit explicit-import` should rewrite the project.",
        );
        assert!(fs::symlink_metadata(proj.join("lib.fix"))
            .unwrap()
            .file_type()
            .is_symlink());
        assert_ne!(
            fs::read_to_string(proj.join("src/lib.fix")).unwrap(),
            LIB_SOURCE
        );
    }

    /// `fix edit explicit-import` leaves a source file in a directory beside the project as it is
    /// when that directory's name starts with the project's, as `proj_lib` beside `proj` does.
    #[test]
    fn edit_leaves_a_source_file_in_a_sibling_sharing_the_name_prefix_as_it_is() {
        let (root, proj) = setup_explicit_import_env("../proj_lib/lib.fix");
        let sibling = root.path().join("proj_lib");
        fs::create_dir_all(&sibling).unwrap();
        fs::write(sibling.join("lib.fix"), LIB_SOURCE).unwrap();
        let output = run_fix(&proj, &["edit", "explicit-import"]);
        assert_succeeded(
            &output,
            "`fix edit explicit-import` should rewrite the project.",
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("it lies outside the project"),
            "stderr: {}",
            stderr
        );
        assert_eq!(
            fs::read_to_string(sibling.join("lib.fix")).unwrap(),
            LIB_SOURCE
        );
    }

    /// `fix edit explicit-import` rewrites a source file that the project file lists twice, under
    /// two spellings, once, and keeps the code after its imports.
    #[test]
    fn edit_rewrites_a_source_file_listed_twice_once() {
        let (_root, proj) = setup_explicit_import_env("./main.fix");
        fs::write(
            proj.join("main.fix"),
            "module Main;\n\nmain : IO ();\nmain = println(3.to_string);\n\nimport Std;\n",
        )
        .unwrap();
        assert_succeeded(
            &run_fix(&proj, &["edit", "explicit-import"]),
            "`fix edit explicit-import` should rewrite the project.",
        );
        let main = fs::read_to_string(proj.join("main.fix")).unwrap();
        assert!(main.contains("main : IO ();"), "main.fix: {}", main);
        assert_succeeded(
            &run_fix(&proj, &["check"]),
            "the rewritten project should check.",
        );
    }
}
