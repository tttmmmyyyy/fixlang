use crate::{
    configuration::Configuration,
    constants::COMPILER_TEST_WORKING_PATH,
    misc::function_name,
    tests::test_util::{test_source, test_source_with_c},
};
use std::fs;

/// `read_file_string` reads back the whole string `write_file_string` wrote, and `read_line` reads
/// it back line by line, each line keeping its trailing newline and the last line having none.
#[test]
pub fn test_write_read_file_string() {
    // Create a working directory.
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);
    let tmp_file = format!("{}/{}.txt", COMPILER_TEST_WORKING_PATH, function_name!());

    let source = format!(
        r#"
        module Main; 
        main : IO ();
        main = (
            let file_path = "{}";
            let lines = ["Hello", "World!"];
            let content = Iterator::from_array(lines).intersperse("\n").concat_iter;
            do {{
                write_file_string(file_path, content);;

                let read_content = *read_file_string(file_path);
                assert_eq(|_|"case 1", content, read_content).lift;;

                let read_lines = *with_file(file_path, "r", |file| (
                    pure $ [*read_line(file), *read_line(file)]
                ));
                assert_eq(|_|"case 2", read_lines.@(0), lines.@(0) + "\n").lift;;
                assert_eq(|_|"case 3", read_lines.@(1), lines.@(1)).lift;;

                pure()
            }}.try(exit_with_msg(1))
        );
    "#,
        tmp_file
    );
    test_source(&source, Configuration::develop_mode());
    fs::remove_file(tmp_file).unwrap();
}

/// `is_eof` answers `true` once `read_string` has read the file to its end.
#[test]
pub fn test_is_eof() {
    // Create a working directory.
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);
    let tmp_file = format!("{}/{}.txt", COMPILER_TEST_WORKING_PATH, function_name!());

    let source = format!(
        r#"
        module Main; 
        
        main : IO ();
        main = (
            let file_path = "{}";
            let content = "Hello World!";
            do {{
                write_file_string(file_path, content);;

                let read_content = *with_file(file_path, "r", |file| (
                    let content = *read_string(file);
                    let is_eof = *is_eof(file).lift;
                    assert(|_|"file had not reached to EOF!", is_eof).lift;;
                    pure $ content
                ));
            
                assert_eq(|_|"read_content != content", content, read_content).lift;;

                pure()
            }}.try(exit_with_msg(1))
        );
    "#,
        tmp_file
    );
    test_source(&source, Configuration::develop_mode());
    fs::remove_file(tmp_file).unwrap();
}

/// `read_file_bytes` reads back the bytes `write_file_bytes` wrote, for data longer than the
/// 1024-byte chunks `read_bytes` reads it in.
#[test]
pub fn test_write_read_file_bytes() {
    // Create a working directory.
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);
    let tmp_file = format!("{}/{}.dat", COMPILER_TEST_WORKING_PATH, function_name!());

    // Test write_file_bytes, read_file_bytes.
    let source = format!(
        r#"
        module Main; 
        main : IO ();
        main = (
            let file_path = "{}";
            let data = Array::from_map(1024 + 512, |n| n.u8);
            do {{
                write_file_bytes(file_path, data);;

                let read = *read_file_bytes(file_path);
                assert_eq(|_|"case 1", data, read).lift;;

                pure()
            }}.try(exit_with_msg(1))
        );
    "#,
        tmp_file
    );
    test_source(&source, Configuration::develop_mode());
    fs::remove_file(tmp_file).unwrap();
}

/// `write_bytes` reports an error for a handle that accepts no bytes, and `read_n_bytes` for a
/// handle that gives none.
#[test]
pub fn test_write_read_bytes_report_failure() {
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);
    let tmp_file = format!("{}/{}.dat", COMPILER_TEST_WORKING_PATH, function_name!());

    let source = format!(
        r#"
        module Main;
        main : IO ();
        main = (
            let file_path = "{}";
            let res = *write_file_bytes(file_path, [1_U8, 2_U8, 3_U8]).to_result;
            assert(|_|"setup", res.is_ok);;

            // A handle opened only for reading accepts no bytes.
            let res = *with_file(file_path, "r", |handle| write_bytes(handle, [4_U8])).to_result;
            assert(|_|"write to a read-only handle", res.is_err);;

            // A handle opened only for appending gives no bytes.
            let res = *with_file(file_path, "a", |handle| read_n_bytes(handle, 1)).to_result;
            assert(|_|"read from a write-only handle", res.is_err);;

            pure()
        );
    "#,
        tmp_file
    );
    test_source(&source, Configuration::develop_mode());
    fs::remove_file(tmp_file).unwrap();
}

/// `read_line` returns a line that spans several of `fgets`'s buffers whole, a last line without a
/// newline, and the empty string once the file is exhausted, after which `is_eof` holds;
/// `read_n_bytes` returns nothing for a count of zero, the rest of the file for a count past it, and
/// nothing at the end; and writing no bytes succeeds and leaves the file empty.
#[test]
pub fn test_read_write_boundaries() {
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);
    let tmp_file = format!("{}/{}.txt", COMPILER_TEST_WORKING_PATH, function_name!());

    let source = format!(
        r#"
        module Main;
        main : IO ();
        main = (
            let file_path = "{}";
            let long1 = Array::fill(1023, "a").to_iter.concat_iter;
            let long2 = Array::fill(2500, "b").to_iter.concat_iter;
            let content = long1 + "\n" + long2 + "\n" + "tail";
            let res = *write_file_string(file_path, content).to_result;
            assert(|_|"setup", res.is_ok);;

            let res = *with_file(file_path, "r", |h| (
                let l1 = *read_line(h);
                let l2 = *read_line(h);
                let l3 = *read_line(h);
                let l4 = *read_line(h);
                let eof = *is_eof(h).lift;
                pure $ (l1, l2, l3, l4, eof)
            )).to_result;
            let (l1, l2, l3, l4, eof) = res.as_ok;
            assert_eq(|_|"line of 1023 bytes", l1, long1 + "\n");;
            assert_eq(|_|"line of 2500 bytes", l2, long2 + "\n");;
            assert_eq(|_|"last line without newline", l3, "tail");;
            assert_eq(|_|"read past the end", l4, "");;
            assert(|_|"eof after the end", eof);;

            let res = *read_file_string(file_path).to_result;
            assert_eq(|_|"read_string", res.as_ok, content);;

            let res = *with_file(file_path, "r", |h| (
                let zero = *read_n_bytes(h, 0);
                let all = *read_n_bytes(h, 100000);
                let after = *read_n_bytes(h, 10);
                pure $ (zero, all, after)
            )).to_result;
            let (zero, all, after) = res.as_ok;
            assert_eq(|_|"zero bytes", zero, []);;
            assert_eq(|_|"more than remain", all, content.get_bytes.pop_back);;
            assert_eq(|_|"at the end", after, []);;

            let res = *with_file(file_path, "w", |h| write_bytes(h, []);; write_string(h, "")).to_result;
            assert(|_|"write nothing", res.is_ok);;
            let res = *read_file_bytes(file_path).to_result;
            assert_eq(|_|"file is empty", res.as_ok, []);;

            pure()
        );
    "#,
        tmp_file
    );
    test_source(&source, Configuration::develop_mode());
    fs::remove_file(tmp_file).unwrap();
}

/// `with_file` closes the file when the action it runs succeeds, which leaves the handle's file
/// pointer null.
#[test]
pub fn test_with_file_closed_when_ok() {
    // Create a working directory.
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);
    let tmp_file = format!("{}/{}.txt", COMPILER_TEST_WORKING_PATH, function_name!());

    let source = format!(
        r#"
module Main; 

main : IO ();
main = (
    let file_path = "{}";
    let content = "Hello, This is a test file.";
    do {{
        write_file_string(file_path, content);;
        with_file(file_path, "r", |file|
            let dtor = file.@_data;
            let ptr = *dtor.boxed_to_retained_ptr.lift;
            FFI_CALL_IO[() store(Ptr), ptr].lift;;
            pure() : IOFail ()
        )
    }}.try(|_| pure());;

    let ptr = *FFI_CALL_IO[Ptr load()];
    let dtor = *ptr.boxed_from_retained_ptr;
    let handle = IOHandle {{ _data : dtor }};
    let file_ptr = *get_file_ptr(handle);
    assert_eq(|_|"file_ptr is not closed!", file_ptr, nullptr);;

    pure()
);
    "#,
        tmp_file
    );

    let c_source = r#"
#include <stdlib.h>

void *storage = NULL;

void store(void *ptr)
{
    storage = ptr;
}

void *load()
{
    return storage;
}
    "#;

    test_source_with_c(&source, c_source, function_name!());
    fs::remove_file(tmp_file).unwrap();
}

/// `with_file` closes the file when the action it runs fails, which leaves the handle's file pointer
/// null.
#[test]
pub fn test_with_file_closed_when_err() {
    // Create a working directory.
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);
    let tmp_file = format!("{}/{}.txt", COMPILER_TEST_WORKING_PATH, function_name!());

    let source = format!(
        r#"
module Main; 

main : IO ();
main = (
    let file_path = "{}";
    let content = "Hello, This is a test file.";
    do {{
        write_file_string(file_path, content);;
        with_file(file_path, "r", |file|
            let dtor = file.@_data;
            let ptr = *dtor.boxed_to_retained_ptr.lift;
            FFI_CALL_IO[() store(Ptr), ptr].lift;;
            throw("Error") : IOFail ()
        )
    }}.try(|_| pure());;

    let ptr = *FFI_CALL_IO[Ptr load()];
    let dtor = *ptr.boxed_from_retained_ptr;
    let handle = IOHandle {{ _data : dtor }};
    let file_ptr = *get_file_ptr(handle);
    assert_eq(|_|"file_ptr is not closed!", file_ptr, nullptr);;

    pure()
);
    "#,
        tmp_file
    );

    let c_source = r#"
#include <stdlib.h>

void *storage = NULL;

void store(void *ptr)
{
    storage = ptr;
}

void *load()
{
    return storage;
}
    "#;

    test_source_with_c(&source, c_source, function_name!());
    fs::remove_file(tmp_file).unwrap();
}
