use std::process::Command;

// Run `cargo install --locked --path .`.
pub fn install_fix() {
    let _ = Command::new("cargo")
        .arg("install")
        .arg("--locked")
        .arg("--path")
        .arg(".")
        .output()
        .expect("Failed to run cargo install.");
}
