//! `install.sh` lists the releases newest version first, marks the pre-releases, and offers the
//! newest full release as the default, whatever order the GitHub API returns them in.
//!
//! The script runs under POSIX `sh` against a stand-in `curl` that serves a fixed release list and
//! a stand-in `uname` that names a platform with a pre-built binary. It runs in a session of its
//! own, so it has no `/dev/tty`, takes the non-interactive path, and installs its default.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::path_env_with_dir_in_front;
    use std::fs::{self, Permissions};
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::{Command, Output, Stdio};
    use tempfile::TempDir;

    /// Writes `content` to `path` as a stand-in command that `install.sh` finds on `PATH`.
    fn write_executable(path: &Path, content: &str) {
        fs::write(path, content).expect("Failed to write a stand-in command");
        fs::set_permissions(path, Permissions::from_mode(0o755))
            .expect("Failed to make a stand-in command executable");
    }

    /// Runs `install.sh` against a GitHub API that lists `tags` in the given order, and returns
    /// its stdout.
    fn run_install_script(tags: &[&str]) -> String {
        let (output, _temp_dir) = run_install_script_downloading(tags, true);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "install.sh failed:\nstdout: {}\nstderr: {}",
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
        stdout
    }

    /// Runs `install.sh` against a GitHub API that lists `tags` in the given order, with `HOME`
    /// at `home` inside the returned directory. The download of the binary succeeds when
    /// `download_succeeds`, and otherwise writes part of a body and fails the way a dropped
    /// connection does (curl's exit 18).
    fn run_install_script_downloading(tags: &[&str], download_succeeds: bool) -> (Output, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let bin_dir = temp_dir.path().join("bin");
        let home_dir = temp_dir.path().join("home");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(&home_dir).unwrap();

        let releases_json = tags
            .iter()
            .map(|tag| {
                format!(
                    "  {{\n    \"tag_name\": \"{}\",\n    \"prerelease\": false\n  }}",
                    tag
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");
        let releases_path = temp_dir.path().join("releases.json");
        fs::write(&releases_path, format!("[\n{}\n]\n", releases_json)).unwrap();

        // Serves the release list for the API URL, and a placeholder binary for `-o <dest>`.
        let download = if download_succeeds {
            "echo placeholder > \"$out\""
        } else {
            "echo partial > \"$out\"; exit 18"
        };
        write_executable(
            &bin_dir.join("curl"),
            &format!(
                r#"#!/bin/sh
out=
while [ $# -gt 0 ]; do
  if [ "$1" = -o ]; then out="$2"; shift; fi
  shift
done
if [ -n "$out" ]; then {}; else cat '{}'; fi
"#,
                download,
                releases_path.display()
            ),
        );
        write_executable(
            &bin_dir.join("uname"),
            "#!/bin/sh\ncase \"$1\" in -s) echo Linux ;; -m) echo x86_64 ;; esac\n",
        );

        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh");
        let output = Command::new("perl")
            .args(["-MPOSIX", "-e", "POSIX::setsid(); exec @ARGV or die", "sh"])
            .arg(&script)
            .env("PATH", path_env_with_dir_in_front(bin_dir))
            .env("HOME", &home_dir)
            .stdin(Stdio::null())
            .output()
            .expect("Failed to run install.sh");
        (output, temp_dir)
    }

    /// The file names in the install directory, `~/.local/bin`, of a run's home.
    fn installed_files(temp_dir: &TempDir) -> Vec<String> {
        let install_dir = temp_dir.path().join("home/.local/bin");
        let mut names = fs::read_dir(&install_dir)
            .map(|entries| {
                entries
                    .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    /// Returns the entries of the "Available versions:" list in `stdout`, trimmed.
    fn listed_versions(stdout: &str) -> Vec<String> {
        stdout
            .lines()
            .skip_while(|line| *line != "Available versions:")
            .skip(1)
            .take_while(|line| line.starts_with("  "))
            .map(|line| line.trim().to_string())
            .collect()
    }

    /// The versions are listed newest first in semver order — a full release above its own
    /// pre-releases, `alpha` < `beta` < `rc`, trailing numbers compared as numbers, version
    /// components compared as numbers — and every pre-release carries the mark.
    #[test]
    fn test_install_script_lists_versions_newest_first() {
        let stdout = run_install_script(&[
            "v1.5.0-beta.9",
            "v1.4.0",
            "v1.5.0-rc.1",
            "v1.10.0",
            "v1.5.0-alpha",
            "v1.5.0",
            "v1.5.0-beta.10",
            "v1.9.2",
        ]);
        assert_eq!(
            listed_versions(&stdout),
            vec![
                "v1.10.0",
                "v1.9.2",
                "v1.5.0",
                "v1.5.0-rc.1 (pre-release)",
                "v1.5.0-beta.10 (pre-release)",
                "v1.5.0-beta.9 (pre-release)",
                "v1.5.0-alpha (pre-release)",
                "v1.4.0",
            ],
            "stdout:\n{}",
            stdout
        );
    }

    /// On the repository's own release history, where a suffix-less pre-release (`v1.0.1-rc`,
    /// `v1.2.0-beta`) sits beside numbered ones and alpha numbers reach two digits, the list
    /// and the default come out in semver order.
    #[test]
    fn test_install_script_orders_the_real_release_history() {
        let stdout = run_install_script(&[
            "v1.1.0-alpha.9",
            "v1.0.1-rc",
            "v1.5.0-beta.1",
            "v1.1.0-alpha.10",
            "v0.1.0",
            "v1.2.0-beta",
            "v1.0.1-rc.1",
            "v1.5.0-rc.1",
            "v1.3.0-beta.8",
            "v1.4.0",
            "v1.5.0",
            "v1.3.0",
            "v1.2.0-beta.3",
            "v1.5.0-beta.3",
            "v1.5.0-beta.2",
            "v1.3.0-beta",
        ]);
        assert_eq!(
            listed_versions(&stdout),
            vec![
                "v1.5.0",
                "v1.5.0-rc.1 (pre-release)",
                "v1.5.0-beta.3 (pre-release)",
                "v1.5.0-beta.2 (pre-release)",
                "v1.5.0-beta.1 (pre-release)",
                "v1.4.0",
                "v1.3.0",
                "v1.3.0-beta.8 (pre-release)",
                "v1.3.0-beta (pre-release)",
                "v1.2.0-beta.3 (pre-release)",
                "... (16 versions total)",
            ],
            "stdout:\n{}",
            stdout
        );
        assert!(
            stdout.contains("Version to install [v1.5.0]: v1.5.0"),
            "stdout:\n{}",
            stdout
        );
    }

    /// The default is the newest full release, even when a pre-release of a later version exists
    /// and the API lists it first.
    #[test]
    fn test_install_script_defaults_to_the_newest_full_release() {
        let stdout = run_install_script(&["v1.6.0-rc.1", "v1.4.0", "v1.5.0", "v1.5.0-rc.1"]);
        assert!(
            stdout.contains("Version to install [v1.5.0]: v1.5.0"),
            "stdout:\n{}",
            stdout
        );
    }

    /// A successful install leaves the binary at `~/.local/bin/fix` and nothing else in the
    /// directory, the temporary file it downloaded into included.
    #[test]
    fn test_install_script_moves_the_download_into_place() {
        let (output, temp_dir) = run_install_script_downloading(&["v1.5.0"], true);
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(installed_files(&temp_dir), vec!["fix"]);
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("home/.local/bin/fix")).unwrap(),
            "placeholder\n"
        );
    }

    /// A download that fails part-way leaves nothing in `~/.local/bin`: no truncated `fix`, which
    /// a later run would take for an installed one, and no temporary file.
    #[test]
    fn test_install_script_leaves_nothing_after_a_failed_download() {
        let (output, temp_dir) = run_install_script_downloading(&["v1.5.0"], false);
        assert!(!output.status.success(), "{:?}", output);
        assert_eq!(installed_files(&temp_dir), Vec::<String>::new());
    }
}
