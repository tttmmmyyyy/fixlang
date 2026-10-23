// The approval of what a project asks a build to run: the `preliminary_commands` it lists and the
// `ld_flags` it hands to the linker. Both run code the project chooses (a linker flag such as
// `gcc`'s `-wrapper` starts any program), so a build uses them only once the user has approved
// them. The flow is invoked by `Configuration::approve_build_requests`.
// See the section "Approval of `preliminary_commands` and `ld_flags`" of `Document.md` for the
// user-visible specification.

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

// Which section of a project file a request comes from: `[build]` or `[build.test]`.
// Used as part of the trust-store approval key so build-mode and test-mode approvals are
// tracked independently.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BuildSection {
    Build,
    Test,
}

impl BuildSection {
    // Lowercase identifier used in the `mode` field of `~/.fixtrust.toml` entries.
    pub fn as_str(&self) -> &'static str {
        match self {
            BuildSection::Build => "build",
            BuildSection::Test => "test",
        }
    }
}

// What one section of one project file asks the build to run: its `preliminary_commands` and its
// `ld_flags`. A section that lists neither makes no request. The trust-store lookup granularity is
// one request.
#[derive(Clone)]
pub struct BuildRequest {
    // Absolute path of the project directory, in which the commands run.
    pub work_dir: PathBuf,
    // `[general] name` of the project that made the request (used for display).
    pub project_name: String,
    // Whether the request came from `[build]` or `[build.test]`.
    pub section: BuildSection,
    // Where the requesting project originated from. Combined with `section` (and commit for
    // git deps) to look up the trust-store entry that approves this request.
    pub source: ProjectOrigin,
    // The argv of each preliminary command, each non-empty with the executable first.
    pub preliminary_commands: Vec<Vec<String>>,
    // The flags to hand to the linker.
    pub ld_flags: Vec<String>,
}

// Execute the preliminary command `command` in `work_dir` synchronously. Returns an error if the
// process fails to start or exits non-zero. Scans the command's stdout for lines in the
// `fix.ld_flags=...` format and appends their flags to `config.ld_flags`.
fn run_preliminary_command(
    command: &[String],
    work_dir: &Path,
    config: &mut Configuration,
) -> Result<(), Errors> {
    let mut com = Command::new(&command[0]);
    for arg in &command[1..] {
        com.arg(arg);
    }
    let work_dir = to_absolute_path(work_dir)?;
    com.current_dir(&work_dir);
    let status = com.status().map_err(|e| {
        Errors::from_msg(format!(
            "Failed to run command \"{}\": {:?}",
            command.join(" "),
            e
        ))
    })?;
    if !status.success() {
        return Err(Errors::from_msg(format!(
            "Command \"{}\" failed with exit code {}.",
            command.join(" "),
            status.code().unwrap_or(-1)
        )));
    }

    // Get stdout as String.
    let output = com.output().map_err(|e| {
        Errors::from_msg(format!(
            "Failed to run command \"{}\": {:?}",
            command.join(" "),
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

// ---- Approval flow ----

// Entry point called from `Configuration::approve_build_requests`. Classifies the queued
// requests into approved / pending ones using the user's trust store, resolves the pending ones
// (via the `--trust-projects` flag, a non-interactive failure, or an interactive 3-choice prompt),
// and then grants every request: its `ld_flags` go to the linker, ahead of the flags the command
// line gives, and its preliminary commands run, in the order the requests were registered.
pub fn approve_and_run(config: &mut Configuration) -> Result<(), Errors> {
    if config.build_requests.is_empty() {
        return Ok(());
    }

    let mut trust_store = TrustStore::load()?;

    let mut approved: Vec<Classified> = vec![];
    let mut pending: Vec<Classified> = vec![];
    for request in &config.build_requests {
        let status = classify(request, &trust_store);
        let classified = Classified {
            request: request.clone(),
            status,
        };
        match classified.status {
            RequestStatus::Approved => approved.push(classified),
            _ => pending.push(classified),
        }
    }

    if !approved.is_empty() {
        display_approved(&approved);
    }

    if !pending.is_empty() {
        if config.trust_projects {
            display_auto_approved(&pending);
        } else if !is_interactive() {
            display_non_interactive_error(&pending);
            return Err(Errors::from_msg(
                "preliminary commands and ld_flags require approval.".to_string(),
            ));
        } else {
            prompt_and_record(&pending, &mut trust_store)?;
        }
    }

    let requests = config.build_requests.clone();
    let project_ld_flags = requests
        .iter()
        .flat_map(|request| request.ld_flags.iter().cloned());
    config.ld_flags.splice(0..0, project_ld_flags);
    for request in &requests {
        for command in &request.preliminary_commands {
            run_preliminary_command(command, &request.work_dir, config)?;
        }
    }
    Ok(())
}

// Result of looking a `BuildRequest` up in the trust store.
enum RequestStatus {
    // No matching entry and no previous approval for this (source, section).
    New,
    // A previous approval exists for this (source, section) but with a different commit hash.
    // Only possible for git dependencies; surfaced as `CHANGED from commit abcd…` so the
    // user can review what changed between updates.
    Changed { old_commit: Option<String> },
    // Exact match in the trust store; the request is granted without prompting.
    Approved,
}

// A `BuildRequest` paired with its classification. Produced by `classify` and consumed by the
// approval / display pipeline.
struct Classified {
    request: BuildRequest,
    status: RequestStatus,
}

// Decide whether a request is `Approved`, `Changed`, or `New` relative to the trust store.
// `Changed` is only possible for git dependencies (path-based sources are trusted
// regardless of their pinned commit).
fn classify(request: &BuildRequest, trust_store: &TrustStore) -> RequestStatus {
    if trust_store.is_approved(&request.source, request.section) {
        return RequestStatus::Approved;
    }
    if let ProjectOrigin::Git { .. } = &request.source {
        let src_key = request.source.to_trust_key();
        let mode_str = request.section.as_str();
        if let Some(prev) = trust_store
            .approvals
            .iter()
            .find(|a| a.source == src_key && a.mode == mode_str)
        {
            return RequestStatus::Changed {
                old_commit: prev.commit_hash.clone(),
            };
        }
    }
    RequestStatus::New
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

// Walk pending requests project by project, ask the user, and persist `Yes` answers into
// the trust store immediately. A `No` answer aborts the whole build without prompting
// for any remaining projects. Trust-store write failures are downgraded to a warning so
// the build can still proceed as a one-time approval.
fn prompt_and_record(pending: &[Classified], trust_store: &mut TrustStore) -> Result<(), Errors> {
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
            prompt_style(
                "The following project requests your approval to run preliminary commands \
                 or pass ld_flags to the linker:"
            )
        );
        display_prompt_block(sg, &trust_path);
        match read_choice() {
            Choice::Yes => {
                for entry in &sg.entries {
                    trust_store.record(make_approval(&entry.request));
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
                    "preliminary commands and ld_flags not approved. aborted.".to_string(),
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
    // One entry per section; at most two (build, test), sorted so Build comes first.
    entries: Vec<&'a Classified>,
}

// Re-bucket `Classified` entries by their source (project), preserving the order in
// which sources first appear and sorting inner entries so Build always precedes Test.
fn group_classified_by_source<'a>(pending: &'a [Classified]) -> Vec<SourceGroup<'a>> {
    let mut groups: Vec<SourceGroup<'a>> = vec![];
    for c in pending {
        if let Some(sg) = groups.iter_mut().find(|sg| sg.source == c.request.source) {
            sg.entries.push(c);
        } else {
            groups.push(SourceGroup {
                source: c.request.source.clone(),
                project_name: c.request.project_name.clone(),
                entries: vec![c],
            });
        }
    }
    // Stable sort so build comes before test within each source group.
    for sg in &mut groups {
        sg.entries.sort_by_key(|c| match c.request.section {
            BuildSection::Build => 0,
            BuildSection::Test => 1,
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

// Print what a request asks for: each preliminary command on its own `$ ...` line, and the
// linker flags on an `ld_flags: ...` line.
fn print_request_lines(request: &BuildRequest) {
    for command in &request.preliminary_commands {
        eprintln!("    $ {}", format_argv(command));
    }
    if !request.ld_flags.is_empty() {
        eprintln!("    ld_flags: {}", format_argv(&request.ld_flags));
    }
}

// Print the source and `path:` lines of a project and what each of its requests asks for,
// labelling each request with its section when the project made one for each.
fn print_source_group(sg: &SourceGroup) {
    print_source_line(&sg.source);
    if let Some(first) = sg.entries.first() {
        print_path_line(&first.request.work_dir);
    }
    for e in &sg.entries {
        if sg.entries.len() > 1 {
            eprintln!("    ({})", e.request.section.as_str());
        }
        print_request_lines(&e.request);
    }
}

// Render the `(already approved)` header and the listing for every project whose requests
// were pre-approved by the trust store.
fn display_approved(approved: &[Classified]) {
    info_msg("Running preliminary commands and passing ld_flags (already approved).");
    eprintln!();
    // Re-group by source for cleaner display (same path/source shown once).
    let by_source = group_classified_by_source(approved);
    for sg in &by_source {
        eprintln!("  [{}] (approved)", sg.project_name);
        print_source_group(sg);
        eprintln!();
    }
}

// Render the listing when `--trust-projects` was used. The requests are still granted, but the
// trust store is not updated; the status label reminds the reader that the approval came from the
// CLI flag, not from a recorded trust entry.
fn display_auto_approved(pending: &[Classified]) {
    info_msg(
        "Running preliminary commands and passing ld_flags (auto-approved via --trust-projects).",
    );
    eprintln!();
    let by_source = group_classified_by_source(pending);
    for sg in &by_source {
        eprintln!("  [{}] (auto-approved (--trust-projects))", sg.project_name);
        print_source_group(sg);
        eprintln!();
    }
}

// Render the failure message used when pending approvals remain but stdin is not a tty
// and `--trust-projects` was not supplied. Lists the unresolved projects and points the
// reader at the two ways to approve.
fn display_non_interactive_error(pending: &[Classified]) {
    eprintln!(
        "{}: preliminary commands and ld_flags require approval, but no interactive terminal is available.",
        "error".red().bold()
    );
    eprintln!();
    let by_source = group_classified_by_source(pending);
    for sg in &by_source {
        print_pending_entry_header(sg);
        print_source_group(sg);
        eprintln!();
    }
    eprintln!("To approve:");
    eprintln!("  - Run fix in an interactive terminal and answer the prompt.");
    eprintln!("  - Or pass --trust-projects to approve for this invocation only.");
}

// Render the full prompt block for a single project's pending approval, including the
// 3-choice menu. The wording of the `[y]` line and its sub-bullets is adapted to the
// source kind (git vs local/root), and `trust_path` is inlined so the user can see
// which file will be edited on `y`.
fn display_prompt_block(sg: &SourceGroup, trust_path: &Path) {
    print_pending_entry_header(sg);
    print_source_group(sg);
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
// NEW / CHANGED / section-specific annotations depending on the constituent entries.
fn print_pending_entry_header(sg: &SourceGroup) {
    // If any entry is CHANGED, surface that (it carries more context than NEW).
    if let Some(changed) = sg.entries.iter().find_map(|e| match &e.status {
        RequestStatus::Changed { old_commit } => Some(old_commit.clone()),
        _ => None,
    }) {
        let tag = match changed {
            Some(c) => format!("CHANGED from commit {}", short_commit(&c)),
            None => "CHANGED".to_string(),
        };
        eprintln!("  [{}] ({})", sg.project_name, tag);
        return;
    }
    // All entries are NEW. If only a single section entry exists we annotate which one
    // so the user can see why they are being re-prompted (e.g. after `fix test` when
    // build was approved separately).
    let mode_hint = if sg.entries.len() == 1 {
        match sg.entries[0].request.section {
            BuildSection::Build => "",
            BuildSection::Test => " -- test mode",
        }
    } else {
        ""
    };
    eprintln!("  [{}] (NEW{})", sg.project_name, mode_hint);
}
