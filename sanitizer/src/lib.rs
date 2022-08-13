extern crate rustc_version;
use rustc_version::{version, Version};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::io::{self, Write};
use std::process::exit;
use std::ptr::null;
use std::sync::Mutex;
use std::thread::panicking;
extern crate libc;
use libc::{c_char, c_int, c_ulonglong, c_void};
use once_cell::sync::Lazy;

static OBJECT_ID: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(0));

static OBJECT_INFO: Lazy<Mutex<HashMap<i64, ObjectInfo>>> =
    Lazy::new(|| Mutex::new(Default::default()));

struct ObjectInfo {
    id: i64,
    addr: usize,
    refcnt: i64,
}

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
    let mut object_info = (*OBJECT_INFO).lock().unwrap();
    let info = ObjectInfo {
        id: objid,
        addr: address as usize,
        refcnt: 1,
    };
    object_info.insert(objid, info);
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
    assert_ne!(
        refcnt, 0,
        "Object id={} whose refcnt zero is retained!",
        obj_id
    );
    let mut object_info = (*OBJECT_INFO).lock().unwrap();
    assert!(
        object_info.contains_key(&obj_id),
        "Retain of object id={} is reported but it isn't registered to sanitizer.",
        obj_id
    );
    let info = object_info.get_mut(&obj_id).unwrap();
    assert_eq!(
        info.refcnt, refcnt,
        "The refcnt of object id={} mismatch! reported={}, sanitizer={}",
        obj_id, refcnt, info.refcnt
    );
    info.refcnt = refcnt;
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
    assert_ne!(
        refcnt, 0,
        "Object id={} whose refcnt zero is retained!",
        obj_id
    );
    let mut object_info = (*OBJECT_INFO).lock().unwrap();
    assert!(
        object_info.contains_key(&obj_id),
        "Release of object id={} is reported but it isn't registered to sanitizer.",
        obj_id
    );
    let info = object_info.get_mut(&obj_id).unwrap();
    assert_eq!(
        info.refcnt, refcnt,
        "The refcnt of object id={} mismatch! reported={}, sanitizer={}",
        obj_id, refcnt, info.refcnt
    );
    info.refcnt = refcnt;

    if refcnt == 0 {
        // When deallocated, remove it from OBJECT_INFO
        object_info.remove(&obj_id);
    }
}

const VERBOSE: bool = false;
