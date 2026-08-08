// Expanding a trait alias walks the aliases it names down to the traits themselves, and an alias
// that several of them stand for is arrived at once per path leading to it. The walk therefore has
// to remember the aliases it has expanded: without that memory, aliases sharing what they stand for
// are walked once per path, and the number of paths doubles with every level of sharing.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{fix_build_source_command, wait_within};
    use std::fs::{self, File};
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tempfile::TempDir;

    /// Each level names the level below it twice, so the level below is arrived at along two paths
    /// and the top level along `2^SHARING_LEVELS`. This many levels compiles in under a second
    /// while the walk remembers what it has expanded, and takes minutes while it forgets.
    const SHARING_LEVELS: usize = 24;

    /// Generous next to the second the build takes, with room for a machine several times slower
    /// running the rest of the suite beside it, and far short of what the build costs once every
    /// path is walked.
    const TIMEOUT: Duration = Duration::from_secs(60);

    /// Builds and runs a program whose trait aliases share what they stand for `SHARING_LEVELS`
    /// times over, failing if the build does not finish within `TIMEOUT`.
    ///
    /// The top alias constrains a function the program calls, so the aliases are expanded both
    /// where they are declared and where the constraint is proved.
    #[test]
    fn test_shared_trait_aliases_compile_in_reasonable_time() {
        let mut aliases = String::from("trait Shared0 = ToString;\n");
        for i in 1..=SHARING_LEVELS {
            aliases.push_str(&format!(
                "trait Left{i} = Shared{below};\n\
                 trait Right{i} = Shared{below};\n\
                 trait Shared{i} = Left{i} + Right{i};\n",
                i = i,
                below = i - 1
            ));
        }
        let source = format!(
            "module Main;\n\
             \n\
             {aliases}\n\
             describe : [a : Shared{top}] a -> String;\n\
             describe = |x| x.to_string;\n\
             \n\
             main : IO ();\n\
             main = println(describe(42));\n",
            aliases = aliases,
            top = SHARING_LEVELS
        );

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let program_path = temp_dir.path().join("shared_trait_aliases");

        // The compiler's diagnostics go to a file, which the test reads once the child has exited.
        // A pipe left unread that long fills its buffer and blocks the very build being timed.
        let log_path = temp_dir.path().join("build.log");
        let log = File::create(&log_path).expect("Failed to create the build log");
        let log_for_stderr = log
            .try_clone()
            .expect("Failed to clone the build log handle");

        let mut command = fix_build_source_command(temp_dir.path(), &source, "max");
        command
            .arg("-o")
            .arg(&program_path)
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_for_stderr));
        let mut child = command.spawn().expect("Failed to execute fix build");
        let status = wait_within(
            &mut child,
            TIMEOUT,
            &format!("compiling {} levels of shared trait aliases", SHARING_LEVELS),
        );
        assert!(
            status.success(),
            "compiling {} levels of shared trait aliases failed: {}\n{}",
            SHARING_LEVELS,
            status,
            fs::read_to_string(&log_path).expect("Failed to read the build log")
        );

        let output = Command::new(&program_path)
            .output()
            .expect("Failed to run the compiled program");
        assert!(
            output.status.success(),
            "the compiled program exited with {}",
            output.status
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "42",
            "the constraint on the shared alias reached a wrong implementation"
        );
    }
}
