extern crate rustc_version;
use rustc_version::{version, Version};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::{self, Write};
use std::process::exit;
use std::ptr::null;
use std::sync::Mutex;
use std::thread::panicking;
extern crate libc;
use libc::{c_char, c_int, c_ulonglong, c_void};
use once_cell::sync::Lazy;

static OBJECT_ID: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(0));

static OBJECT_TABLE: Lazy<Mutex<HashMap<i64, ObjectInfo>>> =
    Lazy::new(|| Mutex::new(Default::default()));

struct ObjectInfo {
    id: i64,
    addr: usize,
    refcnt: i64,
    code: String,
}

#[no_mangle]
// Returns reserved object id.
pub extern "C" fn report_malloc(address: *const i8, name: *const i8) -> i64 {
    let name_c_str = unsafe { CStr::from_ptr(name) };
    let name_c_str = name_c_str.to_str();
    if name_c_str.is_err() {
        println!("[report_malloc] Failed to convert given name to &str.");
    }
    let name_c_str = name_c_str.unwrap();
    let mut guard = (*OBJECT_ID).lock().unwrap();
    *guard += 1;
    let objid = *guard;
    if VERBOSE {
        println!(
            "Object id={} is allocated. refcnt=(0 -> 1), addr={:#X}, code = {}",
            objid, address as usize, name_c_str
        );
    }
    let mut object_table = (*OBJECT_TABLE).lock().unwrap();
    let info = ObjectInfo {
        id: objid,
        addr: address as usize,
        refcnt: 1,
        code: String::from(name_c_str),
    };
    object_table.insert(objid, info);
    objid
}

#[no_mangle]
pub extern "C" fn report_retain(address: *const i8, obj_id: i64, refcnt: i64) -> () {
    assert_ne!(
        refcnt, 0,
        "Object id={} whose refcnt zero is retained!",
        obj_id
    );
    let mut object_table = (*OBJECT_TABLE).lock().unwrap();
    assert!(
        object_table.contains_key(&obj_id),
        "Retain of object id={} is reported but it isn't registered to sanitizer.",
        obj_id
    );
    let info = object_table.get_mut(&obj_id).unwrap();
    assert_eq!(
        info.refcnt, refcnt,
        "The refcnt of object id={} in report_retain mismatch! reported={}, sanitizer={}",
        obj_id, refcnt, info.refcnt
    );
    info.refcnt += 1;
    if VERBOSE {
        println!(
            "Object id={} is retained. refcnt=({} -> {}), addr={:#X}, code = {}",
            obj_id,
            refcnt,
            refcnt + 1,
            address as usize,
            info.code
        );
    }
}

#[no_mangle]
pub extern "C" fn report_release(address: *const i8, obj_id: i64, refcnt: i64) -> () {
    assert_ne!(
        refcnt, 0,
        "Object id={} whose refcnt zero is retained!",
        obj_id
    );
    let mut object_info = (*OBJECT_TABLE).lock().unwrap();
    assert!(
        object_info.contains_key(&obj_id),
        "Release of object id={} is reported but it isn't registered to sanitizer.",
        obj_id
    );
    let info = object_info.get_mut(&obj_id).unwrap();
    assert_eq!(
        info.refcnt, refcnt,
        "The refcnt of object id={} in report_release mismatch! reported={}, sanitizer={}",
        obj_id, refcnt, info.refcnt
    );
    info.refcnt -= 1;

    if VERBOSE {
        println!(
            "Object id={} is released. refcnt=({} -> {}), addr={:#X}, code = {}",
            obj_id,
            refcnt,
            refcnt - 1,
            address as usize,
            info.code
        );
    }

    if info.refcnt == 0 {
        // When deallocated, remove it from OBJECT_INFO
        object_info.remove(&obj_id);
    }
}

#[no_mangle]
pub extern "C" fn check_leak() -> () {
    let object_info = (*OBJECT_TABLE).lock().unwrap();
    if object_info.is_empty() {
        return;
    }
    for (id, info) in &*object_info {
        println!(
            "Object id={} is leaked. refcnt={}, addr={:#X}, code = {}",
            id, info.refcnt, info.addr, info.code
        );
    }
    panic!("Some objects leaked!");
}

const VERBOSE: bool = false;
