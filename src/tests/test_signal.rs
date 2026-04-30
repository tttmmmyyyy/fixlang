#[cfg(unix)]
mod tests {
    use crate::{
        commands::run::run,
        configuration::Configuration,
        error::panic_if_err,
        misc::save_temporary_source,
    };
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn test_signal_number_on_abort() {
        // A Fix program that calls `undefined`, which aborts the program.
        let source = r#"
            module Main;
            main : IO ();
            main = println(undefined("reached"));
        "#;
        let src = match save_temporary_source(source, "test_signal") {
            Ok(src) => src,
            Err(e) => panic!("Failed to save temporary source: {}", e),
        };
        let mut config = Configuration::develop_mode();
        config.add_user_source_file(src.file_path);
        let res = panic_if_err(run(config, false));
        let output = res.expect("Failed to run the program");

        // The process should have been terminated by a signal, so code() should be None.
        assert!(
            output.status.code().is_none(),
            "Expected process to be terminated by signal, but got exit code: {:?}",
            output.status.code()
        );

        // The signal should be SIGABRT (6).
        let signal = output.status.signal();
        assert_eq!(
            signal,
            Some(6),
            "Expected SIGABRT (signal 6), but got signal: {:?}",
            signal
        );
    }
}
