extern crate rustc_version;
use rustc_version::{version, Version};
use std::cell::RefCell;
use std::ffi::CString;
use std::io::{self, Write};
use std::process::exit;
use std::ptr::null;
use std::sync::Mutex;
extern crate libc;
use libc::{c_char, c_int, c_ulonglong, c_void};
use once_cell::sync::Lazy;

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

static OBJECT_ID: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(0));

#[no_mangle]
// Returns reserved object id.
pub extern "C" fn report_malloc(address: *const i8) -> i64 {
    let mut guard = (*OBJECT_ID).lock().unwrap();
    *guard += 1;
    let objid = *guard;
    if VERBOSE {
        println!(
            "Object id={} is allocated. refcnt=(0 -> 1), addr={:#X}",
            objid, address as usize
        );
    }
    objid
}

#[no_mangle]
pub extern "C" fn report_retain(address: *const i8, obj_id: i64, refcnt: i64) -> () {
    if VERBOSE {
        println!(
            "Object id={} is retained. refcnt=({} -> {}), addr={:#X}",
            obj_id,
            refcnt,
            refcnt + 1,
            address as usize,
        );
    }
    if refcnt == 0 {
        panic!("Object with refcnt zero is retained!",)
    }
}

#[no_mangle]
pub extern "C" fn report_release(address: *const i8, obj_id: i64, refcnt: i64) -> () {
    if VERBOSE {
        println!(
            "Object id={} is released. refcnt=({} -> {}), addr={:#X}",
            obj_id,
            refcnt,
            refcnt - 1,
            address as usize,
        );
    }
    if refcnt == 0 {
        panic!("Object with refcnt zero is released!",)
    }
}

const VERBOSE: bool = false;
