use std::{
    fs::{self, remove_file, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    misc::{function_name, number_to_varname, split_by_max_size},
    run, test_source, test_source_fail, Configuration, SubCommand, COMPILER_TEST_WORKING_PATH,
    I16_NAME, I32_NAME, I64_NAME, I8_NAME, U16_NAME, U32_NAME, U64_NAME, U8_NAME,
};
use rand::Rng;

#[test]
pub fn test0() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                assert_eq(|_|"case 1", 5 + 3 * 8 / 5 + 7 % 3, 1e1_I64);;
                pure()
            );
        "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_if_semicolon_in_let() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                let x = if true { 1 }; 2; // First semicolon is for `if`, and second semicolon is for `let`.
                assert_eq(|_|"case 1", x, 1);;
                pure()
            );
        "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test1() {
    let source = r#"
            module Main; 
                        
            main : IO ();
            main = (
                assert_eq(|_|"", let x = 5 in -x, -5);;
                pure()
            );
        "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test2() {
    let source = r#"
            module Main; 
            
            main : IO ();
            main = (
                assert_eq(|_|"", let x = 5 in 3, 3);;
                pure()
            );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test3() {
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            assert_eq(|_|"", let n = -5 in let p = 5 in n, -5);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test4() {
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            assert_eq(|_|"", let n = -5 in let p = 5 in p, 5);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test5() {
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            assert_eq(|_|"", let x = -5 in let x = 5 in x, 5);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test6() {
    let source = r#"
        module Main;
        
        main : IO ();
        main = (
            assert_eq(|_|"", let x = let y = 3 in y in x, 3);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test7() {
    let source = r#"
        module Main;         
        main : IO ();
        main = (
            assert_eq(|_|"", (|x| 5)(10), 5);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test8() {
    let source = r#"
        module Main;

        main : IO ();
        main = (
            assert_eq(|_|"", (|x| x) $ 6, 6);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test9_5() {
    let source = r#"
        module Main;         
        main : IO ();
        main = (
            let x = 3;
            let y = 5;
            assert_eq(|_|"", x - y, -2);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test10() {
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"", let x = 5 in 2 + x, 7);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test11() {
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let x = 5 in 
            let y = -3 in
            assert_eq(|_|"", x + y, 2);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test12() {
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let x = 5 in 
            let y = -3 in
            let z = 12 in
            let xy = x + y in
            assert_eq(|_|"", xy + z, 14);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test13() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            let f = add(5) in
            assert_eq(|_|"", f(3), 5+3);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test13_5() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            let f = add(5) in
            assert_eq(|_|"", f(-3) + f(12), 5 - 3 + 5 + 12);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test14() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            let x = 3 in 
            let y = 5 in
            let f = add(x) in
            assert_eq(|_|"", f(y), 3 + 5);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test15() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            let f = |x| 3 + x in
            assert_eq(|_|"", f(5), 3 + 5);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test15_5() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            let x = 3;
            let f = |y| x;
            assert_eq(|_|"", f(5), 3);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test16() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            let f = |x| x + 3 in
            assert_eq(|_|"", f(5), 3 + 5);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test17() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            assert_eq(|_|"", if true { 3 } else { 5 }, 3);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test18() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            assert_eq(|_|"", if false { 3 } else { 5 }, 5);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test19() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            assert_eq(|_|"", if 3 == 3 { 1 } else { 0 }, 1);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test20() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            assert_eq(|_|"", if 3 == 5 { 1 } else { 0 }, 0);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test20_5() {
    let source = r#"
        module Main;         main : IO ();
        main = (
            let ans = (
                if 2 == 0 {
                    0 
                } else if 2 == 1 {
                    1
                } else { 
                    2 
                }
            );
            assert_eq(|_|"", ans, 2);;
            pure ()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test21() {
    let source = r#"
            module Main; 
            main : IO ();
            main = (
                let fact = fix $ |loop, n| if n == 0 { 1 } else { n * loop(n-1) };
                assert_eq(|_|"", fact(5), 5 * 4 * 3 * 2 * 1);;
                pure()
            );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_fix_direct_call() {
    let source = r#"
            module Main; 
            main : IO ();
            main = (
                let fact5 = fix(|loop, n| if n == 0 { 1 } else { n * loop(n-1) }, 5);
                assert_eq(|_|"", fact5, 5 * 4 * 3 * 2 * 1);;
                pure()
            );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test22() {
    // Test recursion function defined by fix with two variables that is tail call.
    let n: i64 = 1000000;
    let source = format!(
        r#"
            module Main;
            main : IO ();
            main = (
                let g = fix $ |loop, a, x|
                            if x == 0 {{ 
                                a 
                            }} else {{
                                let a2 = a + x;
                                let x2 = x + -1;
                                loop(a2, x2)
                            }}
                in 
                    assert_eq(|_|"", g(0, {}), {});;
                    pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    test_source(
        source.as_str(),
        Configuration::release_mode(SubCommand::Run),
    );
}

#[test]
pub fn test22_5() {
    // Test recursion function defined by fix that is not tail-call.
    let source = r#"
        module Main;         main : IO ();
        main = (
            let fib = fix $ |f, n|
                        if n == 0 {
                            0
                        } else if n == 1 {
                            1
                        } else {
                            f(n+-1) + f(n+-2)
                        }
            in 
                assert_eq(|_|"", fib(10), 55);;
                pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test22_7() {
    // Test global recursion function
    let source = r#"
        module Main; 
        fib : I64 -> I64;
        fib = |n| (
            if n == 0 {
                0
            } else if n == 1 {
                1
            } else {
                fib(n-1) + fib(n-2)
            }
        );
        
        main : IO ();
        main = (
            assert_eq(|_|"", fib(30), 832040);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test23() {
    // Test Array::fill of size 0.
    let source = r#"
        module Main;
        main : IO ();
        main = (
            eval Array::fill(0, 42);
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test24() {
    // Test Array::fill of size > 0.
    let source = r#"
        module Main;         main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            assert_eq(|_|"", arr.get_size, 100);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test25() {
    // Test Array::get.
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let elem = arr.@(50);
            assert_eq(|_|"", elem, 42);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test26() {
    // Test Array::set (unique case).
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let arr = arr.set(50, 21);
            assert_eq(|_|"", arr.@(50), 21);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test27() {
    // Test Array::set (shared case).
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr0 = Array::fill(100, 42);
            let arr1 = arr0.set(50, 21);
            assert_eq(|_|"", arr0.@(50) + arr1.@(50), 63);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test27_5() {
    // Test Array of boxed object.
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::from_map(100) $ |i| add(i);
            let arr = arr.set(99, |x| x - 100);
            assert_eq(|_|"", arr.@(99) $ arr.@(50) $ 1, 1 + 50 - 100);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test28() {
    // Calculate Fibonacci sequence using array.
    let source = r#"
        module Main;
        main : IO ();
        
        main = (
            let arr = Array::fill(31, 0);
            let arr = arr.assert_unique(|_|"The array is not unique!").set(0, 0);
            let arr = arr.assert_unique(|_|"The array is not unique!").set(1, 1);
            let loop = fix $ |f, arr: Array I64, n| (
                if n == 31 {
                    arr
                } else {
                    let x = arr.@(add(n, -1));
                    let y = arr.@(add(n, -2));
                    let arr = arr.assert_unique(|_|"The array is not unique!").set(n, x+y);
                    f(arr, n+1)
                }
            );
            let fib = loop(arr, 2);
            assert_eq(|_|"", fib.@(30), 832040);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test29() {
    let source = r#"
        module Main; 
        id : a -> a;
        id = |x| x;

        main : IO ();
        main = (
            assert_eq(|_|"", if id(true) { id(100) } else { 30 }, 100);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test30() {
    // Test dollar combinator
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = |x| x + 3;
            let g = |x| x == 8;
            let ans = g $ f $ 5;
            assert_eq(|_|"", if ans { 1 } else { 0 }, 1);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test31() {
    // Test . combinator
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = |x| x + 3;
            let g = |x| x == 8;
            let ans = 5 .f. g;
            assert_eq(|_|"", if ans { 1 } else { 0 } , 1);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test32() {
    // Test . and $ combinator
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = |x| x + 10;
            assert_eq(|_|"", 5.add $ 3.f, 18);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test33() {
    // Test struct declaration and new, mod.
    let source = r#"
        module Main;
        type I64Bool = box struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool { x: 18, y: false };
            let obj = I64Bool::mod_x(|x| x + 42, obj);
            assert_eq(|_|"", I64Bool::@x(obj), 60);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test34_5() {
    // Test unboxed struct declaration and new, mod.
    let source = r#"
        module Main;
        type I64Bool = unbox struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool { x: 18, y : false};
            let obj = I64Bool::mod_x(|x| x + 42, obj);
            assert_eq(|_|"", I64Bool::@x(obj), 60);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test34() {
    // Test namespace inference.
    let source = r#"
        module Main;
        
        type OtherStruct = box struct {y: I64, x: Bool};
        type I64Bool = box struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool {x: 18, y: false};
            let obj = obj.mod_x(|x| x + 42);
            assert_eq(|_|"", obj.@x, 60);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test35() {
    // Test overloading resolution.
    let source = r#"
        module Main; 
        type A = box struct {x: I64, y: Bool};
        type B = box struct {x: Bool, y: I64};
            
        main : IO ();
        main = (
            let a = A {x: 3, y: true};
            let b = B {x: true, y: 5};
            let ans = add(if a.@y { a.@x } else { 0 }, if b.@x { b.@y } else { 0 });
            assert_eq(|_|"", ans, 8);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test36() {
    // Test modifier composition.
    let source = r#"
        module Main; 
        type A = box struct {x: B};
        type B = box struct {x: I64};
            
        main : IO ();
        main = (
            let a = A{x: B{x: 16}};
            let a = a.(mod_x $ mod_x $ |x| x + 15);
            let ans = a . @x . @x;
            assert_eq(|_|"", ans, 31);;
            pure ()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test37_5() {
    // Test shared modField.
    let source = r#"
        module Main; 
        type A = box struct {x: B};
        type B = box struct {x: I64};

        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let b = a.(mod_x $ mod_x $ |x| x + 15);
            let ans = a.@x.@x + b.@x.@x;
            assert_eq(|_|"", ans, (16 + 15) + 16);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_annotation() {
    // Test type annotation.
    let source = r#"
        module Main; 
        type A = box struct {x: B};
        type B = box struct {x: I64};

        main : IO ();
        main = (    
            let a = A {x: B {x: 16}};
            let f = |a| (a : A) . (mod_x $ mod_x $ |x| x + 15);
            let a = a.f;
            let ans = a.@x.@x;
            assert_eq(|_|"", ans, 31);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_annotation_2() {
    // Test type annotation.
    let source = r#"
        module Main; 
        type A = box struct {x: B};
        type B = box struct {x: I64};
        
        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let f = |a| a . ((mod_x : (B -> B) -> A -> A) $ mod_x $ |x| x + 15);
            let a = a.f;
            let ans = a.@x.@x;
            assert_eq(|_|"", ans, 31);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_annotated_pattern() {
    // Test type annotation at let-binding.
    let source = r#"
        module Main; 
        type A = box struct {x: B};
        type B = box struct {x: I64};
        
        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let f: A -> A = |a| a.(mod_x $ mod_x $ |x| x + 15);
            let a = a .f;
            let ans = a .@x .@x;
            assert_eq(|_|"", ans, 31);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_annotated_pattern_2() {
    // Test type annotation at let-binding.
    let source = r#"
        module Main;         
        main : IO ();
        main = (
            let x: I64 -> I64 = |x| x;
            let ans = x(42);
            assert_eq(|_|"", ans, 42);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test41_5() {
    // Test type annotation at lambda
    let source = r#"
        module Main;         
        main : IO ();
        main = (
            let x = |x: I64| x;
            let ans = x(42);
            assert_eq(|_|"", ans, 42);;
            pure()
        );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test42() {
    // Recursion function using global variable (not tail call).
    let n = 10000;
    let source = format!(
        r#"
            module Main;             
            loop : I64 -> I64;
            loop = |x| if x == 0 {{ 0 }} else {{ add(x) $ loop $ add(x, -1) }};
    
            main : IO ();
            main = (
                let ans = Main::loop({});
                assert_eq(|_|"", ans, {});;
                pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    test_source(source.as_str(), Configuration::compiler_develop_mode());
}

#[test]
pub fn test43() {
    // Recursion function using global variable (tail call).
    let n: i64 = 10000000;
    let source = format!(
        r#"
            module Main;             
            my_loop : I64 -> I64 -> I64;
            my_loop = |x, acc| if x == 0 {{ acc }} else {{ my_loop(x + -1, acc + x) }};
    
            main : IO ();
            main = (
                let ans = my_loop({}, 0);
                assert_eq(|_|"", ans, {});;
                pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    test_source(
        source.as_str(),
        Configuration::release_mode(SubCommand::Run),
    );
}

#[test]
pub fn test44() {
    // Test basic use of traits.
    let source = r#"
        module Main; 
        trait a : ToI64 {
            toI64 : a -> I64;
        }

        impl I64 : Main::ToI64 {
            toI64 = |x| x;
        }

        impl Bool : Main::ToI64 {
            toI64 = |b| if b { 0 } else { -1 };
        }

        add_head_and_next : [a: Main::ToI64] Array a -> I64; 
        add_head_and_next = |arr| (
            let head = arr.@(0).toI64;
            let next = arr.@(1).toI64;
            add(head, next)
        );

        main : IO ();
        main = (
            let arr0 = Array::fill(2, false);
            let arr0 = arr0.set(0, true);
            let x = add_head_and_next(arr0);

            let arr1 = Array::fill(2, 3);
            let arr1 = arr1.set(1, 5);
            let z = add_head_and_next(arr1);

            let y = toI64(5) + toI64(false);
            let ans = x + y + z;
            assert_eq(|_|"", ans, 11);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test44_5() {
    // Test Array::from_map.
    let source = r#"
        module Main; 
        sum : Array I64 -> I64;
        sum = |arr| (
            let loop = fix $ |loop, idx, sum| (
                if idx == arr.get_size { sum };
                loop(idx + 1, sum + arr.@(idx))
            );
            loop(0, 0)
        );

        main : IO ();
        main = (
            let arr = Array::from_map(10, |x| x * x);
            let ans = Main::sum(arr);
            assert_eq(|_|"", ans, 285);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test45() {
    // Test HKT.
    let source = r#"
        module Main; 
        trait [f:*->*] f : MyFunctor {
            my_map : (a -> b) -> f a -> f b;
        }

        impl Array : MyFunctor {
            my_map = |f, arr| (
                Array::from_map(arr.get_size, |idx| f $ arr.@(idx))
            );
        }

        sum : Array I64 -> I64;
        sum = |arr| (
            let loop = fix $ |loop, idx, sum| (
                if idx == arr.get_size { sum };
                loop(idx + 1, sum + arr.@(idx))
            );
            loop(0, 0)
        );

        main : IO ();
        main = (
            let arr = Array::from_map(10, |x| x);
            let arr = arr.my_map(|x| x * x);
            let ans = arr.sum;
            assert_eq(|_|"", ans, 285);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test46() {
    // Test confliction of global name and local name.
    let source = r#"
        module Main; 
        x : I64;
        x = 5;

        y : I64;
        y = 7;

        main : IO ();
        main = (
            let ans = (let x = 3 in let y = 2 in add(x, Main::y)) + x;
            assert_eq(|_|"", ans, 15);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_union_basic_unboxed() {
    // Basic use of union.
    let source = r#"
        module Main; 
        type I64OrBool = union {int : I64, bool: Bool};

        main : IO ();
        main = (
            let int_union = int(2).mod_bool(not).mod_int(add(1));
            let bool_union = bool(false).mod_bool(not).mod_int(add(1));
            let int_val = if int_union.is_int { int_union.as_int } else { 0 };
            let bool_val = if bool_union.is_bool { bool_union.as_bool } else { false };
            let ans = if bool_val { int_val } else { 0 };
            assert_eq(|_|"", ans, 3);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_union_basic_boxed() {
    // Basic use of boxed union.
    let source = r#"
        module Main; 
        type I64OrBool = box union {int : I64, bool: Bool};

        main : IO ();
        main = (
            let int_union = int(2).mod_bool(not).mod_int(add(1));
            let bool_union = bool(false).mod_bool(not).mod_int(add(1));
            let int_val = if int_union.is_int { int_union.as_int } else { 0 };
            let bool_val = if bool_union.is_bool { bool_union.as_bool } else { false };
            let ans = if bool_val { int_val } else { 0 };
            assert_eq(|_|"", ans, 3);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_union_mod_closure() {
    let source = r#"
        module Main; 
        type Union = union {val: I64, func: I64 -> I64};

        main : IO ();
        main = (
            let five = 5;
            let val = Union::val(3);
            let func = Union::func(|x| x + five).mod_func(|f||x|f(x)+2); // x -> x + 5 + 2
            let ans = func.as_func $ val.as_val;
            assert_eq(|_|"", ans, 7 + 3);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_union_mod_array() {
    // Test union for array.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let uni = Option::some([1,2,3]).mod_some(|arr| arr.append([4,5,6]));

            let arr = uni.as_some;
            assert_eq(|_|"", arr.get_size, 6);;
            assert_eq(|_|"", arr.@(0), 1);;
            assert_eq(|_|"", arr.@(1), 2);;
            assert_eq(|_|"", arr.@(2), 3);;
            assert_eq(|_|"", arr.@(3), 4);;
            assert_eq(|_|"", arr.@(4), 5);;
            assert_eq(|_|"", arr.@(5), 6);;

            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_shared_union_mod() {
    // Test union for array.
    let source = r#"
        module Main; 

        type MyOption a = box union { some : a, none : () };

        main : IO ();
        main = (
            let uni0 = MyOption::some([1,2,3]);
            let uni1 = uni0.mod_some(|arr| arr.append([4,5,6]));

            let arr = uni0.as_some;
            assert_eq(|_|"", arr.get_size, 3);;
            assert_eq(|_|"", arr.@(0), 1);;
            assert_eq(|_|"", arr.@(1), 2);;
            assert_eq(|_|"", arr.@(2), 3);;

            let arr = uni1.as_some;
            assert_eq(|_|"", arr.get_size, 6);;
            assert_eq(|_|"", arr.@(0), 1);;
            assert_eq(|_|"", arr.@(1), 2);;
            assert_eq(|_|"", arr.@(2), 3);;
            assert_eq(|_|"", arr.@(3), 4);;
            assert_eq(|_|"", arr.@(4), 5);;
            assert_eq(|_|"", arr.@(5), 6);;

            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test48() {
    // Parametrised struct.
    let source = r#"
        module Main; 
        type Vec a = box struct {data: Array a};

        main : IO ();
        main = (
            let int_vec = Vec {data: Array::fill(2, 5)};
            let int_vec = int_vec.mod_data(|arr| arr.set(0, 3));
            let head = int_vec.@data.@(0);
            let next = int_vec.@data.@(1);
            let ans = add(head, next);
            assert_eq(|_|"", ans, 8);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test49() {
    // Parametrised union.
    let source = r#"
        module Main; 
        type Either a b = union {left: a, right: b};

        main : IO ();
        main = (
            let int_left = Either::left(5);
            let ans = (
                if int_left.is_left {
                    int_left.as_left
                } else if int_left.as_right {
                    1
                } else {
                    0
                } 
            );
            assert_eq(|_|"", ans, 5);;
            pure()
        );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test50() {
    // test loop.
    let n = 100;
    let source = format!(
        r#"
            module Main;     
            main : IO ();
            main = (
                let ans = (
                    loop((0, 0), |state|
                        let i = state.@0;
                        let sum = state.@1;
                        if i == {} {{
                            break(sum)
                        }} else {{
                            continue $ (i+1, sum+i)
                        }} 
                    )
                );
                assert_eq(|_|"", ans, {});;
                pure()
            );
        "#,
        n,
        (n * (n - 1)) / 2
    );
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test50_3() {
    // test loop_iter, loop_iter_m.
    let source = r#"
        module Main;         
        main : IO ();
        main = (
            let sum = Iterator::count_up(0).loop_iter(0, |n, sum| (
                if n > 100 { break $ sum };
                continue $ sum + n
            ));
            assert_eq(|_|"case-loop", sum, 100 * 101 / 2);;

            let sum = *Iterator::count_up(0).loop_iter_m(0, |n, sum| (
                if n > 5 { break_m $ sum };
                (print $ n.to_string + " ");;
                continue_m $ sum + n
            ));
            assert_eq(|_|"case-loop_m", sum, 5 * 6 / 2);;

            pure()
        );
            "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test51() {
    // test trait bounds.
    let source = r#"
    module Main;     
    search : [a: Eq] a -> Array a -> I64;
    search = |elem, arr| loop(0) $ |idx| (
        if idx == arr.get_size {
            break $ -1
        } else if arr.@(idx) == elem { 
            break $ idx
        } else { 
            continue $ idx + 1 
        } 
    );
    
    main : IO ();
    main = (
        let arr = Array::fill(4, (0, false));
        let arr = arr.set(0, (0, false));
        let arr = arr.set(1, (0, true));
        let arr = arr.set(2, (1, false));
        let arr = arr.set(3, (1, true));
        let ans = arr.search((1, false)); // evaluates to 2
        assert_eq(|_|"", ans, 2);;
        pure()
    );
        "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test52() {
    // Test loop with boxed state / break.
    let source = r#"
    module Main; 
    type SieveState = box struct {i: I64, arr: Array Bool};
    
    // Calculate a Bool array whose element is true iff idx is prime.
    is_prime : I64 -> Array Bool;
    is_prime = |n| (
        let arr = Array::fill(n, true);
        let arr = arr.assert_unique(|_|"The array is not unique!").set(0, false);
        let arr = arr.assert_unique(|_|"The array is not unique!").set(1, false);
        loop(SieveState {i: 2, arr: arr}, |state|
            let i = state.@i;
            let arr = state.@arr;
            if i*i > n { break $ arr };
            let next_arr = if arr.@(i) {
                loop(SieveState {i: i+i, arr: arr}, |state|
                    let q = state.@i;
                    let arr = state.@arr;
                    if n-1 < q { 
                        break $ arr
                    } else {
                        continue $ SieveState{ i: q + i, arr: arr.assert_unique(|_|"The array is not unique!").set(q, false) }
                    }
                )
            } else {
                arr
            };
            continue $ SieveState{i: i + 1, arr: next_arr}
        )
    );

    // Count the appearance of a value in an array.
    count : [a: Eq] a -> Array a -> I64;
    count = |elem, arr| (
        loop((0, 0)) $ |state| (
            let i = state.@0;
            let sum = state.@1;
            if arr.get_size == i { break $ sum };
            let sum = sum + (if arr.@(i) == elem {1} else {0});
            continue $ (i+1, sum)
        )
    );
    
    main : IO ();
    main = (
        let ans = (is_prime $ 100).count(true);
        assert_eq(|_|"", ans, 25);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test53() {
    // Test mutation of unique unboxed struct (e.g., tuple).
    let source = r#"
    module Main;     
    main : IO ();
    main = (
        let pair = (13, Array::fill(1, 0));
        let pair = pair.assert_unique(|_|"The pair is not unique!").mod_0(|x| x + 3);
        let pair = pair.assert_unique(|_|"The pair is not unique!").mod_1(|arr| arr.assert_unique(|_|"The array is not unique!").set(0, 5));
        let x = pair.@0;
        let y = pair.@1.@(0);
        let ans = x + y;
        assert_eq(|_|"", ans, 13 + 3 + 5);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test54() {
    // Test mutation of shared unboxed struct (e.g., tuple).
    let source = r#"
    module Main;     
    main : IO ();
    main = (
        let pair0 = (13, Array::fill(1, 0));
        let pair1 = pair0.mod_1(|arr| arr.set(0, 5));
        let pair2 = pair0.mod_0(|x| x + 3);
        let x = pair1.@1.@(0);
        let y = pair2.@0;
        let ans = x + y;
        assert_eq(|_|"", ans, 13 + 3 + 5);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test55() {
    // Test <= operator
    let source = r#"
    module Main;     
    main : IO ();
    main = (
        let ans = (
            if 0 <= -1 && -1 >= 0 {
                0
            } else if 0 <= 0 && 0 <= 1 && 0 >= 0 && 1 >= 0 {
                1
            } else {
                2
            }
        );
        assert_eq(|_|"", ans, 1);;
        pure ()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test56() {
    // Test && and || operator
    let source = r#"
    module Main;     
    main : IO ();
    main = (
        let ans = (
            if false || false == false 
            && false || true == true 
            && true || false == true 
            && true || true == true 
            {1} else {0}
        );
        assert_eq(|_|"", ans, 1);;
        pure ()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test57() {
    // Test ! operator
    let source = r#"
    module Main;     
    main : IO ();
    main = (
        let ans = (
            if !false == true && !true == false {
                1
            } else {
                0
            }
        );
        assert_eq(|_|"", ans, 1);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test58() {
    // Test != operator
    let source = r#"
    module Main;     
    main : IO ();
    main = (
        let ans = (
            if false != true && true != false && !(true != true) && !(false != false) {
                1
            } else {
                0
            }
        );
        assert_eq(|_|"", ans, 1);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test59() {
    // Test namespace definition
    let source = r#"
    module Main;     
    namespace A {
        x : I64;
        x = 3;

        y : I64;
        y = 1;
    }

    namespace B {
        x : I64;
        x = 5;

        y : Bool;
        y = true;
    }

    main : IO ();
    main = (
        let ans = (if y {A::x + B::x + A::y} else {0});
        assert_eq(|_|"", ans, 9);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test60() {
    // Test unit.
    let source = r"
    module Main;     
    unit : ();
    unit = ();

    main : IO ();
    main = let u = unit; pure ();
    ";
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test61() {
    // Test Hello world.
    let source = r#"
    module Main; 
    main_loop : I64 -> IO ();
    main_loop = |counter| (
        if counter == 0 {
            pure()
        } else {
            println("Hello World! (" + counter.to_string + ")");;
            main_loop(counter - 1)
        }
    );

    main : IO ();
    main = main_loop(3);
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test61_5() {
    // Test Hello world.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        loop_m(0, |i| (
            if i == 3 { break_m $ () };
            println("Hello World! (" + i.to_string + ")");;
            continue_m $ i + 1
        ))
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test62() {
    // Test String length.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let len = "Hello World!".get_size;
        assert_eq(|_|"", len, 12);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test63() {
    // Test I64 ToString.
    // See also test98.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let min = -9223372036854775808;
        assert_eq(|_|"", min.to_string, "-9223372036854775808");;
        println $ min.to_string
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_literal() {
    // Test escape sequence.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        assert_eq(|_|"heart", "\u2764", "❤");;
        assert_eq(|_|"tab", "あ\tいうえお", "あ	いうえお");;
        assert_eq(|_|"tab", "あ\nいうえお", "あ
いうえお");;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test65() {
    // Test tuple pattern matching.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let sum = loop((0, 0), |state| 
            let (i, sum) = state;
            if i == 10 {
                break $ sum
            } else {
                continue $ (i+1, sum+i)
            }
        );
        assert_eq(|_|"", sum, 45);;
        pure ()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test66() {
    // Test unboxed struct pattern matching.
    let source = r#"
    module Main; 
    type State = unbox struct {idx: I64, sum: I64};

    main : IO ();
    main = (
        let sum = loop(State{idx:0, sum:0}, |state|
            let State {idx: i, sum: sum} = state;
            if i == 10 {
                break $ sum
            } else {
                continue $ State{idx: i+1, sum: sum+i}
            }
        );
        assert_eq(|_|"", sum, 45);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test67() {
    // Test boxed struct pattern matching.
    let source = r#"
    module Main; 
    type State = box struct {idx: I64, sum: I64};

    main : IO ();
    main = (
        let sum = loop(State{idx: 0, sum: 0}, |state|
            let State {idx: i, sum: sum} = state;
            if i == 10 {
                break $ sum
            } else {
                continue $ State{idx: i+1, sum: sum+i}
            }
        );
        assert_eq(|_|"", sum, 45);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test72() {
    // Test pattern matching on argment.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let sum = loop((0, 0), |(i, sum)|
            if i == 10 {
                break $ sum
            } else {
                continue $ (i + 1, sum + i)
            }
        );
        assert_eq(|_|"", sum, 45);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test73() {
    // Test pattern matching on argment.
    let source = r#"
    module Main; 
    type I64Bool = box struct {x: I64, y: Bool};

    main : IO ();
    main = (
        let int_bool = I64Bool { y: true, x: 42 };
        assert_eq(|_|"", int_bool.@x, 42);;
        assert_eq(|_|"", int_bool.@y, true);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test74() {
    // Test setter function of struct / tuple.
    let source = r#"
    module Main; 
    type UnboxStr = unbox struct {x: I64, y: Bool};
    type BoxStr = box struct {x: I64, y: Bool};

    main : IO ();
    main = (
        // Setter / getter of unboxed struct.
        let int_bool = UnboxStr { y: false, x: 0 };
        let int_bool = int_bool.set_x(3);
        assert_eq(|_|"case 0", int_bool.@x, 3);;
        let int_bool = int_bool.set_x(5);
        assert_eq(|_|"case 1", int_bool.@x, 5);;

        // Setter / getter of pair.
        let pair = (false, 0);
        let pair = pair.set_0(true);
        assert_eq(|_|"case 2", pair.@0, true);;
        let pair = pair.set_0(false);
        assert_eq(|_|"case 3", pair.@0, false);;

        // Setter / getter of boxed struct.
        let int_bool = BoxStr { y: false, x: 0 };
        let int_bool = int_bool.set_y(true);
        assert_eq(|_|"case 4", int_bool.@y, true);;
        let int_bool = int_bool.set_y(false);
        assert_eq(|_|"case 5", int_bool.@y, false);;

        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test75() {
    // Test iterator.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let iter = Iterator::from_map(|i| i*i );
        let (iter, n) = iter.advance.as_some;
        assert_eq(|_|"", n, 0*0);;
        let (iter, n) = iter.advance.as_some;
        assert_eq(|_|"", n, 1*1);;
        let (iter, n) = iter.advance.as_some;
        assert_eq(|_|"", n, 2*2);;
        let (iter, n) = iter.advance.as_some;
        assert_eq(|_|"", n, 3*3);;
        let (iter, n) = iter.advance.as_some;
        assert_eq(|_|"", n, 4*4);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test76() {
    // Test array modifier.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let array = Array::from_map(3, |_i| Array::from_map(3, |_j| 0));
        let array = array.mod(1, |arr| arr.assert_unique(|_|"The array is not unique!").set(1, 9));
        assert_eq(|_|"", array.@(1).@(1), 9);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test77() {
    // Test Iterator::zip / map / take / fold
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let iter0 = Iterator::count_up(5);
        let iter1 = Iterator::from_map(|i| 2*i);
        let iter = iter0.zip(iter1);
        let iter = iter.map(|(a,b)| a+b).take(3);
        let res = iter.fold(0, add);
        assert_eq(|_|"case 1", res, (5+2*0) + (6+2*1) + (7+2*2));;

        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test78() {
    // Test Iterator::filter
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let iter = Iterator::count_up(1).take(100);
        let iter = iter.filter(|n| n%3 == 0 || n%5 == 0);
        let count = iter.map(|_|1).fold(0, add);
        assert_eq(|_|"", count, 100/3 + 100/5 - 100/15);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test79() {
    // Test Iterator::push_front
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let ls = Iterator::empty;
        let ls = ls.push_front(1).push_front(2);
        let (ls, e) = ls.advance.as_some;
        assert_eq(|_|"", 2, e);;
        let (ls, e) = ls.advance.as_some;
        assert_eq(|_|"", 1, e);;
        assert(|_|"", ls.advance.is_none);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test81() {
    // Test array literal.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let arr = [1,2,3,4];
        assert_eq(|_|"", arr.get_size, 4);;
        let arr: Array Bool = [];
        assert_eq(|_|"", arr.get_size, 0);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test82() {
    // Test Array::append.
    let source = r#"
    module Main; 
    main : IO ();
    main = (

        // Test 0+2
        let v1 = [];
        let v2 = [3,4];
        let v = v1.append(v2);
        assert_eq(|_|"wrong reserved length (0+2)", v.get_capacity, 2);;
        assert_eq(|_|"wrong length (0+2)", v.get_size, 2);;
        assert_eq(|_|"wrong element (0+2)", v.@(0), 3);;
        assert_eq(|_|"wrong element (0+2)", v.@(1), 4);;

        // Test 2+0
        let v1 = [1,2];
        let v2 = [];
        let v = v1.append(v2);
        assert_eq(|_|"wrong reserved length (2+0)", v.get_capacity, 2);;
        assert_eq(|_|"wrong length (2+0)", v.get_size, 2);;
        assert_eq(|_|"wrong element (2+0)", v.@(0), 1);;
        assert_eq(|_|"wrong element (2+0)", v.@(1), 2);;

        // Test 0+0
        let v1: Array (I64 -> Bool) = [];
        let v2 = [];
        let v = v1.append(v2);
        assert_eq(|_|"wrong capacity (0+0)", v.get_capacity, 0);;
        assert_eq(|_|"wrong length (0+0)", v.get_size, 0);;

        // Test boxed elements.
        let v1 = [add(1), add(2)];
        let v2 = [add(3), add(4)];
        let v = v1.append(v2);
        let x = 0;
        let x = v.@(0) $ x;
        assert_eq(|_|"wrong value (boxed) 0+1", x, 0+1);;
        let x = v.@(1) $ x;
        assert_eq(|_|"wrong value (boxed) 0+1+2", x, 0+1+2);;
        let x = v.@(2) $ x;
        assert_eq(|_|"wrong value (boxed) 0+1+2+3", x, 0+1+2+3);;
        let x = v.@(3) $ x;
        assert_eq(|_|"wrong value (boxed) 0+1+2+3+4", x, 0+1+2+3+4);;

        // Test appending shared array.
        let v1 = [add(1), add(2)].reserve(4);
        let v2 = [add(3), add(4)];
        let v = v1.append(v2);
        let w = v2.append(v1);
        let x = 0;
        let x = v.@(0) $ x; // += 1
        let x = w.@(3) $ x; // += 2
        assert_eq(|_|"", x, 3);;

        let res = Array::empty(3);
        let v = [[1], [2], [3]].to_iter.fold(res, |v, res| (
            res.assert_unique(|_|"the array is not unique!").append(v)
        ));
        assert_eq(|_|"", v, [1, 2, 3]);;

        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test83() {
    // Test Array::push_back, pop_back
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        // Unboxed element
        let v = [];
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.push_back(idx);
            continue $ (idx+1, v)
        ));
        loop_m(0, |idx|(
            if idx == 100 { break_m $ () };
            assert_eq(|_|"wrong element", idx, v.@(idx));;
            continue_m $ idx + 1
        ));;
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.pop_back;
            continue $ (idx+1, v)
        ));
        assert_eq(|_|"wrong length after pop", 0, v.get_size);;
        assert(|_|"wrong reserved length after pop", v.get_capacity >= 100);;
    
        // Boxed element
        let v = [];
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.push_back(add(idx));
            continue $ (idx+1, v)
        ));
        let x = loop((0, 0), |(idx, x)|(
            if idx == 100 { break $ x };
            let x = v.@(idx) $ x;
            continue $ (idx + 1, x)
        ));
        assert_eq(|_|"wrong value (boxed)", x, 99 * 100 / 2);;
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.pop_back;
            continue $ (idx+1, v)
        ));
        assert_eq(|_|"wrong length after pop (boxed)", 0, v.get_size);;
        assert(|_|"wrong reserved length after pop (boxed)", v.get_capacity >= 100);;
    
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test84() {
    // Test Eq for Array
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let v1 = [1,2,3];
        let v2 = [1,2,3];
        assert(|_|"", v1 == v2);;
    
        let v1 = [1,2,3];
        let v2 = [0,2,3];
        assert(|_|"", v1 != v2);;
    
        let v1 = [];
        let v2 = [0];
        assert(|_|"", v1 != v2);;
    
        let v1: Array I64 = [];
        let v2 = [];
        assert(|_|"", v1 == v2);;
    
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test85() {
    // Test concat string, compare string.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let s1 = "Hello";
        let s2 = " ";
        let s3 = "World!";
        assert_eq(|_|"", s1.concat(s2).concat(s3), "Hello World!");;
    
        pure()
    );
    
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test86() {
    // Test concat_iter
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let iter = Iterator::from_array(["Hello", " ", "World", "!"]);
        assert_eq(|_|"", iter.concat_iter, "Hello World!");;
        pure()
    );
    
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test87() {
    // Test dynamic iterator comparison
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let lhs = Iterator::from_array([1,2,3]).to_dyn;
        let rhs = Iterator::from_array([1,2,3]).to_dyn;
        assert_eq(|_|"", lhs, rhs);;

        let lhs: DynIterator Bool = Iterator::from_array([]).to_dyn;
        let rhs = Iterator::from_array([]).to_dyn;
        assert_eq(|_|"", lhs, rhs);;

        let lhs = Iterator::from_array([]).to_dyn;
        let rhs = Iterator::from_array([1,2]).to_dyn;
        assert(|_|"", lhs != rhs);;

        pure()
    );
    
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test88() {
    // Test Iterator::intersperse
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let iter = Iterator::from_array([1,2,3]);
        let iter = iter.intersperse(0);
        assert_eq(|_|"", iter.to_array, [1,0,2,0,3]);;
    
        let iter = Iterator::from_array([1]);
        let iter = iter.intersperse(0);
        assert_eq(|_|"", iter.to_array, [1]);;
    
        let iter = Iterator::from_array([]);
        let iter = iter.intersperse(0);
        assert_eq(|_|"", iter.to_array, []);;
    
        pure()
    );
    
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test89() {
    // Test Iterator::append
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([4,5,6]);
        assert_eq(|_|"", lhs.append(rhs).to_array, [1,2,3,4,5,6]);;
    
        let lhs = Iterator::from_array([]);
        let rhs = Iterator::from_array([4,5,6]);
        assert_eq(|_|"", lhs.append(rhs).to_array, [4,5,6]);;

        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([]);
        assert_eq(|_|"", lhs.append(rhs).to_array, [1,2,3]);;

        let lhs : ArrayIterator I64 = Iterator::from_array([]);
        let rhs = Iterator::from_array([]);
        assert_eq(|_|"", lhs.append(rhs).to_array, []);;
    
        pure()
    );
    
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_sort_by() {
    // Test "sort_by" and related functions.
    let source = r#"
module Main;

main : IO ();
main = (
    // Merge sort
    test_sort(sort_stable_by(comparator), sort_stable_by(comparator));;

    // Stability test for merge sort.
    test_sort_stability(sort_stable_by(|((x1, y1), (x2, y2))| x1 < x2));;

    // Heap sort used in the implementation of introsort.
    test_sort(heap_sort(comparator), heap_sort(comparator));;

    // Insertion sort used in the implementation of introsort.
    test_sort(insertion_sort(comparator), insertion_sort(comparator));;

    // Introsort with small depth parameter.
    test_sort(introsort_small_depth(comparator), introsort_small_depth(comparator));;

    // Introsort
    test_sort(sort_by(comparator), sort_by(comparator));;

    pure()
);

heap_sort : ((a, a) -> Bool) -> Array a -> Array a;
heap_sort = |less_than, arr| (
    arr._heap_sort_by_ranged(less_than, 0, arr.get_size)
);

insertion_sort : ((a, a) -> Bool) -> Array a -> Array a;
insertion_sort = |less_than, arr| (
    arr._insertion_sort_by_ranged(less_than, 0, arr.get_size)
);

introsort_small_depth : ((a, a) -> Bool) -> Array a -> Array a;
introsort_small_depth = |less_than, arr| (
    arr._introsort_by_internal(less_than, 0, arr.get_size, 2_U8)
);

comparator : [a : LessThan] (a, a) -> Bool;
comparator = |(lhs, rhs)| lhs < rhs;

type SortMethod a = Array a -> Array a;

type BoxedI64 = box struct { v : I64 };

namespace BoxedI64 {
    make : I64 -> BoxedI64;
    make = |v| BoxedI64 { v : v };
}

impl BoxedI64 : Eq {
    eq = |lhs, rhs| lhs.@v == rhs.@v;
}

impl BoxedI64 : LessThan {
    less_than = |lhs, rhs| lhs.@v < rhs.@v;
}

impl BoxedI64 : LessThanOrEq {
    less_than_or_eq = |lhs, rhs| lhs.@v <= rhs.@v;
}

is_increasing : [a : LessThanOrEq] Array a -> Bool;
is_increasing = |arr| (
    if arr.is_empty { true };
    range(0, arr.get_size - 1).loop_iter(true, |i, _|
        let lhs = arr.@(i);
        let rhs = arr.@(i + 1);
        if lhs <= rhs {
            continue $ true
        } else {
            break $ false
        }
    )
);

test_sort : SortMethod I64 -> SortMethod BoxedI64 -> IO ();
test_sort = |sort_method, sort_method_boxed| (
    cases.to_iter.zip(count_up(1)).fold_m((), |(case, case_n), _|
        // unboxed case
        let xs = case;
        let ys = xs.sort_method;
        assert(|_| "case {}-unboxed", ys.is_increasing);;

        // boxed case
        let xs_boxed = xs.map(BoxedI64::make) : Array BoxedI64;
        let ys_boxed = xs_boxed.sort_method_boxed;
        assert(|_| "case {}-boxed".populate([case_n.to_string]), ys_boxed.is_increasing);;

        pure()
    );;

    pure()
);

// Test sorting stability.
// Give a sorting method by `(x1, y1) < (x2, y2) iff x1 < x2`.
test_sort_stability : SortMethod (I64, I64) -> IO ();
test_sort_stability = |sort_method| (
    cases.to_iter.zip(count_up(1)).fold_m((), |(case, case_n), _|
        let xs = case.to_iter.zip(count_up(0)).to_array;
        let ys = xs.sort_method;
        assert(|_| "case {}-stability".populate([case_n.to_string]), ys.is_increasing);;

        pure()
    )
);

cases : Array (Array I64);
cases = [
    case_0,
    case_1,
    case_2,
    case_stability_0,
    case_random_1,
    case_random_2,
    case_random_3,
    case_random_4,
    case_random_5,
    case_random_6,
    case_random_7,
    case_random_8,
    case_random_9,
];
       
case_0 : Array I64;
case_0 = [];

case_1 : Array I64;
case_1 = [3];

case_2 : Array I64;
case_2 = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];

case_stability_0 : Array I64;
case_stability_0 = [3, 3, 3, 2, 2, 2, 1, 1, 1, 0, 0, 0];

case_random_0 : Array I64;
case_random_0 = [346024429990377868, 245837103567876924, 3986578685004063178, 5805251788053972515, 1417556943926455241, 4845257856352310757, 4555558403327679905, 6902334504764357194, 7801513043249390307, 2949176974545635264, 7059641390372345377, 371268058065748070, 8967105408815998004, 9163707814261219422, 2811786699486158499, 695673114012699563, 7255892546594831251, 5215594425099043756, 7226627387424968494, 3500029154995518523, 73487224687842005, 7622782041994750406, 7620487533683671790, 1948626672868633357, 1716207585624274003, 843382608923212683, 7837714819837928558, 1816086736525301267, 7478592686167993236, 6684621575062124516, 8318977752397448659, 2280779419367863148, 4402166591858481893, 2886005979871858608, 8027251237215604302, 3089788399256501254, 4403629729898519952, 3444861609702597660, 3758594455717637291, 1536276683748698726, 444415575857841953, 1406828580750436238, 309946757719811718, 6469342214276629762, 2907307731075134021, 5390516200439052137, 3467070330460410020, 3131752283003023729, 3701002979343777983, 8757902344019921678, 4468592631788431941, 1988686597499626951, 5098943993242450129, 4563975020031135772, 4558740263275937216, 4136401328193265140, 6221776947277384664, 4218852228729107398, 4693164472015346829, 5926127793208466556, 5980928593819338181, 6358786111999711620, 2510847986880873405, 725552905354199960, 4318305169843662077, 9077270978751322765, 4446216120070684060, 7241182603221941674, 6883294465805350312, 182580429536400213, 6665826731289181158, 4103546543575562161, 2944031420480557330, 7145879548678655791, 684327070863398544, 3141373052076162633, 3376664309606534565, 7748477690866038352, 661097284839573365, 8320186250457439799, 4180671567925332991, 5741176287546058753, 5150928445261445863, 1395554178624938115, 4121093914965231516, 5528476498398069969, 510090779688131913, 1440969282495380048, 1957784169139475426, 7135569354928947870, 3472797929026694304, 2103602003886010196, 7282131254377995390, 8200716464868676255, 940910225290816613, 7646683707254883186, 4235747749241714850, 5832841740041503381, 8357660228540455387, 6794172312654611817];

case_random_1 : Array I64;
case_random_1 = [1473833639386082847, 9150924382999182533, 364959846503117916, 5510915564068011639, 5001696028236419798, 527206022233715959, 5824821149935431831, 3906748580689421172, 7768482083511476659, 8359154916340507916, 3902852956764412553, 6046158790262160252, 8581712180152615751, 3176396128264911235, 2348409379585372700, 4101365204482758944, 7288210760341865593, 6414121132270517156, 5143695022434509882, 2651077910063984462, 1344913895724074887, 5342767887754679416, 5207051254756331697, 1350468791188981477, 1385030731822695446, 183858007790869143, 5113507702702803725, 6339520396931193621, 3580079998021483503, 8566564745352972980, 1658305840877014492, 1751591158197077579, 4391879768313882565, 2371232387645854169, 9042662141072433667, 9089361195348459559, 1471524369948215519, 6180261892294887633, 7081440345965963408, 7730873161584516984, 1591562459371200975, 1199315828220048388, 2921316098480181608, 8142955342882910732, 3687539036234222399, 3854214199505884756, 4758705715804480257, 4565979565169930536, 1064730593010578830, 6990164198590953561, 1327201540052176232, 1974455101914757916, 1800475961785569288, 5496370252211965398, 7774983625575746472, 7968024747269723949, 4883854498878481281, 4983223599292988944, 4916656981933504450, 1070016358598574599, 8764741035156142044, 7356156862140378706, 2567283915008246868, 5101510187363106927, 2780368294076064461, 25263280100917895, 644367617447002604, 1828050620563012530, 436197704863618080, 3814207865941808620, 8230331239852930038, 3766059167097163950, 7129925487870311155, 6393369676053877305, 692825249577049427, 52018319054939993, 2532321329693273170, 6212378965565450708, 8332405240286407789, 3047552744657946662, 2090272279415822357, 7409920481171401760, 5542826447855611425, 6899209979811637317, 6135972976876419700, 1425269702630575297, 5580024993562828442, 4479279751373588959, 452211588614476792, 6315102616080874707, 6570799423701133233, 6803460648019031860, 5204411834599792330, 3090395063312956819, 9195865166657103502, 7745428936671859108, 7851158605663387420, 6106961438388791806, 7244278733968717679, 7424679269158195845];

case_random_2 : Array I64;
case_random_2 = [3735983945753853846, 7391658998971614283, 5773747749435627971, 891184639127069766, 7666756142478998494, 5258068467685168049, 5531629763755378722, 4184470953901519054, 6833971991910637282, 3453735290065541012, 937411429762803020, 5443893140171621388, 8945378597029766441, 950961201968551461, 2545699971354791918, 5952923203495371557, 4100644833358552276, 5302814824288480820, 6335341756313109356, 6617544061281219309, 8857699162724245261, 7150569800502283481, 4674825751406928383, 8781665986602998694, 952605013122176609, 7628901230689731814, 4493353023701228937, 2532681063592667729, 5754363468386433008, 3147312625325849803, 7249852656036338305, 647762666375089108, 4610352930650896770, 521933339245656484, 3729079644759025683, 6167512106039129933, 5500136387127309831, 4493934985486034735, 2822921863525896981, 6336649100189002117, 335180114781373253, 5169395453701782058, 7936381342744020175, 969216146526646944, 3489767304806200396, 3870081683368139720, 8915695158973286640, 4156385229611359870, 3630370554269049889, 4188321087822013810, 5753844463294166143, 7661119179087402899, 2351766302449068612, 34663543209339514, 4173528615489109448, 5497439222901020863, 9040864950571501064, 1158024851109175875, 3054049160883377116, 988982881683111036, 4533078271642029407, 3034338485958616060, 1698924376171944886, 5982194671836232741, 3053316016725600497, 6472928743436287649, 1827159067205000930, 9069843338575206584, 870475205632759170, 3079883231141468177, 8761501595168108166, 1161647259200077008, 4502735132210015906, 6931903262165460088, 5290902511099171704, 4097595643173659703, 85320373141275961, 5997724073234674190, 6745900690460735029, 9158035699048131036, 5429300450294741271, 8698991117524601715, 5000194258969348852, 3054036676972445304, 8401591917617480836, 2583613303661085965, 5969707311408714950, 6826899348649599785, 1666972670086079332, 6274277772199034264, 7506114481220316765, 1362108441573709769, 5714547591011712272, 2046830405129656097, 711400334759581335, 6423030744639085698, 6331061501819952538, 4635168627865828195, 7157652086692954180, 5094728093192878221];

case_random_3 : Array I64;
case_random_3 = [4254814088509248120, 7587844402241195533, 1896028181690980501, 4095302555750356217, 5305599755750235117, 5615462751829089661, 4185665630492252845, 8375136933841205645, 3732563431997416763, 6807856943350113153, 8657112689938273852, 2482211299603057006, 2101588013952248528, 2400472851179813738, 7770553575574753287, 7479074773091242939, 8378974446529309264, 8814315766576697808, 6322825574534027019, 3288265287469014406, 6135360527854074593, 8523803129856416966, 87959952235282533, 316206234419014891, 8530231147393589562, 6488332194330323527, 5496810760044038362, 4297911112211423489, 5137125537796140446, 8279860449798025517, 9196348810620663274, 4178596736539741516, 7213916347683404809, 8547792914732302656, 7297611675372483057, 8466983847583537548, 6088281874057260029, 5107947305576954670, 1311312250865822474, 6913106199103936368, 7280856164737616257, 4393405616541736253, 2958231259538406212, 4399032048847720917, 2598964703760052315, 8320701966392070119, 5549350645553879686, 3137553632811363987, 3687608033056216648, 7241174205012163856, 8414584945004687849, 4209515134588956604, 4646193481306425502, 8000500536508023078, 3609373043218432305, 2710195409590177628, 1870806658129711696, 4423612202995450177, 4329763552556607839, 4671438870218720378, 385800600250319604, 9192669293164231742, 5875290221623150810, 6428395608833850461, 5554544478644319744, 6127404359155431511, 4312571810574718578, 2934611530933693519, 6142017706708610053, 1033276389672138464, 7620571444321158695, 1123189488204397649, 6490005031624750662, 5451289712203163729, 480268465752047588, 2918278141765796473, 1128524718911288955, 2925878743034728027, 3353335543842743365, 2733439829001673968, 5611974287087482329, 9186706811707280454, 8287999331035549279, 6896096830400609553, 4143535739456450616, 8318092278276405101, 4733920994348664006, 1927783757453689685, 8167644521292088004, 7846992851966128439, 231565553474249419, 6806623032671543562, 4608862731533895720, 864017086253478830, 8307986747740822908, 7790711946987144187, 5889425727665311984, 8821269542119236684, 2246423448178164589, 3985368815367845512];

case_random_4 : Array I64;
case_random_4 = [6413185887971907809, 6207174647360241580, 5366304224432848273, 3430763646170375881, 2673735701196449853, 460371746629056585, 5776519683513877440, 5280945856576933447, 2950458880296865358, 7637751929908023620, 7082520184663010790, 5614177452252215979, 8830983632126841373, 9033305420227413286, 2182182116113107838, 1701644828958412125, 7545566810589837171, 5038399583882111205, 5062702330225733470, 6273771883212549316, 3134782956753190920, 6861978545912593812, 7698458376012530283, 8946838408248381449, 4556074831254219747, 8742343983661467683, 8915921618796079893, 9093439411179142010, 3826986255030102890, 1670731906726801739, 6595111424608193032, 7169634376184232328, 4132787722650654538, 4701421682412726327, 7112937551623670860, 4617499232446143662, 5625137272218788657, 5567337380566543734, 7116358278085160760, 3938970551559622599, 7266574059968113157, 4507072350383233586, 5399582904272578, 708419893063578698, 6010583150413231004, 1538752035474906755, 717577078689676939, 3575555964222475082, 2232692712392128466, 7384973132738388302, 677191589403811258, 2372514149318286271, 2382437384396579425, 6755045297735500716, 5679826433827930438, 4569603595865129512, 8899772810969901301, 7877996095818301164, 5651962221071419584, 6620684917143955809, 5009912688072883984, 3705630779159545107, 9173096627355902396, 1410810946686548790, 4701225747680014044, 5241860092221016162, 3807392259901383358, 7700899464263496301, 1216034444466052958, 6609937201512412417, 8163001815953754128, 5675485181357735095, 5518905292528997074, 4558416992500146038, 5520677285770387145, 2594212444988370399, 3975137470668157256, 2622217178817976491, 7607779888574021495, 9036641830249553041, 3199127825125160904, 386563718196113483, 4658764723466686850, 3661735792082485362, 6282408890861804805, 262106074947974955, 6553657865643612962, 7800704714434669338, 4312101409766186078, 1287627342714303154, 3423747554433487110, 710994768591745841, 4293348671685188867, 9029842013309778046, 3112430192633191197, 2024740616263220092, 819018526734220217, 5084871997379271816, 3869097151821873978, 8819015488503821127];

case_random_5 : Array I64;
case_random_5 = [6208053885516411215, 1334115713369660354, 1315942909716909312, 2244982026957277379, 3897233137685366866, 9055450307070912300, 7486423679754089054, 1145689676728901242, 3360429160686546256, 6969179237829500648, 60073074201889331, 5135285449205668393, 3902173258265935256, 4118419475990603892, 8594829430540368165, 2144831905867518983, 9151238279237136175, 6679338271824044687, 7986738180135689991, 5805130940137185621, 1200919947397149619, 1570281798240258263, 238924630229773791, 4381333379274041593, 7638941004275705526, 3920191737643080018, 3121873009651086033, 2382120450439922212, 6176943635923327389, 3522903056653638555, 6550635528538056858, 6279607372471335856, 5767958301763848252, 1238111209041851583, 1258897093755738674, 911380651013235108, 4919421135251209070, 2764191242684087332, 5834837102810554087, 2194148087981746132, 2154949110856645864, 7356053822752554949, 7318150074492532810, 7226237643195222290, 2332429213665718432, 6125701107497396494, 9033734446471861401, 4222027543262017685, 1309826078450500628, 1658158217774223133, 874011405993518171, 1747474265694080405, 353542402415603123, 7875546304743381466, 5366636224983018973, 6272435413696500426, 1769392116915533225, 7572532457559002367, 1762926733959000050, 3584719526879126776, 6820519612752199554, 4130756485781427675, 3886601032121689784, 7806640942374638219, 1306916720177717123, 1791238193175853567, 8468264605159129885, 3781537399213551306, 3432610801362329270, 5989950759278542662, 3667906992250961221, 6837703063983548445, 2347209953626311291, 3633623937610429977, 6821350798231022896, 732450899794772793, 4722020807400186875, 8431920588573445231, 7648348137443808224, 238766145314389876, 3678868348185127155, 5961029311923353337, 5574739062148029284, 7567332859168723096, 435704690043988411, 2086514796030212139, 4180674017576963511, 1797068537698904975, 7226287570449720239, 3810490023345739539, 5290213117643458941, 8240599871767633146, 7738250098171894065, 6537217947318548264, 7281089544284021608, 1284198417395978806, 2750106348236061069, 3303650182221686788, 6342047675178564355, 2951483385596788374];

case_random_6 : Array I64;
case_random_6 = [8948409198585432785, 2999603883131403515, 5192825075437899801, 2149590922052692916, 4334003597541877128, 1031227307741304043, 488454911926523731, 2039511621635876403, 797487881091962117, 7865085611815529384, 5938609180822245808, 7695520662232987020, 2117482400628446525, 2288785242786793399, 4095650794501567638, 4034149726825797387, 2926089375511837168, 6916431300302681739, 3985044132589838166, 7291471476319600125, 8165778072338986812, 3391844396100605900, 7728676327452604410, 3475773153213078600, 833095291786568974, 6289937966582186098, 638225522583745775, 2540355332866113270, 108296444237806815, 6734666717882264107, 7822737607143131330, 1906617099045224485, 846087365666398953, 2716691326243649917, 5373259504050175789, 7662674013768782152, 1670291335563641164, 3869900414152949533, 5328093528585854933, 2931697435687178095, 8545321206606613391, 3950312023622768528, 7273766155191139709, 1362992489106506996, 3307750371253091525, 578341487622274734, 261108128511196774, 5029143086678125710, 3830410133844165089, 8741107392854234756, 4605411985514533023, 3241386163029802175, 7932128749584246006, 1116655945881716477, 3295860325767425656, 1122027795006889078, 9006420945972346038, 1392628126163857832, 7323646991638376148, 839639992781114359, 7876886231530765697, 1672345502678476365, 2477342980886565419, 7278014033037569228, 6000681178156669376, 6112474340665892808, 2461499704693061363, 33007982446095288, 2276675790919051666, 4427333124335926844, 6504338185155789564, 1083342173918919566, 4261004805419409192, 4237101814193117061, 858469626860621700, 8602140452083533202, 2639685558467580746, 4202567620621552223, 7257738176741222851, 8212556720848678637, 5867400550874496795, 4762664335220609804, 5195270654328246633, 9093319664164292576, 1835316005791673087, 5010238442482537429, 6416675081734023554, 6444300204198417890, 2772191178032619197, 7381432724139137648, 7907784806292692157, 7773309969887913100, 5042990035271720491, 370552410293601411, 4764688429530351540, 7272718542133086237, 847771735673855495, 5186554620731603403, 6416138885900801736, 914920679609106839];

case_random_7 : Array I64;
case_random_7 = [8863841324612792694, 5885057137173196594, 5779905728742052152, 8884967702510472095, 7904250920978121827, 1511030099854981275, 8319371864487412104, 5051998925109579553, 6282474140493809903, 2094368062475508715, 8701472282628294672, 9168770789844761051, 4895622536579986067, 2674398741573064461, 7895652812439880347, 5360976436502058368, 2983946739457137714, 295341819036414327, 4302212059711883697, 6186065603362933700, 4816141494802099972, 4002651573682279048, 4696852289207898996, 6110339172492566233, 6999507626192338089, 3518908954856080574, 538411976198106716, 3865171475029115747, 3956144566085404663, 8346680932744442370, 2108836774429662717, 5704405970128129574, 1908175709551149212, 7507508944947133811, 5033919519890983867, 6460445447472083493, 8443995607322215813, 3543643821660218006, 4130294193137280608, 7856312813219131230, 362896769502118357, 6003049729434259541, 6492124450981551481, 4942041458600339700, 5572318388898646041, 7666777968286972250, 2452850554577258450, 7053366827431537744, 8714531118673061423, 2853264294892220865, 4515589049103499655, 5711344222451304855, 7269636998291191914, 8534875304831283385, 6816093692207842400, 2251507348979063320, 4629158779662141611, 4679866356696096465, 1563290120734751629, 3804310925595709832, 6934304487601788178, 8436746030096366018, 6156477139228677440, 1159983770980306723, 9063129178094632526, 8024663554535963130, 6936673191205973728, 6682942862830547485, 8981668248515055709, 6419986844999057208, 9081841244823972628, 3716406608061721345, 7239358354496464889, 4254382557265492073, 8378666406197711642, 5070084852546009580, 8630133017172882069, 7569217529118262587, 6771227638729146776, 8602467392150464979, 6305262211688485777, 969304171376202790, 2396869390485126520, 147183888271058564, 9017631276870332590, 7909582817990941955, 4086318072853835586, 3733856123180573131, 6991342308488125344, 3281538948648234357, 7655845297538283776, 4512488402820174849, 7102138458272615858, 2587822166342624791, 1724085736460294071, 6550392525745169864, 586257238633094584, 7356269545905327502, 6917037645362766946, 3956268230680543989];

case_random_8 : Array I64;
case_random_8 = [5412534809715959128, 4266243707908120482, 3631649333276716887, 268099286276283016, 554128782268631809, 5504689917839198978, 7358881858703796468, 7908163506745122406, 4878351282790491317, 5105946860657061450, 3851024307926278009, 7565451178885350474, 9032866367583096460, 5956798620394806796, 2681174851630863331, 2978011586600811715, 9006672499500605567, 2940455659960941281, 5144870596811312139, 6799968193253345622, 7445060574259474518, 7372793243201673919, 671146017797004214, 5378937565297698164, 1482624434313147464, 6093854983670946598, 5712333347298706075, 2390211801453225996, 8513250819636401156, 5310124319688354378, 4445840128422746286, 1775118706483351160, 495606413402406721, 1026591740810658191, 8705358192567111257, 5985527642452098499, 983818389517200382, 3655128258496273962, 3217191439907635750, 5858477308185933792, 4883319580311178029, 6697529670711830828, 51581802521631046, 4172037137013922698, 8776253257194226073, 4397896750996626437, 628840376068788454, 5416518708036788674, 1417691048709643243, 4533928543425582390, 4611038481401338129, 5599348696792433415, 4519094492785983585, 1311839473394794206, 522428013416675992, 7320027344517293869, 4785986402010699781, 7439874614819219866, 12490402884167122, 4498906637899974317, 5725586903015977525, 9164844176129656456, 7531388026297990948, 8600508182676705086, 2081501464673401019, 2229040221020212875, 3101626670667751218, 2446833741940814429, 2677675314345173618, 7212084889679795221, 382452865148902085, 1084970855296098656, 5236202795835083071, 65555354167814581, 6952742361048918860, 2035401883301164760, 7189947878238459208, 7747612895329508015, 8488021776811730599, 4898334665199608482, 7192884226956618028, 1977431124477349632, 633255890448389293, 9221126280718266787, 1507197757636385264, 480728641383034295, 8520892950563146422, 3254460466543268943, 8752595344589352156, 1171326306283262695, 4037536746272169950, 3854993227140902808, 7938666969898816989, 3896700802560339186, 6138941098788014699, 9048458495979188944, 3542053284846188536, 2673547570552034406, 3298928068761687610, 7138283428866517070];

case_random_9 : Array I64;
case_random_9 = [4811598823819225076, 945484849661270666, 3974642354777520028, 4158615994782851664, 2040134951037265503, 621157294625813268, 3420487196631744732, 8562789904150147509, 5726358921314649798, 5928356466053871855, 8973472345727263532, 4321215444607834704, 1419855537442860296, 2946349638552198896, 3481697262845158844, 4457169083154390199, 5742834600950974549, 6628679474349114391, 6784760729456999349, 865826070387573194, 5530670473241415591, 601960353818051893, 6940402831568588974, 1732888253346101831, 4064404537667084724, 463761504442288433, 851336403815555575, 4639882715506635517, 6156893648373566092, 2969518003217349877, 2576159349018145231, 8027510836498749087, 5149690187504693590, 8170630613445223100, 3648177171994296369, 7960190375053187226, 7263759542935751979, 3646755495938702849, 2176536960399349655, 8141763272899095526, 3152061100973745480, 8007005789339314019, 3960077814532411535, 1634345857783987769, 2373108331636701566, 1573355672903350727, 5445783405907903183, 3632275878089886926, 865542142112551140, 1560189317266206736, 4246728935508888062, 3731406940966692971, 4661417546444554281, 6714980452447875351, 1405853297966008052, 4866171316862003271, 4973715192799189631, 1654600401431482492, 5446299239438257290, 3212011753927910491, 6024857320095543, 5406301184340140945, 1842946635044363761, 8660803471712160155, 3474713044059450171, 6252972300581927837, 2244065612450317730, 4403217036547330969, 5592475621107541030, 6816858066830382012, 2319959547431359006, 4158163126853362947, 5862511634031442965, 6931304623600721638, 716712285699896675, 2437089756382021811, 522736709662127418, 3016288545856079198, 6300832290710693081, 796782230489727455, 4447400415547626842, 1844504230435377378, 4614130631843119404, 6274547284856733090, 8151077492150368419, 8275979515983809512, 8966097668492587054, 1680326380418336100, 3552567939733686233, 5623847191189992407, 31236258416721182, 1956086183739573280, 694371266853340607, 8991084936991929338, 2121010902239701563, 6677708078258950571, 4737885433227917146, 2294435783382887082, 3318326582083415987, 1341629257581308951];
    
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_sort_by_immutability() {
    let source = r#"
module Main;

main : IO ();
main = (
    let x = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10];
    let y = x.sort_by(|(lhs, rhs)| lhs < rhs);
    assert_eq(|_|"", y, [-10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);;
    assert_eq(|_|"", x, [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10]);;

    pure()
);
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_sort() {
    let source = r#"
module Main;

import Std hiding { FFI::Destructor::make };

type Pair = struct { x : I64, y : String };
make : I64 -> String -> Pair = |x, y| Pair { x : x, y : y };

impl Pair : LessThan {
    less_than = |lhs, rhs| (
        lhs.@x < rhs.@x
    );
}
impl Pair : Eq {
    eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y;
}

main : IO ();
main = (
    let x = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10];
    let y = x.sort;
    assert_eq(|_|"", y, [-10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);;
    assert_eq(|_|"", x, [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10]);;

    let x = [make(5, "a"), make(5, "b"), make(4, "c"), make(4, "d"), make(4, "e"), make(3, "f"), make(2, "g"), make(2, "h"), make(1, "i"), make(1, "j"), make(1, "k")];
    let y = x.sort_stable;
    assert_eq(|_|"", y, [make(1, "i"), make(1, "j"), make(1, "k"), make(2, "g"), make(2, "h"), make(3, "f"), make(4, "c"), make(4, "d"), make(4, "e"), make(5, "a"), make(5, "b")]);;
    assert_eq(|_|"", x, [make(5, "a"), make(5, "b"), make(4, "c"), make(4, "d"), make(4, "e"), make(3, "f"), make(2, "g"), make(2, "h"), make(1, "i"), make(1, "j"), make(1, "k")]);;

    pure()
);
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test92() {
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let buf = [].reserve(5);
        let vec = buf;
        eval vec.push_back(0);
        eval buf.push_back(1);
        pure()
    );

    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test93() {
    // Test try to make circular reference (and fail).
    let source = r#"
    module Main; 

    type Leaker = box struct { data : Option Leaker };

    main : IO ();
    main = (
        let leaker = Leaker { data : Option::none() };
        // let leaker = leaker.set_data!(Option::some(leaker)); // panics
        eval leaker.set_data(Option::some(leaker)); // doesn't make circular reference in fact.
        pure()
    );

    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_ffi_call() {
    // Test FFI
    let source = r#"
            module Main;     
            main : IO ();
            main = (
                eval "Hello C function! Number = %d\n".borrow_c_str(|ptr|
                    FFI_CALL[I32 printf(Ptr, I32), ptr, 42.to_I32]
                );
                pure()
            );
        "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_ffi_call_ios() {
    // Test FFI
    let source = r#"
            module Main;     

            main : IO ();
            main = (
                IO::from_runner(|ios|
                    "Hello C function! Number = %d\n".borrow_c_str(|ptr|
                        FFI_CALL_IOS[I32 printf(Ptr, I32), ptr, 42.to_I32, ios]
                    )
                );;
                pure()
            );
        "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_ffi_call_io() {
    // Test FFI
    let source = r#"
            module Main;     

            main : IO ();
            main = (
                "Hello C function! Number = %d\n".@_data.borrow_boxed_io(|ptr|
                    FFI_CALL_IO[I32 printf(Ptr, I32), ptr, 42.to_I32]
                );;
                pure()
            );
        "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test95() {
    // Test Std::unsafe_is_unique, Debug::assert_unique
    let source = r#"
            module Main; 
                
            main : IO ();
            main = (
                // For unboxed value, it returns true even if the value is used later.
                let int_val = 42;
                let (unique, _) = int_val.unsafe_is_unique;
                let use = int_val + 1;
                eval use;
                assert_eq(|_|"fail: int_val is shared", unique, true);;

                // For boxed value, it returns true if the value isn't used later.
                let arr = Array::fill(10, 10);
                let (unique, arr) = arr.unsafe_is_unique;
                let use = arr.@(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
                eval use;
                assert_eq(|_|"fail: arr is shared", unique, true);;

                // Fox boxed value, it returns false if the value will be used later.
                let arr = Array::fill(10, 10);
                let (unique, _) = arr.unsafe_is_unique;
                let use = arr.@(0);
                eval use;
                assert_eq(|_|"fail: arr is unique", unique, false);;

                let int_val = 42;
                eval int_val.assert_unique(|_|"fail: int_val is shared (2)");
                let use = int_val + 1;
                eval use;

                let arr = Array::fill(10, 10);
                let arr = arr.assert_unique(|_|"fail: arr is shared (2)");
                let use = arr.@(0);
                eval use;

                pure()
            );
        "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_u8_literal() {
    // Test U8 literal
    let source = r#"
            module Main;             
            main : IO ();
            main = (
                assert_eq(|_|"", 255_U8, 255_U8);;
                assert_eq(|_|"", 'A', 65_U8);;
                assert_eq(|_|"", '\0', 0_U8);;
                assert_eq(|_|"", '\t', 9_U8);;
                assert_eq(|_|"", '\r', 13_U8);;
                assert_eq(|_|"", '\n', 10_U8);;
                assert_eq(|_|"", '\\', 92_U8);;
                assert_eq(|_|"", '\'', 39_U8);;
                assert_eq(|_|"", '\x7f', 127_U8);;
                pure()
            );
        "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test97() {
    // Test arithmetic operation of U8, I32
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"1", -(1_U8), 255_U8);;
            assert_eq(|_|"2", 255_U8 + 3_U8, 2_U8);;
            assert_eq(|_|"3", 1_U8 - 3_U8, 254_U8);;
            assert_eq(|_|"4", 20_U8 * 30_U8, 88_U8);;
            assert_eq(|_|"5", 10_U8 / 3_U8, 3_U8);;
            assert_eq(|_|"6", 10_U8 % 3_U8, 1_U8);;
            assert_eq(|_|"7", 255_U8 > 0_U8, true);;
            assert_eq(|_|"8", 255_U8 >= 0_U8, true);;

            assert_eq(|_|"9", 2147483647_I32 + 2_I32, -2147483647_I32);;
            assert_eq(|_|"10", -2147483647_I32 - 2_I32, 2147483647_I32);;
            assert_eq(|_|"11", 2147483647_I32 * 2_I32, -2_I32);;
            assert_eq(|_|"12", 10_I32 / -3_I32, -3_I32);;
            assert_eq(|_|"13", 10_I32 % -3_I32, 1_I32);;
            assert_eq(|_|"14", -1_I32 < 0_I32, true);;
            
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test98() {
    // Test to_string, from_string for integrals
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            // I8
            assert_eq(|_|"I8 1", -128_I8.to_string, "-128");;
            assert_eq(|_|"I8 2", 127_I8.to_string, "127");;
            assert_eq(|_|"I8 3", -128_I8, "-128".from_string.as_ok);;

            // U8
            assert_eq(|_|"", 0_U8.to_string, "0");;
            assert_eq(|_|"", 255_U8.to_string, "255");;
            assert_eq(|_|"", 255_U8, "255".from_string.as_ok);;

            // I16
            assert_eq(|_|"I16 1", -32768_I16.to_string, "-32768");;
            assert_eq(|_|"I16 2", 32767_I16.to_string, "32767");;
            assert_eq(|_|"I16 3", -32768_I16, "-32768".from_string.as_ok);;

            // U16
            assert_eq(|_|"", 0_U16.to_string, "0");;
            assert_eq(|_|"", 65535_U16.to_string, "65535");;
            assert_eq(|_|"", 65535_U16, "65535".from_string.as_ok);;

            // I32
            assert_eq(|_|"", -2147483648_I32.to_string, "-2147483648");;
            assert_eq(|_|"", 2147483647_I32.to_string, "2147483647");;
            assert_eq(|_|"", -2147483648_I32, "-2147483648".from_string.as_ok);;

            // U32
            assert_eq(|_|"", 0_U32.to_string, "0");;
            assert_eq(|_|"", 4294967295_U32.to_string, "4294967295");;
            assert_eq(|_|"", 4294967295_U32, "4294967295".from_string.as_ok);;

            // I64
            assert_eq(|_|"", -9223372036854775808_I64.to_string, "-9223372036854775808");;
            assert_eq(|_|"", 9223372036854775807_I64.to_string, "9223372036854775807");;
            assert_eq(|_|"", -9223372036854775808_I64, "-9223372036854775808".from_string.as_ok);;

            // U64
            assert_eq(|_|"", 0_U64.to_string, "0");;
            assert_eq(|_|"", 18446744073709551615_U64.to_string, "18446744073709551615");;
            assert_eq(|_|"", 18446744073709551615_U64, "18446744073709551615".from_string.as_ok);;

            // Cases from_string fails.

            let res: Result ErrMsg I64 = "Hello World!".from_string;
            assert(|_|"Case: from_string invalid format", res.is_err);;

            let res: Result ErrMsg I64 = " 42".from_string;
            assert(|_|"Case: from_string invalid format (whitespace)", res.is_err);;

            let res: Result ErrMsg I64 = "1844674407370955161518446744073709551615".from_string;
            assert(|_|"Case: from_string out of range", res.is_err);;
            
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test99() {
    // Test cast between integral types.
    let mut rng = rand::thread_rng();
    let mut cases: Vec<String> = vec![];
    let tys = &[
        I8_NAME, U8_NAME, I16_NAME, U16_NAME, I32_NAME, U32_NAME, I64_NAME, U64_NAME,
    ];
    fn cast(num: i128, ty: &str) -> i128 {
        match ty {
            I8_NAME => (num as i8) as i128,
            U8_NAME => (num as u8) as i128,
            I16_NAME => (num as i16) as i128,
            U16_NAME => (num as u16) as i128,
            I32_NAME => (num as i32) as i128,
            U32_NAME => (num as u32) as i128,
            I64_NAME => (num as i64) as i128,
            U64_NAME => (num as u64) as i128,
            _ => {
                unreachable!()
            }
        }
    }
    for ty1 in tys {
        for ty2 in tys {
            let num = rng.gen::<i128>();
            let num1 = cast(num, ty1);
            let num2 = cast(num1, ty2);
            let lhs = format!("{}_{}.to_{}", num1, ty1, ty2);
            let rhs = format!("{}_{}", num2, ty2);
            let msg = format!(
                r#""{} != {}, lhs=" + {}.to_string + ", rhs=" + {}.to_string"#,
                lhs, rhs, lhs, rhs
            );
            let case = format!("assert_eq(|_|{}, {}, {});;", msg, lhs, rhs);
            cases.push(case);
        }
    }
    let source = format!(
        r#"
            module Main;     
            main : IO ();
            main = (
                {}
                pure()
            );
        "#,
        cases.join("\n")
    );
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test99_5() {
    // Test cast float to integral types.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"", -3.14_F32.to_I8, -3_I8);;
            assert_eq(|_|"", 3.14_F32.to_U8, 3_U8);;
            assert_eq(|_|"", -3.14_F32.to_I16, -3_I16);;
            assert_eq(|_|"", 3.14_F32.to_U16, 3_U16);;
            assert_eq(|_|"", -3.14_F32.to_I32, -3_I32);;
            assert_eq(|_|"", 3.14_F32.to_U32, 3_U32);;
            assert_eq(|_|"", -3.14_F32.to_I64, -3_I64);;
            assert_eq(|_|"", 3.14_F32.to_U64, 3_U64);;

            assert_eq(|_|"", -3.14_F64.to_I8, -3_I8);;
            assert_eq(|_|"", 3.14_F64.to_U8, 3_U8);;
            assert_eq(|_|"", -3.14_F64.to_I16, -3_I16);;
            assert_eq(|_|"", 3.14_F64.to_U16, 3_U16);;
            assert_eq(|_|"", -3.14_F64.to_I32, -3_I32);;
            assert_eq(|_|"", 3.14_F64.to_U32, 3_U32);;
            assert_eq(|_|"", -3.14_F64.to_I64, -3_I64);;
            assert_eq(|_|"", 3.14_F64.to_U64, 3_U64);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test99_51() {
    // Test cast integral to float types.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"", -123_I8.to_F32, -123.0_F32);;
            assert_eq(|_|"", 123_U8.to_F32, 123.0_F32);;
            assert_eq(|_|"", -123_I16.to_F32, -123.0_F32);;
            assert_eq(|_|"", 123_U16.to_F32, 123.0_F32);;
            assert_eq(|_|"", -123_I32.to_F32, -123.0_F32);;
            assert_eq(|_|"", 123_U32.to_F32, 123.0_F32);;
            assert_eq(|_|"", -123_I64.to_F32, -123.0_F32);;
            assert_eq(|_|"", 123_U64.to_F32, 123.0_F32);;

            assert_eq(|_|"", -123_I8.to_F64, -123.0);;
            assert_eq(|_|"", 123_U8.to_F64, 123.0);;
            assert_eq(|_|"", -123_I16.to_F64, -123.0);;
            assert_eq(|_|"", 123_U16.to_F64, 123.0);;
            assert_eq(|_|"", -123_I32.to_F64, -123.0);;
            assert_eq(|_|"", 123_U32.to_F64, 123.0);;
            assert_eq(|_|"", -123_I64.to_F64, -123.0);;
            assert_eq(|_|"", 123_U64.to_F64, 123.0);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test100() {
    // Test u8 literal
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"case 1", 'A', 65_U8);;
            assert_eq(|_|"case 2", '0', 48_U8);;
            assert_eq(|_|"case 3", '\n', 10_U8);;
            assert_eq(|_|"case 4", '\x7f', 127_U8);;
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test101() {
    // Test Array::is_empty, get_first, get_last.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let cap = 42;
            let arr: Array (() -> I64) = [];
            assert_eq(|_|"case 1", arr.is_empty, true);;
            assert_eq(|_|"case 2", arr.get_first.is_none, true);;
            assert_eq(|_|"case 3", arr.get_last.is_none, true);;

            let cap = 42;
            let arr: Array (() -> I64) = [|_|cap];
            assert_eq(|_|"case 4", arr.is_empty, false);;
            assert_eq(|_|"case 5", arr.get_first.as_some $ (), 42);;
            assert_eq(|_|"case 6", arr.get_last.as_some $ (), 42);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_get_first_get_tail() {
    let source = r#"
        module Main; 
        main : IO ();

        main = (
            let iter = [1,2,3,4].to_iter;
            assert_eq(|_|"case 1", iter.get_first, some(1));;
            assert(|_|"case 2", iter.get_tail.as_some.is_equal([2, 3, 4].to_iter));;

            let iter = ([] : Array I64).to_iter;
            assert_eq(|_|"case 2", iter.get_first, none());;
            assert(|_|"case 3", iter.get_tail.is_none);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_take_while() {
    // take_while : (a -> Bool) -> Iterator a -> Iterator a;
    let source = r#"
        module Main; 
        main : IO ();

        main = (
            let iter = Iterator::from_array([1,2,3,4,5,6,7,8,9,10]);
            let iter = iter.take_while(|x| x < 5);
            assert(|_|"case 1", iter.is_equal(Iterator::from_array([1,2,3,4])));;

            let iter = Iterator::from_array([1,2,3,4,5,6,7,8,9,10]);
            let iter = iter.take_while(|x| x < 100);
            assert(|_|"case 2", iter.is_equal(Iterator::from_array([1,2,3,4,5,6,7,8,9,10])));;

            let iter = Iterator::from_array([1,2,3,4,5,6,7,8,9,10]);
            let iter = iter.take_while(|x| x < 0);
            assert(|_|"case 3", iter.is_equal(Iterator::from_array([])));;

            let iter = Iterator::from_array([] : Array I64);
            let iter = iter.take_while(|x| x < 100);
            assert(|_|"case 4", iter.is_equal(Iterator::from_array([])));;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test102() {
    // Test I64 : Eq, LessThan, LessThanEq
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"case 1", 0 == 0, true);;
            assert_eq(|_|"case 2", 0 == 1, false);;
            assert_eq(|_|"case 3", 0 != 0, false);;
            assert_eq(|_|"case 4", 0 != 1, true);;

            assert_eq(|_|"case 5", 0 < 0, false);;
            assert_eq(|_|"case 6", 0 > 0, false);;
            assert_eq(|_|"case 7", 0 < 1, true);;
            assert_eq(|_|"case 8", 0 > 1, false);;

            assert_eq(|_|"case 9", 0 <= 0, true);;
            assert_eq(|_|"case 10", 0 >= 0, true);;
            assert_eq(|_|"case 11", 0 <= 1, true);;
            assert_eq(|_|"case 12", 0 >= 1, false);;
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test103() {
    // Test Bool : Eq
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"case 1", false == false, true);;
            assert_eq(|_|"case 2", false == true, false);;
            assert_eq(|_|"case 3", true == false, false);;
            assert_eq(|_|"case 4", true == true, true);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test104() {
    // Test Bool : ToString
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"case 1", true.to_string, "true");;
            assert_eq(|_|"case 2", false.to_string, "false");;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test105() {
    // Test String::get_first_byte, get_last_byte, is_empty
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"case 1", "".is_empty, true);;
            assert_eq(|_|"case 2", "".get_first_byte.is_none, true);;
            assert_eq(|_|"case 3", "".get_last_byte.is_none, true);;
            assert_eq(|_|"case 4", "abc".is_empty, false);;
            assert_eq(|_|"case 5", "abc".get_first_byte.as_some, 'a');;
            assert_eq(|_|"case 6", "abc".get_last_byte.as_some, 'c');;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test106() {
    // Test [a : Eq] Option a : Eq
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let lhs: Option I64 = Option::none();
            let rhs: Option I64 = Option::none();
            assert(|_|"case 1", lhs == rhs);;

            let lhs: Option I64 = Option::none();
            let rhs: Option I64 = Option::some(42);
            assert(|_|"case 2", lhs != rhs);;

            let lhs: Option I64 = Option::some(84);
            let rhs: Option I64 = Option::some(42);
            assert(|_|"case 3", lhs != rhs);;

            let lhs: Option I64 = Option::some(42);
            let rhs: Option I64 = Option::some(42);
            assert(|_|"case 4", lhs == rhs);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test107() {
    // Test String::pop_back_byte, strip_last_bytes, strip_last_newlines.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"case 1", "".pop_back_byte, "");;
            assert_eq(|_|"case 2", "a".pop_back_byte, "");;

            assert_eq(|_|"case 3", "".strip_last_bytes(|c|c == 'x'), "");;
            assert_eq(|_|"case 4", "abc".strip_last_bytes(|_|true), "");;
            assert_eq(|_|"case 5", "".strip_last_bytes(|_|true), "");;
            assert_eq(|_|"case 6", "x".strip_last_bytes(|c|c == 'x'), "");;
            assert_eq(|_|"case 7", "y".strip_last_bytes(|c|c == 'x'), "y");;
            assert_eq(|_|"case 8", "yx".strip_last_bytes(|c|c == 'x'), "y");;
            assert_eq(|_|"case 9", "yxz".strip_last_bytes(|c|c == 'x'), "yxz");;

            assert_eq(|_|"case 10", "abc\n\r".strip_last_newlines, "abc");;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test108() {
    // Test write_file_string, read_file_string, read_line.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let file_path = "test_uAfQDPwJ7sS6.txt";
            let lines = ["Hello", "World!"];
            let content = Iterator::from_array(lines).intersperse("\n").concat_iter;
            do {
                write_file_string(file_path, content);;

                let read_content = *read_file_string(file_path);
                assert_eq(|_|"case 1", content, read_content).lift;;

                let read_lines = *with_file(file_path, "r", |file| (
                    pure $ [*read_line(file), *read_line(file)]
                ));
                assert_eq(|_|"case 2", read_lines.@(0), lines.@(0) + "\n").lift;;
                assert_eq(|_|"case 3", read_lines.@(1), lines.@(1)).lift;;

                pure()
            }.try(exit_with_msg(1))
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
    remove_file("test_uAfQDPwJ7sS6.txt").unwrap();
}

#[test]
pub fn test_is_eof() {
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            let file_path = "test_bUeW9baGGZmE.txt";
            let content = "Hello World!";
            do {
                write_file_string(file_path, content);;

                let read_content = *with_file(file_path, "r", |file| (
                    let content = *read_string(file);
                    let is_eof = *is_eof(file).lift;
                    assert(|_|"file had not reached to EOF!", is_eof).lift;;
                    pure $ content
                ));
            
                assert_eq(|_|"read_content != content", content, read_content).lift;;

                pure()
            }.try(exit_with_msg(1))
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
    remove_file("test_bUeW9baGGZmE.txt").unwrap();
}

#[test]
pub fn test108_5() {
    // Test write_file_bytes, read_file_bytes.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let file_path = "test_vgZNhmj4gPbF.dat";
            let data = Array::from_map(1024 + 512, |n| n.to_U8);
            do {
                write_file_bytes(file_path, data);;

                let read = *read_file_bytes(file_path);
                assert_eq(|_|"case 1", data, read).lift;;

                pure()
            }.try(exit_with_msg(1))
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
    remove_file("test_vgZNhmj4gPbF.dat").unwrap();
}

#[test]
pub fn test109() {
    // Test monad syntax.
    let source = r#"
        module Main; 
        add_opt_int : Option I64 -> Option I64 -> Option I64;
        add_opt_int = |lhs, rhs| pure $ *lhs + *rhs;

        sequence : [m : Monad, m : Functor, it : Iterator, Item it = m a] it -> m (Array a);
        sequence = |iter| (
            match iter.advance {
                none() => pure $ [],
                some((iter, a)) => (
                    pure $ [*a].append(*sequence(iter))
                )
            }
        );

        main : IO ();
        main = (
            let one = Option::some(1);
            let two = Option::some(2);
            let three = Option::some(3);
            let none = Option::none();

            assert_eq(|_|"case 1", add_opt_int(one, two), three);;
            assert_eq(|_|"case 2", add_opt_int(none, two), none);;
            assert_eq(|_|"case 3", add_opt_int(one, none), none);;
            assert_eq(|_|"case 4", add_opt_int(none, none), none);;

            let res0 = Result::ok(0) : Result String I64;
            let res1 = Result::ok(1);
            let res2 = Result::ok(2);
            let res3 = Result::ok(3);
            let res_iter = Iterator::from_array([res0, res1, res2, res3]).sequence;
            assert_eq(|_|"case 5", res_iter.is_ok, true);;
            assert_eq(|_|"case 6", res_iter.as_ok, [0, 1, 2, 3]);;

            let res0 = Result::ok(0) : Result String I64;
            let res1 = Result::ok(1);
            let res2 = Result::err("Error 2");
            let res3 = Result::err("Error 3");
            let res_iter = Iterator::from_array([res0, res1, res2, res3]).sequence;
            assert_eq(|_|"case 5", res_iter.is_err, true);;
            assert_eq(|_|"case 6", res_iter.as_err, "Error 2");;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test110a() {
    // Test basic float operations, cast between floats, to_string, from_string, to_string_with_precision
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let x = -3.1415_F32;
            let y = 3.1415_F32;
            assert(|_|"case 1", x.abs == y);;
            assert(|_|"case 2", y.abs == y);;

            let x = -3.1415;
            let y = 3.1415;
            assert(|_|"case 3", x.abs == y);;
            assert(|_|"case 4", y.abs == y);;

            let x = 3.1415_F32;
            let y = 3.1415_F32;
            assert(|_|"case 5", x == y);;

            let x = 3.1415;
            let y = 3.1415;
            assert(|_|"case 6", x == y);;

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            assert(|_|"case 7", x != y);;

            let x = 3.1415;
            let y = 2.7183;
            assert(|_|"case 8", x != y);;

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 5.8598_F32;
            assert(|_|"case 9", (x + y - z).abs < 1.0e-4_F32);;

            let x = 3.1415;
            let y = 2.7183;
            let z = 5.8598;
            assert(|_|"case 10", (x + y - z).abs < 1.0e-4);;

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 8.5395_F32;
            assert(|_|"case 11", (x * y - z).abs < 1.0e-4_F32);;

            let x = 3.1415;
            let y = 2.7183;
            let z = 8.5395;
            assert(|_|"case 12", (x * y - z).abs < 1.0e-4);;

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 1.1557_F32;
            assert(|_|"case 13", (x / y - z).abs < 1.0e-4_F32);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test110b() {
    // Test basic float operations, cast between floats, to_string, from_string, to_string_with_precision
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let x = 3.1415;
            let y = 2.7183;
            let z = 1.1557;
            assert(|_|"case 14", (x / y - z).abs < 1.0e-4);;

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            assert(|_|"case 15", x > y);;

            let x = 3.1415;
            let y = 2.7183;
            assert(|_|"case 16", x > y);;

            let x = 3.1415_F32;
            let y = 3.1415_F32;
            assert(|_|"case 17", x >= y);;

            let x = 3.1415;
            let y = 3.1415;
            assert(|_|"case 18", x >= y);;

            let x = 3.1415_F32;
            let y = 3.1415_F32;
            assert(|_|"case 19.1", x.to_F32 == y);;

            let x = 3.1415;
            let y = 3.1415;
            assert(|_|"case 19.1", x.to_F64 == y);;

            let x = 3.1415_F32;
            let y = 3.1415;
            assert(|_|"case 19.3", (x.to_F64 - y) < 1.0e-4);;

            let x = 3.1415;
            let y = 3.1415_F32;
            assert(|_|"case 19.4", (x.to_F32 - y) < 1.0e-4_F32);;

            let x = 3141;
            let y = 3141.0;
            assert(|_|"case 20", x.to_F64 == y);;

            let x = 3141.0;
            let y = 3141;            
            assert(|_|"case 21", x.to_I64 == y);;

            let x = 3.14;
            let z : F64 = 3.14.to_string.from_string.as_ok;

            println(z.to_string);;

            assert_eq(|_|"case 22", 3.14, z);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test111() {
    // Test function composition operators
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let f = |x| x + 2;
            let g = |x| 3*x + 4;

            let f_g = f << g;
            let g_f = f >> g;

            assert_eq(|_|"case 1", f_g(0), 6);;
            assert_eq(|_|"case 2", g_f(0), 10);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test112() {
    // Test Iterator::generate
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let iter = Iterator::generate(0, |_| Option::none());
            let ans = [] : Array I64;
            assert_eq(|_|"case 1", iter.to_array, ans);;

            let iter = Iterator::generate(0, |i| if i == 3 { Option::none() } else { Option::some $ (i+1, i) });
            let ans = [0, 1, 2];
            assert_eq(|_|"case 1", iter.to_array, ans);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test113() {
    // Test bit operations.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            // Shift
            let x = 10_U8.shift_right(2_U8);
            assert_eq(|_|"case 1", x, 2_U8);;

            let x = -10_I32.shift_right(2_I32);
            assert_eq(|_|"case 1", x, -3_I32);;

            let x = 10_U8.shift_left(2_U8);
            assert_eq(|_|"case 1", x, 40_U8);;

            // Xor, Or, And
            let x = 10.bit_xor(12);
            assert_eq(|_|"case 1", x, 6);;

            let x = 10.bit_or(12);
            assert_eq(|_|"case 1", x, 14);;

            let x = 10.bit_and(12);
            assert_eq(|_|"case 1", x, 8);;

            // Not
            let x = 0b01000100_U8.bit_not;
            assert_eq(|_|"case 1", x, 0b10111011_U8);;

            let x = 0b1000000000000000_U16.bit_not;
            assert_eq(|_|"case 1", x, 0b0111111111111111_U16);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test114() {
    // Test Array::find_by
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let arr = [0,1,2,3];

            let res = arr.find_by(|x| x % 5 == 2);
            assert_eq(|_|"case 1", res, Option::some(2));;

            let res = arr.find_by(|x| x % 5 == 4);
            assert_eq(|_|"case 1", res, Option::none());;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_destructor() {
    // Test Std::Destructor
    let source = r#"
        module Main; 
        main : IO ();
        main = (

            // Boxed case
            let dtor0 = Destructor { 
                _value : [1,2,3], 
                dtor : |val| (
                    let arr_str = val.to_iter.map(to_string).join(", ");
                    println("dtor0 destructed. val: " + arr_str);;
                    pure $ val
                )
            };

            // Unboxed case
            let dtor1 = Destructor { 
                _value : 42, 
                dtor : |val| (
                    println("dtor1 destructed. val: " + val.to_string);;
                    pure $ val
                )
            };

            // Dtor in dtor
            let dtor3 = Destructor { 
                _value : 2, 
                dtor : |val| (
                    println("dtor3 destructed. val: " + val.to_string);;
                    pure $ val
                )
            };
            let dtor2 = Destructor { 
                _value : dtor3, 
                dtor : |val| (
                    println("dtor2 destructed. val.@_value: " + val.@_value.to_string);;
                    pure $ val
                )
            };

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test117() {
    // Test String::from_c_str
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let str = String::_unsafe_from_c_str([65_U8, 66_U8, 67_U8, 0_U8, 0_U8]);
            assert_eq(|_|"case 1", str, "ABC");;
            assert_eq(|_|"case 2", str.get_size, 3);;
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test118() {
    // Test fold_m
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            count_up(0).take(10).fold_m(0, |s, i| (
                let s = s + i;
                print("Sum upto " + i.to_string + " is " + s.to_string + ". ");;
                pure $ s
            ));;
            println("");;
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test119() {
    // Test namespace and MakeStruct, Pattern.
    let source = r#"
        module Main; 
        namespace A {
            type S = box struct { data : () };
            type U = union { data : () };
        }

        namespace B {
            type S = box struct { data : () };
            type U = union { data : () };
        }

        main : IO ();
        main = (
            let s = A::S { data : () };
            let A::S { data : _ } = s;
            let s = B::S { data : () };
            let B::S { data : _ } = s;
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_undefined() {
    // Test undefined of type Array or function.
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
pub fn test_punched_array_0() {
    // Test PunchedArray.
    let source = r#"
        module Main; 

        type MyBox = box struct { x : I64 };

        main : IO ();
        main = (
            // Case 1-1: Punch an array of two boxed values and release parray.
            let arr = [MyBox { x : 5 }, MyBox { x : 7 }];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 1-1", five.@x, 5);;

            // Case 1-2: Punch an array of two boxed values and plug-in the same element.
            let arr = [MyBox { x : 5 }, MyBox { x : 7 }];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 1-2-a", five.@x, 5);;
            let arr = parr._plug_in(five);
            assert_eq(|_|"case 1-2-b", arr.@(0).@x + arr.@(1).@x, 5 + 7);;

            // Case 1-3: Punch an array of two boxed values and plug-in the other element.
            let seven = MyBox { x : 7 };
            let arr = [MyBox { x : 5 }, seven];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 1-3-a", five.@x, 5);;
            let arr = parr._plug_in(seven);
            assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 7);;

            // Case 1-4: Punch an array of two boxed values and plug-in another value.
            let arr = [MyBox { x : 5 }, MyBox { x : 7 }];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 1-3-a", five.@x, 5);;
            let arr = parr._plug_in(MyBox { x : 11 });
            assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 11);;

            // Case 2-1: Punch an array of two shared boxed values and release parray.
            let five = MyBox { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 2-1", five.@x, 5);;

            // Case 2-2: Punch an array of two shared boxed values and plug-in the same element.
            let five = MyBox { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 2-2-a", five.@x, 5);;
            let arr = parr._plug_in(five);
            assert_eq(|_|"case 2-2-b", arr.@(0).@x + arr.@(1).@x, 5 + 5);;

            // Case 2-3: Punch an array of two shared boxed values and plug-in the value again.
            let five = MyBox { x : 5 };
            let arr = [five, five];
            let (parr, five1) = arr._unsafe_punch(0);
            assert_eq(|_|"case 2-3-a", five1.@x, 5);;
            let arr = parr._plug_in(five);
            assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 5 + 5);;

            // Case 2-4: Punch an array of two shared boxed values and plug-in another value.
            let five = MyBox { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 2-3-a", five.@x, 5);;
            let arr = parr._plug_in(MyBox { x : 7 });
            assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 5);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_punched_array_1() {
    // Test PunchedArray.
    let source = r#"
        module Main; 

        type MyBox = box struct { x : I64 };

        main : IO ();
        main = (
            // Case 3-1: Punch an array of one boxed values and release parray.
            let arr = [MyBox { x : 5 }];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 3-1", five.@x, 5);;

            // Case 3-2: Punch an array of two boxed values and plug-in the same element.
            let arr = [MyBox { x : 5 }];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 3-2-a", five.@x, 5);;
            let arr = parr._plug_in(five);
            assert_eq(|_|"case 3-2-b", arr.@(0).@x, 5);;

            // Case 4-1: Punch an array of two unboxed values and release parray.
            let arr = [5, 7];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 1-1", five, 5);;

            // Case 4-2: Punch an array of two boxed values and plug-in a value.
            let arr = [5, 7];
            let (parr, five) = arr._unsafe_punch(0);
            assert_eq(|_|"case 4-2-a", five, 5);;
            let arr = parr._plug_in(13);
            assert_eq(|_|"case 4-2-b", arr.@(0) + arr.@(1), 13 + 7);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_act_0() {
    // Test Array::act
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            pure();; // To make the `arr` defined below not global and therefore unique.

            // If the array and the element is both unique, the action should receive an unique value.
            let arr = [[1,2,3], [4,5,6]];
            let arr = arr.act(0, |arr| let arr = arr.assert_unique(|_|"the array is not unique!"); (arr.to_iter.sum, []));
            assert_eq(|_|"case 1", arr, (6, [[], [4,5,6]]));;

            // Case where the array is shared.
            let arr = [[1,2,3], [4,5,6]];
            let arr1 = arr.act(0, |arr| (arr.to_iter.sum, []));
            assert_eq(|_|"case 2", arr1, (6, [[], [4,5,6]]));;
            assert_eq(|_|"case 3", arr, [[1,2,3], [4,5,6]]);;

            // Case where the element is shared.
            let elem = [1,2,3];
            let arr = [elem, [4,5,6]];
            let arr = arr.act(0, |arr| (arr.to_iter.sum, []));
            assert_eq(|_|"case 4", arr, (6, [[], [4,5,6]]));;
            assert_eq(|_|"case 5", elem, [1,2,3]);;

            // Case where the array and the element is both shared.
            let elem = [1,2,3];
            let arr = [elem, [4,5,6]];
            let arr1 = arr.act(0, |arr| (arr.to_iter.sum, []));
            assert_eq(|_|"case 6", arr1, (6, [[], [4,5,6]]));;
            assert_eq(|_|"case 7", arr, [[1,2,3], [4,5,6]]);;
            assert_eq(|_|"case 8", elem, [1,2,3]);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_act_1() {
    // Test Array::act
    let source = r#"
        module Main; 
        
        type MyBox = box struct { x : I64 };

        main : IO ();
        main = (
            let act0: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ v.assert_unique(|_|"not unique!").mod_x(add(5)) } else { Option::none() }
            );
            let act01: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ v.mod_x(add(5)) } else { Option::none() }
            );
            let act1: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ MyBox { x : 5 } } else { Option::none() }
            );

            // Case 0-0-0-0: Box element, unique array, act0 succeeds.
            let case = "0-0-0-0";
            let arr = [MyBox { x : 0 }, MyBox { x : 3 }];
            let opt_arr = arr.act(0, act0);
            assert(|_|"Case " + case + "-a", opt_arr.is_some);;
            assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);;
            assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0).@x, 5);;
            assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1).@x, 3);;

            // Case 0-0-0-1: Box element, unique array, act0 fails.
            let case = "0-0-0-1";
            let arr = [MyBox { x : 1 }, MyBox { x : 3 }];
            let opt_arr = arr.act(0, act0);
            assert(|_|"Case " + case + "-a", opt_arr.is_none);;

            // Case 0-0-1-0: Box element, unique array, act1 succeeds.
            let case = "0-0-1-0";
            let arr = [MyBox { x : 0 }, MyBox { x : 3 }];
            let opt_arr = arr.act(0, act1);
            assert(|_|"Case " + case + "-a", opt_arr.is_some);;
            assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);;
            assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0).@x, 5);;
            assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1).@x, 3);;

            // Case 0-0-1-1: Box element, unique array, act1 fails.
            let case = "0-0-1-1";
            let arr = [MyBox { x : 1 }, MyBox { x : 3 }];
            let opt_arr = arr.act(0, act1);
            assert(|_|"Case " + case + "-a", opt_arr.is_none);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_act_2() {
    // Test Array::act
    let source = r#"
        module Main; 
        
        type MyBox = box struct { x : I64 };

        main : IO ();
        main = (
            let act0: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ v.assert_unique(|_|"not unique!").mod_x(add(5)) } else { Option::none() }
            );
            let act01: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ v.mod_x(add(5)) } else { Option::none() }
            );
            let act1: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ MyBox { x : 5 } } else { Option::none() }
            );

            // Case 0-1-0-0: Box element, shared array, act01 succeeds.
            let case = "0-1-0-0";
            let arr = [MyBox { x : 0 }, MyBox { x : 3 }];
            let opt_arr = arr.act(0, act01);
            assert(|_|"Case " + case + "-a", opt_arr.is_some);;
            assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);;
            assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0).@x, 5);;
            assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1).@x, 3);;
            assert_eq(|_|"Case " + case + "-e", arr.@(0).@x + arr.@(1).@x, 3);;

            // Case 0-1-0-1: Box element, shared array, act0 fails.
            let case = "0-1-0-1";
            let arr = [MyBox { x : 1 }, MyBox { x : 3 }];
            let opt_arr = arr.act(0, act0);
            assert(|_|"Case " + case + "-a", opt_arr.is_none);;
            assert_eq(|_|"Case " + case + "-e", arr.@(0).@x + arr.@(1).@x, 4);;

            // Case 0-1-1-0: Box element, shared array, act1 succeeds.
            let case = "0-1-1-0";
            let arr = [MyBox { x : 0 }, MyBox { x : 3 }];
            let opt_arr = arr.act(0, act1);
            assert(|_|"Case " + case + "-a", opt_arr.is_some);;
            assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);;
            assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0).@x, 5);;
            assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1).@x, 3);;
            assert_eq(|_|"Case " + case + "-e", arr.@(0).@x + arr.@(1).@x, 3);;

            // Case 0-1-1-1: Box element, shared array, act1 fails.
            let case = "0-1-1-1";
            let arr = [MyBox { x : 1 }, MyBox { x : 3 }];
            let opt_arr = arr.act(0, act1);
            assert(|_|"Case " + case + "-a", opt_arr.is_none);;
            assert_eq(|_|"Case " + case + "-e", arr.@(0).@x + arr.@(1).@x, 4);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_act_3() {
    // Test Array::act
    let source = r#"
        module Main; 
        
        type MyBox = box struct { x : I64 };

        main : IO ();
        main = (
            let act0: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ v.assert_unique(|_|"not unique!").mod_x(add(5)) } else { Option::none() }
            );
            let act01: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ v.mod_x(add(5)) } else { Option::none() }
            );
            let act1: MyBox -> Option MyBox = |v| (
                if v.@x == 0 { Option::some $ MyBox { x : 5 } } else { Option::none() }
            );
            let act2: I64 -> Option I64 = |v| (
                if v == 0 { Option::some $ v + 5 } else { Option::none() }
            );

            // Case 1-0-0-0: Unboxed element, unique array, act2 succeeds.
            let case = "1-0-0-0";
            let arr = [0, 3];
            let opt_arr = arr.act(0, act2);
            assert(|_|"Case " + case + "-a", opt_arr.is_some);;
            assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);;
            assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0), 5);;
            assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1), 3);;

            // Case 1-0-0-1: Unboxed element, unique array, act2 fails.
            let case = "1-0-0-1";
            let arr = [1, 3];
            let opt_arr = arr.act(0, act2);
            assert(|_|"Case " + case + "-a", opt_arr.is_none);;

            // Case 1-1-0-0: Unboxed element, shared array, act2 succeeds.
            let case = "1-1-0-0";
            let arr = [0, 3];
            let opt_arr = arr.act(0, act2);
            assert(|_|"Case " + case + "-a", opt_arr.is_some);;
            assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);;
            assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0), 5);;
            assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1), 3);;
            assert_eq(|_|"Case " + case + "-e", arr.@(0) + arr.@(1), 3);;

            // Case 1-1-0-1: Unboxed element, shared array, act2 fails.
            let case = "1-1-0-1";
            let arr = [1, 3];
            let opt_arr = arr.act(0, act2);
            assert(|_|"Case " + case + "-a", opt_arr.is_none);;
            assert_eq(|_|"Case " + case + "-e", arr.@(0) + arr.@(1), 4);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_act_4() {
    // Test Array::act (case 2)
    let source = r#"
        module Main; 

        
        main : IO ();
        main = (
            let act2: I64 -> Option I64 = |v| (
                if v == 0 { Option::some $ v + 5 } else { Option::none() }
            );

            let case = "2-0-0";
            let arr = [[1, 2, 3], [4, 0, 6], [7, 8, 9]];
            let opt_arr = arr.act(1, act(1, act2));
            assert(|_|"Case " + case + "-a", opt_arr.is_some);;
            assert_eq(|_|"Case " + case + "-b", opt_arr.as_some, [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);;

            // Case 2-0-1: Fails updating an element of unique two-dimensional array by act2.
            let case = "2-0-1";
            let arr = [[1, 2, 3], [4, 1, 6], [7, 8, 9]];
            let opt_arr = arr.act(1, act(1, act2));
            assert(|_|"Case " + case + "-a", opt_arr.is_none);;

            // Case 2-1-0: Succeeds updating an element of shared two-dimensional array by act2.
            let case = "2-1-0";
            let arr = [[1, 2, 3], [4, 0, 6], [7, 8, 9]];
            let opt_arr = arr.act(1, act(1, act2));
            assert(|_|"Case " + case + "-a", opt_arr.is_some);;
            assert_eq(|_|"Case " + case + "-b", opt_arr.as_some, [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);;
            assert_eq(|_|"Case " + case + "-c", arr, [[1, 2, 3], [4, 0, 6], [7, 8, 9]]);;

            // Case 2-1-1: Fails updating an element of shared two-dimensional array by act2.
            let case = "2-1-1";
            let arr = [[1, 2, 3], [4, 1, 6], [7, 8, 9]];
            let opt_arr = arr.act(1, act(1, act2));
            assert(|_|"Case " + case + "-a", opt_arr.is_none);;
            assert_eq(|_|"Case " + case + "-c", arr, [[1, 2, 3], [4, 1, 6], [7, 8, 9]]);;

            // Case 3: `plug_in` is called multiple times.
            let case = "3";
            let arr = [[0], [1], [2]];
            let arr = arr.act(0, |x| [[], x, x.push_back(1)]);
            println(arr.to_string);;
            assert_eq(|_|"Case " + case, arr, [[[], [1], [2]], [[0], [1], [2]], [[0, 1], [1], [2]]]);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test124() {
    // Test Array : Functor, Array : Monad
    let source = r#"
        module Main; 

        
        main : IO ();
        main = (
            // flatten
            assert_eq(|_|"case 1", [[1,2,3], [], [4, 5, 6]].flatten, [1, 2, 3, 4, 5, 6]);;
            assert_eq(|_|"case 2", [[]].flatten, []: Array I64);;
            assert_eq(|_|"case 3", [].flatten, []: Array I64);;

            // bind
            let arr = do {
                let x = *[1,2,3];
                let y = *['a','b','c'];
                pure $ (x, y)
            };
            assert_eq(|_|"case 4", arr, [(1, 'a'), (1, 'b'), (1, 'c'), (2, 'a'), (2, 'b'), (2, 'c'), (3, 'a'), (3, 'b'), (3, 'c')]);;

            let arr = do {
                let x = *[1,2,3];
                [x, x]
            };
            assert_eq(|_|"case 5", arr, [1, 1, 2, 2, 3, 3]);;

            let arr = do {
                let x = *[1,2,3];
                []
            };
            assert_eq(|_|"case 6", arr, [] : Array I64);;

            let arr = do {
                let x = *[];
                [x]
            };
            assert_eq(|_|"case 7", arr, [] : Array I64);;

            // map
            assert_eq(|_|"case 8", [1, 2, 3].map(|i| i*i), [1, 4, 9]);;
            assert_eq(|_|"case 9", [].map(|i| i*i), [] : Array I64);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test125() {
    // Test () : Eq
    let source = r#"
        module Main; 

        
        main : IO ();
        main = (
            let arr = [(), ()];
            assert_eq(|_|"", arr.@(0), arr.@(1));;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_sum() {
    // Test Iterator::sum.
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            // Integer
            let n = 100;
            let v = Iterator::range(0, n+1).sum;
            assert_eq(|_|"", v, n*(n+1)/2);;

            // String
            let v = ["Hello", " ", "World!"].to_iter.sum;
            assert_eq(|_|"", v, "Hello World!");;

            // Array
            let v = [[1,2,3], [4,5,6]].to_iter.sum;
            assert_eq(|_|"", v, [1,2,3,4,5,6]);;

            // DynIterator
            let v = [range(0, 3).to_dyn, range(3, 6).to_dyn].to_iter.sum;
            assert_eq(|_|"", v.to_array, [0,1,2,3,4,5]);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_trait_alias() {
    // Test trait alias.
    // Basic example are Additive and Iterator::sum, which are tested in other tests.
    let source = r#"
        module Main; 
        
        // Higher kinded trait alias.

        trait [f : * -> *] f : MyFunctorZero {
            my_fzero : f a;
        }
        trait [f : * -> *] f : MyFunctorPlus {
            my_fplus : f a -> f a -> f a;
        }
        trait MyMonadAdditive = Monad + MyFunctorZero + MyFunctorPlus;

        impl Option : MyFunctorZero {
            my_fzero = Option::none();
        }
        impl Option : MyFunctorPlus {
            my_fplus = |rhs, lhs| if lhs.is_some { lhs } else { rhs };
        }
        my_msum : [m : MyMonadAdditive, it : Iterator, Item it = m a] it -> m a;
        my_msum = |iter| (
            let next = iter.advance;
            if next.is_none { my_fzero };
            let (iter, act) = next.as_some;
            act.my_fplus(my_msum(iter))
        );

        // Using trait alias as precondition of trait implementation.

        type Vector2 a = box struct { x : a, y : a };
        impl [a : Additive] Vector2 a : Zero {
            zero = Vector2 { x : Zero::zero, y : Zero::zero };
        }
        impl [a : Additive] Vector2 a : Add {
            add = |lhs, rhs| Vector2 { x : lhs.@x + rhs.@x, y : lhs.@y + rhs.@y };
        }

        // Define trait alias of a trait alias.
        trait MyAdditive = Additive;
        my_sum : [a : MyAdditive] Array a -> a;
        my_sum = |arr| (
            loop((0, Zero::zero), |(i, sum)| (
                if i == arr.get_size { break $ sum };
                continue $ (i+1, sum + arr.@(i))
            ))
        );

        // Error (cannot implement trait alias directly)
        // impl [a : Additive] Vector2 a : Additive {}

        // Error (circular aliasing)
        // trait MyTraitA = MyTraitB + ToString;
        // trait MyTraitB = MyTraitA + Eq;

        main : IO ();
        main = (
            let sum_vec = [Vector2{x : 1, y : 2}, Vector2{x : 3, y : 4}].to_iter.sum;
            assert_eq(|_|"case 1", sum_vec.@x, 4);;
            assert_eq(|_|"case 2", sum_vec.@y, 6);;

            let opts = [Option::some(1), Option::some(2)].to_iter;
            let opt_sum = opts.my_msum;
            assert_eq(|_|"case 3", opt_sum.as_some, 1);;

            let opts = [Option::none(), Option::some(2)].to_iter;
            let opt_sum = opts.my_msum;
            assert_eq(|_|"case 4", opt_sum.as_some, 2);;

            let opts = [Option::none(), Option::none()].to_iter;
            let opt_sum : Option I64 = opts.my_msum;
            assert_eq(|_|"case 5", opt_sum.is_none, true);;

            let opts = [].to_iter;
            let opt_sum : Option I64 = opts.my_msum;
            assert_eq(|_|"case 6", opt_sum.is_none, true);;

            assert_eq(|_|"case 7", [1,2,3,4,5].my_sum, 1+2+3+4+5);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_trait_alias_kind_mismatch() {
    let source = r#"
        module Main; 
        
        trait BadTrait = Functor + ToString;

        main : IO ();
        main = pure();
    "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Kind mismatch in the definition of trait alias `Main::BadTrait`.",
    );
}

#[test]
pub fn test_trait_alias_implement_trait_alias_directly() {
    let source = r#"
        module Main; 
        
        type Vector2 a = box struct { x : a, y : a };
        impl [a : Additive] Vector2 a : Zero {
            zero = Vector2 { x : Zero::zero, y : Zero::zero };
        }
        impl [a : Additive] Vector2 a : Add {
            add = |lhs, rhs| Vector2 { x : lhs.@x + rhs.@x, y : lhs.@y + rhs.@y };
        }

        // Error (cannot implement trait alias directly)
        impl [a : Additive] Vector2 a : Additive {}

        main : IO ();
        main = (
            pure()
        );
    "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "A trait alias cannot be implemented directly. Implement each aliased trait instead.",
    );
}

#[test]
pub fn test_trait_alias_circular_aliasing() {
    let source = r#"
        module Main; 
        
        // Error (circular aliasing)
        trait MyTraitA = MyTraitB + ToString;
        trait MyTraitB = MyTraitA + Eq;

        main : IO ();
        main = (
            pure()
        );
    "#;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Circular aliasing detected in trait alias `Main::MyTrait",
    );
}

#[test]
pub fn test129() {
    // Test ToBytes/FromBytes
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            // U8
            let case = "U8";
            let n = 1;
            let x = 127_U8;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            let y : Result ErrMsg U8 = Array::fill(n-1, 127_U8).from_bytes;
            assert(|_|case + " 2", y.is_err);;

            // I8
            let case = "I8";
            let n = 1;
            let x = 127_U8;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            let y : Result ErrMsg I8 = Array::fill(n-1, 127_U8).from_bytes;
            assert(|_|case + " 2", y.is_err);;

            // U16
            let case = "U16";
            let n = 2;
            let x = 65535_U16;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            // let y : Result ErrMsg U16 = Array::fill(n-1, 127_U8).from_bytes;
            // assert(|_|case + " 2", y.is_err);;

            // I16
            // let case = "I16";
            // let n = 2;
            // let x = -32768_I16;
            // assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            // let y : Result ErrMsg I16 = Array::fill(n-1, 127_U8).from_bytes;
            // assert(|_|case + " 2", y.is_err);;

            // U32
            let case = "U32";
            let n = 4;
            let x = 90123456_U32;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            let y : Result ErrMsg U32 = Array::fill(n-1, 127_U8).from_bytes;
            assert(|_|case + " 2", y.is_err);;

            // I32
            let case = "I32";
            let n = 4;
            let x = -12345678_I32;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            let y : Result ErrMsg I32 = Array::fill(n-1, 127_U8).from_bytes;
            assert(|_|case + " 2", y.is_err);;

            // U64
            let case = "U64";
            let n = 8;
            let x = 123456789012345678_U64;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            let y : Result ErrMsg U64 = Array::fill(n-1, 127_U8).from_bytes;
            assert(|_|case + " 2", y.is_err);;

            // I64
            let case = "I64";
            let n = 8;
            let x = 123456789012345678_I64;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            let y : Result ErrMsg I64 = Array::fill(n-1, 127_U8).from_bytes;
            assert(|_|case + " 2", y.is_err);;

            // F32
            let case = "F32";
            let n = 4;
            let x = 3.14_F32;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            let y : Result ErrMsg F32 = Array::fill(n-1, 127_U8).from_bytes;
            assert(|_|case + " 2", y.is_err);;

            // F64
            let case = "F64";
            let n = 8;
            let x = 3.14_F64;
            assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);;
            let y : Result ErrMsg F64 = Array::fill(n-1, 127_U8).from_bytes;
            assert(|_|case + " 2", y.is_err);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_consumed_time() {
    // Test Debug module
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            let (r, t) = consumed_time_while_lazy(|_| (
                loop((0, 0), |(i, sum)| if i == 1000000000 { break $ sum } else { continue $ (i + 1, sum + i) })
            ));
            println("loop time : " + t.to_string + ", sum : " + r.to_string);;

            let (_, t) = *consumed_time_while_io(
                let file_path = "test_tMB3iCfTeeES.txt";
                write_file_string(file_path, "Hello World!").try(exit_with_msg(1));;
                let read_content = *read_file_string(file_path).try(exit_with_msg(1));
                println $ read_content
            );
            println("write/read/println time : " + t.to_string);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
    remove_file("test_tMB3iCfTeeES.txt").unwrap();
}

#[test]
pub fn test_signed_integral_abs() {
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            assert_eq(|_|"", -123_I8.abs, 123_I8);;
            assert_eq(|_|"", 123_I8.abs, 123_I8);;

            assert_eq(|_|"", -123_I16.abs, 123_I16);;
            assert_eq(|_|"", 123_I16.abs, 123_I16);;

            assert_eq(|_|"", -123_I32.abs, 123_I32);;
            assert_eq(|_|"", 123_I32.abs, 123_I32);;

            assert_eq(|_|"", -123.abs, 123);;
            assert_eq(|_|"", 123.abs, 123);;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_float_to_string_precision() {
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let x = 3.14_F32;
            assert_eq(|_|"case to_string_precision F32 0", x.to_string_precision(0_U8), "3");;
            assert_eq(|_|"case to_string_precision F32 255", x.to_string_precision(255_U8), "3.140000104904174804687500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");;

            let x = -3.14;
            assert_eq(|_|"case to_string_precision F64 0", x.to_string_precision(0_U8), "-3");;
            assert_eq(|_|"case to_string_precision F64 255", x.to_string_precision(255_U8), "-3.140000000000000124344978758017532527446746826171875000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_float_to_string_exp() {
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let x = 123.45_F32;
            assert_eq(|_|"case to_string_exp F32", x.to_string_exp, "1.234500e+02");;

            let x = -123.45_F64;
            assert_eq(|_|"case to_string_exp F64", x.to_string_exp, "-1.234500e+02");;
        
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_float_to_string_exp_precision() {
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let x = 123.45_F32;
            assert_eq(|_|"", x.to_string_exp_precision(0_U8), "1e+02");;
            assert_eq(|_|"", x.to_string_exp_precision(255_U8), "1.234499969482421875000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e+02");;

            let x = -123.45_F64;
            assert_eq(|_|"", x.to_string_exp_precision(0_U8), "-1e+02");;
            assert_eq(|_|"", x.to_string_exp_precision(255_U8), "-1.234500000000000028421709430404007434844970703125000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e+02");;
        
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_unsafe_from_c_str_ptr() {
    let source = r#"
        module Main;
        
        main : IO ();
        main = (
            let src = "Hello World!";
            let cpy = src.borrow_c_str(String::_unsafe_from_c_str_ptr);
            assert_eq(|_|"", src, cpy);;
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_loop_lines() {
    let source = r#"
    module Main;
    
    sum_up_while : Path -> IOFail I64;
    sum_up_while = |file_path| (
        // Process lines of a file.
        with_file(file_path, "r", |file| (
            loop_lines(file, 0, |cnt, line| (
                // Sum up the number while line can be parsed as an integer.
                let parse_res = from_string(line.strip_last_spaces); // Remove the trailing newline ("\n") and parse as `I64`.
                if parse_res.is_ok {
                    let res = parse_res.as_ok;
                    continue $ cnt + res
                } else {
                    break $ cnt
                }
            ))
        ))
    );
    
    main : IO ();
    main = (
        let file_path = "test_GndeZP399tLX.txt";
        do {
            write_file_string(file_path, ["0", "1", "2", "X", "3", "4"].to_iter.join("\n"));;
            assert_eq(|_|"", *sum_up_while(file_path), 0 + 1 + 2).lift;;

            write_file_string(file_path, ["0", "1", "2", "3", "4"].to_iter.join("\n"));;
            assert_eq(|_|"", *sum_up_while(file_path), 0 + 1 + 2 + 3 + 4).lift;;

            write_file_string(file_path, [].to_iter.join("\n"));;
            assert_eq(|_|"", *sum_up_while(file_path), 0).lift;;

            pure()
        }.try(exit_with_msg(1))
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
    remove_file("test_GndeZP399tLX.txt").unwrap();
}

#[test]
pub fn test_array_get_sub() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        // Unboxed case
        let arr = [0, 1, 2, 3, 4];
        let n = arr.get_size;
        assert_eq(|_|"1", arr.get_sub(2, 4), [2, 3]);;
        assert_eq(|_|"2", arr.get_sub(0, 0), []);;
        assert_eq(|_|"3", arr.get_sub(3, n+1), [3, 4]);;
        assert_eq(|_|"4", arr.get_sub(1, n-1), [1, 2, 3]);;
        assert_eq(|_|"5", arr.get_sub(0, n), [0, 1, 2, 3, 4]);;
    
        let arr : Array I64 = [];
        assert_eq(|_|"6", arr.get_sub(2, 4), []);;
    
        // Boxed case
        let arr = [[0], [1], [2], [3], [4]];
        let n = arr.get_size;
        assert_eq(|_|"7", arr.get_sub(2, 4), [[2], [3]]);;
        assert_eq(|_|"8", arr.get_sub(0, 0), []);;
        assert_eq(|_|"9", arr.get_sub(3, n+1), [[3], [4]]);;
        assert_eq(|_|"10", arr.get_sub(1, n-1), [[1], [2], [3]]);;
        assert_eq(|_|"11", arr.get_sub(0, n), [[0], [1], [2], [3], [4]]);;
    
        let arr : Array (Array I64) = [];
        assert_eq(|_|"12", arr.get_sub(2, 4), []);;
    
        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_get_sub() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        let str = "Hello";
        let n = str.get_size;
        assert_eq(|_|"", str.get_sub(2, 4), "ll");;
        assert_eq(|_|"", str.get_sub(0, 0), "");;
        assert_eq(|_|"", str.get_sub(3, n+1), "lo");;
        assert_eq(|_|"", str.get_sub(1, n-1), "ell");;
    
        assert_eq(|_|"", "".get_sub(2, 4), "");;
    
        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_strip_first_spaces() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", "".strip_first_spaces, "");;
        assert_eq(|_|"", "Hello".strip_first_spaces, "Hello");;
        assert_eq(|_|"", " Hello".strip_first_spaces, "Hello");;
        assert_eq(|_|"", " \tHello".strip_first_spaces, "Hello");;
        assert_eq(|_|"", " ".strip_first_spaces, "");;
        assert_eq(|_|"", "  ".strip_first_spaces, "");;
    
        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_loop_lines_io() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        let content1 = "Hello\nWorld!";
        let file1 = "test_MsuHh3QEXKYN.txt";
        let file2 = "test_9A5bu4U57xTd.txt";
        do {
            write_file_string(file1, content1);;

            with_file(file1, "r", |file1| (
                with_file(file2, "w", |file2| (
                    loop_lines_io(file1, (), |_, line| (
                        continue_m $ *write_string(file2, line)
                    ))
                ))
            ));;

            let content2 = *read_file_string(file2);

            assert_eq(|_|"", content2, content1).lift;;

            pure()
        }.try(exit_with_msg(1))
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
    remove_file("test_MsuHh3QEXKYN.txt").unwrap();
    remove_file("test_9A5bu4U57xTd.txt").unwrap();
}

#[test]
pub fn test_string_find() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"1", "abcdef".find("ab", 0), Option::some(0));;
        assert_eq(|_|"2", "abcdef".find("bc", 0), Option::some(1));;
        assert_eq(|_|"3", "abcdef".find("ef", 0), Option::some(4));;
        assert_eq(|_|"4", "abcdef".find("xyz", 0), Option::none());;
        assert_eq(|_|"5", "abcdef".find("", 0), Option::some(0));;
        assert_eq(|_|"6", "".find("xyz", 0), Option::none());;
        assert_eq(|_|"7", "".find("", 0), Option::some(0));;

        assert_eq(|_|"8", "abcdef".find("ab", 1), Option::none());;
        assert_eq(|_|"9", "abcdef".find("bc", 1), Option::some(1));;
        assert_eq(|_|"10", "abcdef".find("ef", 1), Option::some(4));;
        assert_eq(|_|"11", "abcdef".find("xyz", 1), Option::none());;
        assert_eq(|_|"12", "abcdef".find("", 1), Option::some(1));;
        assert_eq(|_|"13", "".find("xyz", 1), Option::none());;
        assert_eq(|_|"14", "".find("", 1), Option::some(0));;

        assert_eq(|_|"15", "abcdef".find("ab", 7), Option::none());;
        assert_eq(|_|"16", "abcdef".find("bc", 7), Option::none());;
        assert_eq(|_|"17", "abcdef".find("ef", 7), Option::none());;
        assert_eq(|_|"18", "abcdef".find("xyz", 7), Option::none());;
        assert_eq(|_|"19", "abcdef".find("", 7), Option::some(6));;
        assert_eq(|_|"20", "".find("xyz", 7), Option::none());;
        assert_eq(|_|"21", "".find("", 7), Option::some(0));;

        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_names_literal_prefix() {
    let source = r#"
    module Main;
    
    true_global_val : I64;
    true_global_val = 42;

    false_global_val : I64;
    false_global_val = 0;

    nullptr_global_val : I64;
    nullptr_global_val = 0;
    
    main : IO ();
    main = (
        let true_local_num = 42;
        let false_local_num = 0;
        let nullptr_local_num = 0;

        assert_eq(|_|"", true_global_val + false_global_val + nullptr_global_val + true_local_num + false_local_num + nullptr_local_num, 42 + 42);;

        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_split() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (

        assert_eq(|_|"Ex. 1", "ab,c,".split(",").to_array, ["ab", "c", ""]);;
        assert_eq(|_|"Ex. 2", "abc".split(",").to_array, ["abc"]);;
        assert_eq(|_|"Ex. 3", "abc".split("").to_array, ["a", "b", "c"]);; // Special behavior when the separator is empty.

        assert_eq(|_|"1", "--ab---cde----".split("--").to_array, ["", "ab", "-cde", "", ""]);;
        assert_eq(|_|"2", "ab---cde----".split("--").to_array, ["ab", "-cde", "", ""]);;
        assert_eq(|_|"3", "--ab---cde".split("--").to_array, ["", "ab", "-cde"]);;
        assert_eq(|_|"3", "ab---cde".split("--").to_array, ["ab", "-cde"]);;
        assert_eq(|_|"4", "--".split("--").to_array, ["", ""]);;
        assert_eq(|_|"5", "a".split("--").to_array, ["a"]);;
        assert_eq(|_|"6", "".split("--").to_array, [""]);;

        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_ptr_to_string() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", nullptr.add_offset(3134905646).to_string, "00000000badadd2e");;
        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_tarai() {
    let source = r#"
    module Main;
    
    tarai : (I64, I64, I64) -> I64;
    tarai = |(x, y, z)| (
        if x <= y {
            y
        } else {
            let a = tarai $ (x-1, y, z);
            let b = tarai $ (y-1, z, x);
            let c = tarai $ (z-1, x, y);
            tarai $ (a, b, c)
        }
    );

    main : IO ();
    main = (
        let n = tarai $ (12, 6, 0);
        assert_eq(|_|"", n, 12);;
        pure()
    );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_get_args() {
    let source = r##"
    module Main;

    main : IO ();
    main = (
        let args = *get_args;
        args.to_iter.join(", ").println
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_float_inf_nan() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", F32::infinity.to_string, "inf");;

        assert_eq(|_|"", F64::infinity.to_string, "inf");;

        assert_eq(|_|"", (-F32::infinity).to_string, "-inf");;

        assert_eq(|_|"", (-F64::infinity).to_string, "-inf");;

        assert_eq(|_|"", F32::quiet_nan.to_bytes, [255_U8, 255_U8, 255_U8, 127_U8]);;

        assert_eq(|_|"", F64::quiet_nan.to_bytes, [255_U8, 255_U8, 255_U8, 255_U8, 255_U8, 255_U8, 255_U8, 127_U8]);;

        assert_eq(|_|"", F32::quiet_nan.to_string, "nan");;

        assert_eq(|_|"", F64::quiet_nan.to_string, "nan");;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_large_tuple() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        let x = (1,2,3,4,5,6,7,8,9,10);
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_fold() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        let n = 100;
        let ans = n * (n - 1) / 2;
        let res = Iterator::range(0, n).fold(0, |sum, i| sum + i);
        assert_eq(|_|"", res, ans);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_loop_iter() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        let n = 100;
        let ans = n * (n - 1) / 2;

        let res = Iterator::range(0, 2*n).loop_iter(0, |i, sum| if i == n { break(sum) } else { continue(sum + i) });
        assert_eq(|_|"", res, ans);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_to_array() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (

        let arr = ["0", "1", "2", "3", "4"];
        assert_eq(|_|"", arr.to_iter.to_array, arr);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_flatten() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", [[1, 2, 3], [], [4, 5, 6]].to_iter.map(to_iter).flatten.to_array, [1, 2, 3, 4, 5, 6]);;
        assert_eq(|_|"", [[] : Array I64].to_iter.map(Array::to_iter).flatten.to_array, []);;
        assert_eq(|_|"", ([] : Array (Array I64)).to_iter.map(Array::to_iter).flatten.to_array, []);;
        assert_eq(|_|"", [Iterator::range(1, 4), Iterator::range(4, 4), Iterator::range(4, 7)].to_iter.flatten.to_array, Iterator::range(1, 7).to_array);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_run_examples() {
    test_files_in_directory(Path::new("./examples"));
}

// Run all "*.fix" files in the specified directory.
// If the directory contains subdirectories, run Fix program consists of all "*.fix" files in each subdirectory.
pub fn test_files_in_directory(path: &Path) {
    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let mut config = Configuration::compiler_develop_mode();
        if path.is_dir() {
            // Skip hidden directories.
            if path.file_name().unwrap().to_str().unwrap().starts_with(".") {
                continue;
            }

            // For each directory in "tests" directory, run Fix program which consists of "*.fix" files in the directory.
            let files = fs::read_dir(&path).unwrap();
            for file in files {
                let file = file.unwrap().path();
                if file.extension().is_none() || file.extension().unwrap() != "fix" {
                    continue;
                }
                config.source_files.push(file);
            }
        } else {
            // For each file which has extention "fix" in "tests" directory, run it as Fix program.
            if path.extension().is_none() || path.extension().unwrap() != "fix" {
                continue;
            }
            config.source_files.push(path.clone());
        }
        println!("[{}]:", path.to_string_lossy().to_string());
        run(config);
        remove_file("test_process_text_file.txt").unwrap_or(());
    }
}

#[test]
pub fn test_comment_0() {
    // block comment
    let source = r"/* head */ module Main;
        main : IO ();
        main = (
            let x = 5 in 
            let y = -3 in
            /* If the closing symbol is put on the end of this line, g will evaluate.
            let g = fix \f -> \x -> if x == 0 {0} else {add x (f (add x -1))};
            g 100
            /* */
            //
            /*
            multiple line 
            block comment
            */
            let z = /* sub 1 */add(x,/* This comment is parsed as a separater */y)/* comment */;
            pure()
        );
        /*tail*/";
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_comment_1() {
    // ilne comment
    let source = r"
        module Main; //// /* */
        main : IO ();
        main = (
            let x = 5 in
            // let x = 3 in
// some excellent and brilliant comment
            let y = -3 in// comment
            let z = add(x, y);
            pure()
        //
        );";
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_hex_oct_bin_lit() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", 0x0, 0);;
        assert_eq(|_|"", 0o0, 0);;
        assert_eq(|_|"", 0b0, 0);;
        assert_eq(|_|"", -0x0, 0);;
        assert_eq(|_|"", -0o0, 0);;
        assert_eq(|_|"", -0b0, 0);;
        assert_eq(|_|"", 0x0123456789abcdef, 81985529216486895);;
        assert_eq(|_|"", 0x0123456789ABCDEF, 81985529216486895);;
        assert_eq(|_|"", 0o01234567, 342391);;
        assert_eq(|_|"", 0b01, 1);;
        assert_eq(|_|"", -0x0123456789abcdef, -81985529216486895);;
        assert_eq(|_|"", -0x0123456789ABCDEF, -81985529216486895);;
        assert_eq(|_|"", -0o01234567, -342391);;
        assert_eq(|_|"", -0b01, -1);;
        assert_eq(|_|"", 0xdeadbeef, 3735928559);;
        assert_eq(|_|"", 0o33653337357, 3735928559);;
        assert_eq(|_|"", 0b11011110101011011011111011101111, 3735928559);;
        assert_eq(|_|"", 0x7FFFFFFFFFFFFFFF, 9223372036854775807);;
        assert_eq(|_|"", -0x8000000000000000, -9223372036854775808);;
        assert_eq(|_|"", 0o0777777777777777777777, 9223372036854775807);;
        assert_eq(|_|"", -0o1000000000000000000000, -9223372036854775808);;
        assert_eq(|_|"", 0b0111111111111111111111111111111111111111111111111111111111111111, 9223372036854775807);;
        assert_eq(|_|"", -0b1000000000000000000000000000000000000000000000000000000000000000, -9223372036854775808);;
        assert_eq(|_|"", 0xFFFFFFFFFFFFFFFF_U64, 18446744073709551615_U64);;
        assert_eq(|_|"", 0o1777777777777777777777_U64, 18446744073709551615_U64);;
        assert_eq(|_|"", 0b1111111111111111111111111111111111111111111111111111111111111111_U64, 18446744073709551615_U64);;
        assert_eq(|_|"", 0x7FFFFFFF_I32, 2147483647_I32);;
        assert_eq(|_|"", -0x80000000_I32, -2147483648_I32);;
        assert_eq(|_|"", 0xFFFFFFFF_U32, 4294967295_U32);;
        assert_eq(|_|"", 0xffffffffffffffff, -1);;
        assert_eq(|_|"", 0b11111111_I8, -1_I8);;
        assert_eq(|_|"", 0x000ffffffffffffffff, -1);;
        assert_eq(|_|"", 0b00011111111_I8, -1_I8);;
        assert_eq(|_|"", 0o377_U8, 255_U8);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_integer_string_literal_error0() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", 0xffffffffffffffffff, -1);; // out of range of `I64`
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "out of range",
    );
}

#[test]
pub fn test_integer_string_literal_error1() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", 0xffffffffffffffffff_U64, -1);; // out of range of `U64`
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "out of range",
    );
}

#[test]
pub fn test_integer_string_literal_error2() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", 256_I8, -1);; // out of range of `I8`
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "out of range",
    );
}

#[test]
pub fn test_integer_string_literal_error3() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", 256_U8, -1);; // out of range of `I8`
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "out of range",
    );
}

#[test]
pub fn test_integer_string_literal_error4() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", 0o377_I8, -1);; // out of range of `I8`
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "out of range",
    );
}

#[test]
pub fn test_array_to_string() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", ([] : Array Bool).to_string, "[]");;
        assert_eq(|_|"", [1, 2, 3].to_string, "[1, 2, 3]");;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_option_to_string() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", (Option::none() : Option Bool).to_string, "none()");;
        assert_eq(|_|"", (Option::some(42) : Option I64).to_string, "some(42)");;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unit_tuple_to_string() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", ().to_string, "()");;
        assert_eq(|_|"", (42, true).to_string, "(42, true)");;
        assert_eq(|_|"", (42, true, "truth").to_string, "(42, true, truth)");;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unit_tuple_eq() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert(|_|"", () == ());;
        assert(|_|"", (42, true) == (42, true));;
        assert(|_|"", (0, true) != (42, true));;
        assert(|_|"", (42, false) != (42, true));;

        assert(|_|"", (42, true, "truth") == (42, true, "truth"));;
        assert(|_|"", (0, true, "truth") != (42, true, "truth"));;
        assert(|_|"", (42, false, "truth") != (42, true, "truth"));;
        assert(|_|"", (42, false, "falsy") != (42, true, "truth"));;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_tuple_less_than_and_less_than_or_eq() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"", (1, 2) < (2, 1), true);;
        assert_eq(|_|"", (2, 1) < (1, 2), false);;
        assert_eq(|_|"", (1, 2) < (1, 1), false);;
        assert_eq(|_|"", (1, 1) < (1, 2), true);;
        assert_eq(|_|"", (1, 1) < (1, 1), false);;

        assert_eq(|_|"", (1, 2) <= (2, 1), true);;
        assert_eq(|_|"", (2, 1) <= (1, 2), false);;
        assert_eq(|_|"", (1, 2) <= (1, 1), false);;
        assert_eq(|_|"", (1, 1) <= (1, 2), true);;
        assert_eq(|_|"", (1, 1) <= (1, 1), true);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_result_to_string() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        let res : Result String Bool = Result::ok(true);
        assert_eq(|_|"", res.to_string, "ok(true)");;
        let res : Result String Bool = Result::err("error");
        assert_eq(|_|"", res.to_string, "err(error)");;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_result_eq() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        let ok1 = Result::ok(true);
        let ok2 = Result::ok(false);
        let err1 = Result::err("err1");
        let err2 = Result::err("err2");

        let ress = [ok1, ok2, err1, err2];

        let indices = do {
            let i = *Iterator::range(0, ress.get_size).to_dyn;
            let j = *Iterator::range(0, ress.get_size).to_dyn;
            pure $ (i, j)
        };
        indices.loop_iter_m((), |(i, j), _| (
            if i == j {
                assert_eq(|_|"", ress.@(i) == ress.@(j), true)
            } else {
                assert_eq(|_|"", ress.@(i) == ress.@(j), false)
            };;
            continue_m $ ()
        ));;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_less_than_and_less_than_or_eq() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        let arr1 = [];
        let arr2 = [1];
        let arr3 = [1, 1];
        let arr4 = [1, 2];
        let arr5 = [2];

        let arrs = [arr1, arr2, arr3, arr4, arr5];

        let indices = do {
            let i = *Iterator::range(0, arrs.get_size).to_dyn;
            let j = *Iterator::range(0, arrs.get_size).to_dyn;
            pure $ (i, j)
        };
        indices.loop_iter_m((), |(i, j), _| (
            assert_eq(|_|"", arrs.@(i) < arrs.@(j), i < j);;
            assert_eq(|_|"", arrs.@(i) <= arrs.@(j), i <= j);;
            continue_m $ ()
        ));;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_less_than_and_less_than_or_eq() {
    let source = r##"
    module Main;
        
    main : IO ();
    main = (
        let s1 = "";
        let s2 = "a";
        let s3 = "ab";
        let s4 = "ac";
        let s5 = "b";

        let ss = [s1, s2, s3, s4, s5];

        let indices = do {
            let i = *Iterator::range(0, ss.get_size).to_dyn;
            let j = *Iterator::range(0, ss.get_size).to_dyn;
            pure $ (i, j)
        };
        indices.loop_iter_m((), |(i, j), _| (
            assert_eq(|_|"", ss.@(i) < ss.@(j), i < j);;
            assert_eq(|_|"", ss.@(i) <= ss.@(j), i <= j);;
            continue_m $ ()
        ));;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_overlapping_trait_and_function() {
    let source = r##"
    module Main;
        
    trait a : Show {
        show : a -> String;
    }
    
    impl I64 : Show {
        show = |val| "(trait) " + val.to_string;
    }
    
    show : I64 -> String;
    show = |val| "(function) " + val.to_string;
    
    main : IO ();
    main = (
        assert_eq(|_|"", Main::show(42), "(function) 42");;
        assert_eq(|_|"", Show::show(42), "(trait) 42");;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_orphan_rule_1() {
    let source = r##"
    module Main;

    impl Array a : ToString {
        to_string = "array!";
    }
    
    main : IO ();
    main = (
        println $ [1,2,3].to_string
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Implementing trait `Std::ToString` for type `Std::Array a` in module `Main` is illegal; it is not allowed to implement an external trait for an external type.",
    );
}

#[test]
pub fn test_orphan_rule_2() {
    let source = r##"
    module Main;

    type MyType = unbox struct { data : () };

    impl Array MyType : ToString {
        to_string = "array!";
    }
    
    main : IO ();
    main = (
        println $ [1,2,3].to_string
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Implementing trait `Std::ToString` for type `Std::Array Main::MyType` in module `Main` is illegal; it is not allowed to implement an external trait for an external type.",
    );
}

#[test]
pub fn test_orphan_rule_3() {
    let source = r##"
    module Main;

    type MyType = unbox struct { data : () };

    impl MyType -> MyType : ToString {
        to_string = "mytype!";
    }
    
    main : IO ();
    main = (
        println $ [1,2,3].to_string
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Implementing trait `Std::ToString` for type `Main::MyType -> Main::MyType` in module `Main` is illegal; it is not allowed to implement an external trait for an external type.",
    );
}

#[test]
pub fn test_implement_trait_on_arrow_1() {
    let source = r##"
    module Main;
    
    trait a : MyToString {
        to_string : a -> String;
    }

    impl [b : ToString] I64 -> b : MyToString {
        to_string = |f| "f(0) = " + f(0).ToString::to_string + ", f(1) = " + f(1).ToString::to_string;
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"fail", (|x| x + 1).to_string, "f(0) = 1, f(1) = 2");;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_implement_trait_on_arrow_2() {
    let source = r##"
    module Main;
    
    trait a : MyToString {
        to_string : a -> String;
    }

    impl a -> b : MyToString {
        to_string = |f| "arrow";
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"fail", (|x| x + 1).to_string, "arrow");;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_overlapping_instances_1() {
    let source = r##"
    module Main;
    
    trait a : MyToString {
        to_string : a -> String;
    }

    impl Array a : MyToString {
        to_string = |f| "array";
    }

    impl [a : ToString] Array a : MyToString {
        to_string = |f| "array";
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"fail", [1,2,3].to_string, "array");;
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Two trait implementations for `Main::MyToString` are overlapping.",
    );
}

#[test]
pub fn test_overlapping_instances_2() {
    let source = r##"
    module Main;
    
    trait a : MyToString {
        to_string : a -> String;
    }

    impl Array a : MyToString {
        to_string = |f| "array";
    }

    impl Array I64 : MyToString {
        to_string = |f| "array";
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"fail", [1,2,3].to_string, "array");;
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Two trait implementations for `Main::MyToString` are overlapping.",
    );
}

#[test]
pub fn test_overlapping_instances_3() {
    let source = r##"
    module Main;
    
    trait a : MyToString {
        to_string : a -> String;
    }

    impl a -> b : MyToString {
        to_string = |f| "arrow";
    }

    impl [b : ToString] I64 -> b : MyToString {
        to_string = |f| "f(0) = " + f(0).ToString::to_string + ", f(1) = " + f(1).ToString::to_string;
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"fail", [1,2,3].to_string, "array");;
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Two trait implementations for `Main::MyToString` are overlapping.",
    );
}

#[test]
pub fn test_overlapping_instances_4() {
    let source = r##"
    module Main;
    
    trait a : MyToString {
        to_string : a -> String;
    }

    impl [e : MyToString] Result e a : MyToString {
        to_string = |f| "result";
    }

    impl [a : MyToString] Result e a : MyToString {
        to_string = |f| "result";
    }

    impl I64 : MyToString {
        to_string = |_| "I64";
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"fail", Result::ok(1), "result");;
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Two trait implementations for `Main::MyToString` are overlapping.",
    );
}

#[test]
pub fn test_eval_non_unit() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        eval 1;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unrelated_trait_method() {
    let source = r##"
    module Main;

    trait a : MyTrait {
        value : I64;
    }
    
    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Type variable `a` used in trait definition has to appear in the type of a member `value`.",
    );
}

#[test]
pub fn test_duplicated_symbols() {
    let source = r##"
    module Main;

    truth : I64;
    truth = 42;

    truth : I64;
    truth = 0;
    
    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Duplicate definition for global value: `Main::truth`.",
    );
}

#[test]
pub fn test_duplicated_trait_member() {
    let source = r##"
    module Main;

    trait a : MyToString {
        to_string : a -> String;
        to_string : a -> String;
    }
    
    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Duplicate definitions of member `to_string`.",
    );
}

#[test]
pub fn test_duplicated_trait_member_impl() {
    let source = r##"
    module Main;

    trait a : MyToString {
        to_string : a -> String;
    }

    impl I64 : MyToString {
        to_string = |val| "(1) " + val.to_string;
        to_string = |val| "(2) " + val.to_string;
    }
    
    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Duplicate implementation of member `to_string`.",
    );
}

#[test]
pub fn test_typedef_unknown_tyvar() {
    let source = r##"
    module Main;

    type Hoge = unbox struct { data : a };
    
    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Unknown type variable `a` in the definition of type `Main::Hoge`.",
    );
}

#[test]
pub fn test_typedef_trait_precondition() {
    let source = r##"
    module Main;

    type [a : ToString] Hoge a = unbox struct { data : a };
    
    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "In the constraint of type definition, only kind signature is allowed.",
    );
}

#[test]
pub fn test_typedef_specify_kind_twice() {
    let source = r##"
    module Main;

    type [a : *, a : *->*] Hoge a = unbox struct { data : a };
    
    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Kind of type variable `a` is specified more than once.",
    );
}

#[test]
pub fn test_typedef_kind_mismatch() {
    let source = r##"
    module Main;

    type Hoge a b = unbox struct { data : a b };
    
    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Kind mismatch",
    );
}

#[test]
pub fn test_typedef_struct_higher_kinded_type_variable() {
    let source = r##"
    module Main;
    
    type [m : *->*] X m a = struct { data : m a };

    main : IO ();
    main = (
        let x : X IO I64 = X { data : pure(42) };
        assert_eq(|_|"", *x.@data, 42);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_state_t() {
    let source = r##"
    module Main;
    
    type [m : *->*] StateT m s a = unbox struct {
        runner : s -> m (a, s)
    };

    impl [m : Monad] StateT m s : Monad {
        pure = |a| StateT { runner : |s| pure $ (a, s) };
        bind = |f, m| StateT { runner : |s| (
            let (a, s) = *(m.@runner)(s);
            (f(a).@runner)(s)
        ) };
    }

    namespace StateT {

        get_state : [m : Monad] StateT m s s;
        get_state = StateT { runner : |s| pure $ (s, s) };
    
        set_state : [m : Monad] s -> StateT m s ();
        set_state = |s| StateT { runner : |_| pure $ ((), s) };

        lift : [m : Monad] m a -> StateT m s a;
        lift = |m| StateT { runner : |s| (
            let a = *m;
            pure $ (a, s)
        ) };
    }

    namespace IOCounter {

        increment : StateT IO I64 ();
        increment = (
            let state = *get_state;
            set_state(state + 1)
        );

        print_counter : StateT IO I64 ();
        print_counter = (
            let state = *get_state;
            lift $ println $ "Counter: " + state.to_string
        );
    }

    main : IO ();
    main = (
        let action = do {
            print_counter;;
            increment;;
            increment;;
            increment;;
            print_counter;;
            pure()
        };
        let ((), counter) = *(action.@runner)(0);
        assert_eq(|_|"", counter, 3);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_split_by_max_size() {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let result = split_by_max_size(v, 3);
    assert_eq!(result, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]);

    let v = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let result = split_by_max_size(v, 3);
    assert_eq!(result, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8]]);
}

#[test]
pub fn test_duplicated_struct_name() {
    let source = r##"
    module Main;
    
    type Hoge = unbox struct { data : I64 };
    type Hoge = unbox struct { data : I32 };

    main : IO ();
    main = (
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Duplicate definitions of type `Main::Hoge`.",
    );
}

#[test]
pub fn test_ambiguous_struct_name() {
    let source = r##"
    module Main;
    
    namespace A {
        type Hoge = unbox struct { data : I64 };
    }

    namespace B {
        type Hoge = unbox struct { data : I64 };
    }

    main : IO ();
    main = (
        let x = Hoge { data : 42 };
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Name `Hoge` is ambiguous: there are `Main::A::Hoge`, `Main::B::Hoge`.",
    );
}

#[test]
pub fn test_duplicated_trait_name() {
    let source = r##"
    module Main;
    
    trait a : Hoge {
        val : a;
    }
    trait a : Hoge {
        val : a;
    }

    main : IO ();
    main = (
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Duplicate definition for trait Main::Hoge.",
    );
}

#[test]
pub fn test_ambiguous_trait_name() {
    let source = r##"
    module Main;
    
    namespace A {
        trait a : Hoge {
            val : a;
        }
    }

    namespace B {
        trait a : Hoge {
            val : a;
        }
    }

    get_val : [a : Hoge] a;
    get_val = val;

    main : IO ();
    main = (
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Name `Hoge` is ambiguous: there are `Main::A::Hoge`, `Main::B::Hoge`.",
    );
}

#[test]
pub fn test_import_empty() {
    let source = r##"
    module Main;
    import Std::{};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Unknown type or associated type name",
    );
}

#[test]
pub fn test_import_any() {
    let source = r##"
    module Main;
    import Std::*;

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_hiding_any() {
    let source = r##"
    module Main;
    import Std::* hiding *;

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Unknown type or associated type name",
    );
}

#[test]
pub fn test_import_only_necessary() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::println, Array, U8};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_hierarchy() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, Array, U8, IO::{println, eprintln}};

    main : IO ();
    main = (
        eprintln("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_any_in_namespace() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::*, Array, U8};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_insufficient() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, IO::println};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Unknown type or associated type name `Std::String`.",
    );
}

#[test]
pub fn test_import_hiding_necessary() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::println} hiding IO;

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Unknown type or associated type name `IO`.",
    );
}

#[test]
pub fn test_import_hiding_unnecessary() {
    let source = r##"
    module Main;
    import Std hiding Tuple2;

    type Tuple2 a b = struct { fst : a, snd : b };

    impl [a : ToString, b : ToString] Tuple2 a b : ToString {
        to_string = |t| "(" + t.@fst.to_string + ", " + t.@snd.to_string + ")";
    }

    main : IO ();
    main = println $ Tuple2 { fst : "Hello", snd : "World!" }.to_string;
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_hiding_associated_type() {
    let source = r##"
    module Main;
    import Std hiding Iterator::Item;

    type Item = I64;

    main : IO ();
    main = (
        assert_eq(|_|"", 42 : Item, 42 : I64)
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_and_trait_name_collision() {
    let source = r##"
    module Main;

    type Piyo = unbox struct { data : String };
    trait a : Piyo {
        val : a;
    }

    main : IO ();
    main = (
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Name confliction: `Main::Piyo` is both a type and a trait.",
    );
}

#[test]
pub fn test_import_unknown_module() {
    let source = r##"
    module Main;

    import Piyo;

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Cannot find module `Piyo`.",
    );
}

#[test]
pub fn test_import_unknown_symbol() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, Monad::pure, piyo};

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Cannot find value named `Std::piyo`.",
    );
}

#[test]
pub fn test_import_unknown_symbol_hiding() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, Monad::pure} hiding piyo;

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Cannot find value named `Std::piyo`.",
    );
}

#[test]
pub fn test_import_unknown_type_or_trait() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, Monad::pure, Piyo};

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Cannot find entity named `Std::Piyo`.",
    );
}

#[test]
pub fn test_import_unknown_namespace() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, Monad::pure, Piyo::*};

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Namespace `Std::Piyo` is not defined or empty.",
    );
}

#[test]
pub fn test_associated_type_collects() {
    let source = r##"
    module Main;

    import Std::* hiding Indexable::Elem;
    
    trait c : Collects {
        type Elem c;
        empty : c;
        insert : Elem c -> c -> c;
        to_array : c -> Array (Elem c);
    }

    impl Array a : Collects {
        type Elem (Array a) = a;
        empty = [];
        insert = |x, xs| xs.push_back(x);
        to_array = |xs| xs;
    }

    impl DynIterator a : Collects {
        type Elem (DynIterator a) = a;
        empty = DynIterator::empty;
        insert = |x, xs| xs.push_front(x).to_dyn;
        to_array = Iterator::to_array;
    }

    triple : [c : Collects, Elem c = e] e -> e -> e -> c;
    triple = |x, y, z| Collects::empty.insert(x).insert(y).insert(z);

    extend : [c1 : Collects, c2 : Collects, Elem c1 = e, Elem c2 = e] c1 -> c2 -> c2;
    extend = |xs, ys| xs.to_array.to_iter.fold(ys, |x, ys| ys.insert(x));

    has_equal_elements : [c1 : Collects, c2 : Collects, Elem c1 = e, Elem c2 = e, e : Eq] c1 -> c2 -> Bool;
    has_equal_elements = |xs, ys| xs.to_array == ys.to_array;

    stringify : [c : Collects, Elem c = e, e : ToString] c -> String;
    stringify = |xs| xs.to_array.to_iter.map(to_string).join(", ");

    type Wrapper c = struct { data : c };

    impl [c : Collects, Elem c = e, e : ToString] Wrapper c : ToString {
        to_string = |xs| xs.@data.to_array.to_iter.map(to_string).join(", ");
    }

    impl [c : Collects] Wrapper c : Collects {
        type Elem (Wrapper c) = Elem c;
        empty = Wrapper { data : Collects::empty };
        insert = |x, w| Wrapper { data : w.@data.insert(x) };
        to_array = |w| w.@data.to_array;
    }

    sum_elements1 : [c : Collects, Elem c = I64] c -> I64;
    sum_elements1 = |xs| xs.to_array.to_iter.fold(0, |x, acc| acc + x);

    sum_elements3 : [c : Collects, Elem c = e, e : Additive] c -> Elem c;
    sum_elements3 = |xs| xs.to_array.to_iter.sum;

    main : IO ();
    main = (
        assert_eq(|_|"", [].insert(1).insert(2).insert(3), [1, 2, 3]);;
        assert_eq(|_|"", DynIterator::empty.insert(3).insert(2).insert(1).Collects::to_array, [1, 2, 3]);;
        assert_eq(|_|"", triple(1, 2, 3).extend([4, 5, 6]), [1, 2, 3, 4, 5, 6]);;
        assert_eq(|_|"", triple(1, 2, 3).extend([4, 5, 6].to_iter.to_dyn), [1, 2, 3, 4, 5, 6]);;
        assert_eq(|_|"", [1, 2, 3].to_iter.to_dyn.extend([4, 5, 6]).Collects::to_array, [6, 5, 4, 1, 2, 3]);;
        assert_eq(|_|"", [1, 2, 3].to_iter.to_dyn.extend([4, 5, 6].to_iter.to_dyn).Collects::to_array, [6, 5, 4, 1, 2, 3]);;
        assert_eq(|_|"", [1, 2, 3].has_equal_elements([1, 2, 3]), true);;
        assert_eq(|_|"", [1, 2, 3].stringify, "1, 2, 3");;
        assert_eq(|_|"", Wrapper { data : [false, true, true] }.to_string, "false, true, true");;
        assert_eq(|_|"", Wrapper { data : [false, true, true] }.to_array, [false, true, true]);;
        assert_eq(|_|"", [1, 2, 3].sum_elements1, 6);;
        assert_eq(|_|"", [1, 2, 3].sum_elements3, 6);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_associated_type_type_level_arithmetic() {
    let source = r##"
    module Main;
    
    type Zero = unbox struct { data : () };
    type Succ n = unbox struct { data : () };

    type One = Succ Zero;
    type Two = Succ One;
    type Three = Succ Two;

    type Value n = unbox struct { data : I64 };

    trait n : Nat {
        type Add n m;
        value : Value n;
    }
    impl Zero : Nat {
        type Add Zero m = m;
        value = Value { data : 0 };
    }
    impl [n : Nat] Succ n : Nat {
        type Add (Succ n) m = Succ (Add n m);
        value = (
            let n = (Nat::value : Value n).@data;
            Value { data : n + 1 }
        );
    }

    main : IO ();
    main = (
        assert_eq(|_|"", (Nat::value : Value Zero).@data, 0);;
        assert_eq(|_|"", (Nat::value : Value One).@data, 1);;
        assert_eq(|_|"", (Nat::value : Value Two).@data, 2);;
        assert_eq(|_|"", (Nat::value : Value (Add One Two)).@data, 3);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_extra_comma() {
    let source = r##"
    module Main;
    
    type MyStruct0 a b = struct { fst : a, snd : b, };
    type MyStruct1 a b = struct { fst : a, snd : b ,};
    type MyStruct2 a b = struct { fst : a, snd : b , };
    
    type MyUnion0 a b = union { fst : a, snd : b, };
    type MyUnion1 a b = union { fst : a, snd : b ,};
    type MyUnion2 a b = union { fst : a, snd : b , };

    main : IO ();
    main = (
        let _ = MyStruct0 { fst : 0, snd : false, };

        assert_eq(|_|"", [1, 2, 3,], [1, 2, 3]);;
        assert_eq(|_|"", [1, 2, 3, ], [1, 2, 3]);;
        assert_eq(|_|"", [,], [] : Array Bool);;

        assert_eq(|_|"", (42), 42);;

        let zero_tuple : () = ();
        assert_eq(|_|"", zero_tuple, ());;

        let two_tuple : (I64, I64, ) = (0, 1,);
        assert_eq(|_|"", two_tuple : (I64, I64), (0, 1));;

        let one_tuple : (I64,) = (0,);
        let one_tuple = one_tuple.set_0(42);
        assert_eq(|_|"", one_tuple, (42, ));;
        assert_eq(|_|"", one_tuple.to_string, "(42,)");;

        let unwrap_one_tuple = |(x,)| x;
        assert_eq(|_|"", one_tuple.unwrap_one_tuple, 42);;

        let (one_tuple_elem,) = one_tuple;
        assert_eq(|_|"", one_tuple_elem, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_textual_name_of_tuples() {
    let source = r##"
    module Main;
    
    compare_tuple2 : Tuple2 I64 Bool -> (I64, Bool) -> Bool;
    compare_tuple2 = |x, y| x == y;

    compare_tuple1 : Tuple1 I64 -> (I64,) -> Bool;
    compare_tuple1 = |x, y| x == y;

    compare_tuple0 : Tuple0 -> () -> Bool;
    compare_tuple0 = |x, y| x == y;

    main : IO ();
    main = (
        assert_eq(|_|"", compare_tuple2((42, true), (42, true)), true);;
        assert_eq(|_|"", compare_tuple1((42,), (42,)), true);;
        assert_eq(|_|"", compare_tuple0((), ()), true);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_product() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1, 2, 3].to_iter.product(['a', 'b'].to_iter).to_array, [(1, 'a'), (2, 'a'), (3, 'a'), (1, 'b'), (2, 'b'), (3, 'b')]);;
        assert_eq(|_|"", [1, 2, 3].to_iter.product(([] : Array U8).to_iter).to_array, []);;
        assert_eq(|_|"", ([] : Array I64).to_iter.product(['a', 'b'].to_iter).to_array, []);;
        assert_eq(|_|"", ([] : Array I64).to_iter.product(([] : Array U8).to_iter).to_array, []);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_filtermap() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        let n = 50;
        let (triples, time) = consumed_time_while_lazy(|_|
            let r1 = range(1, n);
            let r2 = range(1, 2*n*n);
            r1.product(r1).product(r2).filter_map(|((a, b), c)|
                if a > b { none() };
                if a*a + b*b != c*c { none() };
                some((a, b, c))
            ).to_array
        );
        println("pythagorean_triple: " + time.to_string);;
        assert_eq(|_|"pythagorean_triple", triples, [
            (3, 4, 5), (6, 8, 10), (5, 12, 13), (9, 12, 15), (8, 15, 17), (12, 16, 20), (15, 20, 25), (7, 24, 25), (10, 24, 26),
            (20, 21, 29), (18, 24, 30), (16, 30, 34), (21, 28, 35), (12, 35, 37), (15, 36, 39), (24, 32, 40), (9, 40, 41), (27, 36, 45),
            (30, 40, 50), (14, 48, 50), (24, 45, 51), (20, 48, 52), (28, 45, 53), (33, 44, 55), (40, 42, 58), (36, 48, 60)
        ]);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_get_size() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1, 2, 3].to_iter.get_size, 3);;
        assert_eq(|_|"", ([] : Array I64).to_iter.get_size, 0);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_empty() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", (empty : EmptyIterator I64).get_size, 0);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_pop_first() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", range(1, 4).pop_first.to_array, [2, 3]);;
        assert_eq(|_|"", range(0, 0).pop_first.to_array, []);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_is_empty() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", range(1, 4).is_empty, false);;
        assert_eq(|_|"", range(0, 0).is_empty, true);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_fold_m() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        range(0, 10).fold_m(0, |i, sum|
            let sum = sum + i;
            assert_eq(|_|"", sum, i * (i + 1) / 2);;
            pure $ sum
        );;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_loop_iter_m() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        range(0, 20).loop_iter_m(0, |i, sum|
            if i == 10 { break_m $ sum };
            let sum = sum + i;
            assert_eq(|_|"", sum, i * (i + 1) / 2);;
            continue_m $ sum
        );;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_generate() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        let iter = Iterator::generate(0, |_| Option::none());
        assert_eq(|_|"", iter.to_array, [] : Array I64);;

        let iter = Iterator::generate(0, |i| if i == 3 { Option::none() } else { Option::some $ (i+1, i) });
        assert_eq(|_|"", iter.to_array, [0, 1, 2]);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_intersperse() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", range(1, 4).intersperse(0).to_array, [1, 0, 2, 0, 3]);;
        assert_eq(|_|"", range(1, 2).intersperse(0).to_array, [1]);;
        assert_eq(|_|"", range(1, 1).intersperse(0).to_array, [] : Array I64);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_take() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", range(1, 4).take(2).to_array, [1, 2]);;
        assert_eq(|_|"", range(1, 4).take(0).to_array, [] : Array I64);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_zip() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", range(1, 4).zip(range(4, 10)).to_array, [(1, 4), (2, 5), (3, 6)]);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_enumerate() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", range(0, 0).enumerate.to_array, []);;
        assert_eq(|_|"", range(2, 5).enumerate.to_array, [(0, 2), (1, 3), (2, 4)]);;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_monad() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        let iter = do {
            let x = *Iterator::range(1, 4).to_dyn;
            let y = *['A', 'B'].to_iter.to_dyn;
            pure $ (x, y)
        };
        assert_eq(|_|"", iter.to_array, [(1, 'A'), (1, 'B'), (2, 'A'), (2, 'B'), (3, 'A'), (3, 'B')]);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_collect_m() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        let res = [some(1), some(2), some(3)].to_iter.collect_m;
        assert_eq(|_|"", res, some([1, 2, 3]));;

        let res = [some(1), some(2), none()].to_iter.collect_m;
        assert_eq(|_|"", res, none());;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_range() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"1", Iterator::range(0, -2).to_array, []);;
        assert_eq(|_|"1", Iterator::range(0, -1).to_array, []);;
        assert_eq(|_|"1", Iterator::range(0, 0).to_array, []);;
        assert_eq(|_|"1", Iterator::range(0, 1).to_array, [0]);;
        assert_eq(|_|"1", Iterator::range(0, 2).to_array, [0, 1]);;
        assert_eq(|_|"1", Iterator::range(0, 3).to_array, [0, 1, 2]);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_range_step_1() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"A-2", Iterator::range_step(0, 10, -1).get_size, 0);;
        // assert_eq(|_|"A-1", Iterator::range_step(0, 10, 0).take(100).get_size, 100);;
        // assert_eq(|_|"A0", Iterator::range_step(0, 10, 0).take(100).get_size, 100);;
        assert_eq(|_|"A1", Iterator::range_step(0, 10, 1).to_array, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);;
        assert_eq(|_|"A2", Iterator::range_step(0, 10, 2).to_array, [0, 2, 4, 6, 8]);;
        assert_eq(|_|"A3", Iterator::range_step(0, 10, 3).to_array, [0, 3, 6, 9]);;
        assert_eq(|_|"A4", Iterator::range_step(0, 10, 4).to_array, [0, 4, 8]);;
        assert_eq(|_|"A5", Iterator::range_step(0, 10, 5).to_array, [0, 5]);;
        assert_eq(|_|"A6", Iterator::range_step(0, 10, 6).to_array, [0, 6]);;
        assert_eq(|_|"A7", Iterator::range_step(0, 10, 7).to_array, [0, 7]);;
        assert_eq(|_|"A8", Iterator::range_step(0, 10, 8).to_array, [0, 8]);;
        assert_eq(|_|"A9", Iterator::range_step(0, 10, 9).to_array, [0, 9]);;
        assert_eq(|_|"A10", Iterator::range_step(0, 10, 10).to_array, [0]);;
        assert_eq(|_|"A11", Iterator::range_step(0, 10, 11).to_array, [0]);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_range_step_2() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (

        assert_eq(|_|"B2", Iterator::range_step(10, 0, 2).get_size, 0);;
        // assert_eq(|_|"B1", Iterator::range_step(10, 0, 1).take(100).get_size, 100);;
        // assert_eq(|_|"B0", Iterator::range_step(10, 0, 0).take(100).get_size, 100);;
        assert_eq(|_|"B-1", Iterator::range_step(10, 0, -1).to_array, [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);;
        assert_eq(|_|"B-2", Iterator::range_step(10, 0, -2).to_array, [10, 8, 6, 4, 2]);;
        assert_eq(|_|"B-3", Iterator::range_step(10, 0, -3).to_array, [10, 7, 4, 1]);;
        assert_eq(|_|"B-4", Iterator::range_step(10, 0, -4).to_array, [10, 6, 2]);;
        assert_eq(|_|"B-5", Iterator::range_step(10, 0, -5).to_array, [10, 5]);;
        assert_eq(|_|"B-6", Iterator::range_step(10, 0, -6).to_array, [10, 4]);;
        assert_eq(|_|"B-7", Iterator::range_step(10, 0, -7).to_array, [10, 3]);;
        assert_eq(|_|"B-8", Iterator::range_step(10, 0, -8).to_array, [10, 2]);;
        assert_eq(|_|"B-9", Iterator::range_step(10, 0, -9).to_array, [10, 1]);;
        assert_eq(|_|"B-10", Iterator::range_step(10, 0, -10).to_array, [10]);;
        assert_eq(|_|"B-11", Iterator::range_step(10, 0, -11).to_array, [10]);;

        assert_eq(|_|"C1", Iterator::range_step(0, 0, 1).get_size, 0);;
        // assert_eq(|_|"C0", Iterator::range_step(0, 0, 0).get_size, 0);;
        assert_eq(|_|"C-1", Iterator::range_step(0, 0, -1).get_size, 0);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_iterator_flat_map() {
    let source = r##"
    module Main;
    
    pythagorean_triples : I64 -> DynIterator (I64, I64, I64);
    pythagorean_triples = |limit| (
        Iterator::range(1, limit+1).flat_map(|a| (
            Iterator::range(a, limit+1).flat_map(|b| (
                Iterator::range(b, limit+1).filter(|c| (
                    a*a + b*b == c*c
                )).map(|c| (a, b, c))
            ))
        )).to_dyn
    );

    main : IO ();
    main = (
        let f = |x| range(0, max(0, x));
        let it = [-1, 0, 1, 2, 3].to_iter.flat_map(f);
        assert(|_|"", it.is_equal([0, 0, 1, 0, 1, 2].to_iter));;

        assert(|_|"", [].to_iter.flat_map(f).is_equal(Iterator::empty));;

        assert(|_|"", pythagorean_triples(100).is_equal([(3, 4, 5), (5, 12, 13), (6, 8, 10), (7, 24, 25), (8, 15, 17), (9, 12, 15), (9, 40, 41), (10, 24, 26), (11, 60, 61), (12, 16, 20), (12, 35, 37), (13, 84, 85), (14, 48, 50), (15, 20, 25), (15, 36, 39), (16, 30, 34), (16, 63, 65), (18, 24, 30), (18, 80, 82), (20, 21, 29), (20, 48, 52), (21, 28, 35), (21, 72, 75), (24, 32, 40), (24, 45, 51), (24, 70, 74), (25, 60, 65), (27, 36, 45), (28, 45, 53), (28, 96, 100), (30, 40, 50), (30, 72, 78), (32, 60, 68), (33, 44, 55), (33, 56, 65), (35, 84, 91), (36, 48, 60), (36, 77, 85), (39, 52, 65), (39, 80, 89), (40, 42, 58), (40, 75, 85), (42, 56, 70), (45, 60, 75), (48, 55, 73), (48, 64, 80), (51, 68, 85), (54, 72, 90), (57, 76, 95), (60, 63, 87), (60, 80, 100), (65, 72, 97)].to_iter));;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_option_iterator() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert(|_|"", some(42).to_iter.is_equal(Iterator::empty.push_front(42)));;
        assert(|_|"", (none() : Option I64).to_iter.is_equal(Iterator::empty));;
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_alias() {
    // Test type alias.
    let source = r#"
        module Main; 
        
        // Test of higher kinded type alias is covered by Std::Lazy.

        type Name = String;

        // Type alias in declaration of global value.
        greet : Name -> String;
        greet = |name| "My name is " + name;

        // Type alias in type definition.
        type Person = box struct { name : Name };

        // Type alias in definition of trait.
        trait a : Named {
            get_name : a -> Name;
        }
        impl Person : Named {
            get_name = @name;
        }
        
        // Implement trait on type alias.
        trait a : MyToString {
            to_string : a -> String;
        }
        impl Name : MyToString {
            to_string = |s : Name| s;
        }

        main : IO ();
        main = (
            assert_eq(|_|"", "John".greet + " " + get_name(Person { name : "Smith" }), "My name is John Smith");;

            // Type alias in type annotation.
            let names : Array Name = ["John Smith"];
            assert_eq(|_|"", names.@(0).MyToString::to_string, "John Smith");;

            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_alias_higner_kinded_argument() {
    // Test type alias.
    let source = r#"
        module Main; 
        
        type Swap a f = f a;

        main : Swap () IO;
        main = (
            pure()
        );
    "#;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_circular_aliasing_issue42() {
    let source = r##"
    module Main;

    type MyArray a = Array a;
    
    main: IO ();
    main = (
        let arr: MyArray (MyArray I64)  = [ [] ];
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unsaturated_type_alias() {
    let source = r##"
    module Main;

    type Hoge a b = a -> b;

    impl Hoge a : Functor {
        map = |f, g| g >> f;
    }
    
    main: IO ();
    main = (
        let hoge : Hoge I64 String = to_string;
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Cannot resolve type alias `Main::Hoge` in `Main::Hoge a` because it is not fully applied.",
    );
}

#[test]
pub fn test_detect_circular_type_aliasing_0() {
    let source = r##"
    module Main;

    type Fizz = Buzz;
    type Buzz = Fizz;
    
    main: IO ();
    main = (
        let fizz : Fizz = 42;
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Circular type aliasing is found in `Main::Fizz`.",
    );
}

#[test]
pub fn test_detect_circular_type_aliasing_1() {
    let source = r##"
    module Main;

    type Hoge = Array Hoge;
    
    main: IO ();
    main = (
        let hoge : Hoge = [];
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Circular type aliasing is found in `Main::Hoge`.",
    );
}

#[test]
pub fn test_detect_circular_type_aliasing_2() {
    let source = r##"
    module Main;

    type Hoge a = Hoge I64;
    
    main: IO ();
    main = (
        let hoge : Hoge Bool = [];
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Circular type aliasing is found in `Main::Hoge Std::I64`.",
    );
}

#[test]
pub fn test_call_unimplemented_trait_method_regression_issue_43() {
    let source = r##"
        module Main;

        trait a: Foo {
            foo: a -> IO ();
        }
        
        main: IO ();
        main = (
            123.foo
        );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "`Main::Foo::foo` of type `[a : Main::Foo] a -> Std::IO ()` does not match the expected type `Std::I64 -> Std::IO ()` since `Std::I64 : Main::Foo` cannot be deduced.",
    );
}

#[test]
pub fn test_c_type_aliases() {
    let source = r##"
        module Main;
                
        main: IO ();
        main = (
            let x : CChar = 42.to_CChar;
            let x : CChar = 42.0.to_CChar;

            let x : CUnsignedChar = 42.to_CUnsignedChar;
            let x : CUnsignedChar = 42.0.to_CUnsignedChar;
            
            let x : CShort = 42.to_CShort;
            let x : CShort = 42.0.to_CShort;

            let x : CUnsignedShort = 42.to_CUnsignedShort;
            let x : CUnsignedShort = 42.0.to_CUnsignedShort;

            let x : CInt = 42.to_CInt;
            let x : CInt = 42.0.to_CInt;

            let x : CUnsignedInt = 42.to_CUnsignedInt;
            let x : CUnsignedInt = 42.0.to_CUnsignedInt;

            let x : CLong = 42.to_CLong;
            let x : CLong = 42.0.to_CLong;

            let x : CUnsignedLong = 42.to_CUnsignedLong;
            let x : CUnsignedLong = 42.0.to_CUnsignedLong;

            let x : CLongLong = 42.to_CLongLong;
            let x : CLongLong = 42.0.to_CLongLong;

            let x : CUnsignedLongLong = 42.to_CUnsignedLongLong;
            let x : CUnsignedLongLong = 42.0.to_CUnsignedLongLong;

            let x : CSizeT = 42.to_CSizeT;
            let x : CSizeT = 42.0.to_CSizeT;

            let x : CFloat = 42.to_CFloat;
            let x : CFloat = 42.0.to_CFloat;

            let x : CDouble = 42.to_CDouble;
            let x : CDouble = 42.0.to_CDouble;

            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_mutate_boxed() {
    let source = r##"
        module Main;
                
        main: IO ();
        main = (
            let x : Box I32 = Box { value : 0_I32 };
            let (x, _) = x.mutate_boxed(|ptr| IO::from_runner $ |ios|
                "%d".borrow_c_str(|c_str|
                    FFI_CALL_IOS[CInt snprintf(Ptr, CSizeT, Ptr, CInt), ptr, 4.to_CSizeT, c_str, 123.to_CInt, ios]
                )
            );
            assert_eq(|_|"", x.@value, 0x00333231_I32);; // '1' = 0x31, '2' = 0x32, '3' = 0x33, '\0' = 0x00
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_mutate_boxed_io() {
    let source = r##"
        module Main;
                
        main: IO ();
        main = (
            println("test_mutate_boxed_io");; // This makes the `x` local and therefore unique.
            let x : Box I32 = Box { value : 0_I32 };
            let (x, _) = *x.mutate_boxed_io(|ptr| IO::from_runner $ |ios|
                "%d".borrow_c_str(|c_str|
                    FFI_CALL_IOS[CInt snprintf(Ptr, CSizeT, Ptr, CInt), ptr, 4.to_CSizeT, c_str, 123.to_CInt, ios]
                )
            );
            assert_eq(|_|"", x.@value, 0x00333231_I32);; // '1' = 0x31, '2' = 0x32, '3' = 0x33, '\0' = 0x00
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_mutate_boxed_shared() {
    let source = r##"
        module Main;
                
        main: IO ();
        main = (
            let x : Box I32 = Box { value : 0_I32 };
            let (y, _) = x.mutate_boxed(|ptr| IO::from_runner $ |ios|
                "%d".borrow_c_str(|c_str|
                    FFI_CALL_IOS[CInt snprintf(Ptr, CSizeT, Ptr, CInt), ptr, 4.to_CSizeT, c_str, 123.to_CInt, ios]
                )
            );
            assert_eq(|_|"", x.@value, 0_I32);;
            assert_eq(|_|"", y.@value, 0x00333231_I32);; // '1' = 0x31, '2' = 0x32, '3' = 0x33, '\0' = 0x00
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_mutate_boxed_io_shared() {
    let source = r##"
        module Main;
                
        main: IO ();
        main = (
            let x : Box I32 = Box { value : 0_I32 };
            let (y, _) = *x.mutate_boxed_io(|ptr| IO::from_runner $ |ios|
                "%d".borrow_c_str(|c_str|
                    FFI_CALL_IOS[CInt snprintf(Ptr, CSizeT, Ptr, CInt), ptr, 4.to_CSizeT, c_str, 123.to_CInt, ios]
                )
            );
            assert_eq(|_|"", x.@value, 0_I32);;
            assert_eq(|_|"", y.@value, 0x00333231_I32);; // '1' = 0x31, '2' = 0x32, '3' = 0x33, '\0' = 0x00
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_get_errno() {
    let source = r##"
        module Main;
                
        main: IO ();
        main = (
            let errno = *(IO::from_runner $ |state| "a_path_where_no_file_exists".borrow_c_str(|file|
                "invalid_file_mode".borrow_c_str(|mode|
                    let (state, _) = (clear_errno.@runner)(state);
                    let (state, _) = FFI_CALL_IOS[Ptr fopen(Ptr, Ptr), file, mode, state];
                    (get_errno.@runner)(state)
                )
            ));
            assert(|_|"", errno != 0.to_CInt);;
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_monadic_bind_and_make_struct_ordering() {
    let source = r##"
        module Main;
        
        type Pair a b = struct { x : a, y : b };

        impl [a : ToString, b : ToString] Pair a b : ToString {
            to_string = |p| "(" + p.@x.to_string + ", " + p.@y.to_string + ")";
        }

        main: IO ();
        main = (
            let pairs = do { pure $ Pair { x : *[1, 2], y : *["a", "b"] } }; // Fix `x` first, and move `y`
            assert_eq(|_|"", 
                            pairs.to_iter.map(to_string).join(", "), 
                            "(1, a), (1, b), (2, a), (2, b)");;

            let pairs = do { pure $ Pair { y : *["a", "b"], x : *[1, 2] } }; // Fix `y` first, and move `x`.
            assert_eq(|_|"", 
                            pairs.to_iter.map(to_string).join(", "), 
                            "(1, a), (2, a), (1, b), (2, b)");;
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_monadic_bind_and_function_application_ordering() {
    let source = r##"
        module Main;
    
        main: IO ();
        main = (
            let funs = [|x| (x, 0), |x| (x, 1)];
            let vals = [false, true];

            let xs = do { pure $ (*funs)(*vals) };
            assert_eq(|_|"", xs, [(false, 0), (true, 0), (false, 1), (true, 1)]);;

            let xs = do { pure $ (*funs) $ (*vals) };
            assert_eq(|_|"", xs, [(false, 0), (true, 0), (false, 1), (true, 1)]);;

            let xs = do { pure $ (*vals).(*funs) };
            assert_eq(|_|"", xs, [(false, 0), (false, 1), (true, 0), (true, 1)]);;

            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_struct_act() {
    let source = r##"
        module Main;
                
        // Boxed struct with a boxed field.
        type BB = box struct { x : Array Bool, y : Array I64, z : I64 };
        impl BB : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        // Boxed struct with an unboxed field.
        type BU = box struct { x : Bool, y : Array I64, z : I64 };
        impl BU : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        // Unboxed struct with a boxed field.
        type UB = unbox struct { x : Array Bool, y : Array I64, z : I64 };
        impl UB : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        // Unboxed struct with an unboxed field.
        type UU = unbox struct { x : Bool, y : Array I64, z : I64 };
        impl UU : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        // Generic boxed struct with a boxed field.
        type GB a = box struct { x : Array a, y : Array I64, z : I64 };
        impl [a : Eq] GB a : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        main: IO ();
        main = (
            pure();; // To make the `s` defined below not global and therefore unique.

            let actor_array = |x| let x = x.assert_unique(|_|""); if x.Array::get_size > 0 { Option::some(x) } else { Option::none() };
            let actor_bool = |x| if x { Option::some(x) } else { Option::none() };

            // BB case 1
            let s = BB { x : [true], y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::some(BB { x : [true], y : [1, 2], z : 3 }));;

            // BB case 2
            let s = BB { x : [], y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::none());;

            // BU case 1
            let s = BU { x : true, y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_bool), Option::some(BU { x : true, y : [1, 2], z : 3 }));;

            // BU case 2
            let s = BU { x : false, y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_bool), Option::none());;

            // UB case 1
            let s = UB { x : [true], y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::some(UB { x : [true], y : [1, 2], z : 3 }));;

            // UB case 2
            let s = UB { x : [], y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::none());;

            // UU case 1
            let s = UU { x : true, y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_bool), Option::some(UU { x : true, y : [1, 2], z : 3 }));;

            // UU case 2
            let s = UU { x : false, y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_bool), Option::none());;

            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_struct_act2() {
    let source = r##"
        module Main;
                
        // Boxed struct with a boxed field.
        type BB = box struct { x : Array Bool, y : Array I64, z : I64 };
        impl BB : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        // Boxed struct with an unboxed field.
        type BU = box struct { x : Bool, y : Array I64, z : I64 };
        impl BU : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        // Unboxed struct with a boxed field.
        type UB = unbox struct { x : Array Bool, y : Array I64, z : I64 };
        impl UB : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        // Unboxed struct with an unboxed field.
        type UU = unbox struct { x : Bool, y : Array I64, z : I64 };
        impl UU : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        // Generic boxed struct with a boxed field.
        type GB a = box struct { x : Array a, y : Array I64, z : I64 };
        impl [a : Eq] GB a : Eq {
            eq = |lhs, rhs| lhs.@x == rhs.@x && lhs.@y == rhs.@y && lhs.@z == rhs.@z;
        }

        main: IO ();
        main = (
            let actor_bool = |x| if x { Option::some(x) } else { Option::none() };

            // GB case 1
            let actor_array = |x| if x.Array::get_size > 0 { Option::some(x) } else { Option::none() };
            let s = GB { x : [true], y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::some(GB { x : [true], y : [1, 2], z : 3 }));;

            // GB case 2
            let s = GB { x : [], y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::none());;

            // Case where BB is shared.
            let actor_array = |x| if x.Array::get_size > 0 { Option::some(x.set(0, false)) } else { Option::none() };
            let s = BB { x : [true], y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::some(BB { x : [false], y : [1, 2], z : 3 }));;
            assert_eq(|_|"", s, BB { x : [true], y : [1, 2], z : 3 });;

            // Case where field is shared.
            let x = [true];
            let s = BB { x : x, y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::some(BB { x : [false], y : [1, 2], z : 3 }));;
            assert_eq(|_|"", x, [true]);;

            // Case where both of BB and field are shared.
            let x = [true];
            let s = BB { x : x, y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor_array), Option::some(BB { x : [false], y : [1, 2], z : 3 }));;
            assert_eq(|_|"", x, [true]);;
            assert_eq(|_|"", s, BB { x : [true], y : [1, 2], z : 3 });;

            // Case where `#plug_in(ps)` is called multiple times.
            let actor = |x| [x, x.push_back(false), x.push_back(true)];
            let s = BB { x : [true], y : [1, 2], z : 3 };
            assert_eq(|_|"", s.act_x(actor), [
                BB { x : [true],              y : [1, 2], z : 3 },
                BB { x : [true, false],       y : [1, 2], z : 3 },
                BB { x : [true, true],        y : [1, 2], z : 3 }
            ]);;

            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_tuple_functor() {
    let source = r##"
        module Main;
                
        main: IO ();
        main = (
            assert_eq(|_|"", (1,).map(|x| x + 1), (2,));;
            assert_eq(|_|"", (1, 2).map(|x| x + 1), (1, 3));;

            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}
#[test]
pub fn test_empty_struct() {
    let source = r##"
        module Main;
        
        type Empty = struct {};
        impl Empty : ToString {
            to_string = |e| "Empty";
        }

        type BoxEmpty = box struct {  };
        impl BoxEmpty : ToString {
            to_string = |e| "Box Empty";
        }

        type UnBoxEmpty = unbox struct {    };
        impl UnBoxEmpty : ToString {
            to_string = |e| "Unbox Empty";
        }
        
        main: IO ();
        main = (
            let empty = Empty {};
            assert_eq(|_|"", empty.to_string, "Empty");;
            let box_empty = BoxEmpty {};
            assert_eq(|_|"", box_empty.to_string, "Box Empty");;
            let unbox_empty = UnBoxEmpty {};
            assert_eq(|_|"", unbox_empty.to_string, "Unbox Empty");;
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_make_struct_to_union() {
    let source = r##"
        module Main;

        // declare Foo as a union
        type Foo = unbox union {
            foo: String
        };

        // create Foo as a struct
        make_foo: String -> Foo;
        make_foo = |str| Foo { foo: str };

        main: IO ();
        main = (
            let foo = make_foo("aaa");
            pure()
        );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Type `Main::Foo` is not a struct.",
    );
}

#[test]
pub fn test_regression_issue_46() {
    let source = r##"
        module Main;
        main: IO ();
        main = (
            // No problem
            if !(1 == 2) {
                println ("hello")
            };
            // Syntax error
            if ! (1 == 2) {
                println ("world")
            };
            println("foo")
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_read_file_after_close() {
    let source = r##"
        module Main;
        
        main: IO ();
        main = do {
            let fh = *open_file("/dev/null", "r");
            close_file(fh).lift;;
            let line = *read_line(fh);
            println(line).lift
        }.try(|msg|
            assert_eq(|_|"", msg, "`Std::IO::_read_line_inner` failed!: the IOHandle is already closed.");;
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_circular_type_definition() {
    let source = r##"
        module Main;
        type Foo = unbox struct { func: Foo -> Foo };
        type Bar = unbox union { func: Bar -> Bar };

        main: IO ();
        main = (
            let foo = Foo { func: |x| x };
            let bar = Bar::func(|x| x);
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_number_to_varname() {
    assert_eq!(number_to_varname(0), "a");
    assert_eq!(number_to_varname(1), "b");
    assert_eq!(number_to_varname(25), "z");
    assert_eq!(number_to_varname(26), "a1");
    assert_eq!(number_to_varname(27), "b1");
    assert_eq!(number_to_varname(51), "z1");
    assert_eq!(number_to_varname(52), "a2");
}

#[test]
pub fn test_export() {
    let source = r##"
        module Main;
        
        value : CInt;
        value = 42.to_CInt;
        FFI_EXPORT[value, c_value];

        increment : CInt -> CInt;
        increment = |x| x + 1.to_CInt;
        FFI_EXPORT[increment, c_increment];

        two_variable : CInt -> CInt -> CInt;
        two_variable = |x, y| 2.to_CInt * x + y;
        FFI_EXPORT[two_variable, c_two_variable];

        io_action : IO ();
        io_action = println("io_action");
        FFI_EXPORT[io_action, c_io_action];

        io_action2 : CInt -> IO ();
        io_action2 = |x| do {
            println("io_action2: " + x.to_string);;
            pure()
        };
        FFI_EXPORT[io_action2, c_io_action2];

        io_action3 : CInt -> IO CInt;
        io_action3 = |x| do {
            println("io_action3");;
            pure(x + 1.to_CInt)
        };
        FFI_EXPORT[io_action3, c_io_action3];

        main: IO ();
        main = (
            let res = FFI_CALL[CInt call_fix_values()];
            assert_eq(|_|"", res, 0.to_CInt);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdio.h>

        int c_value();
        int c_increment(int x);
        int c_two_variable(int x, int y);
        void c_io_action();
        void c_io_action2(int x);
        int c_io_action3(int x);

        int call_fix_values() {
            int x = c_value();
            if (x != 42) {
                return 1;
            }

            int y = c_increment(42);
            if (y != 43) {
                return 1;
            }

            if (c_two_variable(3, 2) != 8) {
                return 1;
            }

            c_io_action();

            c_io_action2(42);

            int z = c_io_action3(42);
            if (z != 43) {
                return 1;
            }

            return 0;
        }
    "##;

    test_fix_linked_with_c(&source, &c_source, function_name!());
}

#[test]
pub fn test_unsafe_get_release_retain_function_of_boxed_value_decltype_technique_1() {
    // Actual usage of `get_funptr_release` is tested in asynctask.fix.
    let source = r##"
        module Main;

        type VoidType = box struct {};
        // No constructor for `VoidType` is provided.

        main: IO ();
        main = (
            let release = (|_| undefined("") : VoidType).get_funptr_release;
            let retain = (|_| undefined("") : VoidType).get_funptr_retain;
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unsafe_get_release_retain_function_of_boxed_value_decltype_technique_2() {
    // Actual usage of `get_funptr_release` is tested in asynctask.fix.
    let source = r##"
        module Main;

        get_release_func_of_codom : [b : Boxed] (a -> b) -> Ptr;
        get_release_func_of_codom = |f| (
            let lazy_b = |_| f(undefined(""));
            lazy_b.get_funptr_release
        );

        get_release_func_of_codom_2 : [b : Boxed] (a -> b) -> Ptr;
        get_release_func_of_codom_2 = |f| (
            let lazy_b = |_| undefined("") : b;
            lazy_b.get_funptr_release
        );

        get_retain_func_of_dom : [a : Boxed] (a -> b) -> Ptr;
        get_retain_func_of_dom = |f| (
            let lazy_a = |_| let x = undefined(""); let _ = f(x); x;
            lazy_a.get_funptr_release
        );

        get_retain_func_of_dom_2 : [a : Boxed] (a -> b) -> Ptr;
        get_retain_func_of_dom_2 = |f| (
            let lazy_a : Lazy a = |_| undefined("");
            lazy_a.get_funptr_release
        );

        main: IO ();
        main = (
            let release = get_release_func_of_codom(|_ : I64| [42]);
            let retain = get_retain_func_of_dom(|_ : Array I64| 42);
            let release_2 = get_release_func_of_codom_2(|_ : I64| [42]);
            let retain_2 = get_retain_func_of_dom_2(|_ : Array I64| 42);
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_get_funptr_release_error() {
    let source = r##"
        module Main;

        main: IO ();
        main = (
            let release = (|_| undefined("") : I64).get_funptr_release;
            pure()
        );
    "##;
    test_source_fail(&source, Configuration::compiler_develop_mode(), "");
}

#[test]
pub fn test_get_funptr_retain_error() {
    let source = r##"
        module Main;

        main: IO ();
        main = (
            let retain = (|_| undefined("") : I64).get_funptr_retain;
            pure()
        );
    "##;
    test_source_fail(&source, Configuration::compiler_develop_mode(), "");
}

#[test]
pub fn test_double_bind() {
    let source = r##"
        module Main;

        main: IO ();
        main = (
            let x = Option::some $ Option::some $ 42;
            let y = do { pure $ **x };
            assert(|_|"", y.is_some);;
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_double_semicolon_in_let() {
    let source = r##"
        module Main;

        main: IO ();
        main = (
            let x = [(), ()];; [1, 2, 3];
            assert_eq(|_|"", x, [1, 2, 3, 1, 2, 3]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_unsafe_perform_bug() {
    let source = r##"
        module Main;

        type Wrapper = struct {
            _0 : Box I64
        };

        main: IO ();
        main = (
            let x = do {
                pure $ Wrapper { _0 : Box { value : 1234 } }
            }.unsafe_perform;

            assert_eq(|_|"", x.@_0.@value, 1234)            
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_debug_println() {
    let source = r##"
        module Main;

        main: IO ();
        main = (
            eval debug_println("stdout");
            eval debug_eprintln("stderr");
            pure()
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_impl_boxed_by_hand() {
    let source = r##"
        module Main;

        type MyType = box struct {
            value : I64
        };

        impl MyType : Boxed {}

        main: IO ();
        main = (
            pure()
        );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Implementing `Std::Boxed` by hand is not allowed.",
    );
}

#[test]
pub fn test_get_boxed_data_ptr_for_union() {
    let source = r##"
        module Main;

        type MyUnion = box union {
            a : I64,
            b : F64
        };

        main: IO ();
        main = (
            let instance = MyUnion::a(42);
            let val = instance.with_retained(|instance|
                let ptr = instance._get_boxed_ptr;
                FFI_CALL[I64 deref(Ptr), ptr]
            );
            assert_eq(|_|"", val, 42)
        );
    "##;
    let c_source = r##"
        #include <stdio.h>
        #include <stdint.h>

        int64_t deref(int64_t* ptr) {
            return *ptr;
        }
    "##;
    test_fix_linked_with_c(&source, &c_source, function_name!());
}

#[test]
pub fn test_mutate_boxed_union() {
    let source = r##"
        module Main;

        type MyUnion = box union {
            a : I64,
            b : Bool
        };

        main: IO ();
        main = (
            let uni = MyUnion::a(42);
            let (uni, _) = uni.mutate_boxed(|ptr| FFI_CALL_IO[() negate(Ptr), ptr]);
            assert_eq(|_|"", uni.as_a, -42)
        );
    "##;
    let c_source = r##"
        #include <stdio.h>
        #include <stdint.h>

        void negate(int64_t* ptr) {
            *ptr = -(*ptr);
        }
    "##;
    test_fix_linked_with_c(&source, &c_source, function_name!());
}

#[test]
pub fn test_mutate_boxed_io_union() {
    let source = r##"
        module Main;

        type MyUnion = box union {
            a : I64,
            b : Bool
        };

        main: IO ();
        main = (
            let uni = MyUnion::a(42);
            let (uni, _) = *uni.mutate_boxed_io(|ptr| FFI_CALL_IO[() negate(Ptr), ptr]);
            assert_eq(|_|"", uni.as_a, -42)
        );
    "##;
    let c_source = r##"
        #include <stdio.h>
        #include <stdint.h>

        void negate(int64_t* ptr) {
            *ptr = -(*ptr);
        }
    "##;
    test_fix_linked_with_c(&source, &c_source, function_name!());
}

#[test]
pub fn test_mutate_boxed_union_shared() {
    let source = r##"
        module Main;

        type MyUnion = box union {
            a : I64,
            b : Bool
        };

        main: IO ();
        main = (
            let uni = MyUnion::a(42);
            let (uni2, _) = uni.mutate_boxed(|ptr| FFI_CALL_IO[() negate(Ptr), ptr]);
            assert_eq(|_|"", uni.as_a, 42);;
            assert_eq(|_|"", uni2.as_a, -42)
        );
    "##;
    let c_source = r##"
        #include <stdio.h>
        #include <stdint.h>

        void negate(int64_t* ptr) {
            *ptr = -(*ptr);
        }
    "##;
    test_fix_linked_with_c(&source, &c_source, function_name!());
}

#[test]
pub fn test_mutate_boxed_io_union_shared() {
    let source = r##"
        module Main;

        type MyUnion = box union {
            a : I64,
            b : Bool
        };

        main: IO ();
        main = (
            let uni = MyUnion::a(42);
            let (uni2, _) = *uni.mutate_boxed_io(|ptr| FFI_CALL_IO[() negate(Ptr), ptr]);
            assert_eq(|_|"", uni.as_a, 42);;
            assert_eq(|_|"", uni2.as_a, -42)
        );
    "##;
    let c_source = r##"
        #include <stdio.h>
        #include <stdint.h>

        void negate(int64_t* ptr) {
            *ptr = -(*ptr);
        }
    "##;
    test_fix_linked_with_c(&source, &c_source, function_name!());
}

#[test]
pub fn test_type_variable_in_type_annotation() {
    let source = r##"
        module Main;

        empty_array : Array a;
        empty_array = [] : Array a;

        main: IO ();
        main = (
            assert_eq(|_|"", empty_array : Array I64, [])
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_variable_in_type_annotated_pattern() {
    let source = r##"
        module Main;

        empty_array : Array a;
        empty_array = (
            let x : Array a = [];
            x
        );

        main: IO ();
        main = (
            assert_eq(|_|"", [] : Array I64, [])
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_higher_kinded_type_variable_in_type_annotation() {
    let source = r##"
        // The following test code was written by pt9999, and is licensed under the MIT License.
        // Copyright (c) 2023 pt9999

        // Tests for type annotations which use type variables
        module Main;

        type Reader e a = unbox struct {
            data: e -> a
        };

        make_reader: (e -> a) -> Reader e a;
        make_reader = |f| Reader { data: f };

        run_reader: e -> Reader e a -> a;
        run_reader = |e, reader| (reader.@data)(e);

        // Test for identity reader
        test_reader: e -> e;
        test_reader = (
            let reader: Reader e e = make_reader(|env| env);
            |env| reader.run_reader(env)
        );


        type [m: *->*] ReaderT e m a = unbox struct {
            data: e -> m a
        };

        make_reader_t: [m: Monad] (e -> m a) -> ReaderT e m a;
        make_reader_t = |f| ReaderT { data: f };

        run_reader_t: [m: Monad] e -> ReaderT e m a -> m a;
        run_reader_t = |e, reader| (reader.@data)(e);

        // Test for identity reader_t
        test_reader_t: [m: Monad] e -> m e;
        test_reader_t = (
            let reader: ReaderT e m e = make_reader_t(|env| pure(env));
            |env| reader.run_reader_t(env)
        );

        main: IO ();
        main = (
            let str = test_reader("abc");
            assert_eq(|_|"test_reader", str, "abc");;
            let str = *test_reader_t("abc");
            assert_eq(|_|"test_reader_t", str, "abc")
        );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_create_dylib() {
    let fix_src = r##"
        module Main;
    
        get_truth : IO CInt;
        get_truth = pure $ 42.to_CInt;

        FFI_EXPORT[get_truth, get_truth];
    "##;
    let c_str = r##"
        #include <stdio.h>
        #include <stdint.h>

        int get_truth();

        int main() {
            int x = get_truth();
            if (x != 42) {
                return 1;
            }
            return 0;
        }
    "##;
    install_fix();

    // Recreate working directory for this test.
    let work_dir = PathBuf::from(format!(
        "{}/{}",
        COMPILER_TEST_WORKING_PATH,
        function_name!()
    ));
    let _ = fs::remove_dir_all(&work_dir);
    let _ = fs::create_dir_all(&work_dir);

    // Save `fix_src` to a file.
    let fix_file = format!("{}/lib.fix", work_dir.to_str().unwrap());
    let mut file = File::create(&fix_file).unwrap();
    file.write_all(fix_src.as_bytes()).unwrap();

    // Create dynamic library using `fix`.
    let output = Command::new("fix")
        .arg("build")
        .arg("--output-type")
        .arg("dylib")
        .arg("--file")
        .arg("lib.fix")
        .arg("--output")
        .arg("libfixtest.so")
        .current_dir(&work_dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));

    // Save `c_str` to a file.
    let c_file = format!("{}/main.c", work_dir.to_str().unwrap());
    let mut file = File::create(&c_file).unwrap();
    file.write_all(c_str.as_bytes()).unwrap();

    // Build the C program, link it with the dynamic library, and run it.
    let output = Command::new("gcc")
        .arg("-o")
        .arg("main")
        .arg("main.c")
        .arg("-L.")
        .arg("-lfixtest")
        .current_dir(&work_dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));

    // Run the C program.
    let output = Command::new("./main")
        .current_dir(&work_dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));

    // Remove the working directory.
    let _ = fs::remove_dir_all(&work_dir);
}

#[test]
pub fn test_match_option() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        // Value is unboxed 
        let x = Option::some(42);
        let v = match x {
            some(v) => v,
            none(_) => 0
        };
        assert_eq(|_|"", v, 42);;

        let x = Option::none();
        let v = match x {
            some(v) => v,
            none(_) => 0
        };
        assert_eq(|_|"", v, 0);;

        // Value is boxed
        let x = Option::some(Box::make(42));
        let v = match x {
            some(v) => v,
            none(_) => Box::make(0)
        };
        assert_eq(|_|"", v.@value, 42);;

        let x : Option (Box I64) = Option::none();
        let v = match x {
            some(v) => v,
            none(_) => Box::make(0)
        };
        assert_eq(|_|"", v.@value, 0);;

        // Value is boxed and shared
        let x = Option::some(Box::make(42));
        let v = match x {
            some(v) => v,
            none(_) => Box::make(0)
        };
        assert_eq(|_|"", v.@value, 42);;
        assert_eq(|_|"", x.as_some.@value, 42);;

        // Value is a closure
        let x = Option::some(|x| x + 1);
        let v = match x {
            some(v) => v(41),
            none(_) => 0
        };
        assert_eq(|_|"", v, 42);;

        // Value is a closure and shared
        let x = Option::some(|x| x + 1);
        let v = match x {
            some(v) => v(41),
            none(_) => 0
        };
        assert_eq(|_|"", v, 42);;
        assert_eq(|_|"", (x.as_some)(41), 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_match_extra_comma() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::some(42);
        let v = match x {
            some(v) => v,
            none(_) => 0, // Extra comma
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_match_boxed_union() {
    let source = r##"
    module Main;

    type MyEither a b = box union {
        left : a,
        right : b
    };

    main: IO ();
    main = (
        // Value is unboxed
        let x = MyEither::left(42);
        let v = match x {
            left(v) => v,
            right(_ : Bool) => 0
        };
        assert_eq(|_|"", v, 42);;

        let x = MyEither::right(false);
        let v = match x {
            left(v) => v,
            right(_) => 0
        };
        assert_eq(|_|"", v, 0);;

        // Value is boxed
        let x = MyEither::left(Box::make(42));
        let v = match x {
            left(v) => v,            
            right(_ : Bool) => Box::make(0)
        };
        assert_eq(|_|"", v.@value, 42);;

        let x = MyEither::right(false);
        let v = match x {
            left(v) => v,
            right(_ : Bool) => Box::make(0)
        };
        assert_eq(|_|"", v.@value, 0);;

        // Value is boxed and shared
        let x = MyEither::left(Box::make(42));
        let v = match x {
            left(v) => v,
            right(_ : Bool) => Box::make(0)
        };
        assert_eq(|_|"", v.@value, 42);;
        assert_eq(|_|"", x.as_left.@value, 42);;

        // Value is a closure
        let x = MyEither::left(|x| x + 1);
        let v = match x {
            left(v) => v(41),
            right(_ : Bool) => 0
        };
        assert_eq(|_|"", v, 42);;

        // Value is a closure and shared
        let x = MyEither::left(|x| x + 1);
        let v = match x {
            left(v) => v(41),
            right(_ : Bool) => 0
        };
        assert_eq(|_|"", v, 42);;
        assert_eq(|_|"", (x.as_left)(41), 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_match_non_exhaustive() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::some(42);
        let v = match x {
            some(v) => v
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Variant `none` of union `Std::Option` is not covered.",
    );
}

#[test]
pub fn test_match_otherwise() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::none();
        let v = match x {
            some(v) => v,
            x => if x.is_none { 42 } else { 0 }
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_early_otherwise() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::none();
        let v = match x {
            x => if x.is_none { 42 } else { 0 },
            some(v) => v
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Pattern after `x` is unreachable.",
    );
}

#[test]
pub fn test_match_bad_variant() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::none();
        let v = match x {
            foo(v) => v
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "`foo` is not a variant of union `Std::Option`.",
    );
}

#[test]
pub fn test_match_variant_with_namespace() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::none();
        let v = match x {
            Option::some(v) => v,
            Option::none(_) => 42
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_match_variant_with_bad_namespace() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::none();
        let v = match x {
            LoopState::some(v) => v,
            Option::none(_) => 42
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "`LoopState::some` is not a variant of union `Std::Option`.",
    );
}

#[test]
pub fn test_match_single_variant() {
    let source = r##"
    module Main;

    type MyPair a b = box struct { first : a, second : b };

    main: IO ();
    main = (
        let x = Option::some(MyPair { first: 6, second: 7 });
        let v = match x {
            some(MyPair { first: a, second: b }) => a * b,
            none(_) => 0
        };
        assert_eq(|_|"", v, 42);;

        let x = Option::some((6, 7));
        let v = match x {
            some((a, b)) => a * b,
            none(_) => 0
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_tuple_or_struct_in_match() {
    let source = r##"
    module Main;

    type MyUnion = box union { a : I64 };

    main: IO ();
    main = (
        let x = MyUnion::a(42);
        let v = match x {
            a(v) => v
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_match_on_nonunion_types() {
    let source = r##"
    module Main;

    type MyStruct = box struct { a : I64, b : I64 };

    main: IO ();
    main = (
        let x = Box::make(42);
        let v = match x {
            y => y.@value
        };
        assert_eq(|_|"", v, 42);;

        let v = match (6, 7) {
            (x, y) => x * y
        };
        assert_eq(|_|"", v, 42);;

        let x = MyStruct { a: 6, b: 7 };
        let v = match x {
            MyStruct { a: x, b: y } => x * y
        };
        assert_eq(|_|"", v, 42);;
        assert_eq(|_|"", x.@a * x.@b, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_match_on_variant_for_nonunion() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let v = match [] {
            foo(_) => 0,
            _ => 42,
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(&source, Configuration::compiler_develop_mode(), "The matched value has non-union type `Std::Array a`, but it is matched on a variant pattern `foo(_)`.");
}

#[test]
pub fn test_match_omit_parentheses() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::none();
        let v = match x {
            some(v) => v,
            none() => if x.is_none { 42 } else { 0 } // `none()` instead of `none(())`
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_arrow_functor() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let f = Array::to_iter;
        let g = Iterator::advance;
        let h = f.map(g);
        assert_eq(|_|"", h([1,2,3,4]).as_some.@1, 1)
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_arrow_monad() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x1 = |x| x;
        let x2 = |x| x*x;
        let x3 = |x| x*x*x;
        let x4 = |x| x*x*x*x;

        let p1 = do { pure $ 
            x4(*x1 + 1)
        };
        let p2 = do { pure $ 
            (*x4) + 4*(*x3) + 6*(*x2) + 4*(*x1) + 1
        };
        assert_eq(|_|"", p1(42), p2(42))
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_kind_arrow_right_associative() {
    let source = r##"
    module Main;

    trait [p: * -> * -> *] p: Profunctor {
        dimap: (a -> b) -> (c -> d) -> p b c -> p a d; // If `* -> * -> *` is parsed as `(* -> *) -> * -> *`, this line will cause a type error.
    }

    impl Arrow: Profunctor {
        dimap = |f, g, arr| f >> arr >> g;
    }

    main: IO ();
    main = (
        let arr: Arrow I64 String = |i| i.to_string;
        let arr = arr.dimap(|i| i * 2, |s| "s=" + s);
        assert_eq(|_|"", arr(42), "s=84")
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_arrow_associativity() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let app2 = |f| f(6)(7); // Should be parsed as `|f| (f(6)(7))`, not `|(f| f(6))(7)`.
        assert_eq(|_|"", app2(|x, y| x * y), 42);;

        let arr = [1,2,3];
        let arr2 = arr.set(1)(42); // Should be parsed as `arr.(set(1)(42))`, not `(arr.set(1))(42)`.
        assert_eq(|_|"", arr2, [1,42,3]);;

        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_52() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let a : U64 = *pure();
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "6 |         let a : U64 = *pure();",
    );
}

#[test]
pub fn test_regression_issue_54() {
    let source = r##"
module Main;

type Pipe a b r = box union {
    ret: r,
};

namespace Pipe {
    read: Pipe a b a;
    read = undefined("");

    write: b -> Pipe a b ();
    write = undefined("");
}

impl Pipe a b: Monad {
//impl Pipe x y: Monad {        // Ok if this line is used instead of the above line.
    pure = undefined("");
    bind = undefined("");
}

main: IO ();
main = (
    let pipe: Pipe I64 String () = do {
        write((*read).to_string)
    };
    eval pipe;
    pure()
);
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Program terminated abnormally.", // This implies that the compilation went well, and the program started running.
    );
}

#[test]
pub fn test_string_from_u8() {
    let source = r##"
module Main;

main: IO ();
main = (
    assert_eq(|_|"", String::from_U8('a'), "a");;
    assert_eq(|_|"", String::from_U8('\x00'), "");;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_absolute_namespace_value() {
    let source = r##"
module Main;

x : I64;
x = 0;

namespace Main {
    x : I64;
    x = 1;
}

main: IO ();
main = (
    // assert_eq(|_|"", Main::x, 0);; // ambiguous
    assert_eq(|_|"", ::Main::x, 0);; // top-level `x`
    assert_eq(|_|"", Main::Main::x, 1);; // `Main::x` in `Main` namespace
    assert_eq(|_|"", ::Main::Main::x, 1);; // `Main::x` in `Main` namespace
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_ambiguous_namespace_value() {
    let source = r##"
module Main;

x : I64;
x = 0;

namespace Main {
    x : I64;
    x = 1;
}

main: IO ();
main = (
    assert_eq(|_|"", Main::x, 0);; // ambiguous
    pure()
);
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Name `Main::x` is ambiguous",
    );
}

#[test]
pub fn test_absolute_namespace_type() {
    let source = r##"
module Main;

type X = I64;

namespace Main {
    type X = I64;
}

main: IO ();
main = (
    // assert_eq(|_|"", 0 : X, 0);; // ambiguous
    assert_eq(|_|"", 0 : ::Main::X, 0);; // top-level `X`
    assert_eq(|_|"", 0 : Main::Main::X, 0);; // `Main::X` in `Main` namespace
    assert_eq(|_|"", 0 : ::Main::Main::X, 0);; // `Main::X` in `Main` namespace
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_ambiguous_namespace_type() {
    let source = r##"
module Main;

type X = I64;

namespace Main {
    type X = I64;
}

main: IO ();
main = (
    assert_eq(|_|"", 0 : X, 0);; // ambiguous
    pure()
);
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Name `X` is ambiguous",
    );
}

#[test]
pub fn test_array_search_partition_point() {
    let source = r##"
    module Main;

    main : IO ();
    main = (
        // Basic
        let arr = [-3, -2, -1, 0, 1, 2, 3];
        let pred = |x| x < 0;
        let x = search_partition_point(pred, arr);
        assert_eq(|_|"1", x, 3);;

        // Empty
        let arr = [];
        let x = search_partition_point(pred, arr);
        assert_eq(|_|"2", x, 0);;

        // All elements satisfy the condition
        let arr = [-3, -2, -1];
        let x = search_partition_point(pred, arr);
        assert_eq(|_|"3", x, 3);;

        // No element satisfies the condition
        let arr = [0, 1, 2, 3];
        let x = search_partition_point(pred, arr);
        assert_eq(|_|"4", x, 0);;

        // `pred(x)` changes at the index 1
        let arr = [-1, 0, 1, 2];
        let x = search_partition_point(pred, arr);
        assert_eq(|_|"5", x, 1);;

        // `pred(x)` changes at the index n-1
        let arr = [-3, -2, -1, 0];
        let x = search_partition_point(pred, arr);
        assert_eq(|_|"6", x, 3);;
        
        pure()
    );
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_f28ea22() {
    let source = r##"
// Compiling this code causes the compiler panic at f28ea221719275887e67220495f23b758ee2368e.

module Main;

trait c : C {
    type T c;
    call : c -> T;
}

main : IO ();
main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "All appearance of associated type has to be saturated.",
    );
}

#[test]
pub fn test_nobreak_space() {
    let source = r##"
module Main;
main : IO ();
main = (
    assert_eq(|_|"", 42, 42);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_external_project_math() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-math.git",
        "fixlang-math",
    );
}

#[test]
pub fn test_external_project_hashmap() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-hashmap.git",
        "fixlang-hashmap",
    );
}

#[test]
pub fn test_external_project_hashset() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-hashset.git",
        "fixlang-hashset",
    );
}

#[test]
pub fn test_external_project_random() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-random.git",
        "fixlang-random",
    );
}

#[test]
pub fn test_external_project_time() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-time.git",
        "fixlang-time",
    );
}

#[test]
pub fn test_external_project_character() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-character.git",
        "fixlang-character",
    );
}

#[test]
pub fn test_external_project_subprocess() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-subprocess.git",
        "fixlang-subprocess",
    );
}

#[test]
pub fn test_external_project_regexp() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-regexp.git",
        "fixlang-regexp",
    );
}

#[test]
pub fn test_external_project_asynctask() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-asynctask.git",
        "fixlang-asynctask",
    );
}

#[test]
pub fn test_external_project_gmp() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-gmp.git",
        "fixlang-gmp",
    );
}

#[test]
pub fn test_external_project_misc_algos() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-misc-algos.git",
        "fixlang-misc-algos",
    );
}

#[test]
pub fn test_external_project_binary_heap() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-binary-heap.git",
        "fixlang-binary-heap",
    );
}

#[test]
pub fn test_external_project_cp_library() {
    test_external_project("https://github.com/tttmmmyyyy/cp-library", "cp-library");
}

// Run `cargo install --locked --path .`.
pub fn install_fix() {
    let _ = Command::new("cargo")
        .arg("install")
        .arg("--locked")
        .arg("--path")
        .arg(".")
        .output()
        .expect("Failed to run cargo install.");
}

pub fn test_external_project(url: &str, test_name: &str) {
    println!("Testing external project: {}", url);
    install_fix();

    // Recreate working directory for this test.
    let work_dir = PathBuf::from(format!("{}/{}", COMPILER_TEST_WORKING_PATH, test_name));
    let _ = fs::remove_dir_all(&work_dir);
    let _ = fs::create_dir_all(&work_dir);

    // Run `git clone {url}`.
    let _ = Command::new("git")
        .arg("clone")
        .arg(url)
        .current_dir(&work_dir)
        .output()
        .expect("Failed to run git clone.");

    // Get the created directory name.
    let dir_name = url
        .split("/")
        .last()
        .unwrap()
        .to_string()
        .replace(".git", "");

    // Run `fix test`.
    let output = Command::new("fix")
        .arg("test")
        .current_dir(work_dir.join(dir_name))
        .output()
        .expect("Failed to run fix test.");

    // Check the result.
    assert_eq!(
        output.status.code(),
        Some(0),
        "Failed to run fix test of \"{}\"",
        url
    );
}

#[test]
pub fn test_generate_docs() {
    install_fix();

    // Run `fix doc -m Std` in `std_doc` directory.
    let _ = Command::new("fix")
        .arg("docs")
        .arg("-m")
        .arg("Std")
        .arg("-o")
        .arg(".")
        .current_dir("std_doc")
        .output()
        .expect("Failed to run fix doc.");
}

fn test_fix_linked_with_c(fix_src: &str, c_src: &str, test_name: &str) {
    // Create a working directory.
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);

    // Save `c_source` to a file.
    let c_file = format!("{}/{}.c", COMPILER_TEST_WORKING_PATH, test_name);
    let mut file = File::create(&c_file).unwrap();
    file.write_all(c_src.as_bytes()).unwrap();

    // Build `c_source` into a shared library.
    let lib_name = test_name;
    let so_file_path = format!("lib{}.so", lib_name);
    let mut com = Command::new("gcc");
    let output = com
        .arg("-shared")
        .arg("-fPIC")
        .arg("-o")
        .arg(so_file_path.clone())
        .arg(&c_file)
        .output()
        .expect("Failed to run gcc.");
    if output.stderr.len() > 0 {
        eprintln!(
            "{}",
            String::from_utf8(output.stderr)
                .unwrap_or("(failed to parse stderr from gcc as UTF8.)".to_string())
        );
    }

    // Link the shared library to the Fix program.
    let mut config = Configuration::compiler_develop_mode();
    config.add_dynamic_library(lib_name);
    // Add the library search path.
    config.library_search_paths.push(PathBuf::from("."));

    // Run the Fix program.
    test_source(&fix_src, config);

    // Remove the shared library.
    let _ = fs::remove_file(so_file_path);
}

#[test]
pub fn test_regression_on_associated_type_bug() {
    let source = r##"
module Main;

import Std::* hiding Indexable::Elem;

type BinaryHeap p e = unbox struct { _d : Array e, _p : p };

trait p : Priority {
    type Elem p;
}

type PriorityMinimum e = struct {};

impl [e : LessThan] PriorityMinimum e : Priority {
    type Elem (PriorityMinimum e) = e;
}

type MinBinaryHeap e = BinaryHeap (PriorityMinimum e) e;

namespace MinBinaryHeap {
    // Create an empty minimum heap.
    empty : MinBinaryHeap e;
    empty = BinaryHeap { _d: [], _p: PriorityMinimum {} }; // Dummy implementation.
}

namespace BinaryHeap {
    push : [p : Priority, Elem p = e] e -> BinaryHeap p e -> BinaryHeap p e;
    push = |_, heap| heap; // Dummy implementation.
}

main: IO ();
main = (
    let heap = MinBinaryHeap::empty.push(0); // Error occurred here.
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_string_from_bytes_to_bytes() {
    let source = r##"
module Main;

main: IO ();
main = (
    let s = from_bytes(['a', 'b', 'c', '\0']);
    assert_eq(|_|"", s.as_ok, "abc");;
    assert_eq(|_|"", s.as_ok.to_bytes, ['a', 'b', 'c', '\0']);;

    let s = from_bytes(['\0']);
    assert_eq(|_|"", s.as_ok, "");;
    assert_eq(|_|"", s.as_ok.to_bytes, ['\0']);;

    let s = from_bytes(['a', '\0', 'b']);
    assert_eq(|_|"", s.as_ok, "a");;
    assert_eq(|_|"", s.as_ok.to_bytes, ['a', '\0']);;

    let s : Result ErrMsg String = from_bytes(['a', 'b', 'c']);
    assert(|_|"", s.is_err);;

    let s : Result ErrMsg String = from_bytes([]);
    assert(|_|"", s.is_err);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_recursive_closure_capturing() {
    let source = r##"
module Main;

rec : (I64 -> I64) -> I64 -> Array I64 -> Array I64;
rec = |f, n, arr| (
    if n == 0 { arr };
    let g = |x| x + n;
    let h = f >> g;
    rec(h, n - 1, arr.push_back(h(n)))
);

main: IO ();
main = (
    let arr = rec(|x| x, 3, []);
    assert_eq(|_|"", arr, [6, 7, 7]);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_57() {
    // The following code should not take too long time to be parsed.
    let source = r##"
module Main;

f : I64 -> I64;
f = |x| (
    let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;
    let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;
    let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;
    let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;
    let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;
    let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;
    let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;
    let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;let x = x;

    let x = x // forgot to add semicolon here
    x
);
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Expected `;`",
    );
}

#[test]
pub fn test_let_nospace_equal() {
    let source = r##"
module Main;

main : IO ();
main = (
    let x=42; // No space between `x=` and `42`
    assert_eq(|_|"", x, 42);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_check_all() {
    let source = r##"
module Main;

main : IO ();
main = (
    let is_positive = |x| x > 0;
    assert_eq(|_|"", [].to_iter.check_all(is_positive), true);;
    assert_eq(|_|"", [1, 2, 3].to_iter.check_all(is_positive), true);;
    assert_eq(|_|"", [0, 1, 2].to_iter.check_all(is_positive), false);;
    assert_eq(|_|"", [-1, -2, -3].to_iter.check_all(is_positive), false);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_check_any() {
    let source = r##"
module Main;

main : IO ();
main = (
    let is_positive = |x| x > 0;
    assert_eq(|_|"", [].to_iter.check_any(is_positive), false);;
    assert_eq(|_|"", [1, 2, 3].to_iter.check_any(is_positive), true);;
    assert_eq(|_|"", [0, 1, 2].to_iter.check_any(is_positive), true);;
    assert_eq(|_|"", [-1, -2, -3].to_iter.check_any(is_positive), false);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_loop_iter_s() {
    let source = r##"
module Main;

main : IO ();
main = (
    // Look at natural numbers, and break when a multiple of 5 is found.

    // 1,2,3,4 => ends with continue(4)
    let res = range(1, 5).loop_iter_s(0, |n, _| if n % 5 == 0 { break $ n } else { continue $ n } );
    assert(|_|"loop_iter_s 1", res.is_continue);;
    assert_eq(|_|"loop_iter_s 1", res.as_continue, 4);;

    // 1,2,3,4,5 => ends with break(5)
    let res = range(1, 6).loop_iter_s(0, |n, _| if n % 5 == 0 { break $ n } else { continue $ n } );
    assert(|_|"loop_iter_s 2", res.is_break);;
    assert_eq(|_|"loop_iter_s 2", res.as_break, 5);;

    // [] => ends with continue(0)
    let res = range(1, 1).loop_iter_s(0, |n, _| if n % 5 == 0 { break $ n } else { continue $ n } );
    assert(|_|"loop_iter_s 3", res.is_continue);;
    assert_eq(|_|"loop_iter_s 3", res.as_continue, 0);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_loop_iter_ms() {
    let source = r##"
module Main;

main : IO ();
main = (
    // Look at natural numbers, and break when a multiple of 5 is found.

    // 1,2,3,4 => ends with continue(4)
    let res = *range(1, 5).loop_iter_ms(0, |n, _| println(n.to_string);; if n % 5 == 0 { break_m $ n } else { continue_m $ n } );
    assert(|_|"loop_iter_ms 1", res.is_continue);;
    assert_eq(|_|"loop_iter_ms 1", res.as_continue, 4);;

    // 1,2,3,4,5 => ends with break(5)
    let res = *range(1, 6).loop_iter_ms(0, |n, _| println(n.to_string);; if n % 5 == 0 { break_m $ n } else { continue_m $ n } );
    assert(|_|"loop_iter_ms 2", res.is_break);;
    assert_eq(|_|"loop_iter_ms 2", res.as_break, 5);;

    // [] => ends with continue(0)
    let res = *range(1, 1).loop_iter_ms(0, |n, _| println(n.to_string);; if n % 5 == 0 { break_m $ n } else { continue_m $ n } );
    assert(|_|"loop_iter_ms 3", res.is_continue);;
    assert_eq(|_|"loop_iter_ms 3", res.as_continue, 0);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_flush() {
    let source = r##"
module Main;

main : IO ();
main = (
    println("Hello, World!");;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_populate() {
    let source = r##"
module Main;

main : IO ();
main = (
    assert_eq(|_|"case 1", "".populate([]), "");;
    assert_eq(|_|"case 2", "123".populate([]), "123");;
    assert_eq(|_|"case 3", "{{".populate([]), "{");;
    assert_eq(|_|"case 4", "123{{".populate([]), "123{");;
    assert_eq(|_|"case 5", "{{123".populate([]), "{123");;
    assert_eq(|_|"case 6", "123{{123".populate([]), "123{123");;
    assert_eq(|_|"case 7", "}}".populate([]), "}");;
    assert_eq(|_|"case 8", "123}}".populate([]), "123}");;
    assert_eq(|_|"case 9", "}}123".populate([]), "}123");;
    assert_eq(|_|"case 10", "123}}123".populate([]), "123}123");;
    assert_eq(|_|"case 11", "{}".populate(["123"]), "123");;
    assert_eq(|_|"case 12", "{}{}".populate(["123", "456"]), "123456");;
    assert_eq(|_|"case 13", "{} {}".populate(["123", "456"]), "123 456");;
    assert_eq(|_|"case 14", " {} {}".populate(["123", "456"]), " 123 456");;
    assert_eq(|_|"case 15", " {} {} ".populate(["123", "456"]), " 123 456 ");;

    assert_eq(|_|"case example 1", "{}, {}!".populate(["Hello", "world"]), "Hello, world!");;

    assert_eq(|_|"case example 2", "{{ x = {}, y = {} }}".populate([1.to_string, 2.to_string]), "{ x = 1, y = 2 }");;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_59() {
    let source = r##"
module Main;

trait a : Action {
    type Set a;
}

f : [a : Action, Set a = m] (m, a) -> m;
f = |tree| (
    undefined("")
);

g : /*[a : Action, Set a = m]*/ (m, a) -> m; // I forgot to add predicates here.
g = f;

main : IO ();
main = (
    let _ = ((), ()).g;
    pure()
);
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "`a : Main::Action` cannot be deduced",
    );
}

#[test]
pub fn test_regression_issue_60() {
    let source = r##"
module Main;

type FFTDirection = union {
    forward : (),
    inverse : (),
};

fft_zp : FFTDirection -> I64 -> I64;
fft_zp = |dir, n| (
    match dir {
        forward => n, // I forgot to add `()` here, then the compiler panics.
        inverse => 2*n,
    }
);

main : IO ();
main = (
    let res = fft_zp(inverse(), 42);
    assert_eq(|_|"", res, 84);;
    pure()
);
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Pattern after `forward` is unreachable.",
    );
}

#[test]
pub fn test_array_resize() {
    let source = r##"
module Main;

main : IO ();
main = (
    let arr = [1, 2, 3];
    let resized = arr.resize(5, 0);
    assert_eq(|_|"1", resized, [1, 2, 3, 0, 0]);;

    let arr = arr.resize(2, 0);
    assert_eq(|_|"2", arr, [1, 2]);;

    let arr = [[1], [2], [3]];
    let resized = arr.resize(5, []);
    assert_eq(|_|"3", resized, [[1], [2], [3], [], []]);;

    let arr = arr.resize(2, []);
    assert_eq(|_|"4", arr, [[1], [2]]);;

    let arr = arr.resize(0, []);
    assert_eq(|_|"5", arr, []);;

   
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_reverse() {
    let source = r##"
module Main;

main : IO ();
main = (
    // Basic case
    let arr = [] : Array I64;
    let arr = arr.reverse;
    assert_eq(|_|"1", arr, []);;

    let arr = [1];
    let arr = arr.reverse;
    assert_eq(|_|"2", arr, [1]);;

    let arr = [0, 1];
    let arr = arr.reverse;
    assert_eq(|_|"3", arr, [1, 0]);;

    let arr = [0, 1, 2];
    let arr = arr.reverse;
    assert_eq(|_|"3", arr, [2, 1, 0]);;

    // Boxed & shared case
    let arr = [[0], [1], [2]];
    let rev = arr.reverse;
    assert_eq(|_|"4", rev, [[2], [1], [0]]);;
    assert_eq(|_|"5", arr, [[0], [1], [2]]);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_concise_defn() {
    let source = r##"
module Main;

fib:I64->I64=|n|(
    if n <= 1 {
        1
    } else {
        fib(n-1) + fib(n-2)
    }
);

main : IO () = (
    let res = fib(5);
    res.to_string.println;;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_dedup1() {
    let source = r##"
module Main;

test_unboxed : IO () = (
    let x = [] : Array I64;
    let y = x.dedup;
    assert_eq(|_|"empty", y, []);;
    assert_eq(|_|"empty,immu", x, []);;

    let x = [1];
    let y = x.dedup;
    assert_eq(|_|"single", y, [1]);;
    assert_eq(|_|"single,immu", x, [1]);;

    let x = [1, 2];
    let y = x.dedup;
    assert_eq(|_|"double,nodup", y, [1, 2]);;
    assert_eq(|_|"double,nodup,immu", x, [1, 2]);;

    let x = [1, 2, 3];
    let y = x.dedup;
    assert_eq(|_|"triple,nodup", y, [1, 2, 3]);;
    assert_eq(|_|"triple,nodup,immu", x, [1, 2, 3]);;

    let x = [1, 1];
    let y = x.dedup;
    assert_eq(|_|"double,dup", y, [1]);;
    assert_eq(|_|"double,dup,immu", x, [1, 1]);;

    let x = [1, 1, 3];
    let y = x.dedup;
    assert_eq(|_|"triple,dup1", y, [1, 3]);;
    assert_eq(|_|"triple,dup1,immu", x, [1, 1, 3]);;

    let x = [1, 2, 1];
    let y = x.dedup;
    assert_eq(|_|"triple,nodup2", y, [1, 2, 1]);;
    assert_eq(|_|"triple,nodup2,immu", x, [1, 2, 1]);;

    let x = [1, 1, 2];
    let y = x.dedup;
    assert_eq(|_|"triple,dup2", y, [1, 2]);;
    assert_eq(|_|"triple,dup2,immu", x, [1, 1, 2]);;

    let x = [1, 1, 1];
    let y = x.dedup;
    assert_eq(|_|"triple,dup3", y, [1]);;
    assert_eq(|_|"triple,dup3,immu", x, [1, 1, 1]);;

    let x = [1, 2, 2, 3, 3, 3, 4];
    let y = x.dedup;
    assert_eq(|_|"mixed_dups", y, [1, 2, 3, 4]);;
    
    let x = [5, 5, 5, 5, 5];
    let y = x.dedup;
    assert_eq(|_|"all_same", y, [5]);;

    pure()
);

main : IO ();
main = (
    test_unboxed;;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_array_dedup2() {
    let source = r##"
module Main;

test_boxed : IO () = (
    let x = [] : Array (Array I64);
    let y = x.dedup;
    assert_eq(|_|"empty", y, []);;
    assert_eq(|_|"empty,immu", x, []);;

    let x = [[1]];
    let y = x.dedup;
    assert_eq(|_|"single", y, [[1]]);;
    assert_eq(|_|"single,immu", x, [[1]]);;

    let x = [[1], [2]];
    let y = x.dedup;
    assert_eq(|_|"double,nodup", y, [[1], [2]]);;
    assert_eq(|_|"double,nodup,immu", x, [[1], [2]]);;

    let x = [[1], [2], [3]];
    let y = x.dedup;
    assert_eq(|_|"triple,nodup", y, [[1], [2], [3]]);;
    assert_eq(|_|"triple,nodup,immu", x, [[1], [2], [3]]);;

    let x = [[1], [1]];
    let y = x.dedup;
    assert_eq(|_|"double,dup", y, [[1]]);;
    assert_eq(|_|"double,dup,immu", x, [[1], [1]]);;

    let x = [[1], [1], [3]];
    let y = x.dedup;
    assert_eq(|_|"triple,dup1", y, [[1], [3]]);;
    assert_eq(|_|"triple,dup1,immu", x, [[1], [1], [3]]);;

    let x = [[1], [2], [1]];
    let y = x.dedup;
    assert_eq(|_|"triple,nodup2", y, [[1], [2], [1]]);;
    assert_eq(|_|"triple,nodup2,immu", x, [[1], [2], [1]]);;

    let x = [[1], [1], [2]];
    let y = x.dedup;
    assert_eq(|_|"triple,dup2", y, [[1], [2]]);;
    assert_eq(|_|"triple,dup2,immu", x, [[1], [1], [2]]);;

    let x = [[1], [1], [1]];
    let y = x.dedup;
    assert_eq(|_|"triple,dup3", y, [[1]]);;
    assert_eq(|_|"triple,dup3,immu", x, [[1], [1], [1]]);;

    pure()
);

main : IO ();
main = (
    test_boxed;;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_indeterminate_type_variable() {
    let source = r##"
module Main;

f : Bool = g == g;

g : Array a;
g = [];

main : IO ();
main = (
    f.to_string.println
);
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Cannot infer the type of this expression",
    );
}

#[test]
pub fn test_indeterminate_type_variable2() {
    let source = r##"
module Main;

main : IO ();
main = (
    assert_eq(|_|"", [].get_size, 0)
);
    "##;
    test_source_fail(
        &source,
        Configuration::compiler_develop_mode(),
        "Cannot infer the type of this expression",
    );
}

#[test]
pub fn test_regression_issue_62() {
    let source = r##"
module Main;

test : IO ();
test = (
    let x = FFI_CALL[CUnsignedShort f(CUnsignedShort), undefined("")];
    let x = FFI_CALL[CUnsignedLongLong f(CUnsignedLongLong), undefined("")];
    let x = FFI_CALL[CUnsignedLong f(CUnsignedLong), undefined("")];
    let x = FFI_CALL[CUnsignedInt f(CUnsignedInt), undefined("")];
    let x = FFI_CALL[CUnsignedChar f(CUnsignedChar), undefined("")];
    let x = FFI_CALL[CSizeT f(CSizeT), undefined("")];
    let x = FFI_CALL[CShort f(CShort), undefined("")];
    let x = FFI_CALL[CLongLong f(CLongLong), undefined("")];
    let x = FFI_CALL[CLong f(CLong), undefined("")];
    let x = FFI_CALL[CInt f(CInt), undefined("")];
    let x = FFI_CALL[CFloat f(CFloat), undefined("")];
    let x = FFI_CALL[CDouble f(CDouble), undefined("")];
    let x = FFI_CALL[CChar f(CChar), undefined("")];
    pure()
);

main : IO ();
main = (
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unwrap_newtype_partial_application() {
    let source = r##"
module Main;

type Foo a b = struct { data : a -> b };

type [f : *->*] Bar f = struct { data : f Bool, dummy : () };
// By having `dummy`, `Bar` is no longer a newtype, and `unwrap_newtype` in types.rs is called for `Foo I64`.

main : IO () = (
    let f : Bar (Foo I64) = Bar { data : Foo { data : |x| (x + 1) > 0 }, dummy : () };
    assert_eq(|_|"", (f.@data.@data)(1), true);;
    assert_eq(|_|"", (f.@data.@data)(-1), false);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unwrap_newtype_bug_regression() {
    // A regression test for a bug that existed in the implementation of `unwrap-newtype`.
    let source = r##"
module Main;

type [m: * -> *] Reader e a = unbox struct {
    data: e -> a
};

reader: (e -> a) -> Reader e a;
reader = |f| Reader { data: f };

run_reader: e -> Reader e a -> a;
run_reader = |e, ra| (ra.@data)(e);

main : IO () = (
    let r_sub  = |p:I64| reader $ |q:I64| (p,q);
    let r_main  = reader $ |p:I64| r_sub(p);
    let actual = r_main.run_reader(3).run_reader(4);
    
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_64() {
    // This is also a test for remove-hktvs and unwrap-newtype.
    let source = r##"
module Main;

type [f : (* -> *) -> *] Foo f = struct { data : f IO };
type [f : * -> *] Bar f = struct { data : f () };

main : IO () = (
    let foobar : Foo Bar = Foo { data : Bar { data : println("Hello, World!") } };
    foobar.@data.@data;;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_64_boxed() {
    // This is also a test for remove-hktvs and unwrap-newtype.
    let source = r##"
module Main;

type [f : (* -> *) -> *] Foo f = box struct { data : f IO };
type [f : * -> *] Bar f = box struct { data : f () };

main : IO () = (
    let foobar : Foo Bar = Foo { data : Bar { data : println("Hello, World!") } };
    foobar.@data.@data;;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_64_more_complex() {
    // This is also a test for remove-hktvs and unwrap-newtype.
    let source = r##"
module Main;

type [f : (* -> *) -> *] Foo f = struct { data : f (Reader I64) }; 
type [f : * -> *] Bar f = struct { data : f I64 };
// → Foo Bar is isomorphic to Reader I64 I64

type [m: * -> *] Reader e a = unbox struct {
    data: e -> a
};

impl Reader e : Monad {
    pure = |a| Reader { data: |_| a };
    bind = |f, ra| Reader { data: |e|
        let a = (ra.@data)(e);
        let rb = f(a);
        (rb.@data)(e)
    };
}   

impl Reader e : Functor {
    map = |f, ra| Reader { data: |e| f((ra.@data)(e)) };
}

reader: (e -> a) -> Reader e a;
reader = |f| Reader { data: f };

run_reader: e -> Reader e a -> a;
run_reader = |e, ra| (ra.@data)(e);

get_env : Reader e e;
get_env = Reader { data: |e| e };

main : IO () = (
    let foobar : Foo Bar = Foo { data : Bar { data : get_env } };
    let res = (foobar.@data.@data).run_reader(42);
    assert_eq(|_|"1", res, 42);;

    let foobar = foobar.mod_data(mod_data(map(|x| x / 6)));
    let res = (foobar.@data.@data).run_reader(42);
    assert_eq(|_|"2", res, 7);;

    let foobar = foobar.mod_data(set_data(reader $ |r| r + 1));
    let res = (foobar.@data.@data).run_reader(41);
    assert_eq(|_|"2", res, 42);;    

    let foobar = foobar.(Foo::act_data << Bar::act_data $ |r| some(r));
    let res = (foobar.as_some.@data.@data).run_reader(41);
    assert_eq(|_|"3", res, 42);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_64_more_complex_boxed() {
    // This is also a test for remove-hktvs and unwrap-newtype.
    let source = r##"
module Main;

type [f : (* -> *) -> *] Foo f = box struct { data : f (Reader I64) }; 
type [f : * -> *] Bar f = box struct { data : f I64 };
// → Foo Bar is isomorphic to Reader I64 I64

type [m: * -> *] Reader e a = unbox struct {
    data: e -> a
};

impl Reader e : Monad {
    pure = |a| Reader { data: |_| a };
    bind = |f, ra| Reader { data: |e|
        let a = (ra.@data)(e);
        let rb = f(a);
        (rb.@data)(e)
    };
}   

impl Reader e : Functor {
    map = |f, ra| Reader { data: |e| f((ra.@data)(e)) };
}

reader: (e -> a) -> Reader e a;
reader = |f| Reader { data: f };

run_reader: e -> Reader e a -> a;
run_reader = |e, ra| (ra.@data)(e);

get_env : Reader e e;
get_env = Reader { data: |e| e };

main : IO () = (
    let foobar : Foo Bar = Foo { data : Bar { data : get_env } };
    let res = (foobar.@data.@data).run_reader(42);
    assert_eq(|_|"1", res, 42);;

    let foobar = foobar.mod_data(mod_data(map(|x| x / 6)));
    let res = (foobar.@data.@data).run_reader(42);
    assert_eq(|_|"2", res, 7);;

    let foobar = foobar.mod_data(set_data(reader $ |r| r + 1));
    let res = (foobar.@data.@data).run_reader(41);
    assert_eq(|_|"2", res, 42);;    

    let foobar = foobar.(Foo::act_data << Bar::act_data $ |r| some(r));
    let res = (foobar.as_some.@data.@data).run_reader(41);
    assert_eq(|_|"3", res, 42);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unwrap_newtype_cont() {
    let source = r##"
// This test code is based on the continuation monad transformer implementation in:
// https://github.com/pt9999/fixlang-minilib-monad/blob/4f1b67d9f4328ce6ae8ec14f38cde5b4bea1737a/lib/monad/cont.fix
// 
// MIT License
// 
// Copyright (c) 2024 pt9999
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

module Main;

type [m: * -> *] ContT r m a = unbox struct {
    data: (a -> m r) -> m r
};

trait MonadCont = Monad + MonadContIF;

// A trait for generic continuation  monads.
trait [cm: * -> *] cm: MonadContIF {
    // `call_cc(f)` calls `f` with the current continuation, and returns a continuation monad.
    // `f` takes the current continuation (the exit function) and should return a continuation monad.
    // For example, `call_cc(|exit| ... if condition { exit(false) }; ... pure(true))`.
    // The exit function can be passed to another function.
    call_cc: ((a -> cm b) -> cm a) -> cm a;
}

type [cm: * -> *] JmpBuf cm a b = box struct {
    // the argument of a jump
    arg: a,
    // current continuation of a jump
    cc: JmpBuf cm a b -> cm b
};

namespace JmpBuf {
    setjmp: [cm: MonadCont] a -> cm (JmpBuf cm a b);
    setjmp = |arg| (
        call_cc(|cc| pure $ JmpBuf { arg: arg, cc: cc })
    );

    longjmp: [cm: MonadCont] a -> JmpBuf cm a b -> cm b;
    longjmp = |arg, jmpbuf| (
        (jmpbuf.@cc)(jmpbuf.set_arg(arg))
    );
}

impl [m: Monad] ContT r m: MonadContIF {
    //call_cc: ((a -> cm b) -> cm a) -> cm a;
    call_cc = |fa_cmb_cma| (            // fa_cmb_cma: (a -> ContT r m b) -> ContT r m a
        cont_t $ |fa_mr|                // fa_mr: a -> m r
        let fa_cmb = |a| (              // fa_cmb: a -> ContT r m b  (exit function)
            cont_t $ |fb_mr|            // fb_mr: b -> m r (ignored)
            fa_mr(a)                    // ignores fb_mr and calls the final continuation function
        );
        let cma = fa_cmb_cma(fa_cmb);
        cma.run_cont_t(fa_mr)
    );
}

impl [m: Monad] ContT r m: Monad {
    pure = |a| cont_t $ |famr| famr(a);
    bind = |facmb, cma| (           // facmb: a -> ContT r m b, cma: ContT r m a
        cont_t $ |fbmr|             // fbmr: b -> m r
        cma.run_cont_t(|a|
            let cmb = facmb(a);     // cmb: ContT r m b
            cmb.run_cont_t(fbmr)
        )
    );
}

// Creates a ContT monad from a function which receives a continuation function and returns the monadic value of the result.
cont_t: [m: Monad] ((a -> m r) -> m r) -> ContT r m a;
cont_t = |f| ContT { data: f };

// Runs a ContT monad with the supplied continuation function.
run_cont_t: [m: Monad] (a -> m r) -> ContT r m a -> m r;
run_cont_t = |fa_mr, cma| (
    (cma.@data)(fa_mr)
);

main: IO ();
main = (
    let cma: ContT String IOFail String = do {
        let jmpbuf = *setjmp(0);
        let i = jmpbuf.@arg;
        //eval *eprintln("i=" + i.to_string).lift.lift_t;
        if i < 5 { jmpbuf.longjmp(i + 1) };
        pure(i.to_string)
    };
    let actual = *cma.run_cont_t(|i| pure $ i).try(exit_with_msg(1));
    let expected = "5";
    assert_eq(|_|"", actual, expected)
);

    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_generic_type_annotation() {
    let source = r##"
module Main;

apply : (a -> b) -> a -> b;
apply = |f : a -> b, x : a| f(x : a) : b;

main: IO ();
main = (
    let x = apply(|y| y + 1, 41);
    assert_eq(|_|"", x, 42);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_unwrap_newtype() {
    let source = r##"
module Main;

type Header = (String, String);

type Headers = unbox struct {
    iter: DynIterator Header
};

main: IO ();
main = (
    let headers = Headers { iter : DynIterator::empty };
    let headers = headers.mod_iter(|iter| iter.push_front(("Content-Type", "text/plain")).to_dyn);
    let (_, (key, value)) = headers.@iter.advance.as_some;
    assert_eq(|_|"", key, "Content-Type");;
    assert_eq(|_|"", value, "text/plain");;
    pure()
);

    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_63() {
    let source = r##"
module Main;

type State s a = unbox struct { run : s -> (s, a) };

impl State s : Monad {
    bind = |f, x| State { run : |state| (
        let (state, r) = (x.@run)(state);
        (f(r).@run)(state)
    )};
    pure = |v| State { run : |state| (state, v) };
}

eval_state : s -> State s a -> a;
eval_state = |s, ma| let (s, a) = (ma.@run)(s); a;

type LargeState = (U64, U64, U64, U64, U64, U64, U64, U64, U64, U64, U64, U64);

init : LargeState = (0_U64, 0_U64, 0_U64, 0_U64, 0_U64, 0_U64, 0_U64, 0_U64, 0_U64, 0_U64, 0_U64, 0_U64);

type MyStateM = State LargeState;

main : IO ();
main = (
    eval range(0, 10000000).fold_m(false, |_, x| x.pure : MyStateM Bool).eval_state(init);
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_eval_0() {
    // This program verifies memory management of GenerationContext::eval_eval.
    let source = r##"
module Main;

create_array : Array I64 = (
    let arr = [1,2,3,4,5];
    eval debug_println(arr.to_string); // eval unboxed value
    eval arr; // eval boxed value
    arr
);

main : IO () = (
    let sum = range(0, 100).fold(0, |_, sum| 
        let arr = create_array;
        sum + arr.get_size
    );
    assert_eq(|_|"", sum, 5*100);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_66() {
    let source = r##"
module Main;

type Obj = unbox struct {
    arr: Array I64,
    f: I64 -> I64,
};
execute: Obj -> Obj = |obj| (
    let f = obj.@f;
    obj.mod_arr(|arr|
        arr.assert_unique(|_| "arr")
        .push_back(f(4))
    )
);
main: IO () = (
    let obj: Obj = Obj { arr: [1, 2, 3], f: add(10) };
    let obj = obj.execute;
    println(obj.@arr.to_string)
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_regression_issue_67() {
    let source = r##"
module Main;

type Obj = unbox struct {
    arr: Array I64,
};

execute: I64 -> Obj -> (Obj, I64) = |i, obj| (
    let val = obj.@arr.@(i);
    let obj2 = obj.mod_arr(|arr|
        arr.assert_unique(|_| "arr")
    );
    (obj2, val)
);

main: IO () = (
    let obj = Obj { arr: [ 42 ] };
    let (obj, val) = obj.execute(0);
    println(val.to_string)
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_index_syntax_0() {
    let source = r##"
module Main;

main : IO () = (
    let arr = [0, 1, 2];
    let x = arr[1].iget;
    assert_eq(|_|"", x, 1);;

    let arr2 = [[0, 0, 0], [0, 1, 0], [0, 0, 0]];
    let x = arr2[1][1].iget;
    assert_eq(|_|"", x, 1);;

    let arr = [0, 0, 0];
    let arr = arr[1].iset(42);
    assert_eq(|_|"", arr, [0, 42, 0]);;

    let arr2 = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let arr2 = arr2[1][1].iset(42);
    assert_eq(|_|"", arr2, [[0, 0, 0], [0, 42, 0], [0, 0, 0]]);;

    let arr2 = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let arr2 = arr2[1][1].imod(add(42));
    assert_eq(|_|"", arr2, [[0, 0, 0], [0, 42, 0], [0, 0, 0]]);;

    let arr2 = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let arr2 = arr2[1][1].iact(some);
    assert_eq(|_|"", arr2, some $ [[0, 0, 0], [0, 0, 0], [0, 0, 0]]);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_index_syntax_1() {
    let source = r##"
module Main;

type A = box struct { i : (Array I64,) };
type B = box struct { a : A };
type C = struct { x : B };
type D = struct { x : C };

main: IO () = (
    let a = A { i : ([0],) };
    let b = B { a : a };
    let c = C { x : b };
    let d = D { x : c };
    let d = d[^x][^x][^a][^i][^0][0].iset(42);
    let v = d[^D::x][^::Main::C::x][^a][^i][^Tuple1::0][0].iget;
    assert_eq(|_|"", v, 42);;
    let d = d[^x][^x][^a][^i][^0].imod(|arr| arr.push_back(84));
    let v = d[^x][^x][^a][^i][^0][1].iget;
    assert_eq(|_|"", v, 84);;
    let (d, v) = d[^x][^x][^a][^i][^0][1].ixchg(42);
    assert_eq(|_|"", v, 84);;
    let v = d[^x][^x][^a][^i][^0][1].iget;
    assert_eq(|_|"", v, 42);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_index_syntax_2() {
    let source = r##"
module Main;

main: IO () = (
    let arr = [[1,2,3],[4,5,6],[7,8,9]][1][1].imod(add(10));
    assert_eq(|_|"", arr, [[1,2,3],[4,15,6],[7,8,9]]);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_index_syntax_3() {
    let source = r##"
module Main;

type MyType = struct { data : (I64, I64, I64) };

impl MyType : Indexable {
    type Elem (MyType) = I64;
    type Index (MyType) = Array I64;
    act_at_index = |i, f, obj| (
        let i = i.to_iter.sum % 3;
        if i == 0 {
            obj[^data][^0].iact(f)
        } else if i == 1 {
            obj[^data][^1].iact(f)
        } else {
            obj[^data][^2].iact(f)
        }
    );
}

main: IO () = (
    let obj = MyType { data : (10, 20, 30) };
    let obj = obj[[42]].iget;
    assert_eq(|_|"", obj, 10);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_index_syntax_4() {
    // Forbid using user-defined `act_foo` with index syntax
    let source = r#"
    module Main;

    act_foo : [f : Functor] (a -> f a) -> (a,) -> f (a,) = act_0;
    
    main : IO ();
    main = (
        let x = (42,)[^foo].iget;
        assert_eq(|_|"", x, 42);;
        pure()
    );
    "#;
    test_source_fail(source, Configuration::compiler_develop_mode(), "Unknown");
}

#[test]
pub fn test_index_syntax_2d_array() {
    let source = r##"
module Main;

type Array2d a = struct {
    width : I64,
    data : Array a
};

impl Array2d a : Indexable {
    type Elem (Array2d a) = a;
    type Index (Array2d a) = (I64, I64);
    act_at_index = |(i, j), f, arr| (
        let i = i * arr.@width + j;
        arr[^data][i].iact(f)
    );
}

main: IO () = (
    let arr = Array2d {
        width : 3,
        data : [
            0, 1, 2, 
            3, 4, 5, 
            6, 7, 8
        ]
    };
    let x = arr[(0, 1)].iget + arr[(1, 0)].iget + arr[(2, 1)].iget + arr[(1, 2)].iget;
    assert_eq(|_|"", x, 16);;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_identity_monad() {
    // Test Identity Monad: pure and bind
    let source = r#"
    module Main;
    
    main : IO ();
    main = (
        // Test pure
        let id_val = pure(42);
        assert_eq(|_|"pure", id_val.@data, 42);;
        
        // Test bind with a simple function
        let id_result = id_val.bind(|x| pure(x + 10));
        assert_eq(|_|"bind", id_result.@data, 52);;
        
        // Test bind chaining
        let id_chain = pure(5)
            .bind(|x| pure(x * 2))
            .bind(|x| pure(x + 3))
            .bind(|x| pure(x * x));
        assert_eq(|_|"bind chain", id_chain.@data, 169);;
        
        // Test monad laws
        // Left identity: pure(a).bind(f) == f(a)
        let left_identity_lhs = pure(10).bind(|x| pure(x + 5));
        let left_identity_rhs = pure(10 + 5);
        assert_eq(|_|"left identity", left_identity_lhs.@data, left_identity_rhs.@data);;
        
        // Right identity: m.bind(pure) == m
        let right_identity_m = pure(20);
        let right_identity_result = right_identity_m.bind(pure);
        assert_eq(|_|"right identity", right_identity_result.@data, 20);;
        
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_recursive_reference() {
    // Even with recursive definitions, it should compile as long as the values are not used.
    let source = r#"
    module Main;

    x : I64;
    x = x;

    y : I64;
    y = z;

    z : I64;
    z = y;
    
    main : IO ();
    main = (
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_0() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl Array a : MyToString {
        my_to_string : Array a -> String = |_|"my_to_string";
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "my_to_string");;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_1() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl Array a : MyToString {
        my_to_string : Array a -> String;
        my_to_string = |_|"my_to_string";
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "my_to_string");;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_use_in_annotation() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [a : ToString] Array a : MyToString {
        my_to_string : Array a -> String;
        my_to_string = |arr : Array a| (
            arr.to_iter.map(to_string : a -> String).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_mismatch_type() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : b -> a -> String;
    }

    impl [a : ToString] Array a : MyToString {
        my_to_string : a -> Array a -> String;
        my_to_string = |_: a, arr : Array a| (
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string(1), "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "Type signature of member `my_to_string` is not equivalent to the one in the trait definition.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_redundant_kind_sign() {
    let source = r#"
    module Main;

    trait a : MyTrait {
        my_member : (f, a);
    }

    type MyType = struct {};

    impl MyType : MyTrait {
        my_member : [f:*] (f, MyType);
        my_member = undefined("");
    }
    
    main : IO ();
    main = (
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_undefined_type_var_0() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [b : ToString] Array b : MyToString {
        my_to_string : Array b -> String;
        my_to_string = |arr : a| (
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "Unknown type variable `a`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_undefined_type_var_1() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [b : ToString] Array b : MyToString {
        my_to_string = |arr : a| ( // `a` is defined in the trait, but not in this scope
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "Unknown type variable `a`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_duplicated_type_sig() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [a : ToString] Array a : MyToString {
        my_to_string : Array a -> String;
        my_to_string : Array a -> String = |arr : Array a| (
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "Duplicate the type signature of member `my_to_string`",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_type_sig_on_nonexistent_member() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [a : ToString] Array a : MyToString {
        to_string : Array a -> String;

        my_to_string : Array a -> String = |arr : Array a| (
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "`to_string` is not a member of trait `Main::MyToString`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_use_type_alias_and_trait_alias_0() {
    let source = r#"
    module Main;

    trait a : MyTrait {
        member : [b : Zero, b : Add] b -> b -> () -> a;
    }

    impl I64 : MyTrait {
        member : [b : Additive] b -> b -> Lazy I64 = |x, y, _| eval x; eval y; 42;
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", MyTrait::member(0, 0, ()), 42);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_use_type_alias_and_trait_alias_1() {
    let source = r#"
    module Main;

    trait a : MyTrait {
        member : [b : Additive] b -> b -> Lazy a;
    }

    impl I64 : MyTrait {
        member : [b : Zero, b : Add] b -> b -> () -> I64 = |x, y, _| eval x; eval y; 42;
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", MyTrait::member(0, 0, ()), 42);;
        pure()
    );
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_example_in_changelog_0() {
    let source = r#"
module Main;

trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

type MyType a = struct {};

impl MyType : MyFunctor {
    mymap = |f : a -> b, x : MyType a| MyType{} : MyType b;
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(to_U64);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "Unknown type variable `a`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_example_in_changelog_1() {
    let source = r#"
module Main;

trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

type MyType a = struct {};

impl MyType : MyFunctor {
    mymap : (a -> b) -> MyType a -> MyType b = |f : a -> b, x : MyType a| MyType{} : MyType b;
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(to_U64);
    pure()
);
    "#;
    test_source(source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_lacking_impl() {
    let source = r#"
module Main;

trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

type MyType a = struct {};

impl MyType : MyFunctor {
    mymap : (a -> b) -> MyType a -> MyType b;
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(to_U64);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "Lacking implementation of member `mymap`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_wrong_member_name() {
    let source = r#"
module Main;

trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

type MyType a = struct {};

impl MyType : MyFunctor {
    my_map : (a -> b) -> MyType a -> MyType b;
    mymap = |f, x| MyType{};
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(to_U64);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "`my_map` is not a member of trait `Main::MyFunctor`.",
    );
}

#[test]
pub fn test_impl_undefined_trait() {
    let source = r#"
module Main;

type MyType a = struct {};

impl MyType : MyFunctor {
    mymap = |f, x| MyType{};
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(to_U64);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "Unknown trait name `MyFunctor`.",
    );
}
