use crate::{
    configuration::Configuration,
    tests::util::{test_source, test_source_fail},
};

#[test]
pub fn test_undefined_placeholder() {
    // Test using undefined as a placeholder
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            let x = 3;
            let a = if true { Array::fill(1, |_| x) } else { undefined("") };
            assert_eq(|_|"case 1", (a.@(0))(1), x);;
            let a = if true { |_| x } else { undefined("") };
            assert_eq(|_|"case 1", a(1), x);;
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_undefined_reached() {
    // Undefined reached.
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            eval undefined("Undefined reached") : I64;
            pure()
        );
    "#;
    let mut config = Configuration::compiler_develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Undefined reached");
}
