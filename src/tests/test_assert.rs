use crate::{
    configuration::Configuration,
    tests::test_util::{test_source, test_source_fail},
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
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    // config.set_valgrind(ValgrindTool::None);
    test_source_fail(&source, config, "test_assert_fail");
}
