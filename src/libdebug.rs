use super::*;

const SOURCE: &str = include_str!("libdebug.fix");

pub fn make_debug_mod() -> FixModule {
    let mut fix_module = parse_source(SOURCE);

    fix_module.add_global_value(
        FullName::from_strs(&[DEBUG_NAME], "abort"),
        abort_function(),
    );

    fix_module
}
