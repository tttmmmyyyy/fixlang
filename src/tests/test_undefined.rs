use crate::{
    configuration::Configuration,
    tests::test_util::{test_source, test_source_fail},
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
    test_source(&source, Configuration::develop_mode());
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
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Undefined reached");
}

#[test]
pub fn test_undefined_reached_via_global_eval() {
    // `eval`-ing a global whose initializer aborts must run that initializer. A bare global
    // reference computes nothing of its own, so forcing it — the effect `eval` guarantees — is what
    // triggers the abort.
    let source = r#"
        module Main;

        g : I64;
        g = undefined("Undefined reached");

        main : IO ();
        main = (
            eval g;
            pure()
        );
    "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Undefined reached");
}
