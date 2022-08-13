extern crate rustc_version;
use rustc_version::{version, Version};
use std::ffi::CString;
use std::io::{self, Write};
use std::process::exit;
use std::ptr::null;
extern crate libc;
use libc::{c_char, c_int, c_void};

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
pub extern "C" fn rust_function() -> i32 {
    // unsafe {
    //     let p: *mut c_void = libc::malloc(10);
    //     let p = p as *mut c_char;
    // }
    println!("rust_function called!");
    return 0;
}
