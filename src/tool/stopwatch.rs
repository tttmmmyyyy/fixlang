use std::{cell::Cell, time::Instant};

pub struct StopWatch {
    name: String,
    now: Instant,
    running: Cell<bool>,
}

impl StopWatch {
    #[allow(dead_code)]
    pub fn new(name: &str, start: bool) -> Self {
        Self {
            name: name.to_string(),
            now: Instant::now(),
            running: Cell::new(start),
        }
    }

    pub fn end(&self) {
        if !self.running.get() {
            return;
        }
        let elapsed = self.now.elapsed();
        let time_str = format!("{}.{:03} sec", elapsed.as_secs(), elapsed.subsec_millis());
        let log = format!("{}: {}", self.name, time_str);
        eprintln!("{}", log);
        // write_lsp_log(&log);
        self.running.set(false);
    }
}

impl Drop for StopWatch {
    fn drop(&mut self) {
        self.end();
    }
}
