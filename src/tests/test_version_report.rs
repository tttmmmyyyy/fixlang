//! The version the compiler reports. Every way of asking for it answers with one line, and that
//! line heads every help message, so a bug report that quotes any of them names the build it was
//! made against.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::fix_command;

    /// The output of `fix <args>`, stdout followed by stderr. A `fix` invoked with no arguments
    /// writes its help to stderr, and every other form here writes to stdout.
    fn run(args: &[&str]) -> String {
        let output = fix_command()
            .args(args)
            .output()
            .expect("Failed to execute the fix command");
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        text
    }

    fn first_line(text: &str) -> &str {
        text.lines().next().unwrap_or("")
    }

    /// The version reported by `fix --version`, as the whole line it prints.
    fn version_line() -> String {
        run(&["--version"]).trim_end().to_string()
    }

    /// The long option, the short one, the `version` subcommand, and the header of the help each
    /// answer the same line, so a reader holding any one of them holds the version.
    #[test]
    fn test_every_way_of_asking_the_version_answers_the_same_line() {
        let version = version_line();
        let released = format!("fix {} (", env!("CARGO_PKG_VERSION"));
        let revision = version
            .strip_prefix(&released)
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or_else(|| {
                panic!(
                    "`fix --version` answered `{}`, which does not carry the released version followed by a revision",
                    version
                )
            });
        assert!(
            !revision.is_empty() && !revision.contains(char::is_whitespace),
            "`fix --version` answered `{}`, which names no revision to tell two builds of {} apart",
            version,
            env!("CARGO_PKG_VERSION")
        );

        for form in [vec!["-V"], vec!["version"]] {
            assert_eq!(
                run(&form).trim_end(),
                version,
                "`fix {}` reported a different version",
                form.join(" ")
            );
        }
        for help in [vec!["--help"], vec!["help"], vec![]] {
            assert_eq!(
                first_line(&run(&help)),
                version,
                "the header of `fix {}` reported a different version",
                help.join(" ")
            );
        }
    }

    /// A subcommand's help is headed by the same version, under the name that subcommand is
    /// invoked by, at every depth the command tree reaches.
    #[test]
    fn test_a_subcommand_help_is_headed_by_the_version() {
        let version = version_line();
        let version = version
            .strip_prefix("fix ")
            .expect("`fix --version` answered a line that does not begin with the command name");
        for path in [
            vec!["build"],
            vec!["deps"],
            vec!["edit"],
            vec!["deps", "add"],
            vec!["edit", "explicit-import"],
        ] {
            let mut args = path.clone();
            args.push("--help");
            assert_eq!(
                first_line(&run(&args)),
                format!("fix-{} {}", path.join("-"), version),
                "the header of `fix {} --help` reported a different version",
                path.join(" ")
            );
        }
    }

    /// A subcommand reached with none of its own subcommands prints the help that
    /// `fix <name> --help` prints, so the two paths to one help message cannot drift apart.
    #[test]
    fn test_a_subcommand_without_a_subcommand_prints_the_help_of_that_subcommand() {
        for subcommand in ["deps", "edit"] {
            assert_eq!(
                run(&[subcommand]),
                run(&[subcommand, "--help"]),
                "`fix {}` printed a help other than the one `fix {} --help` prints",
                subcommand,
                subcommand
            );
        }
    }

    /// The line under the version says what `fix` is, so the help alone tells a reader who has
    /// never seen Fix what they are holding.
    #[test]
    fn test_the_help_says_what_fix_is() {
        let help = run(&["--help"]);
        let about = help.lines().nth(1).unwrap_or("");
        assert!(
            about.contains("Fix"),
            "the line under the version does not describe the tool: `{}`",
            about
        );
    }
}
