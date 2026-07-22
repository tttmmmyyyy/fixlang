use std::{fs, path::PathBuf, process::Command};

use crate::{constants::COMPILER_TEST_WORKING_PATH, env_vars, tests::test_util::fix_command};

// Several projects are pinned to their `array-storage-migration` revision. The array/storage
// redesign made `Array` unboxed and dropped its `Boxed` instance, so projects that used `Array`'s
// FFI (`borrow_boxed` / `mutate_boxed` on an array), called `unsafe_is_unique` on their own unbox
// structs, or used the removed unsafe primitives were migrated to the array-specific helpers on that
// branch and are pinned to it here. Projects passing `None` build against their default branch.

#[test]
pub fn test_external_project_math() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-math.git",
        "fixlang-math",
        Some("e072aef74a55bbaae3f77924a06dfbc8ea42d984"),
    );
}

#[test]
pub fn test_external_project_hashmap() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-hashmap.git",
        "fixlang-hashmap",
        None,
    );
}

#[test]
pub fn test_external_project_hashset() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-hashset.git",
        "fixlang-hashset",
        None,
    );
}

#[test]
pub fn test_external_project_random() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-random.git",
        "fixlang-random",
        None,
    );
}

#[test]
pub fn test_external_project_time() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-time.git",
        "fixlang-time",
        Some("42b66a3b9d7a14521747ec11fe1700a5ec124cff"),
    );
}

#[test]
pub fn test_external_project_character() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-character.git",
        "fixlang-character",
        None,
    );
}

#[test]
pub fn test_external_project_subprocess() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-subprocess.git",
        "fixlang-subprocess",
        Some("eb24b00cc5e16dfed57f99f0b12d14a5cd396af1"),
    );
}

#[test]
pub fn test_external_project_regexp() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-regexp.git",
        "fixlang-regexp",
        None,
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
        Some("d659377220cf41d7ab82e7c882aebf49f11a9379"),
    );
}

#[test]
pub fn test_external_project_gmp() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-gmp.git",
        "fixlang-gmp",
        Some("26d1848491dee2e0979b535e5ee158c66d6a2717"),
    );
}

#[test]
pub fn test_external_project_mpfr() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-mpfr.git",
        "fixlang-mpfr",
        Some("5587b6337b4615cd844dbdafa6487562362cc335"),
    );
}

#[test]
pub fn test_external_project_misc_algos() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-misc-algos.git",
        "fixlang-misc-algos",
        Some("d426fc85bcaaa8b0d6f9bed2fe100d256c879a5d"),
    );
}

#[test]
pub fn test_external_project_binary_heap() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-binary-heap.git",
        "fixlang-binary-heap",
        None,
    );
}

#[test]
pub fn test_external_project_cp_library() {
    if env_vars::get_max_opt_level() <= crate::FixOptimizationLevel::None {
        // Skip this test when the optimization level is low since it takes too long time.
        return;
    }
    test_external_project(
        "https://github.com/tttmmmyyyy/cp-library",
        "cp-library",
        Some("f538b64ae07cb225fcf497ca783fb303b315508a"),
    );
}

/// Clone `url`, check out `git_ref` (the default branch when `None`), and run `fix test`.
pub fn test_external_project(url: &str, test_name: &str, git_ref: Option<&str>) {
    println!("Testing external project: {}", url);

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
    let repo_dir = work_dir.join(dir_name);

    // Check out the requested revision.
    if let Some(git_ref) = git_ref {
        let output = Command::new("git")
            .arg("checkout")
            .arg(git_ref)
            .current_dir(&repo_dir)
            .output()
            .expect("Failed to run git checkout.");
        assert!(
            output.status.success(),
            "Failed to check out \"{}\" of \"{}\":\n{}",
            git_ref,
            url,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Run `fix test`. `--allow-preliminary-commands` is supplied because this test is
    // non-interactive and some external projects legitimately ship `preliminary_commands`
    // (e.g. cp-library).
    let mut cmd = fix_command();
    cmd.arg("test")
        .arg("--allow-preliminary-commands")
        .current_dir(&repo_dir);

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
