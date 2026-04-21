// `PreliminaryCommand` data types and the approval flow that runs them.
// The flow is invoked by `Configuration::run_preliminary_commands`.
// See `logs/.../spec.md` for the user-visible specification.

use crate::{
    configuration::Configuration,
    constants::PRELIMINARY_BUILD_LD_FLAGS,
    error::Errors,
    metafiles::{
        project_file::ProjectOrigin,
        trust_store::{make_approval, TrustStore},
    },
    misc::{info_msg, prompt_style, split_string_by_space_not_quated, to_absolute_path, warn_msg},
};
use colored::Colorize;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// ---- Types ----

// Which `preliminary_commands` section a command belongs to (`[build]` or `[build.test]`).
// Used as part of the trust-store approval key so build-mode and test-mode approvals are
// tracked independently.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PreliminaryCommandMode {
    Build,
    Test,
}

impl PreliminaryCommandMode {
    // Lowercase identifier used in the `mode` field of `~/.fixtrust.toml` entries.
    pub fn as_str(&self) -> &'static str {
        match self {
            PreliminaryCommandMode::Build => "build",
            PreliminaryCommandMode::Test => "test",
        }
    }
}

// A single preliminary command queued to run before compilation. One entry is created per
// argv in a `fixproj.toml` `preliminary_commands` array, with `project_name`, `mode`, and
// `source` identifying which project the command came from.
#[derive(Clone)]
pub struct PreliminaryCommand {
    // Absolute path of the directory in which the command will run.
    pub work_dir: PathBuf,
    // argv of the command (must be non-empty; the first element is the executable).
    pub command: Vec<String>,
    // `[general] name` of the project that declared this command (used for display).
    pub project_name: String,
    // Whether this command came from `[build]` or `[build.test]`.
    pub mode: PreliminaryCommandMode,
    // Where the declaring project originated from. Combined with `mode` (and commit for
    // git deps) to look up the trust-store entry that approves this command.
    pub source: ProjectOrigin,
}

impl PreliminaryCommand {
    // Execute this command synchronously. Returns an error if the process fails to start
    // or exits non-zero. Scans the command's stdout for lines in the
    // `fix.ld_flags=...` format and appends their flags to `config.ld_flags`.
    pub fn run(&self, config: &mut Configuration) -> Result<(), Errors> {
        let mut com = Command::new(&self.command[0]);
        for arg in &self.command[1..] {
            com.arg(arg);
        }
        let work_dir = to_absolute_path(&self.work_dir)?;
        com.current_dir(&work_dir);
        let status = com.status().map_err(|e| {
            Errors::from_msg(format!(
                "Failed to run command \"{}\": {:?}",
                self.command.join(" "),
                e
            ))
        })?;
        if !status.success() {
            return Err(Errors::from_msg(format!(
                "Command \"{}\" failed with exit code {}.",
                self.command.join(" "),
                status.code().unwrap_or(-1)
            )));
        }

        // Get stdout as String.
        let output = com.output().map_err(|e| {
            Errors::from_msg(format!(
                "Failed to run command \"{}\": {:?}",
                self.command.join(" "),
                e
            ))
        })?;

        // If the command outputs build flags in the designated format, add them to the configuration.
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout_lines: Vec<&str> = stdout.lines().collect();
        for stdout_line in stdout_lines {
            if stdout_line.starts_with(PRELIMINARY_BUILD_LD_FLAGS) {
                let ld_flags = stdout_line[PRELIMINARY_BUILD_LD_FLAGS.len()..].trim();
                let mut ld_flags = split_string_by_space_not_quated(ld_flags);
                config.ld_flags.append(&mut ld_flags);
            }
        }
        Ok(())
    }
}

// ---- Approval flow ----

// Entry point called from `Configuration::run_preliminary_commands`. Classifies the
// queued `preliminary_commands` into approved / pending groups using the user's trust
// store, resolves the pending ones (via the `--allow-preliminary-commands` flag, a
// non-interactive failure, or an interactive 3-choice prompt), and finally executes all
// commands in the order they were registered.
pub fn approve_and_run(config: &mut Configuration) -> Result<(), Errors> {
    if config.preliminary_commands.is_empty() {
        return Ok(());
    }

    let groups = group_commands(&config.preliminary_commands);
    let mut trust_store = TrustStore::load()?;

    let mut approved: Vec<Classified> = vec![];
    let mut pending: Vec<Classified> = vec![];
    for g in groups {
        let status = classify(&g, &trust_store);
        match status {
            GroupStatus::Approved => approved.push(Classified { group: g, status }),
            _ => pending.push(Classified { group: g, status }),
        }
    }

    if !approved.is_empty() {
        display_approved(&approved);
    }

    if !pending.is_empty() {
        if config.allow_preliminary_commands {
            display_auto_approved(&pending);
        } else if !is_interactive() {
            display_non_interactive_error(&pending);
            return Err(Errors::from_msg(
                "preliminary commands require approval.".to_string(),
            ));
        } else {
            prompt_and_record(&pending, &mut trust_store)?;
        }
    }

    // Execute all preliminary_commands in the order they were registered.
    for cmd in &config.preliminary_commands.clone() {
        cmd.run(config)?;
    }
    Ok(())
}

// All preliminary commands that share the same `(source, mode)` tuple -- i.e. every
// command declared under the same `[build]` or `[build.test]` section of the same
// project. The trust-store lookup granularity is one `Group`.
struct Group {
    source: ProjectOrigin,
    mode: PreliminaryCommandMode,
    // `[general] name` of the declaring project, copied for convenience at display time.
    project_name: String,
    // Commands belonging to this group. Cloned (rather than borrowed) so this type can be
    // moved around freely without wrestling with the caller's lifetimes.
    commands: Vec<PreliminaryCommand>,
}

// Result of looking a `Group` up in the trust store.
enum GroupStatus {
    // No matching entry and no previous approval for this (source, mode).
    New,
    // A previous approval exists for this (source, mode) but with a different commit hash.
    // Only possible for git dependencies; surfaced as `CHANGED from commit abcd…` so the
    // user can review what changed between updates.
    Changed { old_commit: Option<String> },
    // Exact match in the trust store; the group can run without prompting.
    Approved,
}

// A `Group` paired with its classification. Produced by `classify` and consumed by the
// approval / display pipeline.
struct Classified {
    group: Group,
    status: GroupStatus,
}

// Bucket every queued `PreliminaryCommand` into `Group`s keyed by `(source, mode)` while
// preserving the original registration order across groups.
fn group_commands(commands: &[PreliminaryCommand]) -> Vec<Group> {
    let mut groups: Vec<Group> = vec![];
    for cmd in commands {
        if let Some(g) = groups
            .iter_mut()
            .find(|g| g.source == cmd.source && g.mode == cmd.mode)
        {
            g.commands.push(cmd.clone());
        } else {
            groups.push(Group {
                source: cmd.source.clone(),
                mode: cmd.mode,
                project_name: cmd.project_name.clone(),
                commands: vec![cmd.clone()],
            });
        }
    }
    groups
}

// Decide whether a group is `Approved`, `Changed`, or `New` relative to the trust store.
// `Changed` is only possible for git dependencies (path-based sources are trusted
// regardless of their pinned commit).
fn classify(g: &Group, trust_store: &TrustStore) -> GroupStatus {
    if trust_store.is_approved(&g.source, g.mode) {
        return GroupStatus::Approved;
    }
    if let ProjectOrigin::Git { .. } = &g.source {
        let src_key = g.source.to_trust_key();
        let mode_str = g.mode.as_str();
        if let Some(prev) = trust_store
            .approvals
            .iter()
            .find(|a| a.source == src_key && a.mode == mode_str)
        {
            return GroupStatus::Changed {
                old_commit: prev.commit_hash.clone(),
            };
        }
    }
    GroupStatus::New
}

// Determine whether to treat the session as interactive.
// Normally this is `stdin().is_terminal()`. The env var `FIX_TEST_FORCE_INTERACTIVE=1`
// forces the interactive branch for integration tests that pipe stdin.
fn is_interactive() -> bool {
    if std::env::var("FIX_TEST_FORCE_INTERACTIVE").ok().as_deref() == Some("1") {
        return true;
    }
    std::io::stdin().is_terminal()
}

// ---- Interactive prompt ----

// The three answers to the approval prompt: approve permanently, approve once, or reject.
enum Choice {
    Yes,
    Once,
    No,
}

// Prompt the user once and read their answer from stdin. EOF, I/O errors, empty input,
// and any character other than `y` / `o` / `n` (case-insensitive) all fall back to `No`.
fn read_choice() -> Choice {
    eprintln!();
    eprint!("{} ", prompt_style("Approve? [y/o/N]:"));
    let _ = std::io::stderr().flush();
    let stdin = std::io::stdin();
    let mut line = String::new();
    let choice = match stdin.lock().read_line(&mut line) {
        Ok(0) => {
            // EOF. Emit a newline so subsequent output starts on a fresh line.
            eprintln!();
            Choice::No
        }
        Ok(_) => {
            let first = line.trim().chars().next().map(|c| c.to_ascii_lowercase());
            match first {
                Some('y') => Choice::Yes,
                Some('o') => Choice::Once,
                _ => Choice::No,
            }
        }
        Err(_) => {
            eprintln!();
            Choice::No
        }
    };
    // Blank line separating the user's answer from whatever comes next.
    eprintln!();
    choice
}

// Walk pending groups project by project, ask the user, and persist `Yes` answers into
// the trust store immediately. A `No` answer aborts the whole build without prompting
// for any remaining projects. Trust-store write failures are downgraded to a warning so
// the build can still proceed as a one-time approval.
fn prompt_and_record(
    pending: &[Classified],
    trust_store: &mut TrustStore,
) -> Result<(), Errors> {
    // Regroup pending entries by source so we issue one prompt per project.
    let by_source = group_classified_by_source(pending);
    // Resolve the trust-store path once so the prompt can show it to the user as part
    // of the `[y]` explanation. If resolution fails (no HOME), fall back to a generic
    // placeholder -- `save()` below will surface the real error.
    let trust_path = crate::metafiles::trust_store::default_path()
        .unwrap_or_else(|_| std::path::PathBuf::from("~/.fixtrust.toml"));

    for sg in &by_source {
        // Header line that sets the context for every per-project prompt.
        eprintln!(
            "{}",
            prompt_style("The following project requests your approval to run preliminary commands:")
        );
        display_prompt_block(sg, &trust_path);
        match read_choice() {
            Choice::Yes => {
                for entry in &sg.entries {
                    let preview: Vec<Vec<String>> = entry
                        .group
                        .commands
                        .iter()
                        .map(|c| c.command.clone())
                        .collect();
                    let approval = make_approval(
                        &entry.group.source,
                        entry.group.mode,
                        entry.group.project_name.clone(),
                        preview,
                    );
                    trust_store.record(approval);
                }
                if let Err(e) = trust_store.save() {
                    warn_msg(&format!(
                        "Failed to record approval to trust store: {}. \
                         Proceeding as a one-time approval.",
                        e.to_string()
                    ));
                }
            }
            Choice::Once => {
                // Nothing to record.
            }
            Choice::No => {
                return Err(Errors::from_msg(
                    "preliminary commands not approved. aborted.".to_string(),
                ));
            }
        }
    }
    Ok(())
}

// Display-time grouping of `Classified` entries that share the same project source.
// Used so that when a `fix test` invocation has both build-mode and test-mode approvals
// pending for one project, we issue a single prompt covering both.
struct SourceGroup<'a> {
    source: ProjectOrigin,
    project_name: String,
    // One entry per mode; at most two (build, test), sorted so Build comes first.
    entries: Vec<&'a Classified>,
}

// Re-bucket `Classified` entries by their source (project), preserving the order in
// which sources first appear and sorting inner entries so Build always precedes Test.
fn group_classified_by_source<'a>(pending: &'a [Classified]) -> Vec<SourceGroup<'a>> {
    let mut groups: Vec<SourceGroup<'a>> = vec![];
    for c in pending {
        if let Some(sg) = groups.iter_mut().find(|sg| sg.source == c.group.source) {
            sg.entries.push(c);
        } else {
            groups.push(SourceGroup {
                source: c.group.source.clone(),
                project_name: c.group.project_name.clone(),
                entries: vec![c],
            });
        }
    }
    // Stable sort so build comes before test within each source group.
    for sg in &mut groups {
        sg.entries.sort_by_key(|c| match c.group.mode {
            PreliminaryCommandMode::Build => 0,
            PreliminaryCommandMode::Test => 1,
        });
    }
    groups
}

// ---- Display helpers ----

// Format a command's argv into a shell-ready line for display (each argument quoted
// when necessary).
fn format_argv(argv: &[String]) -> String {
    argv.iter()
        .map(|a| shell_escape(a))
        .collect::<Vec<_>>()
        .join(" ")
}

// Minimal POSIX-ish shell escape sufficient for *display* of command lines (not safe
// to feed back into a shell). Strings made only of alphanumerics and `@%_+=:,./-`
// pass through unquoted; anything else is single-quoted with embedded quotes escaped.
fn shell_escape(s: &str) -> String {
    let safe = !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "@%_+=:,./-".contains(c));
    if safe {
        s.to_string()
    } else {
        let escaped = s.replace('\'', "'\\''");
        format!("'{}'", escaped)
    }
}

// Short (7-char) abbreviation of a git commit hash for compact display.
fn short_commit(commit: &str) -> String {
    if commit.len() > 7 {
        commit[..7].to_string()
    } else {
        commit.to_string()
    }
}

// Print the `source: <url> (commit <short>)` line when this project came from git.
// Emits nothing for local/root projects -- their origin is already conveyed by `path:`.
fn print_source_line(source: &ProjectOrigin) {
    if let ProjectOrigin::Git { url, commit } = source {
        eprintln!("    source: {} (commit {})", url, short_commit(commit));
    }
}

// Print the `path: <work_dir>` line, doubling as both the filesystem location and the
// cwd in which the commands will run.
fn print_path_line(work_dir: &Path) {
    eprintln!("    path: {}", work_dir.display());
}

// Print each command on its own `$ ...` line.
fn print_command_lines(commands: &[PreliminaryCommand]) {
    for c in commands {
        eprintln!("    $ {}", format_argv(&c.command));
    }
}

// Render the `(already approved)` header and the command listing for every project
// whose preliminary_commands were pre-approved by the trust store.
fn display_approved(approved: &[Classified]) {
    info_msg("Running preliminary commands (already approved).");
    eprintln!();
    // Re-group by source for cleaner display (same path/source shown once).
    let by_source = group_classified_by_source(approved);
    for sg in &by_source {
        eprintln!("  [{}] (approved)", sg.project_name);
        print_source_line(&sg.source);
        if let Some(first) = sg.entries.first().and_then(|e| e.group.commands.first()) {
            print_path_line(&first.work_dir);
        }
        for e in &sg.entries {
            if sg.entries.len() > 1 {
                eprintln!("    ({})", e.group.mode.as_str());
            }
            print_command_lines(&e.group.commands);
        }
        eprintln!();
    }
}

// Render the command listing when `--allow-preliminary-commands` was used. Commands
// still run, but the trust store is not updated; the status label reminds the reader
// that the approval came from the CLI flag, not from a recorded trust entry.
fn display_auto_approved(pending: &[Classified]) {
    info_msg("Running preliminary commands (auto-approved via --allow-preliminary-commands).");
    eprintln!();
    let by_source = group_classified_by_source(pending);
    for sg in &by_source {
        eprintln!(
            "  [{}] (auto-approved (--allow-preliminary-commands))",
            sg.project_name
        );
        print_source_line(&sg.source);
        if let Some(first) = sg.entries.first().and_then(|e| e.group.commands.first()) {
            print_path_line(&first.work_dir);
        }
        for e in &sg.entries {
            if sg.entries.len() > 1 {
                eprintln!("    ({})", e.group.mode.as_str());
            }
            print_command_lines(&e.group.commands);
        }
        eprintln!();
    }
}

// Render the failure message used when pending approvals remain but stdin is not a tty
// and `--allow-preliminary-commands` was not supplied. Lists the unresolved projects
// and points the reader at the two ways to approve.
fn display_non_interactive_error(pending: &[Classified]) {
    eprintln!(
        "{}: preliminary commands require approval, but no interactive terminal is available.",
        "error".red().bold()
    );
    eprintln!();
    let by_source = group_classified_by_source(pending);
    for sg in &by_source {
        print_pending_entry_header(sg);
        print_source_line(&sg.source);
        if let Some(first) = sg.entries.first().and_then(|e| e.group.commands.first()) {
            print_path_line(&first.work_dir);
        }
        for e in &sg.entries {
            if sg.entries.len() > 1 {
                eprintln!("    ({})", e.group.mode.as_str());
            }
            print_command_lines(&e.group.commands);
        }
        eprintln!();
    }
    eprintln!("To approve:");
    eprintln!("  - Run fix in an interactive terminal and answer the prompt.");
    eprintln!(
        "  - Or pass --allow-preliminary-commands to bypass for this invocation only."
    );
}

// Render the full prompt block for a single project's pending approval, including the
// 3-choice menu. The wording of the `[y]` line and its sub-bullets is adapted to the
// source kind (git vs local/root), and `trust_path` is inlined so the user can see
// which file will be edited on `y`.
fn display_prompt_block(sg: &SourceGroup, trust_path: &Path) {
    print_pending_entry_header(sg);
    print_source_line(&sg.source);
    if let Some(first) = sg.entries.first().and_then(|e| e.group.commands.first()) {
        print_path_line(&first.work_dir);
    }
    for e in &sg.entries {
        if sg.entries.len() > 1 {
            eprintln!("    ({})", e.group.mode.as_str());
        }
        print_command_lines(&e.group.commands);
    }
    eprintln!();
    eprintln!("{}", prompt_style("Choices:"));
    match &sg.source {
        ProjectOrigin::Git { .. } => {
            eprintln!("  [y] Yes, I approve -- and trust this commit from now on");
            eprintln!(
                "        (records this approval in {})",
                trust_path.display()
            );
            eprintln!("  [o] Yes, I approve -- but only for this run");
            eprintln!("  [n] No, I decline");
        }
        ProjectOrigin::Local(p) => {
            eprintln!("  [y] Yes, I approve -- and trust this path from now on");
            eprintln!(
                "        (future changes at {} will not prompt;",
                p.display()
            );
            eprintln!(
                "         records this approval in {})",
                trust_path.display()
            );
            eprintln!("  [o] Yes, I approve -- but only for this run");
            eprintln!("        (recommended unless this is your own project)");
            eprintln!("  [n] No, I decline");
        }
    }
}

// Render the status line for a pending project entry. Combines the project label with
// NEW / CHANGED / mode-specific annotations depending on the constituent entries.
fn print_pending_entry_header(sg: &SourceGroup) {
    // If any entry is CHANGED, surface that (it carries more context than NEW).
    if let Some(changed) = sg.entries.iter().find_map(|e| match &e.status {
        GroupStatus::Changed { old_commit } => Some(old_commit.clone()),
        _ => None,
    }) {
        let tag = match changed {
            Some(c) => format!("CHANGED from commit {}", short_commit(&c)),
            None => "CHANGED".to_string(),
        };
        eprintln!("  [{}] ({})", sg.project_name, tag);
        return;
    }
    // All entries are NEW. If only a single mode entry exists we annotate which one
    // so the user can see why they are being re-prompted (e.g. after `fix test` when
    // build was approved separately).
    let mode_hint = if sg.entries.len() == 1 {
        match sg.entries[0].group.mode {
            PreliminaryCommandMode::Build => "",
            PreliminaryCommandMode::Test => " -- test mode",
        }
    } else {
        ""
    };
    eprintln!("  [{}] (NEW{})", sg.project_name, mode_hint);
}
