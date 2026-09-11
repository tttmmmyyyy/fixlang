//! What the polling the harness waits by promises.
//!
//! Every wait in this directory is written on `poll_every`, so what it promises is what a wait
//! for a response, for a pass, or for the server's exit means.

#[cfg(test)]
mod tests {
    use super::super::lsp_client::poll_every;
    use std::time::{Duration, Instant};

    /// A wait whose condition already holds hands that look's answer back and stops, so that a
    /// test asking for what the server has already sent pays nothing for the asking.
    ///
    /// The timeout is empty, which leaves the wait with no time to spend, and the interval is
    /// long enough that a wait spending any shows it.
    #[test]
    fn test_a_wait_answers_a_condition_that_already_holds() {
        /// The gap between two looks.
        const INTERVAL: Duration = Duration::from_secs(5);

        let mut looks = 0;
        let started = Instant::now();
        let answer = poll_every(INTERVAL, Duration::ZERO, || {
            looks += 1;
            Some(looks)
        });

        assert_eq!(
            answer,
            Some(1),
            "the wait is expected to hand back the answer of the look that answered"
        );
        assert_eq!(
            looks, 1,
            "the wait is expected to stop at the look that answered, but it looked {} times",
            looks
        );
        assert!(
            started.elapsed() < INTERVAL,
            "the wait took {:?}, which is the interval it is expected to spend none of",
            started.elapsed()
        );
    }

    /// A wait looks once more when its time runs out, so that a condition turning true while the
    /// wait sleeps is answered rather than missed.
    ///
    /// The interval is the whole timeout, so the wait sleeps once and the look after that sleep
    /// is the one taken at the deadline. The condition answers nothing on the first look, so a
    /// wait that gives up without taking the second one answers nothing at all.
    #[test]
    fn test_a_wait_looks_once_more_when_its_time_runs_out() {
        let interval = Duration::from_millis(100);
        let mut looks = 0;
        let answer = poll_every(interval, interval, || {
            looks += 1;
            (looks > 1).then_some(looks)
        });

        assert!(
            answer.is_some(),
            "the wait gave up after {} look(s), without looking again when its time ran out",
            looks
        );
    }
}
