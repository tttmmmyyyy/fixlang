extern crate rustc_version;
use rustc_version::{version, Version};
use std::ffi::CString;
use std::io::{self, Write};
use std::process::exit;
use std::ptr::null;
extern crate libc;
use libc::{c_char, c_int, c_ulonglong, c_void};

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
pub extern "C" fn hello_runtime() -> () {
    // unsafe {
    //     let p: *mut c_void = libc::malloc(10);
    //     let p = p as *mut c_char;
    // }
    println!("Hello runtime!");
}

#[no_mangle]
pub extern "C" fn report_malloc(address: *const i8) -> () {
    if VERBOSE {
        println!("Object at {:#X} is allocated (0 -> 1)", address as usize,);
    }
}

#[no_mangle]
pub extern "C" fn report_retain(address: *const i8, refcnt: i64) -> () {
    if VERBOSE {
        println!(
            "Object at {:#X} is retained ({} -> {})",
            address as usize,
            refcnt,
            refcnt + 1
        );
    }
    if refcnt == 0 {
        panic!("Object with refcnt zero is retained!",)
    }
}

#[no_mangle]
pub extern "C" fn report_release(address: *const i8, refcnt: i64) -> () {
    if VERBOSE {
        println!(
            "Object at {:#X} is released ({} -> {})",
            address as usize,
            refcnt,
            refcnt - 1
        );
    }
    if refcnt == 0 {
        panic!("Object with refcnt zero is released!",)
    }
}

const VERBOSE: bool = false;
