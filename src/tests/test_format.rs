// `String::format`, `IO::printf`, `IO::write_format` and the rest of the trait `Format`: each
// program compares the text they write with `to_string` or with a literal.

#[cfg(test)]
mod tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        tests::test_util::{deprecation_report, run_source_capture, test_source, test_source_fail},
    };

    /// The definitions the tests share: checks comparing with `to_string` and with `format`, a user
    /// type implementing `Format`, and a temporary file to write to.
    const HELPERS: &str = r##"// Checks that `format` writes `x` as `to_string` does, alone and after other text.
//
// # Parameters
// * `name` - The name of the case, shown when the check fails.
// * `x` - The value to write.
check_to_string : [a : Format, a : ToString] String -> a -> IO () = |name, x| (
    assert_eq(|_|name + " alone", "{}".format((x,)), x.to_string);;
    assert_eq(|_|name + " after text", "ab{}cd".format((x,)), "ab" + x.to_string + "cd")
);

type Point = struct { x : I64, y : F64 };

impl Point : Format {
    write_text = |p, out| (
        let out = *out.write_text("(");
        let out = *out.write_text(p.@x);
        let out = *out.write_text(", ");
        let out = *out.write_text(p.@y);
        out.write_text(")")
    );
}

type TwoTries a = struct { value : a };

// Writes the value to `out` twice, keeps the second result and drops the first. So `out` is shared
// when the second write starts.
impl [a : Format] TwoTries a : Format {
    write_text = |t, out| (
        let Identity { data : first } = out.write_text(t.@value).bind(write_text(" first"));
        let second = out.write_text(t.@value).bind(write_text(" second"));
        eval first;
        second
    );
}

// Runs `write` on a new temporary file, and returns what it wrote there.
//
// # Parameters
// * `write` - The action that writes to the file.
written_to_file : (IOHandle -> IO ()) -> IO String = |write| (
    let file = *FFI_CALL_IO[Ptr tmpfile()];
    let handle = *from_file_ptr(file);
    write(handle);;
    FFI_CALL_IO[() rewind(Ptr), file];;
    read_string(handle).try(|e| undefined(e))
);

// Checks that `_write_format` writes the text `format` makes of `values` and `template`, followed by
// `ending`.
//
// # Parameters
// * `name` - The name of the case, shown when the check fails.
// * `ending` - The text to write after the text of the template.
// * `values` - The values.
// * `template` - The template.
check_write_format : [args : FormatArgs] String -> String -> args -> String -> IO () = |name, ending, values, template| (
    let written = *written_to_file(|h| _write_format(h, ending, values, template));
    assert_eq(|_|name, written, template.format(values) + ending)
);"##;

    /// Builds and runs `source`, and checks that it succeeds and writes `stdout` to the standard
    /// output and `stderr` to the standard error. The program runs without valgrind, whose report
    /// would go to the standard error as well.
    fn assert_writes(source: &str, stdout: &str, stderr: &str) {
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::None);
        let output = run_source_capture(source, config);
        let written_stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let written_stderr = String::from_utf8_lossy(&output.stderr).to_string();
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            written_stdout,
            written_stderr
        );
        assert_eq!(written_stdout, stdout, "stderr: {}", written_stderr);
        assert_eq!(written_stderr, stderr);
    }

    /// Builds and runs a program made of `HELPERS` and `test`, whose `main` runs the value
    /// `test_name` that `test` defines.
    fn run_format_test(test: &str, test_name: &str) {
        let source = format!(
            "module Main;\n\n{}\n\n{}\n\nmain : IO () = {};\n",
            HELPERS, test, test_name
        );
        test_source(&source, Configuration::develop_mode());
    }

    /// Each `{}` is replaced with the next value, and `{{` and `}}` with a brace.
    #[test]
    fn test_placeholders() {
        run_format_test(
            r##"// Tests that `format` replaces each `{}` with the next value, and `{{` and `}}` with a brace. The
// cases put them at the start, in the middle and at the end, with zero, one and many values.
test_placeholders : IO () = (
    assert_eq(|_|"case 1", "".format(()), "");;
    assert_eq(|_|"case 2", "123".format(()), "123");;
    assert_eq(|_|"case 3", "{{".format(()), "{");;
    assert_eq(|_|"case 4", "123{{".format(()), "123{");;
    assert_eq(|_|"case 5", "{{123".format(()), "{123");;
    assert_eq(|_|"case 6", "}}".format(()), "}");;
    assert_eq(|_|"case 7", "123}}123".format(()), "123}123");;
    assert_eq(|_|"case 8", "{}".format(("123",)), "123");;
    assert_eq(|_|"case 9", "{}{}".format(("123", "456")), "123456");;
    assert_eq(|_|"case 10", " {} {} ".format(("123", "456")), " 123 456 ");;
    assert_eq(|_|"case 11", "{{{}}}".format((1,)), "{1}");;
    assert_eq(|_|"case 12", "{{}}".format(()), "{}");;
    assert_eq(|_|"case 13", "{} + {} = {}".format((1, 2.5, "three")), "1 + 2.5 = three");;
    assert_eq(|_|"case 14", "あ{}い{}う".format(("え", 1)), "あえい1う");;
    assert_eq(|_|"case 15", "{}{}{}{}{}{}{}{}{}{}{}{}".format((1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)), "123456789101112");;
    pure()
);"##,
            "test_placeholders",
        );
    }

    /// Each `Format` implementation of `Std`, the generated ones for tuples included, writes the same text as `to_string`.
    #[test]
    fn test_same_text_as_to_string() {
        run_format_test(
            r##"// Tests that each type with a `Format` implementation in `Std` writes the same text as
// `to_string`, including the smallest and largest values of each type.
test_same_text_as_to_string : IO () = (
    check_to_string("true", true);;
    check_to_string("false", false);;
    check_to_string("unit", ());;
    check_to_string("empty string", "");;
    check_to_string("string", "Hello, 世界");;
    check_to_string("U8 min", 0_U8);; check_to_string("U8 max", 255_U8);;
    check_to_string("I8 min", -128_I8);; check_to_string("I8 max", 127_I8);;
    check_to_string("U16 max", 65535_U16);;
    check_to_string("I16 min", -32768_I16);; check_to_string("I16 max", 32767_I16);;
    check_to_string("U32 max", 4294967295_U32);;
    check_to_string("I32 min", -2147483648_I32);; check_to_string("I32 max", 2147483647_I32);;
    check_to_string("U64 max", 18446744073709551615_U64);;
    check_to_string("I64 min", -9223372036854775808);; check_to_string("I64 max", 9223372036854775807);;
    check_to_string("I64 zero", 0);;
    check_to_string("U128 zero", 0_U128);; check_to_string("U128 max", U128::maximum);;
    check_to_string("U128 above U64", 18446744073709551616_U128);;
    check_to_string("U128 power of ten", 10000000000000000000_U128);;
    check_to_string("I128 min", I128::minimum);; check_to_string("I128 max", I128::maximum);;
    check_to_string("I128 below I64", -18446744073709551616_I128);;
    check_to_string("I128 small", -7_I128);;
    check_to_string("null pointer", nullptr);;
    check_to_string("F32 zero", 0.0_F32);; check_to_string("F32 negative zero", -0.0_F32);;
    check_to_string("F32 tenth", 0.1_F32);; check_to_string("F32 least", -3.4028235e38_F32);;
    check_to_string("F32 large", 1.0e13_F32);; check_to_string("F32 small", 1.0e-10_F32);;
    check_to_string("F32 infinity", F32::infinity);; check_to_string("F32 nan", F32::quiet_nan);;
    check_to_string("F64 tenth", 0.1);; check_to_string("F64 least", -1.7976931348623157e308);;
    check_to_string("F64 subnormal", -2.2250738585072014e-308);; check_to_string("F64 large", 1.0e300);;
    check_to_string("F64 negative infinity", -F64::infinity);; check_to_string("F64 nan", F64::quiet_nan);;
    check_to_string("some", Option::some(5));;
    check_to_string("none", Option::none() : Option I64);;
    check_to_string("ok", Result::ok(1) : Result String I64);;
    check_to_string("err", Result::err("e") : Result String I64);;
    check_to_string("empty array", [] : Array I64);;
    check_to_string("array", [1, 2, 3]);;
    check_to_string("nested", [Option::some((1, "a")), Option::none()]);;
    check_to_string("tuple 1", (7,));;
    check_to_string("tuple 2", (1, "a"));;
    check_to_string("tuple 5", (1, 2.5, "c", true, [()]));;
    check_to_string("tuple 12", (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12));;
    check_to_string("tuple 13", (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, "13"));;
    check_to_string("tuple 20", (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, [20]));;
    pure()
);"##,
            "test_same_text_as_to_string",
        );
    }

    /// `with_precision` writes the same text as `to_string_precision` at every precision.
    #[test]
    fn test_with_precision() {
        run_format_test(
            r##"// Tests that `with_precision` writes the same text as `to_string_precision`, for `F32` and `F64`,
// at every precision, including values with the longest text.
test_with_precision : IO () = (
    let f32s = [-3.4028235e38_F32, 3.14159_F32, -0.0_F32, 1.0e-10_F32];
    let f64s = [-1.7976931348623157e308, 3.14159, -0.0, 1.0e-300];
    range(0, 256).fold_m((), |p, _| (
        let prec = p.u8;
        f32s.to_iter.fold_m((), |v, _|
            assert_eq(|_|"F32 " + v.to_string + " " + p.to_string, "[{}]".format((v.with_precision(prec),)), "[" + v.to_string_precision(prec) + "]")
        );;
        f64s.to_iter.fold_m((), |v, _|
            assert_eq(|_|"F64 " + v.to_string + " " + p.to_string, "[{}]".format((v.with_precision(prec),)), "[" + v.to_string_precision(prec) + "]")
        )
    ))
);"##,
            "test_with_precision",
        );
    }

    /// `with_exp_precision` writes the same text as `to_string_exp_precision` at every precision.
    #[test]
    fn test_with_exp_precision() {
        run_format_test(
            r##"// Tests that `with_exp_precision` writes the same text as `to_string_exp_precision`, for `F32` and
// `F64`, at every precision, including the largest and the subnormal values, and those that are not
// finite.
test_with_exp_precision : IO () = (
    let f32s = [-3.4028235e38_F32, 123.456_F32, -0.0_F32, 1.0e-45_F32, F32::infinity, F32::quiet_nan];
    let f64s = [-1.7976931348623157e308, 123.456, -0.0, 5.0e-324, -F64::infinity, F64::quiet_nan];
    range(0, 256).fold_m((), |p, _| (
        let prec = p.u8;
        f32s.to_iter.fold_m((), |v, _|
            assert_eq(|_|"F32 " + v.to_string + " " + p.to_string, "[{}]".format((v.with_exp_precision(prec),)), "[" + v.to_string_exp_precision(prec) + "]")
        );;
        f64s.to_iter.fold_m((), |v, _|
            assert_eq(|_|"F64 " + v.to_string + " " + p.to_string, "[{}]".format((v.with_exp_precision(prec),)), "[" + v.to_string_exp_precision(prec) + "]")
        )
    ))
);"##,
            "test_with_exp_precision",
        );
    }

    /// `joined_with` writes the separator between the elements, for zero, one and many elements.
    #[test]
    fn test_joined_with() {
        run_format_test(
            r##"// Tests that `joined_with` writes the separator between elements, for zero, one and many
// elements, and for an empty separator.
test_joined_with : IO () = (
    assert_eq(|_|"none", "[{}]".format((([] : Array I64).to_iter.joined_with(", "),)), "[]");;
    assert_eq(|_|"one", "[{}]".format(([1].to_iter.joined_with(", "),)), "[1]");;
    assert_eq(|_|"several", "[{}]".format(([1, 2, 3].to_iter.joined_with(" | "),)), "[1 | 2 | 3]");;
    assert_eq(|_|"empty separator", "[{}]".format((["a", "b"].to_iter.joined_with(""),)), "[ab]");;
    assert_eq(|_|"mapped range", "{}".format((range(1, 4).map(|i| i * i).joined_with(", "),)), "1, 4, 9");;
    // Ten thousand numbers make the text outgrow its capacity many times.
    let many = Array::from_map(10000, |i| i);
    assert_eq(|_|"many", "{}".format((many.to_iter.joined_with(","),)), many.to_iter.map(to_string).join(","));;
    pure()
);"##,
            "test_joined_with",
        );
    }

    /// `right_aligned_to` and `left_aligned_to` pad the text to the width in code points.
    #[test]
    fn test_aligned() {
        run_format_test(
            r##"// Tests that `right_aligned_to` and `left_aligned_to` pad the text to the width in code points. A
// shorter text is padded, and a text of the width or longer is unchanged. Multibyte characters count
// as one each, and the value can be another wrapper. Some cases write other text first, to check
// that the padding goes where the value starts.
test_aligned : IO () = (
    assert_eq(|_|"right narrower", "ab[{}]".format((42.right_aligned_to(5),)), "ab[   42]");;
    assert_eq(|_|"right as wide", "[{}]".format((12345.right_aligned_to(5),)), "[12345]");;
    assert_eq(|_|"right wider", "[{}]".format(("toolong".right_aligned_to(3),)), "[toolong]");;
    assert_eq(|_|"right zero width", "[{}]".format(("x".right_aligned_to(0),)), "[x]");;
    assert_eq(|_|"right multibyte", "[{}]".format(("あい".right_aligned_to(4),)), "[  あい]");;
    assert_eq(|_|"right empty", "[{}]".format(("".right_aligned_to(2),)), "[  ]");;
    assert_eq(|_|"right wrapper", "[{}]".format((3.14159.with_precision(2_U8).right_aligned_to(6),)), "[  3.14]");;
    assert_eq(|_|"left narrower", "ab[{}]".format((42.left_aligned_to(5),)), "ab[42   ]");;
    assert_eq(|_|"left wider", "[{}]".format(("toolong".left_aligned_to(3),)), "[toolong]");;
    assert_eq(|_|"left multibyte", "[{}]".format(("あい".left_aligned_to(4),)), "[あい  ]");;
    assert_eq(|_|"two in a row", "{}{}".format((1.right_aligned_to(3), 2.left_aligned_to(3))), "  12  ");;
    pure()
);"##,
            "test_aligned",
        );
    }

    /// `zero_padded_to` pads the text with `0` after a leading `-`.
    #[test]
    fn test_zero_padded() {
        run_format_test(
            r##"// Tests that `zero_padded_to` pads the text with `0` to the width in code points, after the `-` a
// negative number starts with. A text of the width or longer is unchanged, and the value can be
// another wrapper or a value whose text is not a number. Some cases write other text first, to check
// that the padding goes where the value starts.
test_zero_padded : IO () = (
    assert_eq(|_|"narrower", "ab[{}]".format((9.zero_padded_to(3),)), "ab[009]");;
    assert_eq(|_|"as wide", "[{}]".format((123.zero_padded_to(3),)), "[123]");;
    assert_eq(|_|"wider", "[{}]".format((12345.zero_padded_to(3),)), "[12345]");;
    assert_eq(|_|"negative", "ab[{}]".format(((-5).zero_padded_to(4),)), "ab[-005]");;
    assert_eq(|_|"negative as wide", "[{}]".format(((-5).zero_padded_to(2),)), "[-5]");;
    assert_eq(|_|"I128 min", "{}".format((I128::minimum.zero_padded_to(42),)), "-00170141183460469231731687303715884105728");;
    assert_eq(|_|"float", "[{}]".format(((-3.5).zero_padded_to(6),)), "[-003.5]");;
    assert_eq(|_|"wrapper", "[{}]".format((3.14159.with_precision(2_U8).zero_padded_to(7),)), "[0003.14]");;
    assert_eq(|_|"string", "[{}]".format(("ab".zero_padded_to(4),)), "[00ab]");;
    assert_eq(|_|"empty", "[{}]".format(("".zero_padded_to(2),)), "[00]");;
    assert_eq(|_|"inside right_aligned_to", "[{}]".format((7.zero_padded_to(3).right_aligned_to(5),)), "[  007]");;
    pure()
);"##,
            "test_zero_padded",
        );
    }

    /// A user type that implements `Format` is written by its parts, also inside an array.
    #[test]
    fn test_user_type() {
        run_format_test(
            r##"// Tests that `format` writes a user type that implements `Format` by writing its parts, also inside
// an array.
test_user_type : IO () = (
    let p = Point { x : 3, y : 0.5 };
    assert_eq(|_|"alone", "p = {}".format((p,)), "p = (3, 0.5)");;
    assert_eq(|_|"in an array", "{}".format(([p, p],)), "[(3, 0.5), (3, 0.5)]");;
    pure()
);"##,
            "test_user_type",
        );
    }

    /// A shared `TextOut` is copied before a value is written to it.
    #[test]
    fn test_shared_out() {
        run_format_test(
            r##"// Tests that a shared `TextOut` is copied before a value is written to it, so each copy has only
// its own text.
test_shared_out : IO () = (
    assert_eq(|_|"shared", "[{}]".format((TwoTries { value : 42 },)), "[42 second]");;
    assert_eq(|_|"shared float", "[{}]".format(((TwoTries { value : 7 }, 2.5),)), "[(7 second, 2.5)]");;
    pure()
);"##,
            "test_shared_out",
        );
    }

    /// `printf` and the others write the text `format` makes, also where the text is written out along the way.
    #[test]
    fn test_write_format() {
        run_format_test(
            r##"// Tests that `_write_format`, which `printf` and the others call, writes the same text as
// `format` followed by the ending, also where the text is long enough to be written out many times
// along the way, even in the middle of a value, and that the text goes in order with what is
// written to the handle before and after it.
test_write_format : IO () = (
    check_write_format("no ending", "", (1, "two"), "{} and {}");;
    check_write_format("newline", "\n", (1, "two"), "{} and {}");;
    check_write_format("no placeholders", "\n", (), "abc");;
    let many = Array::from_map(10000, |i| i);
    check_write_format("long value", "", (many,), "<{}>");;
    let nested = Array::from_map(100, |i| Array::from_map(i * 10, |j| (j, some("x"))));
    check_write_format("nested values", "\n", (1, nested, many.to_iter.joined_with(" ")), "{} {} {}");;
    check_write_format("aligned long value", "", (many.right_aligned_to(5),), "{}");;
    let written = *written_to_file(|h|
        write_string(h, "before ").try(|e| undefined(e));;
        _write_format(h, "", (many,), "{}");;
        write_string(h, " after").try(|e| undefined(e))
    );
    assert_eq(|_|"in order", written, "before " + "{}".format((many,)) + " after");;
    pure()
);"##,
            "test_write_format",
        );
    }

    /// `write_format` writes the text `format` makes, and returns the error of a write that fails.
    #[test]
    fn test_write_format_to_handle() {
        run_format_test(
            r##"// Tests that `write_format` writes the same text as `format`, also where the text is long enough to
// be written out many times along the way, and that it returns the error of a write that fails,
// both for a short text and for one written out along the way.
test_write_format_to_handle : IO () = (
    let many = Array::from_map(10000, |i| i);
    let written = *written_to_file(|h| write_format(h, (1, "two"), "{} and {}").try(|e| undefined(e)));
    assert_eq(|_|"short", written, "1 and two");;
    let written = *written_to_file(|h| write_format(h, (many,), "<{}>").try(|e| undefined(e)));
    assert_eq(|_|"long", written, "<{}>".format((many,)));;
    let short = *write_format(stdin, (1,), "{}").to_result;
    assert_eq(|_|"short: the error is returned", short.is_err, true);;
    let long = *write_format(stdin, (many,), "{}").to_result;
    assert_eq(|_|"long: the error is returned", long.is_err, true);;
    pure()
);"##,
            "test_write_format_to_handle",
        );
    }

    /// `flush_if_full` of the writer `printf` runs in writes out a text longer than
    /// `_flush_threshold`, and that of the writer `write_format` runs in returns the error of a write
    /// that fails.
    #[test]
    fn test_flush_if_full() {
        run_format_test(
            r##"// Tests that `flush_if_full` of `_HandleWriter IO` keeps a text of up to `_flush_threshold`
// bytes, and writes out a longer one and returns it emptied, and that `flush_if_full` of
// `_HandleWriter IOFail` returns the error of a write that fails.
test_flush_if_full : IO () = (
    let short = TextOut { _bytes : Array::fill(_flush_threshold, 'a') };
    let written = *written_to_file(|h|
        let out = *((short.flush_if_full : _HandleWriter IO TextOut).@_run)(h);
        assert_eq(|_|"short: kept", out.@_bytes.@size, _flush_threshold)
    );
    assert_eq(|_|"short: not written", written, "");;
    let long = TextOut { _bytes : Array::fill(_flush_threshold + 1, 'a') };
    let written = *written_to_file(|h|
        let out = *((long.flush_if_full : _HandleWriter IO TextOut).@_run)(h);
        assert_eq(|_|"long: emptied", out.@_bytes.@size, 0)
    );
    assert_eq(|_|"long: written", written.@size, _flush_threshold + 1);;
    // The writer of `write_format` returns the error of a write that fails.
    let failed = *((long.flush_if_full : _HandleWriter IOFail TextOut).@_run)(stdin).to_result;
    assert_eq(|_|"long: the error is returned", failed.is_err, true);;
    pure()
);"##,
            "test_flush_if_full",
        );
    }

    /// A template with fewer placeholders than values stops the program with a message naming the
    /// template.
    #[test]
    fn test_format_with_too_many_values() {
        let source = r##"
        module Main;
        main : IO () = println("{} and".format((1, 2)));
        "##;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "[Std::String::format] The template has fewer placeholders than the values: \"{} and\"",
        );
    }

    /// A template with more placeholders than values stops the program with a message naming the
    /// template.
    #[test]
    fn test_format_with_too_few_values() {
        let source = r##"
        module Main;
        main : IO () = println("{} and {}".format((1,)));
        "##;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "[Std::String::format] The template has more placeholders than the values: \"{} and {}\"",
        );
    }

    /// An opening brace at the end of a template, with nothing after it to close it, stops the
    /// program with a message naming the brace and the template.
    #[test]
    fn test_format_with_an_unclosed_brace() {
        let source = r##"
        module Main;
        main : IO () = println("a {".format(()));
        "##;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "[Std::String::format] The template has a \"{\" that is neither a placeholder nor an escape: \"a {\"",
        );
    }

    /// A brace that is neither part of a placeholder nor an escape stops the program with a message
    /// naming the brace and the template.
    #[test]
    fn test_format_with_a_stray_brace() {
        let source = r##"
        module Main;
        main : IO () = println("a } b".format(()));
        "##;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "[Std::String::format] The template has a \"}\" that is neither a placeholder nor an escape: \"a } b\"",
        );
    }

    /// `format` takes a tuple of thirteen or twenty values, since the implementation of
    /// `FormatArgs` is generated for every tuple size a program uses.
    #[test]
    fn test_thirteen_and_twenty_values() {
        run_format_test(
            r##"// Tests that `format` takes a tuple of thirteen or twenty values, of different types.
test_thirteen_and_twenty_values : IO () = (
    assert_eq(|_|"13 values", "{}{}{}{}{}{}{}{}{}{}{}{}{}".format((1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, "13")), "12345678910111213");;
    assert_eq(|_|"20 values", "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}".format((1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, [20])), "12345678910111213141516171819[20]");;
    pure()
);"##,
            "test_thirteen_and_twenty_values",
        );
    }

    /// `printf` and `printfln` write to the standard output, `eprintf` and `eprintfln` to the
    /// standard error, and the `ln` ones add a newline.
    #[test]
    fn test_printf_streams() {
        let source = r##"
module Main;

main : IO () = (
    "a{}|".printf((1,));;
    "b{}".printfln((2,));;
    "c{}|".eprintf((3,));;
    "d{}".eprintfln((4,));;
    pure()
);
"##;
        assert_writes(source, "a1|b2\n", "c3|d4\n");
    }

    /// `debug_printf` and `debug_printfln` write to the standard output, `debug_eprintf` and
    /// `debug_eprintfln` to the standard error, and the `ln` ones add a newline.
    #[test]
    fn test_debug_printf_streams() {
        let source = r##"
module Main;

main : IO () = (
    eval "a{}|".debug_printf((1,));
    eval "b{}".debug_printfln((2,));
    eval "c{}|".debug_eprintf((3,));
    eval "d{}".debug_eprintfln((4,));
    pure()
);
"##;
        assert_writes(source, "a1|b2\n", "c3|d4\n");
    }

    /// Naming `String::populate` is reported as deprecated, with the message its `DEPRECATED`
    /// pragma carries pointing at `format`.
    #[test]
    fn test_populate_is_deprecated() {
        let source = r#"
module Main;

main : IO () = println("{}".populate(["x"]));
"#;
        let report = deprecation_report(source);
        assert!(
            report.contains("Use `Std::String::format` instead."),
            "naming `populate` should be reported with the message its pragma carries:\n{}",
            report,
        );
    }
    /// A width that the text is longer than writes the text unchanged, the least `I64` included.
    #[test]
    fn test_width_at_least_i64() {
        run_format_test(
            r##"// Tests that a width at or near `I64::minimum`, which every text is wider than, writes the text
// unchanged.
test_width_at_least_i64 : IO () = (
    let n = (*IO::get_args).@size.i64;
    let w = I64::minimum + n - 1;
    assert_eq(|_|"right_aligned_to", "[{}]".format((42.right_aligned_to(w),)), "[42]");;
    assert_eq(|_|"left_aligned_to", "[{}]".format((42.left_aligned_to(w),)), "[42]");;
    assert_eq(|_|"zero_padded_to", "[{}]".format((42.zero_padded_to(w),)), "[42]");;
    assert_eq(|_|"right_aligned_to, one above", "[{}]".format((42.right_aligned_to(w + 1),)), "[42]");;
    pure()
);"##,
            "test_width_at_least_i64",
        );
    }

    /// A width too large for any array to hold, given to `right_aligned_to`, `left_aligned_to` or
    /// `zero_padded_to`, stops the program with a message saying so.
    #[test]
    fn test_width_too_large_for_an_array() {
        for wrapper in ["right_aligned_to", "left_aligned_to", "zero_padded_to"] {
            let source = format!(
                r##"
        module Main;
        main : IO () = println("[{{}}]".format((42.{}(I64::maximum),)));
        "##,
                wrapper
            );
            test_source_fail(
                &source,
                Configuration::develop_mode(),
                "[Std::Format] The text is too long for an array to hold.",
            );
        }
    }
}
