use std::fs::{self, remove_file};

use super::*;

// Tests should run sequentially, since OBJECT_TABLE in libfixsanitizer.so is shared between tests and check_leak() asserts OBJECT_TABLE is empty.
#[test]
#[serial]
pub fn test0() {
    let source = r#"
            module Main;
    
            main : IO ();
            main = (
                let _ = assert_eq("", 5 + 3 * 8 / 5 + 7 % 3, 10);
                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test1() {
    let source = r#"
            module Main;
            
            main : IO ();
            main = (
                let u = assert_eq("", let x = 5 in -x, -5);
                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test2() {
    let source = r#"
            module Main;
            main : IO ();
            main = (
                let u = assert_eq("", let x = 5 in 3, 3);
                pure()
            );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test3() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", let n = -5 in let p = 5 in n, -5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test4() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", let n = -5 in let p = 5 in p, 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test5() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", let x = -5 in let x = 5 in x, 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test6() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", let x = let y = 3 in y in x, 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test7() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", (|x| 5)(10), 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test8() {
    let source = r#"
        module Main; 

        main : IO ();
        main = (
            let u = assert_eq("", (|x| x) $ 6, 6);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test9_5() {
    let source = r#"
        module Main;
        
        main : IO ();
        main = (
            let x = 3;
            let y = 5;
            let u = assert_eq("", x - y, -2);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test10() {
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let u = assert_eq("", let x = 5 in 2 + x, 7);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test11() {
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let x = 5 in 
            let y = -3 in
            let u = assert_eq("", x + y, 2);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test12() {
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let x = 5 in 
            let y = -3 in
            let z = 12 in
            let xy = x + y in
            let u = assert_eq("", xy + z, 14);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test13() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = add(5) in
            let u = assert_eq("", f(3), 5+3);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test13_5() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = add(5) in
            let u = assert_eq("", f(-3) + f(12), 5 - 3 + 5 + 12);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test14() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let x = 3 in 
            let y = 5 in
            let f = add(x) in
            let u = assert_eq("", f(y), 3 + 5);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test15() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = |x| 3 + x in
            let u = assert_eq("", f(5), 3 + 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test15_5() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let x = 3;
            let f = |y| x;
            let u = assert_eq("", f(5), 3);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test16() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = |x| x + 3 in
            let u = assert_eq("", f(5), 3 + 5);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test17() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", if true { 3 } else { 5 }, 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test18() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", if false { 3 } else { 5 }, 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test19() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", if 3 == 3 { 1 } else { 0 }, 1);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test20() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let u = assert_eq("", if 3 == 5 { 1 } else { 0 }, 0);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test20_5() {
    let source = r#"
        module Main;
        main : IO ();
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
            let u = assert_eq("", ans, 2);
            pure ()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test21() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let fact = fix $ |loop, n| if n == 0 { 1 } else { n * loop(n-1) };
                let u = assert_eq("", fact(5), 5 * 4 * 3 * 2 * 1);
                pure()
            );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test22() {
    // Test recursion function defined by fix with two variables that is tail-call.
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
                    let u = assert_eq("", g(0, {}), {});
                    pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), Configuration::release());
}

#[test]
#[serial]
pub fn test22_5() {
    // Test recursion function defined by fix that is not tail-call.
    let source = r#"
        module Main;
        main : IO ();
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
                let u = assert_eq("", fib(10), 55);
                pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
            let u = assert_eq("", fib(30), 832040);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test23() {
    // Test Array::fill of size 0.
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::fill(0, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test24() {
    // Test Array::fill of size > 0.
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let u = assert_eq("", arr.get_size, 100);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test25() {
    // Test Array::get.
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let elem = arr.get(50);
            let u = assert_eq("", elem, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test26() {
    // Test Array::set (unique case).
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let arr = arr.set(50, 21);
            let u = assert_eq("", arr.get(50), 21);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test27() {
    // Test Array::set (shared case).
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr0 = Array::fill(100, 42);
            let arr1 = arr0.set(50, 21);
            let u = assert_eq("", arr0.get(50) + arr1.get(50), 63);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test27_5() {
    // Test Array of boxed object.
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::from_map(100) $ |i| add(i);
            let arr = arr.set(99, |x| x - 100);
            let u = assert_eq("", arr.get(99) $ arr.get(50) $ 1, 1 + 50 - 100);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test28() {
    // Calculate Fibonacci sequence using array.
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let arr = Array::fill(31, 0);
            let arr = arr.set!(0, 0);
            let arr = arr.set!(1, 1);
            let loop = fix $ |f, arr: Array I64, n| (
                if n == 31 {
                    arr
                } else {
                    let x = arr.get(add(n, -1));
                    let y = arr.get(add(n, -2));
                    let arr = arr.set!(n, x+y);
                    f(arr, n+1)
                }
            );
            let fib = loop(arr, 2);
            let u = assert_eq("", fib.get(30), 832040);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test29() {
    let source = r#"
        module Main;

        id : a -> a;
        id = |x| x;

        main : IO ();
        main = (
            let u = assert_eq("", if id(true) { id(100) } else { 30 }, 100);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test30() {
    // Test dollar combinator
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = |x| x + 3;
            let g = |x| x == 8;
            let ans = g $ f $ 5;
            let u = assert_eq("", if ans { 1 } else { 0 }, 1);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test31() {
    // Test . combinator
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = |x| x + 3;
            let g = |x| x == 8;
            let ans = 5 .f. g;
            let u = assert_eq("", if ans { 1 } else { 0 } , 1);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test32() {
    // Test . and $ combinator
    let source = r#"
        module Main;
        main : IO ();
        main = (
            let f = |x| x + 10;
            let u = assert_eq("", 5.add $ 3.f, 18);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test33() {
    // Test struct declaration and new, mod.
    let source = r#"
        module Main;
        type I64Bool = struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool { x: 18, y: false };
            let obj = I64Bool::mod_x(|x| x + 42, obj);
            let u = assert_eq("", I64Bool::@x(obj), 60);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test34_5() {
    // Test unboxed struct declaration and new, mod.
    let source = r#"
        module Main;
        type I64Bool = unbox struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool { x: 18, y : false};
            let obj = I64Bool::mod_x(|x| x + 42, obj);
            let u = assert_eq("", I64Bool::@x(obj), 60);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test34() {
    // Test namespace inference.
    let source = r#"
        module Main;        
        
        type OtherStruct = struct {y: I64, x: Bool};
        type I64Bool = struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool {x: 18, y: false};
            let obj = obj.mod_x(|x| x + 42);
            let u = assert_eq("", obj.@x, 60);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test35() {
    // Test overloading resolution.
    let source = r#"
        module Main;

        type A = struct {x: I64, y: Bool};
        type B = struct {x: Bool, y: I64};
            
        main : IO ();
        main = (
            let a = A {x: 3, y: true};
            let b = B {x: true, y: 5};
            let ans = add(if a.@y { a.@x } else { 0 }, if b.@x { b.@y } else { 0 });
            let u = assert_eq("", ans, 8);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test36() {
    // Test modifier composition.
    let source = r#"
        module Main;

        type A = struct {x: B};
        type B = struct {x: I64};
            
        main : IO ();
        main = (
            let a = A{x: B{x: 16}};
            let a = a.(mod_x $ mod_x $ |x| x + 15);
            let ans = a . @x . @x;
            let u = assert_eq("", ans, 31);
            pure ()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test37() {
    // Test unique modField.
    let source = r#"
        module Main;

        type A = struct {x: B};
        type B = struct {x: I64};

        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let b = a . (mod_x! $ mod_x! $ |x| x + 15);
            let ans = b . @x . @x;
            let u = assert_eq("", ans, 31);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test37_5() {
    // Test shared modField.
    let source = r#"
        module Main;

        type A = struct {x: B};
        type B = struct {x: I64};

        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let b = a.(mod_x $ mod_x $ |x| x + 15);
            let ans = a.@x.@x + b.@x.@x;
            let u = assert_eq("", ans, (16 + 15) + 16);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test38() {
    // Test type annotation.
    let source = r#"
        module Main;

        type A = struct {x: B};
        type B = struct {x: I64};

        main : IO ();
        main = (    
            let a = A {x: B {x: 16}};
            let f = |a| (a : A) . (mod_x! $ mod_x! $ |x| x + 15);
            let a = a.f;
            let ans = a.@x.@x;
            let u = assert_eq("", ans, 31);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test39() {
    // Test type annotation.
    let source = r#"
        module Main;

        type A = struct {x: B};
        type B = struct {x: I64};
        
        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let f = |a| a . ((mod_x! : (B -> B) -> A -> A) $ mod_x! $ |x| x + 15);
            let a = a.f;
            let ans = a.@x.@x;
            let u = assert_eq("", ans, 31);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test40() {
    // Test type annotation at let-binding.
    let source = r#"
        module Main;

        type A = struct {x: B};
        type B = struct {x: I64};
        
        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let f: A -> A = |a| a.(mod_x! $ mod_x! $ |x| x + 15);
            let a = a .f;
            let ans = a .@x .@x;
            let u = assert_eq("", ans, 31);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test41() {
    // Test type annotation at let-binding.
    let source = r#"
        module Main;
        
        main : IO ();
        main = (
            let x: I64 -> I64 = |x| x;
            let ans = x(42);
            let u = assert_eq("", ans, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test41_5() {
    // Test type annotation at lambda
    let source = r#"
        module Main;
        
        main : IO ();
        main = (
            let x = |x: I64| x;
            let ans = x(42);
            let u = assert_eq("", ans, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
                let u = assert_eq("", ans, {});
                pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), Configuration::develop_compiler());
}

#[test]
#[serial]
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
                let u = assert_eq("", ans, {});
                pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), Configuration::release());
}

#[test]
#[serial]
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
            let head = arr.get(0).toI64;
            let next = arr.get(1).toI64;
            add(head, next)
        );

        main : IO ();
        main = (
            let arr0 = Array::fill(2, false);
            let arr0 = arr0.set!(0, true);
            let x = add_head_and_next(arr0);

            let arr1 = Array::fill(2, 3);
            let arr1 = arr1.set!(1, 5);
            let z = add_head_and_next(arr1);

            let y = toI64(5) + toI64(false);
            let ans = x + y + z;
            let u = assert_eq("", ans, 11);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test44_5() {
    // Test Array::from_map.
    let source = r#"
        module Main;

        sum : Array I64 -> I64;
        sum = |arr| (
            let loop = fix $ |loop, idx, sum| (
                if idx == arr.get_size { sum };
                loop(idx + 1, sum + arr.get(idx))
            );
            loop(0, 0)
        );

        main : IO ();
        main = (
            let arr = Array::from_map(10, |x| x * x);
            let ans = sum(arr);
            let u = assert_eq("", ans, 285);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test45() {
    // Test HKT.
    let source = r#"
        module Main;

        trait [f:*->*] f : MyFunctor {
            map : (a -> b) -> f a -> f b;
        }

        impl Array : MyFunctor {
            map = |f, arr| (
                Array::from_map(arr.get_size, |idx| f $ arr.get(idx))
            );
        }

        sum : Array I64 -> I64;
        sum = |arr| (
            let loop = fix $ |loop, idx, sum| (
                if idx == arr.get_size { sum };
                loop(idx + 1, sum + arr.get(idx))
            );
            loop(0, 0)
        );

        main : IO ();
        main = (
            let arr = Array::from_map(10, |x| x);
            let arr = arr.map(|x| x * x);
            let ans = arr.sum;
            let u = assert_eq("", ans, 285);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
            let u = assert_eq("", ans, 15);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test47() {
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
            let u = assert_eq("", ans, 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test47_2() {
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
            let u = assert_eq("", ans, 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test47_5() {
    // Test union of closure object
    let source = r#"
        module Main;

        type Union = union {val: I64, func: I64 -> I64};

        main : IO ();
        main = (
            let five = 5;
            let val = Union::val(3);
            let func = Union::func(|x| x + five).mod_func(|f||x|f(x)+2); // x -> x + 5 + 2
            let ans = func.as_func $ val.as_val;
            let u = assert_eq("", ans, 7 + 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test47_6() {
    // Test union of boxed object
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let uni = Option::some([1,2,3]).mod_some(
                |lhs| lhs.force_unique!.append([4,5,6])
            );
            let arr = uni.as_some;
            let _ = assert_eq("", arr.get(0), 1);
            let _ = assert_eq("", arr.get(1), 2);
            let _ = assert_eq("", arr.get(2), 3);
            let _ = assert_eq("", arr.get(3), 4);
            let _ = assert_eq("", arr.get(4), 5);
            let _ = assert_eq("", arr.get(5), 6);
            let _ = assert_eq("", arr.get_size, 6);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test48() {
    // Parametrised struct.
    let source = r#"
        module Main;

        type Vec a = struct {data: Array a};

        main : IO ();
        main = (
            let int_vec = Vec {data: Array::fill(2, 5)};
            let int_vec = int_vec.mod_data!(|arr| arr.set(0, 3));
            let head = int_vec.@data.get(0);
            let next = int_vec.@data.get(1);
            let ans = add(head, next);
            let u = assert_eq("", ans, 8);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
            let u = assert_eq("", ans, 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
                let u = assert_eq("", ans, {});
                pure()
            );
        "#,
        n,
        (n * (n - 1)) / 2
    );
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test51() {
    // test trait bounds.
    let source = r#"
    module Main;
    
    search : [a: Eq] a -> Array a -> I64;
    search = |elem, arr| loop(0) $ |idx| (
        if idx == arr.get_size {
            break $ -1
        } else if arr.get(idx) == elem { 
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
        let u = assert_eq("", ans, 2);
        pure()
    );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test52() {
    // Test loop with boxed state / break.
    let source = r#"
    module Main;

    type SieveState = struct {i: I64, arr: Array Bool};
    
    // Calculate a Bool array whose element is true iff idx is prime.
    is_prime : I64 -> Array Bool;
    is_prime = |n| (
        let arr = Array::fill(n, true);
        let arr = arr.set!(0, false);
        let arr = arr.set!(1, false);
        loop(SieveState {i: 2, arr: arr}, |state|
            let i = state.@i;
            let arr = state.@arr;
            if i*i > n { break $ arr };
            let next_arr = if arr.get(i) {
                loop(SieveState {i: i+i, arr: arr}, |state|
                    let q = state.@i;
                    let arr = state.@arr;
                    if n-1 < q { 
                        break $ arr
                    } else {
                        continue $ SieveState{ i: q + i, arr: arr.set!(q, false) }
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
            let sum = sum + (if arr.get(i) == elem {1} else {0});
            continue $ (i+1, sum)
        )
    );
    
    main : IO ();
    main = (
        let ans = (is_prime $ 100).count(true);
        let u = assert_eq("", ans, 25);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test53() {
    // Test mutation of unique unboxed struct (e.g., tuple).
    let source = r#"
    module Main;
    
    main : IO ();
    main = (
        let pair = (13, Array::fill(1, 0));
        let pair = pair.mod_0!(|x| x + 3);
        let pair = pair.mod_1!(|arr| arr.set!(0, 5));
        let x = pair.@0;
        let y = pair.@1.get(0);
        let ans = x + y;
        let u = assert_eq("", ans, 13 + 3 + 5);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test54() {
    // Test mutation of shared unboxed struct (e.g., tuple).
    let source = r#"
    module Main;
    
    main : IO ();
    main = (
        let pair0 = (13, Array::fill(1, 0));
        let pair1 = pair0.mod_1(|arr| arr.set(0, 5));
        let pair2 = pair0.mod_0!(|x| x + 3);
        let x = pair1.@1.get(0);
        let y = pair2.@0;
        let ans = x + y;
        let u = assert_eq("", ans, 13 + 3 + 5);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", ans, 1);
        pure ()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", ans, 1);
        pure ()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", ans, 1);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", ans, 1);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", ans, 9);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test60() {
    // Test unit.
    let source = r"
    module Main;
    
    unit : ();
    unit = ();

    main : IO ();
    main = let u = unit; pure ();
    ";
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test61() {
    // Test Hello world.
    let source = r#"
    module Main;

    main_loop : I64 -> IO ();
    main_loop = |counter| (
        if counter == 0 {
            pure()
        } else {
            let _ = *println("Hello World! (" + counter.to_string + ")");
            main_loop(counter - 1)
        }
    );

    main : IO ();
    main = main_loop(3);
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test61_5() {
    // Test Hello world.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        loop_m(0, |i| (
            if i == 3 { break_m $ () };
            let _ = *println("Hello World! (" + i.to_string + ")");
            continue_m $ i + 1
        ))
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test62() {
    // Test String length.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let len = "Hello World!".get_size;
        let u = assert_eq("", len, 12);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test63() {
    // Test I64 ToString.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let min = -9223372036854775808;
        println $ min.to_string
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test64() {
    // Test escape sequence.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        println $ "\u2764"
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", sum, 45);
        pure ()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", sum, 45);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", sum, 45);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test68() {
    // Test unboxed union pattern matching.
    let source = r#"
    module Main;

    type I64OrBool = unbox union {int: I64, bool: Bool};

    main : IO ();
    main = (
        let u = I64OrBool::int(42);
        let I64OrBool::int(x) = u;
        let u = assert_eq("", x, 42);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test69() {
    // Test boxed union pattern matching.
    let source = r#"
    module Main;

    type I64OrBool = box union {int: I64, bool: Bool};

    main : IO ();
    main = (
        let u = I64OrBool::bool(true);
        let I64OrBool::bool(x) = u;
        let u = assert_eq("", x, true);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test70() {
    // Test tuple in union pattern.
    let source = r#"
    module Main;

    type Union = union {left: (I64, String), right: Bool};

    main : IO ();
    main = (
        let u = Union::left((42, "truth"));
        let Union::left((x, y)) = u;
        let u = assert_eq("", x, 42);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test71() {
    // Test union in struct pattern.
    let source = r#"
    module Main;

    type Struct = struct {uni: Union, value: I64};
    type Union = union {left: (I64, String), right: Bool};

    main : IO ();
    main = (
        let u = Struct {uni: Union::left((42, "truth")), value: 13};
        let Struct { uni: Union::left((truth, string)), value: val } = u;
        let u = assert_eq("", truth, 42);
        let u = assert_eq("", val, 13);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let u = assert_eq("", sum, 45);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test73() {
    // Test pattern matching on argment.
    let source = r#"
    module Main;

    type I64Bool = struct {x: I64, y: Bool};

    main : IO ();
    main = (
        let int_bool = I64Bool { y: true, x: 42 };
        let u = assert_eq("", int_bool.@x, 42);
        let u = assert_eq("", int_bool.@y, true);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test74() {
    // Test setter function of struct / tuple.
    let source = r#"
    module Main;

    type I64Bool = struct {x: I64, y: Bool};

    main : IO ();
    main = (
        let int_bool = I64Bool { y: false, x: 0 };
        let int_bool = int_bool.=x(3);
        let u = assert_eq("", int_bool.@x, 3);
        let int_bool = int_bool.=x!(5);
        let u = assert_eq("", int_bool.@x, 5);
        let pair = (false, 0);
        let pair = pair.=0(true);
        let u = assert_eq("", pair.@0, true);
        let pair = pair.=0!(false);
        let u = assert_eq("", pair.@0, false);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test75() {
    // Test iterator.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let iter = Iterator::from_map(|i| i*i );
        let Option::some((n, iter)) = iter.advance;
        let _ = assert_eq("", n, 0*0);
        let Option::some((n, iter)) = iter.advance;
        let _ = assert_eq("", n, 1*1);
        let Option::some((n, iter)) = iter.advance;
        let _ = assert_eq("", n, 2*2);
        let Option::some((n, iter)) = iter.advance;
        let _ = assert_eq("", n, 3*3);
        let Option::some((n, iter)) = iter.advance;
        let _ = assert_eq("", n, 4*4);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test76() {
    // Test array modifier.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let array = Array::from_map(3, |_i| Array::from_map(3, |_j| 0));
        let array = array.mod!(1, Array::set!(1, 9));
        let _ = assert_eq("", array.get(1).get(1), 9);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test77() {
    // Test Iterator::zip / map / take / fold.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let iter0 = Iterator::count_up(5);
        let iter1 = Iterator::from_map(|i| 2*i);
        let iter = iter0.zip(iter1);
        let iter = iter.map(|(a,b)| a+b).take(3);
        let res = iter.fold(0, add);
        let _ = assert_eq("", res, (5+2*0) + (6+2*1) + (7+2*2));
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test78() {
    // Test Iterator::filter
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let iter = Iterator::count_up(1).take(100);
        let iter = iter.filter(|n| n%3 == 0 || n%5 == 0);
        let count = iter.map(|_|1).fold(0, add);
        let _ = assert_eq("", count, 100/3 + 100/5 - 100/15);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test79() {
    // Test Iterator::push_front
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let ls = Iterator::empty;
        let ls = ls.push_front(1).push_front(2);
        let (e, ls) = ls.advance.as_some;
        let _ = assert_eq("", 2, e);
        let (e, ls) = ls.advance.as_some;
        let _ = assert_eq("", 1, e);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test80() {
    // Test Iterator::last
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let iter = Iterator::empty.push_front(4).push_front(3).push_front(2).push_front(1);
        let last = iter.find_last.as_some;
        let _ = assert_eq("", last, 4);
        let last: Option Bool = Iterator::empty.find_last;
        let _ = assert("", last.is_none);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test81() {
    // Test array literal.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let arr = [1,2,3,4];
        let _ = assert_eq("", arr.get_size, 4);
        let arr: Array Bool = [];
        let _ = assert_eq("", arr.get_size, 0);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let _ = assert_eq("wrong reserved length (0+2)", v.get_capacity, 2);
        let _ = assert_eq("wrong length (0+2)", v.get_size, 2);
        let _ = assert_eq("wrong element (0+2)", v.get(0), 3);
        let _ = assert_eq("wrong element (0+2)", v.get(1), 4);

        // Test 2+0
        let v1 = [1,2];
        let v2 = [];
        let v = v1.append(v2);
        let _ = assert_eq("wrong reserved length (2+0)", v.get_capacity, 2);
        let _ = assert_eq("wrong length (2+0)", v.get_size, 2);
        let _ = assert_eq("wrong element (2+0)", v.get(0), 1);
        let _ = assert_eq("wrong element (2+0)", v.get(1), 2);

        // Test 0+0
        let v1: Array (I64 -> Bool) = [];
        let v2 = [];
        let v = v1.append(v2);
        let _ = assert_eq("wrong capacity (0+0)", v.get_capacity, 0);
        let _ = assert_eq("wrong length (0+0)", v.get_size, 0);

        // Test boxed elements.
        let v1 = [add(1), add(2)];
        let v2 = [add(3), add(4)];
        let v = v1.append(v2);
        let x = 0;
        let x = v.get(0) $ x;
        let _ = assert_eq("wrong value (boxed) 0+1", x, 0+1);
        let x = v.get(1) $ x;
        let _ = assert_eq("wrong value (boxed) 0+1+2", x, 0+1+2);
        let x = v.get(2) $ x;
        let _ = assert_eq("wrong value (boxed) 0+1+2+3", x, 0+1+2+3);
        let x = v.get(3) $ x;
        let _ = assert_eq("wrong value (boxed) 0+1+2+3+4", x, 0+1+2+3+4);

        // Test appending shared array.
        let v1 = [add(1), add(2)].reserve(4);
        let v2 = [add(3), add(4)];
        let v = v1.append(v2);
        let w = v2.append(v1);
        let x = 0;
        let x = v.get(0) $ x; // += 1
        let x = w.get(3) $ x; // += 2
        let _ = assert_eq("", x, 3);

        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
        let _ = loop(0, |idx|(
            if idx == 100 { break $ () };
            let _ = assert_eq("wrong element", idx, v.get(idx));
            continue $ idx + 1
        ));
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.pop_back;
            continue $ (idx+1, v)
        ));
        let _ = assert_eq("wrong length after pop", 0, v.get_size);
        let _ = assert("wrong reserved length after pop", v.get_capacity >= 100);
    
        // Boxed element
        let v = [];
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.push_back(add(idx));
            continue $ (idx+1, v)
        ));
        let x = loop((0, 0), |(idx, x)|(
            if idx == 100 { break $ x };
            let x = v.get(idx) $ x;
            continue $ (idx + 1, x)
        ));
        let _ = assert_eq("wrong value (boxed)", x, 99 * 100 / 2);
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.pop_back;
            continue $ (idx+1, v)
        ));
        let _ = assert_eq("wrong length after pop (boxed)", 0, v.get_size);
        let _ = assert("wrong reserved length after pop (boxed)", v.get_capacity >= 100);
    
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test84() {
    // Test Eq for Array
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let v1 = [1,2,3];
        let v2 = [1,2,3];
        let _ = assert("", v1 == v2);
    
        let v1 = [1,2,3];
        let v2 = [0,2,3];
        let _ = assert("", v1 != v2);
    
        let v1 = [];
        let v2 = [0];
        let _ = assert("", v1 != v2);
    
        let v1: Array I64 = [];
        let v2 = [];
        let _ = assert("", v1 == v2);
    
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test85() {
    // Test concat string, compare string.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let s1 = "Hello";
        let s2 = " ";
        let s3 = "World!";
        let _ = assert_eq("", s1.concat(s2).concat(s3) == "Hello World!");
    
        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test86() {
    // Test concat_iter
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let iter = Iterator::from_array(["Hello", " ", "World", "!"]);
        let _ = assert_eq("", iter.concat_iter, "Hello World!");
        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test87() {
    // Test iterator comparison
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([1,2,3]);
        let _ = assert_eq("", lhs, rhs);

        let lhs: Iterator Bool = Iterator::from_array([]);
        let rhs = Iterator::from_array([]);
        let _ = assert_eq("", lhs, rhs);

        let lhs = Iterator::from_array([]);
        let rhs = Iterator::from_array([1,2]);
        let _ = assert("", lhs != rhs);

        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test88() {
    // Test iterator comparison
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let iter = Iterator::from_array([1,2,3]);
        let iter = iter.intersperse(0);
        let _ = assert_eq("", iter, Iterator::from_array([1,0,2,0,3]));
    
        let iter = Iterator::from_array([1]);
        let iter = iter.intersperse(0);
        let _ = assert_eq("", iter, Iterator::from_array([1]));
    
        let iter = Iterator::from_array([]);
        let iter = iter.intersperse(0);
        let _ = assert_eq("", iter, Iterator::from_array([]));
    
        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test89() {
    // Test Iterator::append
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([4,5,6]);
        let _ = assert_eq("", lhs + rhs, Iterator::from_array([1,2,3,4,5,6]));
    
        let lhs = Iterator::from_array([]);
        let rhs = Iterator::from_array([4,5,6]);
        let _ = assert_eq("", lhs + rhs, Iterator::from_array([4,5,6]));

        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([]);
        let _ = assert_eq("", lhs + rhs, Iterator::from_array([1,2,3]));

        let lhs: Iterator I64 = Iterator::from_array([]);
        let rhs = Iterator::from_array([]);
        let _ = assert_eq("", lhs + rhs, Iterator::from_array([]));
    
        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test90() {
    // Test Array::sort_by.
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let vec = [5,3,1,7,4,6,9,8,2];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        let _ = assert_eq("wrong result 9", vec, [1,2,3,4,5,6,7,8,9]);

        let vec = [1];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        let _ = assert_eq("wrong result 1", vec, [1]);

        let vec: Array I64 = [];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        let _ = assert_eq("wrong result 0", vec, []);

        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test93() {
    // Test try to make circular reference (and fail).
    let source = r#"
    module Main;

    type SelfRef = box struct { data : Option SelfRef };

    main : IO ();
    main = (
        let ref = SelfRef { data : Option::none() };
        // let ref = ref.=data!(Option::some(ref)); // fails
        let ref = ref.=data(Option::some(ref)); // fails
        pure()
    );

    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test94() {
    // Test FFI
    let source = r#"
            module Main;
    
            main : IO ();
            main = (
                let _ = "Hello C function!\n".borrow_c_str(|ptr|
                    CALL_C[I32 printf(Ptr, ...), ptr]
                );
                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test95() {
    // Test Std::is_unique
    let source = r#"
            module Main;
    
            main : IO ();
            main = (
                // For unboxed value, it returns true even if the value is used later.
                let int_val = 42;
                let (unique, _) = int_val.is_unique;
                let use = int_val + 1;
                let _ = assert_eq("fail: int_val is shared", unique, true);

                // For boxed value, it returns true if the value isn't used later.
                let arr = Array::fill(10, 10);
                let (unique, arr) = arr.is_unique;
                let use = arr.get(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
                let _ = assert_eq("fail: arr is shared", unique, true);

                // Fox boxed value, it returns false if the value will be used later.
                let arr = Array::fill(10, 10);
                let (unique, _) = arr.is_unique;
                let use = arr.get(0);
                let _ = assert_eq("fail: arr is unique", unique, false);

                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test96() {
    // Test U8 literal
    let source = r#"
            module Main;
    
            main : IO ();
            main = (
                let _ = assert_eq("", -1_U8, 255_U8);
                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test97() {
    // Test arithmetic operation of U8, I32
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = assert_eq("", -(1_U8), 255_U8);
            let _ = assert_eq("", 255_U8 + 3_U8, 2_U8);
            let _ = assert_eq("", 1_U8 - 3_U8, 254_U8);
            let _ = assert_eq("", 20_U8 * 30_U8, 88_U8);
            let _ = assert_eq("", 10_U8 / 3_U8, 3_U8);
            let _ = assert_eq("", 10_U8 % 3_U8, 1_U8);
            let _ = assert_eq("", -1_U8 > 0_U8, true);
            let _ = assert_eq("", -1_U8 >= 0_U8, true);

            let _ = assert_eq("", 2147483647_I32 + 2_I32, -2147483647_I32);
            let _ = assert_eq("", -2147483647_I32 - 2_I32, 2147483647_I32);
            let _ = assert_eq("", 2147483647_I32 * 2_I32, -2_I32);
            let _ = assert_eq("", 10_I32 / -3_I32, -3_I32);
            let _ = assert_eq("", 10_I32 % -3_I32, 1_I32);
            let _ = assert_eq("", -1_I32 < 0_I32, true);
            
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test98() {
    // Test to_string for integrals
    let source = r#"
        module Main;

        main : IO ();
        main = (
            // U8
            let _ = assert_eq("", 0_U8.to_string, "0");
            let _ = assert_eq("", 255_U8.to_string, "255");
            
            // I32
            let _ = assert_eq("", -2147483648_I32.to_string, "-2147483648");
            let _ = assert_eq("", 2147483647_I32.to_string, "2147483647");

            // U32
            let _ = assert_eq("", 0_U32.to_string, "0");
            let _ = assert_eq("", 4294967295_U32.to_string, "4294967295");

            // I64
            let _ = assert_eq("", -9223372036854775808_I64.to_string, "-9223372036854775808");
            let _ = assert_eq("", 9223372036854775807_I64.to_string, "9223372036854775807");

            // U64
            let _ = assert_eq("", 0_U64.to_string, "0");
            let _ = assert_eq("", 18446744073709551615_U64.to_string, "18446744073709551615");
            
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test99() {
    // Test cast between integral types.
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = assert_eq("case 1", -2147483648_I32.to_I64, -2147483648_I64);
            let _ = assert_eq("case 2", 2147483647_I32.to_I64, 2147483647_I64);
            let _ = assert_eq("case 3", -10000000000_I64.to_I32, -10000000000_I32);
            let _ = assert_eq("case 4", 10000000000_I64.to_I32, 10000000000_I32);
            let _ = assert_eq("case 5", -10000000000_I32.to_U8, -10000000000_U8);
            let _ = assert_eq("case 6", 255_U8.to_I32, 255_I32);
            let _ = assert_eq("case 7", -1_I32.to_U8, -1_U8);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test100() {
    // Test u8 literal
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = assert_eq("case 1", 'A', 65_U8);
            let _ = assert_eq("case 2", '0', 48_U8);
            let _ = assert_eq("case 3", '\n', 10_U8);
            let _ = assert_eq("case 3", '\x7f', 127_U8);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test101() {
    // Test Array::is_empty, get_first, get_last.
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let cap = 42;
            let arr: Array (() -> I64) = [];
            let _ = assert_eq("case 1", arr.is_empty, true);
            let _ = assert_eq("case 2", arr.get_first.is_none, true);
            let _ = assert_eq("case 3", arr.get_last.is_none, true);

            let cap = 42;
            let arr: Array (() -> I64) = [|_|cap];
            let _ = assert_eq("case 4", arr.is_empty, false);
            let _ = assert_eq("case 5", arr.get_first.as_some $ (), 42);
            let _ = assert_eq("case 6", arr.get_last.as_some $ (), 42);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test102() {
    // Test I64 : Eq, LessThan, LessThanEq
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = assert_eq("case 1", 0 == 0, true);
            let _ = assert_eq("case 2", 0 == 1, false);
            let _ = assert_eq("case 3", 0 != 0, false);
            let _ = assert_eq("case 4", 0 != 1, true);

            let _ = assert_eq("case 5", 0 < 0, false);
            let _ = assert_eq("case 6", 0 > 0, false);
            let _ = assert_eq("case 7", 0 < 1, true);
            let _ = assert_eq("case 8", 0 > 1, false);

            let _ = assert_eq("case 9", 0 <= 0, true);
            let _ = assert_eq("case 10", 0 >= 0, true);
            let _ = assert_eq("case 11", 0 <= 1, true);
            let _ = assert_eq("case 12", 0 >= 1, false);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test103() {
    // Test Bool : Eq
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = assert_eq("case 1", false == false, true);
            let _ = assert_eq("case 2", false == true, false);
            let _ = assert_eq("case 3", true == false, false);
            let _ = assert_eq("case 4", true == true, true);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test104() {
    // Test Bool : ToString
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = assert_eq("case 1", true.to_string, "true");
            let _ = assert_eq("case 2", false.to_string, "false");

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test105() {
    // Test String::get_first_byte, get_last_byte, is_empty
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = assert_eq("case 1", "".is_empty, true);
            let _ = assert_eq("case 2", "".get_first_byte.is_none, true);
            let _ = assert_eq("case 3", "".get_last_byte.is_none, true);
            let _ = assert_eq("case 4", "abc".is_empty, false);
            let _ = assert_eq("case 5", "abc".get_first_byte.as_some, 'a');
            let _ = assert_eq("case 6", "abc".get_last_byte.as_some, 'c');

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test106() {
    // Test [a : Eq] Option a : Eq
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let lhs: Option I64 = Option::none();
            let rhs: Option I64 = Option::none();
            let _ = assert("case 1", lhs == rhs);

            let lhs: Option I64 = Option::none();
            let rhs: Option I64 = Option::some(42);
            let _ = assert("case 2", lhs != rhs);

            let lhs: Option I64 = Option::some(84);
            let rhs: Option I64 = Option::some(42);
            let _ = assert("case 3", lhs != rhs);

            let lhs: Option I64 = Option::some(42);
            let rhs: Option I64 = Option::some(42);
            let _ = assert("case 4", lhs == rhs);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test107() {
    // Test String::pop_back_byte, strip_last_bytes, strip_last_newlines.
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = assert_eq("case 1", "".pop_back_byte, "");
            let _ = assert_eq("case 2", "a".pop_back_byte, "");

            let _ = assert_eq("case 3", "".strip_last_bytes(|c|c == 'x'), "");
            let _ = assert_eq("case 4", "abc".strip_last_bytes(|_|true), "");
            let _ = assert_eq("case 5", "".strip_last_bytes(|_|true), "");
            let _ = assert_eq("case 6", "x".strip_last_bytes(|c|c == 'x'), "");
            let _ = assert_eq("case 7", "y".strip_last_bytes(|c|c == 'x'), "y");
            let _ = assert_eq("case 8", "yx".strip_last_bytes(|c|c == 'x'), "y");
            let _ = assert_eq("case 9", "yxz".strip_last_bytes(|c|c == 'x'), "yxz");

            let _ = assert_eq("case 10", "abc\n\r".strip_last_newlines, "abc");

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test108() {
    // Test write_file!, read_file!, read_line.
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let file_path = Path::parse("test.txt").as_some;
            let lines = ["Hello", "World!"];
            let content = Iterator::from_array(lines).intersperse("\n").concat_iter;
            do {
                let _ = *write_file(file_path, content);

                let read_content = *read_file(file_path);
                let _ = assert_eq("case 1", content, read_content);

                let read_lines = *with_file(file_path, "r", |file| (
                    pure $ [*read_line(file), *read_line(file)]
                ));
                let _ = assert_eq("case 2", read_lines.get(0), lines.get(0) + "\n");
                let _ = assert_eq("case 3", read_lines.get(1), lines.get(1));

                pure()
            }.to_io.map(as_ok)
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
    remove_file("test.txt").unwrap();
}

#[test]
#[serial]
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

            let _ = assert_eq("case 1", add_opt_int(one, two), three);
            let _ = assert_eq("case 2", add_opt_int(none, two), none);
            let _ = assert_eq("case 3", add_opt_int(one, none), none);
            let _ = assert_eq("case 4", add_opt_int(none, none), none);

            let res0 = Result::ok(0) : Result String I64;
            let res1 = Result::ok(1);
            let res2 = Result::ok(2);
            let res3 = Result::ok(3);
            let res_iter = Iterator::from_array([res0, res1, res2, res3]).sequence;
            let _ = assert_eq("case 5", res_iter.is_ok, true);
            let _ = assert_eq("case 6", res_iter.as_ok, Iterator::from_array([0, 1, 2, 3]));

            let res0 = Result::ok(0) : Result String I64;
            let res1 = Result::ok(1);
            let res2 = Result::err("Error 2");
            let res3 = Result::err("Error 3");
            let res_iter = Iterator::from_array([res0, res1, res2, res3]).sequence;
            let _ = assert_eq("case 5", res_iter.is_err, true);
            let _ = assert_eq("case 6", res_iter.as_err, "Error 2");

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test110() {
    // Test basic float operations
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let x = -3.1415_F32;
            let y = 3.1415_F32;
            let _ = assert("case 1", x.abs == y);
            let _ = assert("case 2", y.abs == y);

            let x = -3.1415;
            let y = 3.1415;
            let _ = assert("case 3", x.abs == y);
            let _ = assert("case 4", y.abs == y);

            let x = 3.1415_F32;
            let y = 3.1415_F32;
            let _ = assert("case 5", x == y);

            let x = 3.1415;
            let y = 3.1415;
            let _ = assert("case 6", x == y);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let _ = assert("case 7", x != y);

            let x = 3.1415;
            let y = 2.7183;
            let _ = assert("case 8", x != y);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 5.8598_F32;
            let _ = assert("case 9", (x + y - z).abs < 1.0e-4_F32);

            let x = 3.1415;
            let y = 2.7183;
            let z = 5.8598;
            let _ = assert("case 10", (x + y - z).abs < 1.0e-4);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 8.5395_F32;
            let _ = assert("case 11", (x * y - z).abs < 1.0e-4_F32);

            let x = 3.1415;
            let y = 2.7183;
            let z = 8.5395;
            let _ = assert("case 12", (x * y - z).abs < 1.0e-4);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 1.1557_F32;
            let _ = assert("case 13", (x / y - z).abs < 1.0e-4_F32);

            let x = 3.1415;
            let y = 2.7183;
            let z = 1.1557;
            let _ = assert("case 14", (x / y - z).abs < 1.0e-4);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let _ = assert("case 15", x > y);

            let x = 3.1415;
            let y = 2.7183;
            let _ = assert("case 16", x > y);

            let x = 3.1415_F32;
            let y = 3.1415_F32;
            let _ = assert("case 17", x >= y);

            let x = 3.1415;
            let y = 3.1415;
            let _ = assert("case 18", x >= y);

            let x = 3.1415_F32;
            let y = 3.1415;
            let _ = assert("case 19", (x.to_F64 - y) < 1.0e-4);

            let x = 3.1415;
            let y = 3.1415_F32;
            let _ = assert("case 19", (x.to_F32 - y) < 1.0e-4_F32);

            let x = 3141;
            let y = 3141.0;
            let _ = assert("case 20", x.to_F64 == y);

            let x = 3141.0;
            let y = 3141;            
            let _ = assert("case 21", x.to_I64 == y);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
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

            let _ = assert_eq("case 1", f_g(0), 6);
            let _ = assert_eq("case 2", g_f(0), 10);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test112() {
    // Test Iterator::generate
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let iter = Iterator::generate(0, |_| Option::none());
            let ans = [] : Array I64;
            let _ = assert_eq("case 1", iter.to_array, ans);

            let iter = Iterator::generate(0, |i| if i == 3 { Option::none() } else { Option::some $ (i, i+1) });
            let ans = [0, 1, 2];
            let _ = assert_eq("case 1", iter.to_array, ans);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test113() {
    // Test bit operations.
    let source = r#"
        module Main;

        main : IO ();
        main = (
            // Shift
            let x = 10_U8.shift_right(2_U8);
            let _ = assert_eq("case 1", x, 2_U8);

            let x = -10_I32.shift_right(2_I32);
            let _ = assert_eq("case 1", x, -3_I32);

            let x = 10_U8.shift_left(2_U8);
            let _ = assert_eq("case 1", x, 40_U8);

            // Xor, Or, And
            let x = 10.bit_xor(12);
            let _ = assert_eq("case 1", 6);

            let x = 10.bit_or(12);
            let _ = assert_eq("case 1", 14);

            let x = 10.bit_and(12);
            let _ = assert_eq("case 1", 8);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test114() {
    // Test Array::find_by
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let arr = [0,1,2,3];

            let res = arr.find_by(|x| x % 5 == 2);
            let _ = assert_eq("case 1", res, Option::some(2));

            let res = arr.find_by(|x| x % 5 == 4);
            let _ = assert_eq("case 1", res, Option::none());

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test115() {
    // Test HashMap
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let mp = HashMap::empty(0)
                            .insert(0, 0)
                            .insert(1, 1)
                            .insert(2, 2)
                            .erase(2).insert(2, 2)
                            .insert(3, 3)
                            .insert(4, 4).erase(4)
                            .erase(5)
                            // Do nothing for 6
                            .insert(7, -1).insert(7, 7);
        
            let _ = assert_eq("case 0", mp.find(0), Option::some(0));
            let _ = assert_eq("case 1", mp.find(1), Option::some(1));
            let _ = assert_eq("case 2", mp.find(2), Option::some(2));
            let _ = assert_eq("case 3", mp.find(3), Option::some(3));
            let _ = assert_eq("case 4", mp.find(4), Option::none());
            let _ = assert_eq("case 5", mp.find(5), Option::none());
            let _ = assert_eq("case 6", mp.find(6), Option::none());
            let _ = assert_eq("case 7", mp.find(7), Option::some(7));
            let _ = assert_eq("case size", mp.get_size, 5);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test116() {
    // Test Std::Destructor
    let source = r#"
        module Main;

        main : IO ();
        main = (

            // Boxed case
            let dtor0 = Destructor { 
                value : [1,2,3], 
                dtor : |arr| (
                    let arr_str = arr.to_iter.map(to_string).join(", ");
                    debug_println("dtor0 destructed. val: " + arr_str)
                )
            };

            // Unboxed case
            let dtor1 = Destructor { 
                value : 42, 
                dtor : |val| (
                    debug_println("dtor1 destructed. val: " + val.to_string)
                )
            };

            // Dtor in dtor
            let dtor2 = Destructor { 
                value : 2, 
                dtor : |val| (
                    debug_println("dtor2 destructed. val: " + val.to_string)
                )
            };
            let dtor3 = Destructor { 
                value : dtor2, 
                dtor : |val| (
                    debug_println("dtor3 destructed. val.@value: " + val.@value.to_string)
                )
            };

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test117() {
    // Test String::from_c_str
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let str = String::from_c_str([65_U8, 66_U8, 67_U8, 0_U8, 0_U8]);
            let _ = assert_eq("case 1", str, "ABC");
            let _ = assert_eq("case 2", str.get_size, 3);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test118() {
    // Test fold_m
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let _ = *count_up(0).take(10).fold_m(0, |s, i| (
                let s = s + i;
                let _ = *print("Sum upto " + i.to_string + " is " + s.to_string + ". ");
                pure $ s
            ));
            let _ = *println("");
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test119() {
    // Test namespace and MakeStruct, Pattern.
    let source = r#"
        module Main;

        namespace A {
            type S = struct { data : () };
            type U = union { data : () };
        }

        namespace B {
            type S = struct { data : () };
            type U = union { data : () };
        }

        main : IO ();
        main = (
            let s = A::S { data : () };
            let A::S { data : _ } = s;
            let s = B::S { data : () };
            let B::S { data : _ } = s;
            let u = A::U::data();
            let A::U::data(_) = u;
            let u = B::U::data();
            let B::U::data(_) = u;
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test_run_examples() {
    let paths = fs::read_dir("./examples").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let display = path.display();
        if path.extension().is_none() || path.extension().unwrap() != "fix" {
            continue;
        }
        println!("[run_examples] {}:", display);

        run_file(&path, Configuration::develop_compiler());
    }
}

#[test]
#[serial]
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
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
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
    run_source(source, Configuration::develop_compiler());
}
