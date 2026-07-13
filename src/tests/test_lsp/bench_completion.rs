// Opt-in benchmark for warm-state `textDocument/completion` latency.
//
// This is NOT a correctness test: it measures the round-trip latency the
// editor experiences for a completion request *after* the initial
// diagnostics run has settled (the cache is warm). It is gated behind the
// `FIX_LSP_BENCH` environment variable so it returns immediately during
// normal `cargo test` runs and only does work when explicitly requested:
//
//   FIX_LSP_BENCH=1 cargo test --release \
//     --  tests::test_lsp::bench_completion --nocapture
//
// The reported numbers are end-to-end (send request -> receive response),
// so they include the client-side JSON parse of the response in this test
// harness. For the non-dot case that returns the full candidate list,
// that parse is non-trivial; treat the absolute numbers as an upper bound
// and use the before/after delta as the signal when optimizing.

#[cfg(test)]
mod bench {
    use super::super::lsp_client::LspClient;
    use crate::tests::test_util::copy_dir_recursive;
    use serde_json::json;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    /// Number of measured completion requests per benchmark point.
    const ITERS: usize = 15;
    /// Number of discarded warm-up requests before measuring.
    const WARMUP: usize = 2;
    /// Per-request deadline before the benchmark gives up and panics.
    const TIMEOUT: Duration = Duration::from_secs(60);

    /// Absolute path to the directory holding the LSP test-case projects.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_lsp/cases");
        path
    }

    /// Copy a test-case project into a fresh temp directory and return the
    /// temp dir (kept alive for cleanup) and the canonicalized project path.
    fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src = get_test_cases_dir().join(project_name);
        let dst = temp_dir.path().join(project_name);
        copy_dir_recursive(&src, &dst).expect("Failed to copy test case");
        let dst = dst.canonicalize().expect("canonicalize");
        (temp_dir, dst)
    }

    /// Send one completion request and finely poll for the response,
    /// returning the round-trip duration and the number of items.
    fn timed_completion(
        client: &mut LspClient,
        uri: &str,
        line: u32,
        col: u32,
    ) -> (Duration, usize) {
        let start = Instant::now();
        let id = client
            .send_request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": col }
                }),
            )
            .expect("send completion");
        loop {
            if let Some(resp) = client.get_response(id) {
                let elapsed = start.elapsed();
                let result = resp.get("result").expect("response has result");
                let n = if result.is_array() {
                    result.as_array().unwrap().len()
                } else {
                    result
                        .get("items")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0)
                };
                return (elapsed, n);
            }
            if start.elapsed() > TIMEOUT {
                panic!("completion did not respond within {:?}", TIMEOUT);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Convert a `Duration` to fractional milliseconds.
    fn millis(d: Duration) -> f64 {
        d.as_secs_f64() * 1000.0
    }

    /// Run `ITERS` measured completions at `(line, col)` after `WARMUP`
    /// discarded warm-up calls, and print min / median / mean / max.
    fn bench_one(client: &mut LspClient, uri: &str, line: u32, col: u32, label: &str) {
        let (cold, n_items) = timed_completion(client, uri, line, col);
        for _ in 1..WARMUP {
            timed_completion(client, uri, line, col);
        }
        let mut samples: Vec<Duration> = Vec::with_capacity(ITERS);
        for _ in 0..ITERS {
            let (d, _) = timed_completion(client, uri, line, col);
            samples.push(d);
        }
        samples.sort();
        let min = samples[0];
        let med = samples[samples.len() / 2];
        let max = samples[samples.len() - 1];
        let mean = samples.iter().sum::<Duration>() / samples.len() as u32;
        eprintln!(
            "[bench] {label:<14} items={n_items:>4}  cold={:>7.1}ms  min={:>7.1}ms  med={:>7.1}ms  mean={:>7.1}ms  max={:>7.1}ms  (n={ITERS})",
            millis(cold),
            millis(min),
            millis(med),
            millis(mean),
            millis(max),
        );
    }

    /// Locate the cursor `(line, col)` just after the `.` in the first
    /// occurrence of `needle` (which must contain a `.`).
    fn pos_after_dot(text: &str, needle: &str) -> (u32, u32) {
        let dot_in_needle = needle.find('.').expect("needle has a dot");
        for (i, line) in text.lines().enumerate() {
            if let Some(start) = line.find(needle) {
                return (i as u32, (start + dot_in_needle + 1) as u32);
            }
        }
        panic!("needle {:?} not found in fixture", needle);
    }

    /// Locate the cursor `(line, col)` at the end of the first occurrence
    /// of `needle` (used for a non-dot context where `needle` is a
    /// lowercase identifier, yielding an empty namespace filter -> the
    /// full candidate list).
    fn pos_after(text: &str, needle: &str) -> (u32, u32) {
        for (i, line) in text.lines().enumerate() {
            if let Some(start) = line.find(needle) {
                return (i as u32, (start + needle.len()) as u32);
            }
        }
        panic!("needle {:?} not found in fixture", needle);
    }

    /// Measures warm-state completion latency for a non-dot (full list)
    /// request and two dot-context requests (`Array I64` and `I64`
    /// receivers); opt-in via `FIX_LSP_BENCH`, otherwise a no-op.
    #[test]
    fn bench_completion_warm() {
        if std::env::var("FIX_LSP_BENCH").is_err() {
            eprintln!(
                "[bench] skipped (set FIX_LSP_BENCH=1 to run the completion latency benchmark)"
            );
            return;
        }

        let (_temp_dir, project_dir) = setup_test_env("completion-bench");
        let main_rel = Path::new("main.fix");
        let text = std::fs::read_to_string(project_dir.join(main_rel)).expect("read main.fix");

        let mut client = LspClient::new(&project_dir).expect("start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(10))
            .expect("initialize LSP");
        client.open_document(main_rel).expect("open main.fix");

        // Warm the typecheck cache / snapshot by running diagnostics once
        // and waiting for it to settle. The user considers this initial
        // cost acceptable; we measure what happens *after* this.
        client.trigger_and_wait_for_diagnostics(main_rel);

        let uri = format!("file://{}", project_dir.join(main_rel).display());

        // Non-dot, full candidate list (empty namespace filter).
        let (l, c) = pos_after(&text, "let _ = sz");
        bench_one(&mut client, &uri, l, c, "non-dot/full");

        // Dot on an `Array I64` receiver.
        let (l, c) = pos_after_dot(&text, "arr.get_size");
        bench_one(&mut client, &uri, l, c, "dot/array");

        // Dot on an `I64` receiver.
        let (l, c) = pos_after_dot(&text, "42.compute");
        bench_one(&mut client, &uri, l, c, "dot/i64");

        client
            .shutdown(Duration::from_millis(500))
            .expect("shutdown LSP");
        client.finish().expect("reader thread clean");
    }
}
