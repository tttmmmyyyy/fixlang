use crate::{
    configuration::Configuration,
    tests::test_util::{runtime_checked_config, test_source, test_source_fail},
};

#[test]
pub fn test_assert_pass() {
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            assert(|_|"", true);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_assert_fail() {
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            assert(|_|"test_assert_fail", false);;
            pure()
        );
    "#;
    test_source_fail(&source, runtime_checked_config(), "test_assert_fail");
}
