use crate::{configuration::Configuration, tests::util::test_source};

#[test]
pub fn test_string_unsafe_from_c_str_ptr() {
    let source = r#"
        module Main;
        
        main : IO ();
        main = (
            let src = "Hello World!";
            let cpy = src.borrow_c_str(String::_unsafe_from_c_str_ptr);
            assert_eq(|_|"", src, cpy);;
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_get_sub() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        let str = "Hello";
        let n = str.get_size;
        assert_eq(|_|"", str.get_sub(2, 4), "ll");;
        assert_eq(|_|"", str.get_sub(0, 0), "");;
        assert_eq(|_|"", str.get_sub(3, n+1), "lo");;
        assert_eq(|_|"", str.get_sub(1, n-1), "ell");;
    
        assert_eq(|_|"", "".get_sub(2, 4), "");;
    
        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_strip_first_spaces() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", "".strip_first_spaces, "");;
        assert_eq(|_|"", "Hello".strip_first_spaces, "Hello");;
        assert_eq(|_|"", " Hello".strip_first_spaces, "Hello");;
        assert_eq(|_|"", " \tHello".strip_first_spaces, "Hello");;
        assert_eq(|_|"", " ".strip_first_spaces, "");;
        assert_eq(|_|"", "  ".strip_first_spaces, "");;
    
        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_find() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"1", "abcdef".find("ab", 0), Option::some(0));;
        assert_eq(|_|"2", "abcdef".find("bc", 0), Option::some(1));;
        assert_eq(|_|"3", "abcdef".find("ef", 0), Option::some(4));;
        assert_eq(|_|"4", "abcdef".find("xyz", 0), Option::none());;
        assert_eq(|_|"5", "abcdef".find("", 0), Option::some(0));;
        assert_eq(|_|"6", "".find("xyz", 0), Option::none());;
        assert_eq(|_|"7", "".find("", 0), Option::some(0));;

        assert_eq(|_|"8", "abcdef".find("ab", 1), Option::none());;
        assert_eq(|_|"9", "abcdef".find("bc", 1), Option::some(1));;
        assert_eq(|_|"10", "abcdef".find("ef", 1), Option::some(4));;
        assert_eq(|_|"11", "abcdef".find("xyz", 1), Option::none());;
        assert_eq(|_|"12", "abcdef".find("", 1), Option::some(1));;
        assert_eq(|_|"13", "".find("xyz", 1), Option::none());;
        assert_eq(|_|"14", "".find("", 1), Option::some(0));;

        assert_eq(|_|"15", "abcdef".find("ab", 7), Option::none());;
        assert_eq(|_|"16", "abcdef".find("bc", 7), Option::none());;
        assert_eq(|_|"17", "abcdef".find("ef", 7), Option::none());;
        assert_eq(|_|"18", "abcdef".find("xyz", 7), Option::none());;
        assert_eq(|_|"19", "abcdef".find("", 7), Option::some(6));;
        assert_eq(|_|"20", "".find("xyz", 7), Option::none());;
        assert_eq(|_|"21", "".find("", 7), Option::some(0));;

        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_split() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (

        assert_eq(|_|"Ex. 1", "ab,c,".split(",").to_array, ["ab", "c", ""]);;
        assert_eq(|_|"Ex. 2", "abc".split(",").to_array, ["abc"]);;
        assert_eq(|_|"Ex. 3", "abc".split("").to_array, ["a", "b", "c"]);; // Special behavior when the separator is empty.

        assert_eq(|_|"1", "--ab---cde----".split("--").to_array, ["", "ab", "-cde", "", ""]);;
        assert_eq(|_|"2", "ab---cde----".split("--").to_array, ["ab", "-cde", "", ""]);;
        assert_eq(|_|"3", "--ab---cde".split("--").to_array, ["", "ab", "-cde"]);;
        assert_eq(|_|"3", "ab---cde".split("--").to_array, ["ab", "-cde"]);;
        assert_eq(|_|"4", "--".split("--").to_array, ["", ""]);;
        assert_eq(|_|"5", "a".split("--").to_array, ["a"]);;
        assert_eq(|_|"6", "".split("--").to_array, [""]);;

        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_ptr_to_string() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", nullptr.add_offset(3134905646).to_string, "00000000badadd2e");;
        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_less_than_and_less_than_or_eq() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        let s1 = "";
        let s2 = "a";
        let s3 = "ab";
        let s4 = "ac";
        let s5 = "b";

        let ss = [s1, s2, s3, s4, s5];

        let indices = do {
            let i = *Iterator::range(0, ss.get_size).to_dyn;
            let j = *Iterator::range(0, ss.get_size).to_dyn;
            pure $ (i, j)
        };
        indices.loop_iter_m((), |(i, j), _| (
            assert_eq(|_|"", ss.@(i) < ss.@(j), i < j);;
            assert_eq(|_|"", ss.@(i) <= ss.@(j), i <= j);;
            continue_m $ ()
        ));;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_from_u8() {
    let source = r##"
module Main;

main: IO ();
main = (
    assert_eq(|_|"", String::from_U8('a'), "a");;
    assert_eq(|_|"", String::from_U8('\x00'), "");;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_from_bytes_to_bytes() {
    let source = r##"
module Main;

main: IO ();
main = (
    let s = from_bytes(['a', 'b', 'c', '\0']);
    assert_eq(|_|"", s.as_ok, "abc");;
    assert_eq(|_|"", s.as_ok.to_bytes, ['a', 'b', 'c', '\0']);;

    let s = from_bytes(['\0']);
    assert_eq(|_|"", s.as_ok, "");;
    assert_eq(|_|"", s.as_ok.to_bytes, ['\0']);;

    let s = from_bytes(['a', '\0', 'b']);
    assert_eq(|_|"", s.as_ok, "a");;
    assert_eq(|_|"", s.as_ok.to_bytes, ['a', '\0']);;

    let s : Result ErrMsg String = from_bytes(['a', 'b', 'c']);
    assert(|_|"", s.is_err);;

    let s : Result ErrMsg String = from_bytes([]);
    assert(|_|"", s.is_err);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}
