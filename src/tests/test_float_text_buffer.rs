// The eight values that write a `F32` or a `F64` as text hand a buffer to the C runtime, which
// writes into it with no length to stop at. Each buffer's size is derived from the widest text its
// format can produce, and the derivation holds only if the widest whole part, the widest exponent
// and the null terminator were all counted. A buffer short by even one byte writes past its
// allocation, which the program itself cannot see: the overflowing bytes are readable afterwards,
// so the text still comes back whole. Valgrind is what sees it.
//
// How far the check reaches depends on the size of the buffer. A storage of
// `ARRAY_ALIGNED_ALLOC_THRESHOLD` bytes or more is allocated with `ARRAY_STORAGE_ALLOC_SLACK` bytes
// of room to slide the object onto the buffer alignment, and a write a byte past such a buffer
// lands in that room. `F64::to_string` and `F64::to_string_precision` always allocate more than the
// threshold, so an error of a byte in their bounds passes here and one of `ARRAY_BUF_ALIGNMENT`
// bytes or more is caught; for the other six, a byte is enough.

#[cfg(test)]
mod float_text_buffer_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    #[test]
    pub fn test_widest_text_fits_its_buffer() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let source = r#"
module Main;

main : IO () = (
    // The least value of each type, whose whole part and exponent are the widest either type
    // reaches, written to every precision the functions accept. Each buffer is therefore filled
    // to the width its size was derived for.
    let widest_f32 = -3.4028235e38_F32;
    let widest_f64 = -1.7976931348623157e308;
    let total = range(0, 256).fold(0, |p, total|
        let prec = p.u8;
        total + widest_f32.to_string_precision(prec).@size
              + widest_f32.to_string_exp_precision(prec).@size
              + widest_f64.to_string_precision(prec).@size
              + widest_f64.to_string_exp_precision(prec).@size
    );
    // The four that take no precision write the 6 places their format gives by default.
    let total = total + widest_f32.to_string.@size
                      + widest_f32.to_string_exp.@size
                      + widest_f64.to_string.@size
                      + widest_f64.to_string_exp.@size;
    assert_eq(|_|"the texts of every precision come to their known total", total, 224899);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(source, config);
    }
}
