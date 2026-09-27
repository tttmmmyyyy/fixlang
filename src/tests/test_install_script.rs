//! `install.sh` lists the releases newest version first, marks the pre-releases, and offers the
//! newest full release as the default, whatever order the GitHub API returns them in.
//!
//! The script runs under POSIX `sh` against a stand-in `curl` that serves a fixed release list and
//! a stand-in `uname` that names a platform with a pre-built binary. It runs in a session of its
//! own, so it has no `/dev/tty`, takes the non-interactive path, and installs its default.

#[cfg(test)]
mod integration_tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::{Command, Stdio};
    use tempfile::TempDir;

    fn write_executable(path: &Path, content: &str) {
        fs::write(path, content).expect("Failed to write a stand-in command");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .expect("Failed to make a stand-in command executable");
    }

    /// Runs `install.sh` against a GitHub API that lists `tags` in the given order, and returns
    /// its stdout.
    fn run_install_script(tags: &[&str]) -> String {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let bin_dir = temp_dir.path().join("bin");
        let home_dir = temp_dir.path().join("home");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(&home_dir).unwrap();

        let releases_json = tags
            .iter()
            .map(|tag| format!("  {{\n    \"tag_name\": \"{}\",\n    \"prerelease\": false\n  }}", tag))
            .collect::<Vec<_>>()
            .join(",\n");
        let releases_path = temp_dir.path().join("releases.json");
        fs::write(&releases_path, format!("[\n{}\n]\n", releases_json)).unwrap();

        // Serves the release list for the API URL, and a placeholder binary for `-o <dest>`.
        write_executable(
            &bin_dir.join("curl"),
            &format!(
                r#"#!/bin/sh
out=
while [ $# -gt 0 ]; do
  if [ "$1" = -o ]; then out="$2"; shift; fi
  shift
done
if [ -n "$out" ]; then echo placeholder > "$out"; else cat '{}'; fi
"#,
                releases_path.display()
            ),
        );
        write_executable(
            &bin_dir.join("uname"),
            "#!/bin/sh\ncase \"$1\" in -s) echo Linux ;; -m) echo x86_64 ;; esac\n",
        );

        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh");
        let path_env = format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let output = Command::new("perl")
            .args(["-MPOSIX", "-e", "POSIX::setsid(); exec @ARGV or die", "sh"])
            .arg(&script)
            .env("PATH", path_env)
            .env("HOME", &home_dir)
            .stdin(Stdio::null())
            .output()
            .expect("Failed to run install.sh");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "install.sh failed:\nstdout: {}\nstderr: {}",
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
        stdout
    }

    /// The lines of the "Available versions:" list.
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
}
