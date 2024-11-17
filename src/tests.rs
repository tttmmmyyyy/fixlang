use std::{
    fs::{self, remove_file, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    misc::{function_name, number_to_varname, split_by_max_size},
    run_file, test_source, test_source_fail, Configuration, Graph, SubCommand,
    COMPILER_TEST_WORKING_PATH, I16_NAME, I32_NAME, I64_NAME, I8_NAME, U16_NAME, U32_NAME,
    U64_NAME, U8_NAME,
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test22() {
    // Test recursion function defined by fix with two variables that is tail-call.
    let n: i64 = 1000000;
    let source = format!(
        r#"
            module Main;             main : IO ();
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test23() {
    // Test Array::fill of size 0.
    let source = r#"
        module Main;         main : IO ();
        main = (
            let arr = Array::fill(0, 42);
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test25() {
    // Test Array::get.
    let source = r#"
        module Main;         main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let elem = arr.@(50);
            assert_eq(|_|"", elem, 42);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test26() {
    // Test Array::set (unique case).
    let source = r#"
        module Main;         main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let arr = arr.set(50, 21);
            assert_eq(|_|"", arr.@(50), 21);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test27() {
    // Test Array::set (shared case).
    let source = r#"
        module Main;         main : IO ();
        main = (
            let arr0 = Array::fill(100, 42);
            let arr1 = arr0.set(50, 21);
            assert_eq(|_|"", arr0.@(50) + arr1.@(50), 63);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test27_5() {
    // Test Array of boxed object.
    let source = r#"
        module Main;         main : IO ();
        main = (
            let arr = Array::from_map(100) $ |i| add(i);
            let arr = arr.set(99, |x| x - 100);
            assert_eq(|_|"", arr.@(99) $ arr.@(50) $ 1, 1 + 50 - 100);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test28() {
    // Calculate Fibonacci sequence using array.
    let source = r#"
        module Main;         main : IO ();
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test30() {
    // Test dollar combinator
    let source = r#"
        module Main;         main : IO ();
        main = (
            let f = |x| x + 3;
            let g = |x| x == 8;
            let ans = g $ f $ 5;
            assert_eq(|_|"", if ans { 1 } else { 0 }, 1);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test31() {
    // Test . combinator
    let source = r#"
        module Main;         main : IO ();
        main = (
            let f = |x| x + 3;
            let g = |x| x == 8;
            let ans = 5 .f. g;
            assert_eq(|_|"", if ans { 1 } else { 0 } , 1);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test32() {
    // Test . and $ combinator
    let source = r#"
        module Main;         main : IO ();
        main = (
            let f = |x| x + 10;
            assert_eq(|_|"", 5.add $ 3.f, 18);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test33() {
    // Test struct declaration and new, mod.
    let source = r#"
        module Main;         type I64Bool = box struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool { x: 18, y: false };
            let obj = I64Bool::mod_x(|x| x + 42, obj);
            assert_eq(|_|"", I64Bool::@x(obj), 60);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test34_5() {
    // Test unboxed struct declaration and new, mod.
    let source = r#"
        module Main;         type I64Bool = unbox struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool { x: 18, y : false};
            let obj = I64Bool::mod_x(|x| x + 42, obj);
            assert_eq(|_|"", I64Bool::@x(obj), 60);;
            pure()
        );
        "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source.as_str(), Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test50_3() {
    // test loop_iter, loop_iter_m.
    let source = r#"
        module Main;         
        main : IO ();
        main = (
            let sum = Iterator::count_up(0).loop_iter(0, |sum, n| (
                if n > 100 { break $ sum };
                continue $ sum + n
            ));
            assert_eq(|_|"case-loop", sum, 100 * 101 / 2);;

            let sum = *Iterator::count_up(0).loop_iter_m(0, |sum, n| (
                if n > 5 { break_m $ sum };
                (print $ n.to_string + " ");;
                continue_m $ sum + n
            ));
            assert_eq(|_|"case-loop_m", sum, 5 * 6 / 2);;

            pure()
        );
            "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test64() {
    // Test escape sequence.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        println $ "\u2764"
    );
    "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test75() {
    // Test iterator.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let iter = Iterator::from_map(|i| i*i );
        let (n, iter) = iter.advance.as_some;
        assert_eq(|_|"", n, 0*0);;
        let (n, iter) = iter.advance.as_some;
        assert_eq(|_|"", n, 1*1);;
        let (n, iter) = iter.advance.as_some;
        assert_eq(|_|"", n, 2*2);;
        let (n, iter) = iter.advance.as_some;
        assert_eq(|_|"", n, 3*3);;
        let (n, iter) = iter.advance.as_some;
        assert_eq(|_|"", n, 4*4);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test77() {
    // Test Iterator::zip / map / take / fold / subsequences.
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

        let subs = (Iterator::empty : Iterator I64).subsequences;
        assert_eq(|_|"subsequences 1", subs.get_size, 1);;
        assert_eq(|_|"subsequences 2", subs.advance.as_some.@0.get_size, 0);;

        let subs = [1,2,3].to_iter.subsequences;
        assert_eq(|_|"subsequences 3", subs.map(to_array).to_array, [[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]]);;

        pure()
    );
    "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
        let (e, ls) = ls.advance.as_some;
        assert_eq(|_|"", 2, e);;
        let (e, ls) = ls.advance.as_some;
        assert_eq(|_|"", 1, e);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test80() {
    // Test Iterator::last
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let iter = Iterator::empty.push_front(4).push_front(3).push_front(2).push_front(1);
        let last = iter.find_last.as_some;
        assert_eq(|_|"", last, 4);;
        let last: Option Bool = Iterator::empty.find_last;
        assert(|_|"", last.is_none);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
        let v = [[1], [2], [3]].to_iter.fold(res, |res, v| (
            res.assert_unique(|_|"the array is not unique!").append(v)
        ));
        assert_eq(|_|"", v, [1, 2, 3]);;

        pure()
    );
    "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test87() {
    // Test iterator comparison
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([1,2,3]);
        assert_eq(|_|"", lhs, rhs);;

        let lhs: Iterator Bool = Iterator::from_array([]);
        let rhs = Iterator::from_array([]);
        assert_eq(|_|"", lhs, rhs);;

        let lhs = Iterator::from_array([]);
        let rhs = Iterator::from_array([1,2]);
        assert(|_|"", lhs != rhs);;

        pure()
    );
    
    "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test88() {
    // Test iterator comparison
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let iter = Iterator::from_array([1,2,3]);
        let iter = iter.intersperse(0);
        assert_eq(|_|"", iter, Iterator::from_array([1,0,2,0,3]));;
    
        let iter = Iterator::from_array([1]);
        let iter = iter.intersperse(0);
        assert_eq(|_|"", iter, Iterator::from_array([1]));;
    
        let iter = Iterator::from_array([]);
        let iter = iter.intersperse(0);
        assert_eq(|_|"", iter, Iterator::from_array([]));;
    
        pure()
    );
    
    "#;
    test_source(source, Configuration::develop_compiler_mode());
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
        assert_eq(|_|"", lhs + rhs, Iterator::from_array([1,2,3,4,5,6]));;
    
        let lhs = Iterator::from_array([]);
        let rhs = Iterator::from_array([4,5,6]);
        assert_eq(|_|"", lhs + rhs, Iterator::from_array([4,5,6]));;

        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([]);
        assert_eq(|_|"", lhs + rhs, Iterator::from_array([1,2,3]));;

        let lhs: Iterator I64 = Iterator::from_array([]);
        let rhs = Iterator::from_array([]);
        assert_eq(|_|"", lhs + rhs, Iterator::from_array([]));;
    
        pure()
    );
    
    "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test90() {
    // Test Array::sort_by.
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let vec = [5,3,1,7,4,6,9,8,2];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        assert_eq(|_|"wrong result 9", vec, [1,2,3,4,5,6,7,8,9]);;

        let vec = [1];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        assert_eq(|_|"wrong result 1", vec, [1]);;

        let vec: Array I64 = [];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        assert_eq(|_|"wrong result 0", vec, []);;

        pure()
    );
    
    "#;
    test_source(source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test92() {
    let source = r#"
    module Main; 
    main : IO ();
    main = (
        let buf = [].reserve(5);
        let vec = buf;
        let vec = vec.push_back(0);
        let buf = buf.push_back(1);
        pure()
    );

    "#;
    test_source(source, Configuration::develop_compiler_mode());
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
        let leaker = leaker.set_data(Option::some(leaker)); // doesn't make circular reference in fact.
        pure()
    );

    "#;
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
                assert_eq(|_|"fail: int_val is shared", unique, true);;

                // For boxed value, it returns true if the value isn't used later.
                let arr = Array::fill(10, 10);
                let (unique, arr) = arr.unsafe_is_unique;
                let use = arr.@(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
                assert_eq(|_|"fail: arr is shared", unique, true);;

                // Fox boxed value, it returns false if the value will be used later.
                let arr = Array::fill(10, 10);
                let (unique, _) = arr.unsafe_is_unique;
                let use = arr.@(0);
                assert_eq(|_|"fail: arr is unique", unique, false);;

                let int_val = 42;
                eval int_val.assert_unique(|_|"fail: int_val is shared (2)");
                let use = int_val + 1;

                let arr = Array::fill(10, 10);
                let arr = arr.assert_unique(|_|"fail: arr is shared (2)");
                let use = arr.@(0);

                pure()
            );
        "#;
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test96() {
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
            assert_eq(|_|"case 3", '\x7f', 127_U8);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test108() {
    // Test write_file_string, read_file_string, read_line.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let file_path = Path::parse("test_uAfQDPwJ7sS6.txt").as_some;
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
    test_source(&source, Configuration::develop_compiler_mode());
    remove_file("test_uAfQDPwJ7sS6.txt").unwrap();
}

#[test]
pub fn test_is_eof() {
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
            let file_path = Path::parse("test_bUeW9baGGZmE.txt").as_some;
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
    test_source(&source, Configuration::develop_compiler_mode());
    remove_file("test_bUeW9baGGZmE.txt").unwrap();
}

#[test]
pub fn test108_5() {
    // Test write_file_bytes, read_file_bytes.
    let source = r#"
        module Main; 
        main : IO ();
        main = (
            let file_path = Path::parse("test_vgZNhmj4gPbF.dat").as_some;
            let data = Array::from_map(1024 + 512, |n| n.to_U8);
            do {
                write_file_bytes(file_path, data);;

                let read = *read_file_bytes(file_path);
                assert_eq(|_|"case 1", data, read).lift;;

                pure()
            }.try(exit_with_msg(1))
        );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
    remove_file("test_vgZNhmj4gPbF.dat").unwrap();
}

#[test]
pub fn test109() {
    // Test monad syntax.
    let source = r#"
        module Main; 
        add_opt_int : Option I64 -> Option I64 -> Option I64;
        add_opt_int = |lhs, rhs| pure $ *lhs + *rhs;

        sequence : [m : Monad, m : Functor] Iterator (m a) -> m (Iterator a);
        sequence = |iter| (
            if iter.is_empty { pure $ Iterator::empty };
            let (x, xs_iter) = iter.advance.as_some;
            pure $ Iterator::push_front(*x) $ *sequence(xs_iter)
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
            assert_eq(|_|"case 6", res_iter.as_ok, Iterator::from_array([0, 1, 2, 3]));;

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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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

            let iter = Iterator::generate(0, |i| if i == 3 { Option::none() } else { Option::some $ (i, i+1) });
            let ans = [0, 1, 2];
            assert_eq(|_|"case 1", iter.to_array, ans);;

            pure()
        );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
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

            pure()
        );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 1-1", five.@x, 5);;

            // Case 1-2: Punch an array of two boxed values and plug-in the same element.
            let arr = [MyBox { x : 5 }, MyBox { x : 7 }];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 1-2-a", five.@x, 5);;
            let arr = parr.plug_in(five);
            assert_eq(|_|"case 1-2-b", arr.@(0).@x + arr.@(1).@x, 5 + 7);;

            // Case 1-3: Punch an array of two boxed values and plug-in the other element.
            let seven = MyBox { x : 7 };
            let arr = [MyBox { x : 5 }, seven];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 1-3-a", five.@x, 5);;
            let arr = parr.plug_in(seven);
            assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 7);;

            // Case 1-4: Punch an array of two boxed values and plug-in another value.
            let arr = [MyBox { x : 5 }, MyBox { x : 7 }];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 1-3-a", five.@x, 5);;
            let arr = parr.plug_in(MyBox { x : 11 });
            assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 11);;

            // Case 2-1: Punch an array of two shared boxed values and release parray.
            let five = MyBox { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 2-1", five.@x, 5);;

            // Case 2-2: Punch an array of two shared boxed values and plug-in the same element.
            let five = MyBox { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 2-2-a", five.@x, 5);;
            let arr = parr.plug_in(five);
            assert_eq(|_|"case 2-2-b", arr.@(0).@x + arr.@(1).@x, 5 + 5);;

            // Case 2-3: Punch an array of two shared boxed values and plug-in the value again.
            let five = MyBox { x : 5 };
            let arr = [five, five];
            let (parr, five1) = arr.unsafe_punch(0);
            assert_eq(|_|"case 2-3-a", five1.@x, 5);;
            let arr = parr.plug_in(five);
            assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 5 + 5);;

            // Case 2-4: Punch an array of two shared boxed values and plug-in another value.
            let five = MyBox { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 2-3-a", five.@x, 5);;
            let arr = parr.plug_in(MyBox { x : 7 });
            assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 5);;

            pure()
        );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
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
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 3-1", five.@x, 5);;

            // Case 3-2: Punch an array of two boxed values and plug-in the same element.
            let arr = [MyBox { x : 5 }];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 3-2-a", five.@x, 5);;
            let arr = parr.plug_in(five);
            assert_eq(|_|"case 3-2-b", arr.@(0).@x, 5);;

            // Case 4-1: Punch an array of two unboxed values and release parray.
            let arr = [5, 7];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 1-1", five, 5);;

            // Case 4-2: Punch an array of two boxed values and plug-in a value.
            let arr = [5, 7];
            let (parr, five) = arr.unsafe_punch(0);
            assert_eq(|_|"case 4-2-a", five, 5);;
            let arr = parr.plug_in(13);
            assert_eq(|_|"case 4-2-b", arr.@(0) + arr.@(1), 13 + 7);;

            pure()
        );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_array_act_0() {
    // Test Array::act
    let source = r#"
        module Main; 
        
        main : IO ();
        main = (
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test126() {
    // Test Iterator::sum.
    let source = r#"
        module Main; 

        
        main : IO ();
        main = (
            let n = 100;
            let v = Iterator::range(0, n+1).sum;
            assert_eq(|_|"", v, n*(n+1)/2);;

            pure()
        );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
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
        my_msum : [m : MyMonadAdditive] Iterator (m a) -> m a;
        my_msum = |iter| (
            let next = iter.advance;
            if next.is_none { my_fzero };
            let (act, iter) = next.as_some;
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
    test_source(&source, Configuration::develop_compiler_mode());
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
                let file_path = Path::parse("test_tMB3iCfTeeES.txt").as_some;
                write_file_string(file_path, "Hello World!").try(exit_with_msg(1));;
                let read_content = *read_file_string(file_path).try(exit_with_msg(1));
                println $ read_content
            );
            println("write/read/println time : " + t.to_string);;

            pure()
        );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        let file_path = Path::parse("test_GndeZP399tLX.txt").as_some;
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        assert_eq(|_|"", arr.get_sub(2, 4), [2, 3]);;
        assert_eq(|_|"", arr.get_sub(0, 0), []);;
        assert_eq(|_|"", arr.get_sub(3, 1), [3, 4, 0]);;
        assert_eq(|_|"", arr.get_sub(1, -1), [1, 2, 3]);;
    
        let arr : Array I64 = [];
        assert_eq(|_|"", arr.get_sub(2, 4), []);;
    
        // Boxed case
        let arr = [[0], [1], [2], [3], [4]];
        assert_eq(|_|"", arr.get_sub(2, 4), [[2], [3]]);;
        assert_eq(|_|"", arr.get_sub(0, 0), []);;
        assert_eq(|_|"", arr.get_sub(3, 1), [[3], [4], [0]]);;
        assert_eq(|_|"", arr.get_sub(1, -1), [[1], [2], [3]]);;
    
        let arr : Array (Array I64) = [];
        assert_eq(|_|"", arr.get_sub(2, 4), []);;
    
        pure()
    );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_string_get_sub() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        let str = "Hello";
        assert_eq(|_|"", str.get_sub(2, 4), "ll");;
        assert_eq(|_|"", str.get_sub(0, 0), "");;
        assert_eq(|_|"", str.get_sub(3, 1), "loH");;
        assert_eq(|_|"", str.get_sub(1, -1), "ell");;
    
        assert_eq(|_|"", "".get_sub(2, 4), "");;
    
        pure()
    );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_loop_lines_io() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        let content1 = "Hello\nWorld!";
        let file1 = Path::parse("test_MsuHh3QEXKYN.txt").as_some;
        let file2 = Path::parse("test_9A5bu4U57xTd.txt").as_some;
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_string_split() {
    let source = r#"
    module Main;
        
    main : IO ();
    main = (
        assert_eq(|_|"1", "--ab---cde----".split("--").to_array, ["", "ab", "-cde", "", ""]);;
        assert_eq(|_|"2", "ab---cde----".split("--").to_array, ["ab", "-cde", "", ""]);;
        assert_eq(|_|"3", "--ab---cde".split("--").to_array, ["", "ab", "-cde"]);;
        assert_eq(|_|"3", "ab---cde".split("--").to_array, ["ab", "-cde"]);;
        assert_eq(|_|"4", "--".split("--").to_array, ["", ""]);;
        assert_eq(|_|"5", "".split("--").to_array, [""]);;

        pure()
    );
    "#;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_graph_find_loop() {
    // Test find_loop of graph.rs.

    let g = Graph::new((0..3).collect());
    assert_eq!(g.find_loop(), vec![] as Vec<usize>);

    let mut g = Graph::new((0..3).collect());
    g.connect(0, 1);
    g.connect(1, 2);
    assert_eq!(g.find_loop(), vec![] as Vec<usize>);

    let mut g = Graph::new((0..3).collect());
    g.connect(0, 0);
    assert_eq!(g.find_loop(), vec![0 as usize]);

    let mut g = Graph::new((0..3).collect());
    g.connect(1, 1);
    assert_eq!(g.find_loop(), vec![1 as usize]);

    let mut g = Graph::new((0..3).collect());
    g.connect(0, 1);
    g.connect(2, 2);
    assert_eq!(g.find_loop(), vec![2 as usize]);

    let mut g = Graph::new((0..3).collect());
    g.connect(1, 2);
    g.connect(2, 1);
    assert_eq!(g.find_loop(), vec![1 as usize, 2 as usize]);

    let mut g = Graph::new((0..4).collect());
    g.connect(0, 1);
    g.connect(1, 2);
    g.connect(1, 3);
    g.connect(2, 3);
    assert_eq!(g.find_loop(), vec![] as Vec<usize>);

    let mut g = Graph::new((0..5).collect());
    g.connect(0, 1);
    g.connect(1, 2);
    g.connect(1, 3);
    g.connect(3, 4);
    g.connect(4, 1);
    assert_eq!(
        g.find_loop(),
        vec![1 as usize, 3 as usize, 4 as usize] as Vec<usize>
    );
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
        let mut config = Configuration::develop_compiler_mode();
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
        run_file(config);
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
    test_source(source, Configuration::develop_compiler_mode());
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
    test_source(source, Configuration::develop_compiler_mode());
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
        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
            let i = *Iterator::range(0, ress.get_size);
            let j = *Iterator::range(0, ress.get_size);
            pure $ (i, j)
        };
        indices.loop_iter_m((), |_, (i, j)| (
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
    test_source(&source, Configuration::develop_compiler_mode());
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
            let i = *Iterator::range(0, arrs.get_size);
            let j = *Iterator::range(0, arrs.get_size);
            pure $ (i, j)
        };
        indices.loop_iter_m((), |_, (i, j)| (
            assert_eq(|_|"", arrs.@(i) < arrs.@(j), i < j);;
            assert_eq(|_|"", arrs.@(i) <= arrs.@(j), i <= j);;
            continue_m $ ()
        ));;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
            let i = *Iterator::range(0, ss.get_size);
            let j = *Iterator::range(0, ss.get_size);
            pure $ (i, j)
        };
        indices.loop_iter_m((), |_, (i, j)| (
            assert_eq(|_|"", ss.@(i) < ss.@(j), i < j);;
            assert_eq(|_|"", ss.@(i) <= ss.@(j), i <= j);;
            continue_m $ ()
        ));;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
        "Type variable `a` used in trait definition has to appear in the type of a method `value`.",
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
        Configuration::develop_compiler_mode(),
        "Duplicate definition for global value: `Main::truth`.",
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
        "Name `Hoge` is ambiguous. There are `Main::A::Hoge`, `Main::B::Hoge`.",
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
        "Name `Hoge` is ambiguous. There are `Main::A::Hoge`, `Main::B::Hoge`.",
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
        Configuration::develop_compiler_mode(),
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
        "Unknown type or associated type name",
    );
}

#[test]
pub fn test_import_only_necessary() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::println};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
}
#[test]
pub fn test_import_recursive_group() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::{println, eprintln}};

    main : IO ();
    main = (
        eprintln("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_import_any_in_namespace() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::*};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
        "Namespace `Std::Piyo` is not defined or empty.",
    );
}

#[test]
pub fn test_associated_type_collects() {
    let source = r##"
    module Main;
    
    trait c : Collects {
        type Elem c;
        empty : Elem c;
        insert : Elem c -> c -> c;
        to_iter : c -> Iterator (Elem c);
    }

    impl Array a : Collects {
        type Elem (Array a) = a;
        empty = [];
        insert = |x, xs| xs.push_back(x);
        to_iter = |xs| Array::to_iter(xs);
    }

    impl Iterator a : Collects {
        type Elem (Iterator a) = a;
        empty = Iterator::empty;
        insert = |x, xs| xs.push_front(x);
        to_iter = |xs| xs;
    }

    extend : [c1 : Collects, c2 : Collects, Elem c1 = e, Elem c2 = e] c1 -> c2 -> c2;
    extend = |xs, ys| xs.to_iter.fold(ys, |ys, x| ys.insert(x));

    has_equal_elements : [c1 : Collects, c2 : Collects, Elem c1 = e, Elem c2 = e, e : Eq] c1 -> c2 -> Bool;
    has_equal_elements = |xs, ys| xs.to_iter.to_array == ys.to_iter.to_array;

    stringify : [c : Collects, Elem c = e, e : ToString] c -> String;
    stringify = |xs| xs.to_iter.map(to_string).join(", ");

    type Wrapper c = struct { data : c };

    impl [c : Collects, Elem c = e, e : ToString] Wrapper c : ToString {
        to_string = |xs| xs.@data.to_iter.map(to_string).join(", ");
    }

    impl [c : Collects] Wrapper c : Collects {
        type Elem (Wrapper c) = Elem c;
        empty = Wrapper { data : Collects::empty };
        insert = |x, w| Wrapper { data : w.@data.insert(x) };
        to_iter = |w| w.@data.to_iter;
    }

    sum_elements1 : [c : Collects, Elem c = I64] c -> I64;
    sum_elements1 = |xs| xs.to_iter.fold(0, |acc, x| acc + x);

    sum_elements2 : [c : Collects, Elem c = I64] c -> Elem c;
    sum_elements2 = |xs| xs.to_iter.fold(0, |acc, x| acc + x);

    sum_elements3 : [c : Collects, Elem c = e, e : Additive] c -> Elem c;
    sum_elements3 = |xs| xs.to_iter.sum;

    main : IO ();
    main = (
        assert_eq(|_|"", [].insert(1).insert(2).insert(3), [1, 2, 3]);;
        assert_eq(|_|"", Iterator::empty.insert(3).insert(2).insert(1).to_array, [1, 2, 3]);;
        assert_eq(|_|"", [1, 2, 3].extend([4, 5, 6]), [1, 2, 3, 4, 5, 6]);;
        assert_eq(|_|"", [1, 2, 3].extend([4, 5, 6].Collects::to_iter), [1, 2, 3, 4, 5, 6]);;
        assert_eq(|_|"", [1, 2, 3].Collects::to_iter.extend([4, 5, 6]).to_array, [6, 5, 4, 1, 2, 3]);;
        assert_eq(|_|"", [1, 2, 3].Collects::to_iter.extend([4, 5, 6].Collects::to_iter).to_array, [6, 5, 4, 1, 2, 3]);;
        assert_eq(|_|"", [1, 2, 3].has_equal_elements([1, 2, 3]), true);;
        assert_eq(|_|"", [1, 2, 3].stringify, "1, 2, 3");;
        assert_eq(|_|"", Wrapper { data : [false, true, true] }.to_string, "false, true, true");;
        assert_eq(|_|"", Wrapper { data : [false, true, true] }.Collects::to_iter.to_array, [false, true, true]);;
        assert_eq(|_|"", [1, 2, 3].sum_elements1, 6);;
        assert_eq(|_|"", [1, 2, 3].sum_elements2, 6);;
        assert_eq(|_|"", [1, 2, 3].sum_elements3, 6);;
        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_iterator_product() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1, 2, 3].to_iter.product(['a', 'b'].to_iter).to_array, [(1, 'a'), (2, 'a'), (3, 'a'), (1, 'b'), (2, 'b'), (3, 'b')]);;
        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
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
        Configuration::develop_compiler_mode(),
        "No value named `foo` matches the expected type `Std::I64 -> Std::IO ()`.",
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_read_file_after_close() {
    let source = r##"
        module Main;
        
        main: IO ();
        main = do {
            let fh = *open_file(Path::parse("/dev/null").as_some, "r");
            close_file(fh).lift;;
            let line = *read_line(fh);
            println(line).lift
        }.try(|msg|
            assert_eq(|_|"", msg, "`Std::IO::_read_line_inner` failed!: the IOHandle is already closed.");;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source_fail(&source, Configuration::develop_compiler_mode(), "");
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
    test_source_fail(&source, Configuration::develop_compiler_mode(), "");
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
        Configuration::develop_compiler_mode(),
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
            let val = instance.do_with_retained(|instance|
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
    test_source(&source, Configuration::develop_compiler_mode());
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
            some(v) => v;
            none(_) => 0;
        };
        assert_eq(|_|"", v, 42);;

        let x = Option::none();
        let v = match x {
            some(v) => v;
            none(_) => 0;
        };
        assert_eq(|_|"", v, 0);;

        // Value is boxed
        let x = Option::some(Box::make(42));
        let v = match x {
            some(v) => v;
            none(_) => Box::make(0);
        };
        assert_eq(|_|"", v.@value, 42);;

        let x : Option (Box I64) = Option::none();
        let v = match x {
            some(v) => v;
            none(_) => Box::make(0);
        };
        assert_eq(|_|"", v.@value, 0);;

        // Value is boxed and shared
        let x = Option::some(Box::make(42));
        let v = match x {
            some(v) => v;
            none(_) => Box::make(0);
        };
        assert_eq(|_|"", v.@value, 42);;
        assert_eq(|_|"", x.as_some.@value, 42);;

        // Value is a closure
        let x = Option::some(|x| x + 1);
        let v = match x {
            some(v) => v(41);
            none(_) => 0;
        };
        assert_eq(|_|"", v, 42);;

        // Value is a closure and shared
        let x = Option::some(|x| x + 1);
        let v = match x {
            some(v) => v(41);
            none(_) => 0;
        };
        assert_eq(|_|"", v, 42);;
        assert_eq(|_|"", (x.as_some)(41), 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
            left(v) => v;
            right(_ : Bool) => 0;
        };
        assert_eq(|_|"", v, 42);;

        let x = MyEither::right(false);
        let v = match x {
            left(v) => v;
            right(_) => 0;
        };
        assert_eq(|_|"", v, 0);;

        // Value is boxed
        let x = MyEither::left(Box::make(42));
        let v = match x {
            left(v) => v;
            right(_ : Bool) => Box::make(0);
        };
        assert_eq(|_|"", v.@value, 42);;

        let x = MyEither::right(false);
        let v = match x {
            left(v) => v;
            right(_ : Bool) => Box::make(0);
        };
        assert_eq(|_|"", v.@value, 0);;

        // Value is boxed and shared
        let x = MyEither::left(Box::make(42));
        let v = match x {
            left(v) => v;
            right(_ : Bool) => Box::make(0);
        };
        assert_eq(|_|"", v.@value, 42);;
        assert_eq(|_|"", x.as_left.@value, 42);;

        // Value is a closure
        let x = MyEither::left(|x| x + 1);
        let v = match x {
            left(v) => v(41);
            right(_ : Bool) => 0;
        };
        assert_eq(|_|"", v, 42);;

        // Value is a closure and shared
        let x = MyEither::left(|x| x + 1);
        let v = match x {
            left(v) => v(41);
            right(_ : Bool) => 0;
        };
        assert_eq(|_|"", v, 42);;
        assert_eq(|_|"", (x.as_left)(41), 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_match_non_exhaustive() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::some(42);
        let v = match x {
            some(v) => v;
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_compiler_mode(),
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
            some(v) => v;
            x => if x.is_none { 42 } else { 0 };
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_early_otherwise() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::none();
        let v = match x {
            x => if x.is_none { 42 } else { 0 };
            some(v) => v;
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_compiler_mode(),
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
            foo(v) => v;
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_compiler_mode(),
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
            Option::some(v) => v;
            Option::none(_) => 42;
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_match_variant_with_bad_namespace() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let x = Option::none();
        let v = match x {
            LoopResult::some(v) => v;
            Option::none(_) => 42;
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_compiler_mode(),
        "`LoopResult::some` is not a variant of union `Std::Option`.",
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
            some(MyPair { first: a, second: b }) => a * b;
            none(_) => 0;
        };
        assert_eq(|_|"", v, 42);;

        let x = Option::some((6, 7));
        let v = match x {
            some((a, b)) => a * b;
            none(_) => 0;
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
            a(v) => v;
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
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
            y => y.@value;
        };
        assert_eq(|_|"", v, 42);;

        let v = match (6, 7) {
            (x, y) => x * y;
        };
        assert_eq(|_|"", v, 42);;

        let x = MyStruct { a: 6, b: 7 };
        let v = match x {
            MyStruct { a: x, b: y } => x * y;
        };
        assert_eq(|_|"", v, 42);;
        assert_eq(|_|"", x.@a * x.@b, 42);;

        pure()
    );
    "##;
    test_source(&source, Configuration::develop_compiler_mode());
}

#[test]
pub fn test_match_on_variant_for_nonunion() {
    let source = r##"
    module Main;

    main: IO ();
    main = (
        let v = match [] {
            foo(_) => 0;
            _ => 42;
        };
        assert_eq(|_|"", v, 42);;

        pure()
    );
    "##;
    test_source_fail(&source, Configuration::develop_compiler_mode(), "The condition of `match` has type `Std::Array a` which is not union, but matched on a variant pattern `foo(_)`.");
}

#[test]
pub fn test_external_projects() {
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-math.git",
        "fixlang-math",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-hashmap.git",
        "fixlang-hashmap",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-hashset.git",
        "fixlang-hashset",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-random.git",
        "fixlang-random",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-time.git",
        "fixlang-time",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-character.git",
        "fixlang-character",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-subprocess.git",
        "fixlang-subprocess",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-regexp.git",
        "fixlang-regexp",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-asynctask.git",
        "fixlang-asynctask",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-gmp.git",
        "fixlang-gmp",
    );
    test_external_project(
        "https://github.com/tttmmmyyyy/fixlang-misc-algos.git",
        "fixlang-misc-algos",
    );
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
    let mut config = Configuration::develop_compiler_mode();
    config.add_dyanmic_library(lib_name);
    // Add the library search path.
    config.library_search_paths.push(PathBuf::from("."));

    // Run the Fix program.
    test_source(&fix_src, config);

    // Remove the shared library.
    let _ = fs::remove_file(so_file_path);
}
