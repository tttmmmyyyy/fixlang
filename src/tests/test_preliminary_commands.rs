// Integration tests for the preliminary_commands approval flow.
// See logs/security-preliminary-commands.20260419/{spec,design}.md for the specification.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::install_fix;
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};
    use tempfile::TempDir;

    // ---------- Helpers ----------

    // Environment for a single test case: a project directory and an isolated HOME
    // (so that `~/.fixtrust.toml` resolves into the tempdir).
    struct Env {
        _home_dir: TempDir,
        _proj_dir: TempDir,
        home: PathBuf,
        proj: PathBuf,
    }

    impl Env {
        fn new() -> Self {
            let home_dir = TempDir::new().expect("home tempdir");
            let proj_dir = TempDir::new().expect("proj tempdir");
            let home = home_dir.path().to_path_buf();
            let proj = proj_dir.path().to_path_buf();
            Env {
                _home_dir: home_dir,
                _proj_dir: proj_dir,
                home,
                proj,
            }
        }

        fn trust_file(&self) -> PathBuf {
            self.home.join(".fixtrust.toml")
        }

        fn trust_file_contents(&self) -> Option<String> {
            fs::read_to_string(self.trust_file()).ok()
        }
    }

    // Write `fixproj.toml` and a minimal `main.fix` into the project directory.
    fn write_project(proj: &Path, fixproj_toml: &str) {
        fs::write(proj.join("fixproj.toml"), fixproj_toml).expect("write fixproj.toml");
        fs::write(
            proj.join("main.fix"),
            "module Main;\n\nmain : IO ();\nmain = println(\"ok\");\n",
        )
        .expect("write main.fix");
    }

    // Run `fix <args>` in `env.proj` with `HOME=env.home`.
    // `stdin_input`: optional stdin bytes. If provided, piped in (non-tty unless forced).
    // `interactive`: if true, set `FIX_TEST_FORCE_INTERACTIVE=1` so the prompt branch is taken.
    fn run_fix(env: &Env, args: &[&str], stdin_input: Option<&str>, interactive: bool) -> Output {
        install_fix();
        // `install_fix()` copies the fresh binary into `~/.cargo/bin/fix`. The developer's
        // PATH may resolve `fix` to a stale binary elsewhere (e.g. `~/.local/bin/fix`),
        // so invoke the cargo-installed binary by absolute path instead of by name.
        let fix_path = dirs::home_dir()
            .expect("home for cargo bin")
            .join(".cargo/bin/fix");

        let mut cmd = Command::new(&fix_path);
        cmd.args(args)
            .current_dir(&env.proj)
            .env("HOME", &env.home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if interactive {
            cmd.env("FIX_TEST_FORCE_INTERACTIVE", "1");
        }
        if stdin_input.is_some() {
            cmd.stdin(Stdio::piped());
        } else {
            cmd.stdin(Stdio::null());
        }
        let mut child = cmd.spawn().expect("spawn fix");
        if let Some(input) = stdin_input {
            let mut stdin = child.stdin.take().expect("child stdin");
            stdin.write_all(input.as_bytes()).expect("write stdin");
        }
        child.wait_with_output().expect("wait_with_output")
    }

    // Convenience: produce a fixproj.toml for a simple root project with the given
    // build-mode preliminary_commands lines (each line formatted as a TOML array literal).
    fn fixproj_with_preliminary(name: &str, commands: &[&str]) -> String {
        let joined = commands
            .iter()
            .map(|c| format!("  {},", c))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "[general]\nname = \"{name}\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"main.fix\"]\npreliminary_commands = [\n{joined}\n]\n",
            name = name,
            joined = joined
        )
    }

    fn fixproj_with_build_and_test(
        name: &str,
        build_cmds: &[&str],
        test_cmds: &[&str],
    ) -> String {
        let joined_build = build_cmds
            .iter()
            .map(|c| format!("  {},", c))
            .collect::<Vec<_>>()
            .join("\n");
        let joined_test = test_cmds
            .iter()
            .map(|c| format!("  {},", c))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "[general]\nname = \"{name}\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"main.fix\"]\npreliminary_commands = [\n{jb}\n]\n\n\
             [build.test]\nfiles = [\"test.fix\"]\npreliminary_commands = [\n{jt}\n]\n",
            name = name,
            jb = joined_build,
            jt = joined_test,
        )
    }

    fn stderr_str(o: &Output) -> String {
        String::from_utf8_lossy(&o.stderr).to_string()
    }

    // ---------- (A) No preliminary_commands ----------
    #[test]
    fn case_a_no_preliminary_commands() {
        let env = Env::new();
        write_project(
            &env.proj,
            "[general]\nname = \"noop\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"main.fix\"]\n",
        );
        let out = run_fix(&env, &["build"], None, false);
        assert!(out.status.success(), "stderr: {}", stderr_str(&out));
        // No trust file should have been created.
        assert!(env.trust_file_contents().is_none());
    }

    // ---------- (B) --allow-preliminary-commands ----------
    #[test]
    fn case_b_allow_flag_bypasses_without_recording() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("b-test", &["[\"echo\", \"hello\"]"]),
        );
        let out = run_fix(&env, &["build", "--allow-preliminary-commands"], None, false);
        assert!(out.status.success(), "stderr: {}", stderr_str(&out));
        // No trust file should have been created.
        assert!(env.trust_file_contents().is_none());
        let err = stderr_str(&out);
        assert!(err.contains("auto-approved"));
    }

    // ---------- (C) Non-interactive stdin, no flag ----------
    #[test]
    fn case_c_non_interactive_fails() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("c-test", &["[\"echo\", \"hello\"]"]),
        );
        let out = run_fix(&env, &["build"], None, false);
        assert!(!out.status.success());
        let err = stderr_str(&out);
        assert!(
            err.contains("require approval") && err.contains("no interactive terminal"),
            "stderr: {}",
            err
        );
    }

    // ---------- (D) Trust store has a matching entry → no prompt ----------
    #[test]
    fn case_d_approved_entry_skips_prompt() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("d-test", &["[\"echo\", \"hello\"]"]),
        );
        // Pre-populate trust file with an approval for the project's absolute path.
        let preapproval = format!(
            r#"[[approval]]
source = "{}"
mode = "build"
approved_at = "2026-04-20T10:00:00Z"
project_name = "d-test"
commands_preview = [["echo", "hello"]]
"#,
            env.proj.canonicalize().unwrap().to_string_lossy()
        );
        fs::write(env.trust_file(), preapproval).unwrap();

        let out = run_fix(&env, &["build"], None, false);
        assert!(out.status.success(), "stderr: {}", stderr_str(&out));
        let err = stderr_str(&out);
        assert!(err.contains("(approved)"));
    }

    // ---------- (K) Interactive 'y' → records and runs ----------
    #[test]
    fn case_k_happy_path_y() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("k-test", &["[\"echo\", \"greet\"]"]),
        );
        let out = run_fix(&env, &["build"], Some("y\n"), true);
        assert!(out.status.success(), "stderr: {}", stderr_str(&out));
        let content = env.trust_file_contents().expect("trust file should exist");
        assert!(content.contains("k-test"));
        assert!(content.contains("mode = \"build\""));
    }

    // ---------- (L) Interactive 'n' → aborts ----------
    #[test]
    fn case_l_n_aborts_build() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("l-test", &["[\"echo\", \"oops\"]"]),
        );
        let out = run_fix(&env, &["build"], Some("n\n"), true);
        assert!(!out.status.success());
        let err = stderr_str(&out);
        assert!(err.contains("not approved"), "stderr: {}", err);
        // trust file must not have been written.
        assert!(env.trust_file_contents().is_none());
    }

    // ---------- (I) Interactive 'o' → runs but no record ----------
    #[test]
    fn case_i_o_runs_without_recording() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("i-test", &["[\"echo\", \"once\"]"]),
        );
        let out = run_fix(&env, &["build"], Some("o\n"), true);
        assert!(out.status.success(), "stderr: {}", stderr_str(&out));
        assert!(env.trust_file_contents().is_none());
    }

    // ---------- (M) Path-based trust: commit content change does not re-prompt ----------
    #[test]
    fn case_m_path_trust_survives_command_change() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("m-test", &["[\"echo\", \"one\"]"]),
        );
        let out1 = run_fix(&env, &["build"], Some("y\n"), true);
        assert!(out1.status.success(), "stderr: {}", stderr_str(&out1));

        // Change preliminary_commands and re-run.
        write_project(
            &env.proj,
            &fixproj_with_preliminary("m-test", &["[\"echo\", \"TWO\"]"]),
        );
        let out2 = run_fix(&env, &["build"], None, false);
        assert!(out2.status.success(), "stderr: {}", stderr_str(&out2));
        let err = stderr_str(&out2);
        assert!(err.contains("(approved)"), "stderr: {}", err);
    }

    // ---------- (N) Prompt surfaces the trust-store path up front so the user knows
    // which file will be edited when they press `y`.
    #[test]
    fn case_n_prompt_shows_trust_store_path() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("n-test", &["[\"echo\", \"hi\"]"]),
        );
        // Press `n` so we only observe the prompt text; the path must appear before
        // the user answers.
        let out = run_fix(&env, &["build"], Some("n\n"), true);
        assert!(!out.status.success());
        let err = stderr_str(&out);
        let expected = env.trust_file().to_string_lossy().to_string();
        assert!(
            err.contains("records this approval in") && err.contains(&expected),
            "stderr: {}",
            err
        );
    }

    // ---------- (O) fix run uses the same approval flow ----------
    #[test]
    fn case_o_fix_run_uses_same_flow() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("o-test", &["[\"echo\", \"run-time\"]"]),
        );
        // Non-interactive fix run with preliminary_commands must fail without the flag.
        let out = run_fix(&env, &["run"], None, false);
        assert!(!out.status.success());
        assert!(stderr_str(&out).contains("require approval"));
    }

    // ---------- fix docs is exempt from the approval flow ----------
    // `fix docs` only reads source code to emit Markdown -- it never builds object
    // files, so dependencies' preliminary_commands (typically `make lib.o` and the
    // like) are irrelevant to it. The approval flow must therefore be skipped, even
    // when stdin is non-interactive and the trust store is empty. Otherwise CI
    // pipelines that call `fix docs` would be forced to set
    // `--allow-preliminary-commands`, and the fixlang-docpage-generator workflow
    // would break the moment any documented dependency declares a preliminary
    // command.
    #[test]
    fn fix_docs_does_not_require_preliminary_approval_for_root() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary(
                "docs-root-prelim",
                &["[\"echo\", \"should-not-run\"]"],
            ),
        );
        let out = run_fix(&env, &["docs", "-o", "out"], None, false);
        assert!(
            out.status.success(),
            "fix docs should succeed without preliminary-command approval; stderr: {}",
            stderr_str(&out)
        );
        let err = stderr_str(&out);
        assert!(
            !err.contains("require approval"),
            "fix docs must not surface the approval gate; stderr: {}",
            err
        );
        // The trust store must be untouched -- `fix docs` should not even consult
        // it, much less write to it.
        assert!(env.trust_file_contents().is_none());
    }

    #[test]
    fn fix_docs_does_not_require_preliminary_approval_for_git_dep() {
        let env = Env::new();
        let dep_root = TempDir::new().unwrap();
        let (dep_repo, _) = make_git_dep_repo(
            dep_root.path(),
            "docsdep",
            "[general]\nname = \"docsdep\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"lib.fix\"]\npreliminary_commands = [[\"echo\", \"dep-prelim\"]]\n",
            &[],
        );
        let dep_url = format!("file://{}", dep_repo.to_string_lossy());

        write_project(
            &env.proj,
            &format!(
                "[general]\nname = \"docs-with-gitdep\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
                 [build]\nfiles = [\"main.fix\"]\n\n\
                 [[dependencies]]\nname = \"docsdep\"\ngit = {{ url = \"{url}\" }}\n",
                url = dep_url
            ),
        );

        // `fix docs` requires the lock file to already exist, mirroring how the
        // fixlang-docpage-generator runs `fix deps install` before `fix docs`.
        let setup = run_fix(&env, &["deps", "update"], None, false);
        assert!(
            setup.status.success(),
            "setup `fix deps update` failed: {}",
            stderr_str(&setup)
        );

        let out = run_fix(&env, &["docs", "-o", "out"], None, false);
        assert!(
            out.status.success(),
            "fix docs should succeed even when a git dep declares preliminary_commands; \
             stderr: {}",
            stderr_str(&out)
        );
        let err = stderr_str(&out);
        assert!(
            !err.contains("require approval"),
            "fix docs must not surface the approval gate for dep preliminary_commands; \
             stderr: {}",
            err
        );
        assert!(env.trust_file_contents().is_none());
    }

    // ---------- (P) Corrupt trust file → warn and re-prompt ----------
    #[test]
    fn case_p_corrupt_trust_file_warns_and_reprompts() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_preliminary("p-test", &["[\"echo\", \"hi\"]"]),
        );
        fs::write(env.trust_file(), "this is not valid toml ~~~\n").unwrap();
        // Non-interactive run: we expect the parse warning, and a non-interactive failure
        // (because the corrupt file is effectively empty).
        let out = run_fix(&env, &["build"], None, false);
        assert!(!out.status.success());
        let err = stderr_str(&out);
        assert!(err.contains("Failed to parse trust store"), "stderr: {}", err);
        assert!(err.contains("require approval"), "stderr: {}", err);
    }

    // ---------- (S) Input validation: various non-'y'/'o' inputs default to No ----------
    #[test]
    fn case_s_input_validation_defaults_to_no() {
        for input in ["\n", "Y\n", "x\n", ""] {
            let env = Env::new();
            write_project(
                &env.proj,
                &fixproj_with_preliminary("s-test", &["[\"echo\", \"x\"]"]),
            );
            // Empty input simulates EOF; any non-y/o/n should also default to no.
            let out = run_fix(&env, &["build"], Some(input), true);
            // All of these should FAIL (default is n).
            // Exception: 'Y\n' is uppercase, which current implementation lowercases to 'y'
            // and approves. Skip that case — verify it succeeds instead.
            if input == "Y\n" {
                assert!(out.status.success(), "input {:?} should be accepted", input);
            } else {
                assert!(
                    !out.status.success(),
                    "input {:?} should default to no; stderr: {}",
                    input,
                    stderr_str(&out)
                );
            }
        }
    }

    // ---------- (G/H) Build/test mode handling ----------
    // (G) fix build then fix test: build approved first, test still prompts.
    // (H) fix test with both modes unapproved: single prompt approves both.
    #[test]
    fn case_g_build_approved_then_fix_test_only_prompts_test() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_build_and_test(
                "gh-test",
                &["[\"echo\", \"build-step\"]"],
                &["[\"echo\", \"test-step\"]"],
            ),
        );
        // A test.fix with a trivial `test` entry point.
        fs::write(
            env.proj.join("test.fix"),
            "module Test;\n\ntest : IO ();\ntest = println(\"testing\");\n",
        )
        .expect("write test.fix");

        // First: fix build → approve build mode.
        let out1 = run_fix(&env, &["build"], Some("y\n"), true);
        assert!(out1.status.success(), "stderr: {}", stderr_str(&out1));
        let content = env.trust_file_contents().expect("trust file");
        assert!(content.contains("mode = \"build\""));
        assert!(!content.contains("mode = \"test\""));

        // Second: fix test → build is approved, test requires approval.
        let out2 = run_fix(&env, &["test"], Some("y\n"), true);
        assert!(out2.status.success(), "stderr: {}", stderr_str(&out2));
        let err2 = stderr_str(&out2);
        assert!(
            err2.contains("(NEW -- test mode)"),
            "expected the prompt to mention test mode only; stderr: {}",
            err2
        );
        let content2 = env.trust_file_contents().expect("trust file");
        assert!(content2.contains("mode = \"test\""));
    }

    #[test]
    fn case_h_fix_test_combines_both_modes() {
        let env = Env::new();
        write_project(
            &env.proj,
            &fixproj_with_build_and_test(
                "h-test",
                &["[\"echo\", \"build-step\"]"],
                &["[\"echo\", \"test-step\"]"],
            ),
        );
        fs::write(
            env.proj.join("test.fix"),
            "module Test;\n\ntest : IO ();\ntest = println(\"testing\");\n",
        )
        .expect("write test.fix");

        let out = run_fix(&env, &["test"], Some("y\n"), true);
        assert!(out.status.success(), "stderr: {}", stderr_str(&out));
        let content = env.trust_file_contents().expect("trust file");
        assert!(content.contains("mode = \"build\""));
        assert!(content.contains("mode = \"test\""));
    }

    // ---------- (E, T) Git-dependency cases using a local-file:// repo ----------
    // Build a local git repo that contains a minimal Fix project.
    fn make_git_dep_repo(
        root: &Path,
        name: &str,
        fixproj_toml: &str,
        extra_files: &[(&str, &str)],
    ) -> (PathBuf, String) {
        use git2::{IndexAddOption, Repository, Signature};
        let repo_dir = root.join(format!("{}-repo", name));
        fs::create_dir_all(&repo_dir).unwrap();
        let repo = Repository::init(&repo_dir).expect("git init");
        fs::write(repo_dir.join("fixproj.toml"), fixproj_toml).unwrap();
        fs::write(
            repo_dir.join("lib.fix"),
            format!(
                "module {};\n\ngreet : IO ();\ngreet = println(\"hi from {}\");\n",
                capitalize(name),
                name
            ),
        )
        .unwrap();
        for (rel, content) in extra_files {
            let path = repo_dir.join(rel);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }
        let mut index = repo.index().unwrap();
        index
            .add_all(&["*"], IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let commit_id = repo
            .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
        (repo_dir, commit_id.to_string())
    }

    fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            Some(ch) => ch
                .to_uppercase()
                .chain(c)
                .collect::<String>()
                .replace('-', "_"),
            None => String::new(),
        }
    }

    // (E) CHANGED status for Git deps when commit_hash mismatches.
    // We simulate by pre-populating the trust store with a different commit hash
    // for the same (source, mode) tuple.
    #[test]
    fn case_e_git_dep_changed_fails_non_interactive() {
        let env = Env::new();
        let _dep_root = TempDir::new().unwrap();
        let (dep_repo, _commit) = make_git_dep_repo(
            _dep_root.path(),
            "gitdep",
            "[general]\nname = \"gitdep\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"lib.fix\"]\npreliminary_commands = [[\"echo\", \"dep-step\"]]\n",
            &[],
        );
        let dep_url = format!("file://{}", dep_repo.to_string_lossy());

        write_project(
            &env.proj,
            &format!(
                "[general]\nname = \"root-with-gitdep\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
                 [build]\nfiles = [\"main.fix\"]\n\n\
                 [[dependencies]]\nname = \"gitdep\"\ngit = {{ url = \"{url}\" }}\n",
                url = dep_url
            ),
        );

        // Pre-populate trust store with a mismatching commit for the git dep.
        let preapproval = format!(
            r#"[[approval]]
source = "git+{url}"
mode = "build"
commit_hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
approved_at = "2026-04-20T10:00:00Z"
project_name = "gitdep"
commands_preview = [["echo", "dep-step"]]

[[approval]]
source = "{root}"
mode = "build"
approved_at = "2026-04-20T10:00:00Z"
project_name = "root-with-gitdep"
commands_preview = []
"#,
            url = dep_url,
            root = env.proj.canonicalize().unwrap().to_string_lossy()
        );
        fs::write(env.trust_file(), preapproval).unwrap();

        let out = run_fix(&env, &["build"], None, false);
        // Expect the run to fail non-interactively because the git dep's commit changed.
        assert!(!out.status.success(), "stderr: {}", stderr_str(&out));
        let err = stderr_str(&out);
        assert!(err.contains("CHANGED from commit"), "stderr: {}", err);
    }

    // (R) Git dep prompt wording differs from local prompt wording.
    #[test]
    fn case_r_git_prompt_wording() {
        let env = Env::new();
        let dep_root = TempDir::new().unwrap();
        let (dep_repo, _) = make_git_dep_repo(
            dep_root.path(),
            "gitwording",
            "[general]\nname = \"gitwording\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"lib.fix\"]\npreliminary_commands = [[\"echo\", \"dep\"]]\n",
            &[],
        );
        let dep_url = format!("file://{}", dep_repo.to_string_lossy());

        write_project(
            &env.proj,
            &format!(
                "[general]\nname = \"r-root\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
                 [build]\nfiles = [\"main.fix\"]\n\n\
                 [[dependencies]]\nname = \"gitwording\"\ngit = {{ url = \"{url}\" }}\n",
                url = dep_url
            ),
        );

        // Stdin ends without selecting -> defaults to N -> abort. We still get the
        // prompt printed once, which is what we want to inspect.
        let out = run_fix(&env, &["build"], Some("n\n"), true);
        assert!(!out.status.success());
        let err = stderr_str(&out);
        assert!(
            err.contains("trust this commit from now on"),
            "expected Git-flavored prompt; stderr: {}",
            err
        );
    }
}
