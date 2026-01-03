use crate::{configuration::Configuration, tests::util::test_source_fail};

#[test]
pub fn test_get() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].@(3);
                pure()
            );
        "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Index out of range: index=3, size=3",
    );
}

#[test]
pub fn test_set() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].set(3, 42);
                pure()
            );
        "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Index out of range",
    );
}

#[test]
pub fn test_mod() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].mod(3, |_| 42);
                pure()
            );
        "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Index out of range",
    );
}

#[test]
pub fn test_act() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].act(3, |_| some(42));
                pure()
            );
        "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Index out of range",
    );
}

#[test]
pub fn test_index_syntax() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2][3].iget;
                pure()
            );
        "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Index out of range",
    );
}

#[test]
pub fn test_empty_negative_capacity() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval Array::empty(-1) : Array I64; 
                pure()
            );
        "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Negative array size or capacity: -1",
    );
}

#[test]
pub fn test_fill_negative_size() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval Array::fill(-1, 42);
                pure()
            );
        "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Negative array size or capacity",
    );
}
