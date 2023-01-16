extern crate rustc_version;
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;
extern crate libc;
use once_cell::sync::Lazy;

static OBJECT_ID: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(0));

static OBJECT_TABLE: Lazy<Mutex<HashMap<i64, ObjectInfo>>> =
    Lazy::new(|| Mutex::new(Default::default()));

struct ObjectInfo {
    addr: usize,
    refcnt: i64,
    code: String,
    is_global: bool, // A global object should not be deallocated (in other words, it should be leaked!)
}

#[no_mangle]
// Report malloc.
// Returns reserved object id.
pub extern "C" fn report_malloc(address: *const i8, name: *const i8) -> i64 {
    let name_c_str = unsafe { CStr::from_ptr(name) };
    let name_c_str = name_c_str.to_str();
    if name_c_str.is_err() {
        println!("[Sanitizer] Failed to convert given name to &str.");
    }
    let name_c_str = name_c_str.unwrap();
    let mut guard = (*OBJECT_ID).lock().unwrap();
    *guard += 1;
    let obj_id = *guard;
    if VERBOSE {
        println!(
            "[Sanitizer] Object id={} is allocated. refcnt=(0 -> 1), addr={:#X}, code = {}",
            obj_id, address as usize, name_c_str
        );
    }
    let mut object_table = (*OBJECT_TABLE).lock().unwrap();
    let info = ObjectInfo {
        addr: address as usize,
        refcnt: 1,
        code: String::from(name_c_str),
        is_global: false,
    };
    object_table.insert(obj_id, info);
    obj_id
}

#[no_mangle]
// Mark an object as global.
pub extern "C" fn mark_as_global(obj_id: i64) -> () {
    let mut object_table = (*OBJECT_TABLE).lock().unwrap();
    assert!(
        object_table.contains_key(&obj_id),
        "[Sanitizer] Object of object id={} isn't registered to sanitizer.",
        obj_id
    );
    let info = object_table.get_mut(&obj_id).unwrap();
    info.is_global = true;
    if VERBOSE {
        println!(
            "[Sanitizer] Object id={} is marked as global. refcnt={}, addr={:#X}, code = {}",
            obj_id, info.refcnt, info.addr, info.code
        );
    }
}

// Report retain.
#[no_mangle]
pub extern "C" fn report_retain(address: *const i8, obj_id: i64, refcnt: i64) -> () {
    assert_ne!(
        refcnt, 0,
        "[Sanitizer] Object id={} whose refcnt zero is retained!",
        obj_id
    );
    let mut object_table = (*OBJECT_TABLE).lock().unwrap();
    assert!(
        object_table.contains_key(&obj_id),
        "[Sanitizer] Retain of object id={} is reported but it isn't registered to sanitizer.",
        obj_id
    );
    let info = object_table.get_mut(&obj_id).unwrap();
    if !info.is_global {
        assert_eq!(
            info.refcnt, refcnt,
            "[Sanitizer] The refcnt of object id={} in report_retain mismatch! reported={}, sanitizer={}",
            obj_id, refcnt, info.refcnt
        );
    } else if refcnt != info.refcnt {
        if VERBOSE {
            println!("[Sanitizer] The refcnt of object id={} in report_retain mismatch but it is global. reported={}, sanitizer={}",
            obj_id, refcnt, info.refcnt)
        }
    }
    info.refcnt += 1;
    if VERBOSE {
        println!(
            "[Sanitizer] Object id={} is retained. refcnt=({} -> {}), addr={:#X}, code = {}",
            obj_id,
            refcnt,
            refcnt + 1,
            address as usize,
            info.code
        );
    }
}

// Report release.
#[no_mangle]
pub extern "C" fn report_release(address: *const i8, obj_id: i64, refcnt: i64) -> () {
    assert_ne!(
        refcnt, 0,
        "[Sanitizer] Object id={} whose refcnt zero is retained!",
        obj_id
    );
    let mut object_info = (*OBJECT_TABLE).lock().unwrap();
    assert!(
        object_info.contains_key(&obj_id),
        "[Sanitizer] Release of object id={} is reported but it isn't registered to sanitizer.",
        obj_id
    );
    let info = object_info.get_mut(&obj_id).unwrap();
    if !info.is_global {
        assert_eq!(
            info.refcnt, refcnt,
            "[Sanitizer] The refcnt of object id={} in report_release mismatch! reported={}, sanitizer={}",
            obj_id, refcnt, info.refcnt
        );
        info.refcnt -= 1;
    } else if refcnt != info.refcnt {
        if VERBOSE {
            println!("[Sanitizer] The refcnt of object id={} in report_retain mismatch but it is global. reported={}, sanitizer={}",
            obj_id, refcnt, info.refcnt)
        }
    }
    if VERBOSE {
        println!(
            "[Sanitizer] Object id={} is released. refcnt=({} -> {}), addr={:#X}, code = {}",
            obj_id,
            refcnt,
            refcnt - 1,
            address as usize,
            info.code
        );
    }

    if info.refcnt == 0 {
        assert!(
            !info.is_global,
            "[Sanitizer] Object of object id={} is global but deallocated!",
            obj_id
        );
        // When deallocated, remove it from OBJECT_INFO
        object_info.remove(&obj_id);
    }
}

// Check if all non-global objects had been released.
#[no_mangle]
pub extern "C" fn check_leak() -> () {
    let object_info = (*OBJECT_TABLE).lock().unwrap();
    if object_info.is_empty() {
        return;
    }
    let mut leak = false;
    for (id, info) in &*object_info {
        if !info.is_global {
            leak = true;
            println!(
                "[Sanitizer] Object id={} is leaked. refcnt={}, addr={:#X}, code = {}",
                id, info.refcnt, info.addr, info.code
            );
        }
    }
    if leak {
        panic!("[Sanitizer] Some objects leaked!");
    }
}

const VERBOSE: bool = true;
