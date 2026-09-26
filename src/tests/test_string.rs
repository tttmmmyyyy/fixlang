use crate::{configuration::Configuration, tests::test_util::test_source};

/// A string read back from the C string `borrow_c_str` lends is the string that was lent, so the
/// bytes a `String` keeps are a null-terminated C string.
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

/// `unsafe_from_c_str_ptr` and `unsafe_from_c_str_ptr_io` read back the string a C string holds,
/// the empty one included.
#[test]
pub fn test_string_unsafe_from_c_str_ptr_io_and_empty() {
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let src = "Hello World!";
            let cpy = *src.borrow_c_str_io(String::unsafe_from_c_str_ptr_io);
            assert_eq(|_|"io", cpy, src);;

            let cpy = "".borrow_c_str(String::unsafe_from_c_str_ptr);
            assert_eq(|_|"pure empty", cpy, "");;
            let cpy = *"".borrow_c_str_io(String::unsafe_from_c_str_ptr_io);
            assert_eq(|_|"io empty", cpy, "");;

            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// The substring between two indices, where the range covers part of the string, is empty, or
/// reaches past the end, and where the string itself is empty.
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

/// Leading spaces and tabs are removed and the rest is kept, so a string of nothing but spaces
/// becomes empty and one starting with another byte is unchanged.
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

/// Where a token first appears at or after the index searching starts from, including a token
/// that is not there, the empty token, the empty string, and a start index past the end.
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

/// The parts a separator splits a string into, including the empty parts its edges and its
/// repetitions leave, a separator that does not appear, and the empty separator, which cuts the
/// string into single bytes.
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

/// A pointer is written as sixteen hexadecimal digits, padded with leading zeros, each digit in
/// the place its value puts it.
#[test]
pub fn test_ptr_to_string() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", nullptr.add_offset(3134905646).to_string, "00000000badadd2e");;
        // Every hexadecimal digit, each in the place its value puts it: 0x0123456789abcdef.
        assert_eq(|_|"", nullptr.add_offset(81985529216486895).to_string, "0123456789abcdef");;
        pure()
    );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// `<` and `<=` order strings the way their positions in a sorted list do, so a prefix comes
/// before what extends it and a byte decides the order where two strings first differ.
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

/// A byte becomes the string of that one byte, and the null byte becomes the empty string, since
/// a `String` ends where its null is.
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

/// `from_bytes` reads the text a byte array holds up to its first null, and `to_bytes` answers
/// with those bytes and the null. An array carrying no null, and the empty array, are errors.
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

/// `from_bytes` reads the array it is given without consuming it, so the caller's array holds the
/// size and the bytes it held before the call.
#[test]
pub fn test_string_from_bytes_keeps_argument() {
    let source = r##"
module Main;

main: IO ();
main = (
    // The byte array is still owned by the caller, so `from_bytes` leaves it as it is.
    let arr = ['a', 'b', '\0', 'd', 'e'];
    let s : Result ErrMsg String = arr.from_bytes;
    assert_eq(|_|"string", s.as_ok, "ab");;
    assert_eq(|_|"size", arr.@size, 5);;
    assert_eq(|_|"elements", arr, ['a', 'b', '\0', 'd', 'e']);;

    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// Whether a string starts with a prefix, including the empty prefix, a prefix as long as the
/// string, one longer than it, and one differing only in case.
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

/// Whether a string ends with a suffix, including the empty suffix, a suffix as long as the
/// string, one longer than it, and one differing only in case.
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

/// Whether a substring appears anywhere in a string -- at either end or in the middle, more than
/// once, and in a case that differs -- and where either string is empty.
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

/// The bytes of a string in order, the null left out: one byte for each byte of the text, so a
/// character taking several bytes in UTF-8 comes out as the several bytes it takes.
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

    // A character taking several bytes in UTF-8
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

/// Writing through the bytes a string literal hands out leaves the literal as the source wrote it,
/// and the caller's array holds the write. A literal's bytes are a constant in read-only memory, so
/// the write goes to a copy.
#[test]
pub fn test_writing_through_a_literals_bytes_leaves_the_literal_alone() {
    let source = r#"
        module Main;

        main : IO ();
        main = (
            // A write whose only use of the array is the write itself: the path that updates a
            // uniquely held array in place.
            let written = "abc".get_bytes.set(0, 'x');
            assert_eq(|_|"the write landed on the copy", written.@(0), 'x');;
            assert_eq(|_|"the literal kept its first byte", "abc".get_bytes.@(0), 'a');;
            assert_eq(|_|"the literal reads as it was written", "abc", "abc");;

            // The same through the functorial write.
            assert_eq(|_|"the functorial write landed on the copy",
                "abc".get_bytes.mod(1, |_| 'Z').@(1), 'Z');;
            assert_eq(|_|"the literal kept its second byte", "abc".get_bytes.@(1), 'b');;

            // The empty literal, whose bytes are the null terminator alone.
            assert_eq(|_|"growing the empty literal's bytes",
                "".get_bytes.push_back('a').@size, 2);;
            assert_eq(|_|"the empty literal kept its terminator", "".get_bytes.@(0), '\0');;

            // A freshly built array takes the write in place and keeps it.
            let fresh = Array::fill(4, 'a').set(0, 'x');
            assert_eq(|_|"a freshly built array takes the write", fresh.@(0), 'x');;

            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// Every `Std::Array` primitive that writes, applied to the bytes a string literal hands out,
/// writes into a copy: the literal's storage is a constant in the program's data, so a write that
/// reached it would land in read-only memory. Each write below is the only use of the bytes it is
/// given, so the uniqueness check that makes the copy is one a shared array would fail.
#[test]
pub fn test_every_write_through_a_literals_bytes_lands_on_a_copy() {
    // Long enough that the storage is the aligned kind.
    let literal = "0123456789abcdefghijklmnopqrstuvwxyz".repeat(7) + "0123456789";
    let source = format!(
        r#"
        module Main;

        long : String;
        long = "{literal}";

        // Writes the byte 88 ('X') through the pointer the array lends.
        write_x : Ptr -> IO ();
        write_x = |p| FFI_CALL_IO[() fixruntime_u8_to_bytes(Ptr, U8), p, 88_U8];

        main : IO ();
        main = (
            let n = long.get_bytes.@size;
            assert_eq(|_|"swap", long.get_bytes.swap(0, n - 1).@(0), '\0');;
            assert_eq(|_|"truncate", long.get_bytes.truncate(3).@size, 3);;
            assert_eq(|_|"reserve", long.get_bytes.reserve(4 * n).@size, n);;
            assert_eq(|_|"append", long.get_bytes.append(long.get_bytes).@size, 2 * n);;
            assert_eq(|_|"sort", long.get_bytes.sort.@(0), '\0');;
            assert_eq(|_|"reverse", long.get_bytes.reverse.@(0), '\0');;
            assert_eq(|_|"resize", long.get_bytes.resize(n + 5, 'Z').@(n + 4), 'Z');;
            assert_eq(|_|"pop_back", long.get_bytes.pop_back.@size, n - 1);;
            assert_eq(|_|"get_sub", long.get_bytes.get_sub(0, 4).@size, 4);;
            assert_eq(|_|"dedup", long.get_bytes.dedup.@size, n);;
            let (written, _) = long.get_bytes.mutate_elements(write_x);
            assert_eq(|_|"the pointer write landed on the copy", written.@(0), 'X');;

            assert_eq(|_|"the literal kept its first byte", long.get_bytes.@(0), '0');;
            assert_eq(|_|"the literal kept its last byte", long.get_bytes.@(n - 2), '9');;
            assert_eq(|_|"the literal kept its terminator", long.get_bytes.@(n - 1), '\0');;
            assert_eq(|_|"the literal reads as it was written", long.get_sub(0, 10), "0123456789");;
            pure()
        );
        "#,
        literal = literal,
    );
    test_source(&source, Configuration::develop_mode());
}
