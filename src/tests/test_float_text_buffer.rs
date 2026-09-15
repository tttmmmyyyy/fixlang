// The eight values that write a `F32` or a `F64` as text hand a buffer to the C runtime. Each
// buffer's size is derived from the widest text the value can be asked for, and the derivation
// holds only if the widest digits, the widest exponent and the null terminator were all counted.
//
// The six that go through `snprintf` write into the buffer with no length to stop at, so a buffer
// short by even one byte writes past its allocation — which the program itself cannot see, since
// the overflowing bytes are readable afterwards and the text still comes back whole. Valgrind is
// what sees it. The two `to_string` values build the text in the runtime's own buffer and measure
// it before copying, so an undersized buffer stops the program instead; this file still writes
// their widest texts, which is what proves the measurement never fires.
//
// How far Valgrind's check reaches depends on the size of the buffer. A storage of
// `ARRAY_ALIGNED_ALLOC_THRESHOLD` bytes or more is allocated with `ARRAY_STORAGE_ALLOC_SLACK` bytes
// of room to slide the object onto the buffer alignment, and a write a byte past such a buffer
// lands in that room. `F64::to_string_precision` always allocates more than the threshold, so an
// error of a byte in its bounds passes here and one of `ARRAY_BUF_ALIGNMENT` bytes or more is
// caught. `F32::to_string_exp` and `F64::to_string_exp` always allocate less than the threshold, so
// a byte is enough for them. The remaining three, `F32::to_string_precision`,
// `F32::to_string_exp_precision` and `F64::to_string_exp_precision`, cross the threshold at the
// higher precisions, where a byte again lands in the slack; a byte in their bounds is caught at the
// lower precisions, where the same size constant is exercised with the storage under the threshold.

#[cfg(test)]
mod float_text_buffer_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    /// Writes the widest text each of the eight functions can produce -- the least value of each
    /// type, at every precision they accept -- under Valgrind, so that a buffer sized short of
    /// that text shows up as a write past its allocation.
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
    // The two exponential ones that take no precision write the 6 places their format gives by
    // default.
    let total = total + widest_f32.to_string.@size
                      + widest_f32.to_string_exp.@size
                      + widest_f64.to_string.@size
                      + widest_f64.to_string_exp.@size;
    // `to_string` writes the shortest digits, and its buffer is sized for the widest text those
    // reach: a number whose digits fill the type and whose point sits outside the window written
    // positionally for an `F64`, and one at the far edge of that window for an `F32`.
    let total = total + (-2.2250738585072014e-308).to_string.@size
                      + (-1.0e12_F32).to_string.@size;
    assert_eq(|_|"the texts of every precision come to their known total", total, 224611);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(source, config);
    }
}
