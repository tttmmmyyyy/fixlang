use crate::{configuration::Configuration, tests::test_util::test_source};

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
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_string_get_sub() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        let str = "Hello";
        let n = str.@size;
        assert_eq(|_|"", str.get_sub(2, 4), "ll");;
        assert_eq(|_|"", str.get_sub(0, 0), "");;
        assert_eq(|_|"", str.get_sub(3, n+1), "lo");;
        assert_eq(|_|"", str.get_sub(1, n-1), "ell");;
    
        assert_eq(|_|"", "".get_sub(2, 4), "");;
    
        pure()
    );
    "#;
    test_source(&source, Configuration::develop_mode());
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
    test_source(&source, Configuration::develop_mode());
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
    test_source(&source, Configuration::develop_mode());
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
    test_source(&source, Configuration::develop_mode());
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
    test_source(&source, Configuration::develop_mode());
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
            let i = *Iterator::range(0, ss.@size).to_dyn;
            let j = *Iterator::range(0, ss.@size).to_dyn;
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
    test_source(&source, Configuration::develop_mode());
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
    test_source(&source, Configuration::develop_mode());
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
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_string_starts_with() {
    let source = r##"
module Main;

main: IO ();
main = (
    // Basic tests
    assert(|_|"1", "Hello World".starts_with("Hello"));;
    assert(|_|"2", "Hello World".starts_with("H"));;
    assert(|_|"3", "Hello World".starts_with(""));;
    assert(|_|"4", !"Hello World".starts_with("World"));;
    assert(|_|"5", !"Hello World".starts_with("hello"));;
    
    // Edge cases
    assert(|_|"6", "".starts_with(""));;
    assert(|_|"7", !"".starts_with("Hello"));;
    assert(|_|"8", "abc".starts_with("abc"));;
    assert(|_|"9", !"abc".starts_with("abcd"));;
    
    // Longer prefix
    assert(|_|"10", "abcdefgh".starts_with("abcde"));;
    assert(|_|"11", !"abcdefgh".starts_with("abcdf"));;
    
    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_string_ends_with() {
    let source = r##"
module Main;

main: IO ();
main = (
    // Basic tests
    assert(|_|"1", "Hello World".ends_with("World"));;
    assert(|_|"2", "Hello World".ends_with("d"));;
    assert(|_|"3", "Hello World".ends_with(""));;
    assert(|_|"4", !"Hello World".ends_with("Hello"));;
    assert(|_|"5", !"Hello World".ends_with("world"));;
    
    // Edge cases
    assert(|_|"6", "".ends_with(""));;
    assert(|_|"7", !"".ends_with("Hello"));;
    assert(|_|"8", "abc".ends_with("abc"));;
    assert(|_|"9", !"abc".ends_with("zabc"));;
    
    // Longer suffix
    assert(|_|"10", "abcdefgh".ends_with("defgh"));;
    assert(|_|"11", !"abcdefgh".ends_with("xefgh"));;
    
    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_string_contains() {
    let source = r##"
module Main;

main : IO ();
main = (
    // Basic contains check
    assert(|_|"1", "Hello World".contains("World"));;
    assert(|_|"2", "Hello World".contains("Hello"));;
    assert(|_|"3", "Hello World".contains("o W"));;
    assert(|_|"4", !"Hello World".contains("Goodbye"));;
    assert(|_|"5", !"Hello World".contains("world"));; // Case sensitive
    
    // Edge cases
    assert(|_|"6", "Hello".contains(""));; // Empty string is contained
    assert(|_|"7", !"".contains("Hello"));; // Empty string doesn't contain non-empty
    assert(|_|"8", "".contains(""));; // Empty contains empty
    assert(|_|"9", "abc".contains("abc"));; // Full match
    assert(|_|"10", !"abc".contains("zabc"));; // Longer substring
    
    // Multiple occurrences
    assert(|_|"11", "abcabc".contains("abc"));;
    assert(|_|"12", "abcabc".contains("ca"));;
    
    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_string_to_iter_bytes() {
    let source = r##"
module Main;

main : IO ();
main = (
    // Basic iteration over bytes
    let bytes = "Hello".to_iter_bytes.to_array;
    assert_eq(|_|"1", bytes.@size, 5);;
    assert_eq(|_|"2", bytes.@(0), 72_U8);; // 'H'
    assert_eq(|_|"3", bytes.@(1), 101_U8);; // 'e'
    assert_eq(|_|"4", bytes.@(2), 108_U8);; // 'l'
    assert_eq(|_|"5", bytes.@(3), 108_U8);; // 'l'
    assert_eq(|_|"6", bytes.@(4), 111_U8);; // 'o'

    // Empty string
    let empty_bytes = "".to_iter_bytes.to_array;
    assert_eq(|_|"7", empty_bytes.@size, 0);;

    // Single byte
    let single = "A".to_iter_bytes.to_array;
    assert_eq(|_|"8", single.@size, 1);;
    assert_eq(|_|"9", single.@(0), 65_U8);; // 'A'

    // UTF-8 multi-byte character (こんにちは contains multi-byte characters)
    let utf8_str = "あ";  // U+3042, UTF-8: E3 81 82
    let utf8_bytes = utf8_str.to_iter_bytes.to_array;
    assert_eq(|_|"10", utf8_bytes.@size, 3);;
    assert_eq(|_|"11", utf8_bytes.@(0), 227_U8);; // 0xE3
    assert_eq(|_|"12", utf8_bytes.@(1), 129_U8);; // 0x81
    assert_eq(|_|"13", utf8_bytes.@(2), 130_U8);; // 0x82

    // Using iterator methods
    let count = "test".to_iter_bytes.fold(0, |_, acc| acc + 1);
    assert_eq(|_|"14", count, 4);;

    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}
