//! `install.sh` lists the releases newest version first, marks the pre-releases, and offers the
//! newest full release as the default, whatever order the GitHub API returns them in. It installs
//! the binary by moving a finished download into place, so a failed or interrupted download leaves
//! `~/.local/bin` as it was.
//!
//! The script runs under POSIX `sh` against a stand-in `curl` that serves a fixed release list and
//! a stand-in `uname` that names a platform with a pre-built binary. Most tests run it in a session
//! of its own, so it has no `/dev/tty`, takes the non-interactive path, and installs its default;
//! the tests that answer its prompts run it on a pseudo-terminal.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::path_env_with_dir_in_front;
    use std::fs::{self, Permissions};
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, ExitStatus, Output, Stdio};
    use tempfile::TempDir;

    /// Writes `content` to `path` as a stand-in command that `install.sh` finds on `PATH`.
    fn write_executable(path: &Path, content: &str) {
        fs::write(path, content).expect("Failed to write a stand-in command");
        fs::set_permissions(path, Permissions::from_mode(0o755))
            .expect("Failed to make a stand-in command executable");
    }

    /// A stand-in `curl` download that writes a placeholder binary to `$out`.
    const DOWNLOAD_SUCCEEDS: &str = "echo placeholder > \"$out\"";

    /// A stand-in `curl` download that writes part of a body to `$out` and fails the way a dropped
    /// connection does (curl's exit 18).
    const DOWNLOAD_DROPS: &str = "echo partial > \"$out\"; exit 18";

    /// A stand-in `curl` download that writes part of a body to `$out` and is then interrupted by
    /// `signal`, sent to the whole process group the way a terminal sends Ctrl-C.
    fn download_interrupted_by(signal: &str) -> String {
        format!("echo partial > \"$out\"; kill -s {} 0", signal)
    }

    /// Runs `install.sh` against a GitHub API that lists `tags` in the given order, and returns
    /// its stdout.
    fn run_install_script(tags: &[&str]) -> String {
        run_install_script_asserting_success(&install_script_fixture(tags, DOWNLOAD_SUCCEEDS))
    }

    /// Runs `install.sh` in `temp_dir` without a terminal, asserts that it succeeded, and returns
    /// its stdout.
    fn run_install_script_asserting_success(temp_dir: &TempDir) -> String {
        let output = run_install_script_without_terminal(temp_dir);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "install.sh failed:\nstdout: {}\nstderr: {}",
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
        stdout
    }

    /// Makes a directory in which `install.sh` runs against a GitHub API that lists `tags` in the
    /// given order and downloads the binary by the stand-in `curl` body `download`, which writes to
    /// `$out`. `HOME` is `home` inside the directory; stand-in commands are in `bin`.
    fn install_script_fixture(tags: &[&str], download: &str) -> TempDir {
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

        // Serves the release list for the API URL, and runs `download` for `-o <dest>`.
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
        temp_dir
    }

    /// A command running `program` with the stand-in commands of `temp_dir` in front of `PATH` and
    /// `HOME` set to the home in `temp_dir`.
    fn install_script_command(program: &str, temp_dir: &TempDir) -> Command {
        let mut command = Command::new(program);
        command
            .env(
                "PATH",
                path_env_with_dir_in_front(temp_dir.path().join("bin")),
            )
            .env("HOME", temp_dir.path().join("home"));
        command
    }

    /// The install directory, `~/.local/bin`, of the home in `temp_dir`.
    fn install_dir(temp_dir: &TempDir) -> PathBuf {
        temp_dir.path().join("home/.local/bin")
    }

    /// The path of `install.sh`.
    fn install_script_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh")
    }

    /// Runs `install.sh` in `temp_dir` in a session of its own, so it has no `/dev/tty` and takes
    /// the non-interactive path. The umask is `022`.
    fn run_install_script_without_terminal(temp_dir: &TempDir) -> Output {
        install_script_command("perl", temp_dir)
            .args([
                "-MPOSIX",
                "-e",
                "umask 022; POSIX::setsid(); exec @ARGV or die",
                "sh",
            ])
            .arg(install_script_path())
            .stdin(Stdio::null())
            .output()
            .expect("Failed to run install.sh")
    }

    /// Runs `install.sh` in `temp_dir` on a pseudo-terminal, typing `input` at it, so it takes
    /// the interactive path and reads its answers from `/dev/tty`. The stdout of the output holds
    /// what the terminal showed, the echoed input included. A script still running after 60 seconds, waiting
    /// for an answer `input` did not give, is killed and the run fails with status 124.
    fn run_install_script_on_terminal(temp_dir: &TempDir, input: &str) -> Output {
        /// A Python program that runs the script at its second argument on a pseudo-terminal, types
        /// its first argument at it, copies what the terminal shows to stdout, and exits with the
        /// script's status.
        const RUN_ON_PTY: &str = r#"
import os, pty, select, signal, sys, time
pid, fd = pty.fork()
if pid == 0:
    os.execvp("sh", ["sh", sys.argv[2]])
os.write(fd, sys.argv[1].encode())
deadline = time.monotonic() + 60
while True:
    ready, _, _ = select.select([fd], [], [], max(0, deadline - time.monotonic()))
    if not ready:
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
        sys.exit(124)
    try:
        data = os.read(fd, 4096)
    except OSError:
        break
    if not data:
        break
    sys.stdout.buffer.write(data)
_, status = os.waitpid(pid, 0)
sys.exit(os.waitstatus_to_exitcode(status) & 0xff)
"#;
        install_script_command("python3", temp_dir)
            .args(["-c", RUN_ON_PTY, input])
            .arg(install_script_path())
            .stdin(Stdio::null())
            .output()
            .expect("Failed to run install.sh on a pseudo-terminal")
    }

    /// Runs `install.sh` in `temp_dir` on a pseudo-terminal, taking the default version and
    /// agreeing to overwrite the installed `fix`, and returns its exit status and what the terminal
    /// showed. Asserts that the script asked whether to overwrite and read the answer from the
    /// terminal.
    fn run_install_script_agreeing_to_overwrite(temp_dir: &TempDir) -> (ExitStatus, String) {
        let output = run_install_script_on_terminal(temp_dir, "\ny\n");
        let terminal = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(terminal.contains("Overwrite? [y/N]"), "{}", terminal);
        assert!(!terminal.contains("non-interactive"), "{}", terminal);
        (output.status, terminal)
    }

    /// The file names in the install directory, `~/.local/bin`, of a run's home.
    fn installed_files(temp_dir: &TempDir) -> Vec<String> {
        let mut names = fs::read_dir(install_dir(temp_dir))
            .expect("install.sh did not reach the download: it made no install directory")
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
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
        let temp_dir = install_script_fixture(&["v1.5.0"], DOWNLOAD_SUCCEEDS);
        run_install_script_asserting_success(&temp_dir);
        assert_eq!(installed_files(&temp_dir), vec!["fix"]);
        assert_eq!(
            fs::read_to_string(install_dir(&temp_dir).join("fix")).unwrap(),
            "placeholder\n"
        );
    }

    /// The installed binary gets the mode the umask gives a new file, made executable: `0755`
    /// under the usual umask `022`.
    #[test]
    fn test_install_script_installs_the_binary_with_the_umask_mode() {
        let temp_dir = install_script_fixture(&["v1.5.0"], DOWNLOAD_SUCCEEDS);
        run_install_script_asserting_success(&temp_dir);
        let mode = fs::metadata(install_dir(&temp_dir).join("fix"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o7777, 0o755, "mode {:o}", mode & 0o7777);
    }

    /// A download that fails part-way leaves nothing in `~/.local/bin`: no truncated `fix`, which
    /// a later run would take for an installed one, and no temporary file.
    #[test]
    fn test_install_script_leaves_nothing_after_a_failed_download() {
        let temp_dir = install_script_fixture(&["v1.5.0"], DOWNLOAD_DROPS);
        let output = run_install_script_without_terminal(&temp_dir);
        assert!(!output.status.success(), "{:?}", output);
        assert_eq!(installed_files(&temp_dir), Vec::<String>::new());
    }

    /// Writes `content` to `~/.local/bin/fix` of `temp_dir` as an installed, executable binary, and
    /// returns its path.
    fn install_existing_fix(temp_dir: &TempDir, content: &[u8]) -> PathBuf {
        fs::create_dir_all(install_dir(temp_dir)).unwrap();
        let path = install_dir(temp_dir).join("fix");
        fs::write(&path, content).unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// When the user agrees to overwrite an installed `fix` and the download then fails part-way,
    /// the installed `fix` is left as it was.
    #[test]
    fn test_install_script_keeps_the_installed_binary_after_a_failed_download() {
        let temp_dir = install_script_fixture(&["v1.5.0"], DOWNLOAD_DROPS);
        let installed = install_existing_fix(&temp_dir, b"installed\n");
        let (status, terminal) = run_install_script_agreeing_to_overwrite(&temp_dir);
        assert!(!status.success(), "{}", terminal);
        assert_eq!(installed_files(&temp_dir), vec!["fix"], "{}", terminal);
        assert_eq!(fs::read_to_string(&installed).unwrap(), "installed\n");
    }

    /// A download interrupted by Ctrl-C (`SIGINT`) or `SIGTERM` leaves nothing in
    /// `~/.local/bin`: no truncated `fix` and no temporary file.
    #[test]
    fn test_install_script_leaves_nothing_after_an_interrupted_download() {
        for signal in ["INT", "TERM"] {
            let temp_dir = install_script_fixture(&["v1.5.0"], &download_interrupted_by(signal));
            let output = run_install_script_without_terminal(&temp_dir);
            assert!(!output.status.success(), "SIG{}: {:?}", signal, output);
            assert_eq!(
                installed_files(&temp_dir),
                Vec::<String>::new(),
                "SIG{}: {:?}",
                signal,
                output
            );
        }
    }

    /// `install.sh` replaces an installed `fix` while it is running, for example as an editor's
    /// language server. Linux refuses to write into a running executable (`ETXTBSY`), so the new
    /// binary has to take the old one's name without writing into it.
    #[test]
    fn test_install_script_replaces_a_running_binary() {
        let temp_dir = install_script_fixture(&["v1.5.0"], DOWNLOAD_SUCCEEDS);
        fs::create_dir_all(install_dir(&temp_dir)).unwrap();
        let installed = install_dir(&temp_dir).join("fix");
        // Copied by `cp` so that this process never opens the file for writing: a child that
        // another test forks meanwhile would inherit that descriptor, and the `exec` below would
        // then fail with `ETXTBSY`.
        let copied = Command::new("cp")
            .arg("/bin/sleep")
            .arg(&installed)
            .status()
            .expect("Failed to run cp");
        assert!(copied.success());
        let mut running = Command::new(&installed)
            .arg("60")
            .spawn()
            .expect("Failed to run the installed binary");
        let (status, terminal) = run_install_script_agreeing_to_overwrite(&temp_dir);
        running.kill().unwrap();
        running.wait().unwrap();
        assert!(status.success(), "{}", terminal);
        assert_eq!(installed_files(&temp_dir), vec!["fix"], "{}", terminal);
        assert_eq!(fs::read_to_string(&installed).unwrap(), "placeholder\n");
    }
}
