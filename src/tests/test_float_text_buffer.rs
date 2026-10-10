// `to_string` and the precision functions of `F32` and `F64` write their text into a buffer whose
// size is derived from the widest text they can be asked for, and the derivation holds only if the
// widest digits, the widest exponent and the null terminator were all counted. The writes are not
// checked against the buffer, so this file writes the widest text each function can produce under
// Valgrind, which reports a write past the buffer's allocation.

#[cfg(test)]
mod float_text_buffer_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    /// Writes the widest text each of the functions can produce -- the least value of each
    /// type, whose whole part is the widest, and the greatest negative one, whose exponent is the
    /// widest, at every precision the functions accept -- under Valgrind, so that a buffer sized
    /// short of that text shows up as a write past its allocation.
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
    // The least value of each type, whose whole part is the widest either type reaches, and the
    // greatest negative one, whose exponent is the widest, written to every precision the
    // functions accept. Each buffer is therefore filled to the width its size was derived for.
    let widest_whole_f32 = -3.4028235e38_F32;
    let widest_exponent_f32 = -1.4e-45_F32;
    let widest_whole_f64 = -1.7976931348623157e308;
    let widest_exponent_f64 = -5.0e-324;
    let total = range(0, 256).fold(0, |p, total|
        let prec = p.u8;
        total + widest_whole_f32.to_string_precision(prec).@size
              + widest_exponent_f32.to_string_exp_precision(prec).@size
              + widest_whole_f64.to_string_precision(prec).@size
              + widest_exponent_f64.to_string_exp_precision(prec).@size
    );
    // The four that take no precision. `to_string_exp` writes 6 places; `to_string` writes the
    // shortest digits, whose widest text comes next.
    let total = total + widest_whole_f32.to_string.@size
                      + widest_exponent_f32.to_string_exp.@size
                      + widest_whole_f64.to_string.@size
                      + widest_exponent_f64.to_string_exp.@size;
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
