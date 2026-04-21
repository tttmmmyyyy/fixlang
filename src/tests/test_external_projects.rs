use std::{env, fs, path::PathBuf, process::Command};

use crate::{constants::COMPILER_TEST_WORKING_PATH, env_vars, tests::test_util::install_fix};

#[test]
pub fn test_external_project_math() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-math.git",
        "fixlang-math",
    );
}

#[test]
pub fn test_external_project_hashmap() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-hashmap.git",
        "fixlang-hashmap",
    );
}

#[test]
pub fn test_external_project_hashset() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-hashset.git",
        "fixlang-hashset",
    );
}

#[test]
pub fn test_external_project_random() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-random.git",
        "fixlang-random",
    );
}

#[test]
pub fn test_external_project_time() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-time.git",
        "fixlang-time",
    );
}

#[test]
pub fn test_external_project_character() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-character.git",
        "fixlang-character",
    );
}

#[test]
pub fn test_external_project_subprocess() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-subprocess.git",
        "fixlang-subprocess",
    );
}

#[test]
pub fn test_external_project_regexp() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-regexp.git",
        "fixlang-regexp",
    );
}

#[test]
pub fn test_external_project_asynctask() {
    if env_vars::get_max_opt_level() <= crate::FixOptimizationLevel::None {
        // Skip this test when the optimization level is low since it takes too long time.
        return;
    }
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-asynctask.git",
        "fixlang-asynctask",
    );
}

#[test]
pub fn test_external_project_gmp() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-gmp.git",
        "fixlang-gmp",
    );
}

#[test]
pub fn test_external_project_mpfr() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-mpfr.git",
        "fixlang-mpfr",
    );
}

#[test]
pub fn test_external_project_misc_algos() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-misc-algos.git",
        "fixlang-misc-algos",
    );
}

#[test]
pub fn test_external_project_binary_heap() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-binary-heap.git",
        "fixlang-binary-heap",
    );
}

#[test]
pub fn test_external_project_cp_library() {
    if env_vars::get_max_opt_level() <= crate::FixOptimizationLevel::None {
        // Skip this test when the optimization level is low since it takes too long time.
        return;
    }
    test_external_project("https://github.com/tttmmmyyyy/cp-library", "cp-library");
}

pub fn test_external_project(url: &str, test_name: &str) {
    println!("Testing external project: {}", url);
    install_fix();

    // Recreate working directory for this test.
    let work_dir = PathBuf::from(format!("{}/{}", COMPILER_TEST_WORKING_PATH, test_name));
    let _ = fs::remove_dir_all(&work_dir);
    let _ = fs::create_dir_all(&work_dir);

    // Run `git clone {url}`.
    let _ = Command::new("git")
        .arg("clone")
        .arg(url)
        .current_dir(&work_dir)
        .output()
        .expect("Failed to run git clone.");

    // Get the created directory name.
    let dir_name = url
        .split("/")
        .last()
        .unwrap()
        .to_string()
        .replace(".git", "");

    // Run `fix test`. `install_fix()` writes the freshly built binary to `~/.cargo/bin/fix`;
    // use that absolute path so we don't accidentally pick up a stale `fix` earlier on PATH.
    // `--allow-preliminary-commands` is supplied because this test is non-interactive and
    // some external projects legitimately ship `preliminary_commands` (e.g. cp-library).
    let fix_path = dirs::home_dir()
        .expect("home for cargo bin")
        .join(".cargo/bin/fix");
    let mut cmd = Command::new(&fix_path);
    cmd.arg("test")
        .arg("--allow-preliminary-commands")
        .current_dir(work_dir.join(dir_name));

    // Inherit all environment variables from the parent process
    cmd.envs(env::vars());

    let output = cmd.output().expect("Failed to run fix test.");

    // Check the result.
    if output.status.code() != Some(0) {
        eprintln!("=== fix test stdout ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("=== fix test stderr ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    assert_eq!(
        output.status.code(),
        Some(0),
        "Failed to run fix test of \"{}\"",
        url
    );
}
