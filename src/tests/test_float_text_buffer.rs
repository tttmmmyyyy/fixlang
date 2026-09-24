// The eight values that write a `F32` or a `F64` as text hand a buffer to the C runtime. Each
// buffer's size is derived from the widest text the value can be asked for, and the derivation
// holds only if the widest digits, the widest exponent and the null terminator were all counted.
//
// A buffer short of that text stops the program: the runtime builds every text in a buffer of its
// own and hands its length to `fixruntime_copy_float_text`, which aborts where the text and its
// null do not fit before copying anything. So what this file does is write the widest text each of
// the eight can produce, which is what proves the check never fires — and an undersized buffer is
// caught by the abort wherever the tests run, rather than by the Valgrind this file also asks for.
//
// The run is under Valgrind all the same, because the check answers for the write into the buffer
// and Valgrind answers for everything around it: the `Array` the buffer lives in, the copy into
// it, and the `String` built from it.

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
    // The four that take no precision. The exponential two write the 6 places their format gives
    // by default; `to_string` writes the shortest digits, whose widest text comes next.
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
