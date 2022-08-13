extern crate rustc_version;
use rustc_version::{version, Version};
use std::io::{self, Write};
use std::process::exit;

#[test]
fn test_rustc_version() {
    if version().unwrap() != Version::new(1, 56, 0) {
        writeln!(
            &mut io::stderr(),
            "This crate requires rustc 1.56.0 (to emit llvm-ir by llvm 13.0.0)."
        )
        .unwrap();
        exit(1);
    }
}

#[no_mangle]
pub extern "C" fn rust_function() {
    println!("test function!");
}
