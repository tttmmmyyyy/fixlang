use crate::{
    configuration::Configuration, constants::COMPILER_TEST_WORKING_PATH, misc::function_name,
    tests::util::test_source,
};
use std::fs;

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
    test_source(&source, Configuration::compiler_develop_mode());
    fs::remove_file(tmp_file).unwrap();
}

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
    test_source(&source, Configuration::compiler_develop_mode());
    fs::remove_file(tmp_file).unwrap();
}

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
            let data = Array::from_map(1024 + 512, |n| n.to_U8);
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
    test_source(&source, Configuration::compiler_develop_mode());
    fs::remove_file(tmp_file).unwrap();
}
