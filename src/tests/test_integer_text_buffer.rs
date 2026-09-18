// The nine values that write an integer or a pointer as text hand a buffer to the C runtime. Each
// buffer's size is derived from the widest text the value can be asked for, and the runtime writes
// the digits into it without reading that size, so the derivation is the whole of what keeps the
// write inside the buffer.
//
// This file writes the widest text each of the nine can produce, under Valgrind, so that a write
// past the buffer shows up as an invalid write.

#[cfg(test)]
mod integer_text_buffer_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    /// Writes the widest text each of the nine functions can produce -- the greatest value of an
    /// unsigned type, the least of a signed one, and a pointer whose every digit is written --
    /// under Valgrind, so that a buffer sized short of that text shows up as a write past its
    /// allocation. Their lengths are summed and checked, so a text of the wrong width fails too.
    #[test]
    pub fn test_widest_integer_text_fits_its_buffer() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let source = r#"
module Main;

main : IO ();
main = (
    let total = U8::maximum.to_string.get_size
              + I8::minimum.to_string.get_size
              + U16::maximum.to_string.get_size
              + I16::minimum.to_string.get_size
              + U32::maximum.to_string.get_size
              + I32::minimum.to_string.get_size
              + U64::maximum.to_string.get_size
              + I64::minimum.to_string.get_size
              + nullptr.add_offset(I64::minimum).to_string.get_size;
    assert_eq(|_|"the widest texts come to their known total", total, 95);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(source, config);
    }
}
