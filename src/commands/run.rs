use crate::commands::build::build;
use crate::constants::RUN_PATH;
use crate::error::{panic_if_err, panic_with_msg, Errors};
use crate::Configuration;
use crate::ValgrindTool;
use crate::DOT_FIXLANG;
use rand::Rng;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{self, Command, Output, Stdio};

pub fn run(
    mut config: Configuration,
    inherit_streams: bool,
) -> Result<Result<Output, io::Error>, Errors> {
    fs::create_dir_all(DOT_FIXLANG)
        .expect(format!("Failed to create \"{}\" directory.", DOT_FIXLANG).as_str());
    fs::create_dir_all(RUN_PATH)
        .expect(format!("Failed to create \"{}\" directory.", RUN_PATH).as_str());

    // For parallel execution, use different file name for each execution.
    let exec_path: String = format!("{}/a{}.out", RUN_PATH, rand::thread_rng().gen::<u64>());
    let user_specified_out_path = std::mem::replace(
        &mut config.out_file_path,
        Some(PathBuf::from(exec_path.clone())),
    );

    // Build executable file.
    build(&mut config)?;

    // Run the executable file.
    let mut com = if config.valgrind_tool == ValgrindTool::None {
        Command::new(exec_path.clone())
    } else {
        let mut com = config.valgrind_command()?;
        com.arg(exec_path.clone());
        com
    };
    for arg in &config.run_program_args {
        com.arg(arg);
    }
    if inherit_streams {
        com.stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit());
    }
    let output = com.output();

    // Clean up the temporary executable file.
    match user_specified_out_path {
        Some(out_path) => {
            // Move the temporary executable file to the specified output file.
            if let Err(e) = fs::rename(exec_path.clone(), out_path.clone()) {
                let _ = fs::remove_file(exec_path.clone()); // Ignore the error.
                panic_with_msg(&format!(
                    "Failed to rename \"{}\" to \"{}\": {}",
                    exec_path,
                    out_path.display(),
                    e
                ));
            }
        }
        None => {
            // If the output file is not specified, remove the temporary executable file.
            let _ = fs::remove_file(exec_path.clone()); // Ignore the error.
        }
    }

    Ok(output)
}

// Implementation of `fix run` command.
pub fn run_command(config: &Configuration) {
    let output = run(config.clone(), true);
    let output = panic_if_err(output);

    if let Err(e) = output {
        panic_with_msg(&format!("Failed to run the program: {}", e));
    }
    let output = output.unwrap();

    if output.status.code().is_none() {
        panic_with_msg("Program terminated by signal");
    }
    let code = output.status.code().unwrap();

    process::exit(code);
}
