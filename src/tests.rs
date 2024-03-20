use std::fs::{self, remove_file};

use rand::Rng;

use super::*;

#[test]
pub fn test0() {
    let source = r#"    
            module Main; 
            import Debug;

            main : IO ();
            main = (
                eval assert_eq(|_|"case 1", 5 + 3 * 8 / 5 + 7 % 3, 1e1_I64);
                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test1() {
    let source = r#"
            module Main; 
            import Debug;
            
            main : IO ();
            main = (
                let u = assert_eq(|_|"", let x = 5 in -x, -5);
                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test2() {
    let source = r#"
            module Main; 
            import Debug;

            main : IO ();
            main = (
                let u = assert_eq(|_|"", let x = 5 in 3, 3);
                pure()
            );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test3() {
    let source = r#"
        module Main; 
        import Debug;

        main : IO ();
        main = (
            let u = assert_eq(|_|"", let n = -5 in let p = 5 in n, -5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test4() {
    let source = r#"
        module Main; 
        import Debug;

        main : IO ();
        main = (
            let u = assert_eq(|_|"", let n = -5 in let p = 5 in p, 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test5() {
    let source = r#"
        module Main; 
        import Debug;

        main : IO ();
        main = (
            let u = assert_eq(|_|"", let x = -5 in let x = 5 in x, 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test6() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let u = assert_eq(|_|"", let x = let y = 3 in y in x, 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test7() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let u = assert_eq(|_|"", (|x| 5)(10), 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test8() {
    let source = r#"
        module Main; import Debug; 

        main : IO ();
        main = (
            let u = assert_eq(|_|"", (|x| x) $ 6, 6);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test9_5() {
    let source = r#"
        module Main; import Debug;
        
        main : IO ();
        main = (
            let x = 3;
            let y = 5;
            let u = assert_eq(|_|"", x - y, -2);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test10() {
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let u = assert_eq(|_|"", let x = 5 in 2 + x, 7);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test11() {
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let x = 5 in 
            let y = -3 in
            let u = assert_eq(|_|"", x + y, 2);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test12() {
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let x = 5 in 
            let y = -3 in
            let z = 12 in
            let xy = x + y in
            let u = assert_eq(|_|"", xy + z, 14);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test13() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let f = add(5) in
            let u = assert_eq(|_|"", f(3), 5+3);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test13_5() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let f = add(5) in
            let u = assert_eq(|_|"", f(-3) + f(12), 5 - 3 + 5 + 12);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test14() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let x = 3 in 
            let y = 5 in
            let f = add(x) in
            let u = assert_eq(|_|"", f(y), 3 + 5);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test15() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let f = |x| 3 + x in
            let u = assert_eq(|_|"", f(5), 3 + 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test15_5() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let x = 3;
            let f = |y| x;
            let u = assert_eq(|_|"", f(5), 3);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test16() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let f = |x| x + 3 in
            let u = assert_eq(|_|"", f(5), 3 + 5);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test17() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let u = assert_eq(|_|"", if true { 3 } else { 5 }, 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test18() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let u = assert_eq(|_|"", if false { 3 } else { 5 }, 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test19() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let u = assert_eq(|_|"", if 3 == 3 { 1 } else { 0 }, 1);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test20() {
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let u = assert_eq(|_|"", if 3 == 5 { 1 } else { 0 }, 0);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test20_5() {
    let source = r#"
        module Main; import Debug;
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
            let u = assert_eq(|_|"", ans, 2);
            pure ()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test21() {
    let source = r#"
            module Main; import Debug;

            main : IO ();
            main = (
                let fact = fix $ |loop, n| if n == 0 { 1 } else { n * loop(n-1) };
                let u = assert_eq(|_|"", fact(5), 5 * 4 * 3 * 2 * 1);
                pure()
            );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test22() {
    // Test recursion function defined by fix with two variables that is tail-call.
    let n: i64 = 1000000;
    let source = format!(
        r#"
            module Main; import Debug;
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
                    let u = assert_eq(|_|"", g(0, {}), {});
                    pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), Configuration::release());
}

#[test]
pub fn test22_5() {
    // Test recursion function defined by fix that is not tail-call.
    let source = r#"
        module Main; import Debug;
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
                let u = assert_eq(|_|"", fib(10), 55);
                pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test22_7() {
    // Test global recursion function
    let source = r#"
        module Main; import Debug;

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
            let u = assert_eq(|_|"", fib(30), 832040);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test23() {
    // Test Array::fill of size 0.
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let arr = Array::fill(0, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test24() {
    // Test Array::fill of size > 0.
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let u = assert_eq(|_|"", arr.get_size, 100);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test25() {
    // Test Array::get.
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let elem = arr.@(50);
            let u = assert_eq(|_|"", elem, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test26() {
    // Test Array::set (unique case).
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let arr = Array::fill(100, 42);
            let arr = arr.set(50, 21);
            let u = assert_eq(|_|"", arr.@(50), 21);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test27() {
    // Test Array::set (shared case).
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let arr0 = Array::fill(100, 42);
            let arr1 = arr0.set(50, 21);
            let u = assert_eq(|_|"", arr0.@(50) + arr1.@(50), 63);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test27_5() {
    // Test Array of boxed object.
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let arr = Array::from_map(100) $ |i| add(i);
            let arr = arr.set(99, |x| x - 100);
            let u = assert_eq(|_|"", arr.@(99) $ arr.@(50) $ 1, 1 + 50 - 100);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test28() {
    // Calculate Fibonacci sequence using array.
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let arr = Array::fill(31, 0);
            let arr = arr.set!(0, 0);
            let arr = arr.set!(1, 1);
            let loop = fix $ |f, arr: Array I64, n| (
                if n == 31 {
                    arr
                } else {
                    let x = arr.@(add(n, -1));
                    let y = arr.@(add(n, -2));
                    let arr = arr.set!(n, x+y);
                    f(arr, n+1)
                }
            );
            let fib = loop(arr, 2);
            let u = assert_eq(|_|"", fib.@(30), 832040);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test29() {
    let source = r#"
        module Main; import Debug;

        id : a -> a;
        id = |x| x;

        main : IO ();
        main = (
            let u = assert_eq(|_|"", if id(true) { id(100) } else { 30 }, 100);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test30() {
    // Test dollar combinator
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let f = |x| x + 3;
            let g = |x| x == 8;
            let ans = g $ f $ 5;
            let u = assert_eq(|_|"", if ans { 1 } else { 0 }, 1);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test31() {
    // Test . combinator
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let f = |x| x + 3;
            let g = |x| x == 8;
            let ans = 5 .f. g;
            let u = assert_eq(|_|"", if ans { 1 } else { 0 } , 1);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test32() {
    // Test . and $ combinator
    let source = r#"
        module Main; import Debug;
        main : IO ();
        main = (
            let f = |x| x + 10;
            let u = assert_eq(|_|"", 5.add $ 3.f, 18);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test33() {
    // Test struct declaration and new, mod.
    let source = r#"
        module Main; import Debug;
        type I64Bool = box struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool { x: 18, y: false };
            let obj = I64Bool::mod_x(|x| x + 42, obj);
            let u = assert_eq(|_|"", I64Bool::@x(obj), 60);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test34_5() {
    // Test unboxed struct declaration and new, mod.
    let source = r#"
        module Main; import Debug;
        type I64Bool = unbox struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool { x: 18, y : false};
            let obj = I64Bool::mod_x(|x| x + 42, obj);
            let u = assert_eq(|_|"", I64Bool::@x(obj), 60);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test34() {
    // Test namespace inference.
    let source = r#"
        module Main; import Debug;        
        
        type OtherStruct = box struct {y: I64, x: Bool};
        type I64Bool = box struct {x: I64, y: Bool};

        main : IO ();
        main = (
            let obj = I64Bool {x: 18, y: false};
            let obj = obj.mod_x(|x| x + 42);
            let u = assert_eq(|_|"", obj.@x, 60);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test35() {
    // Test overloading resolution.
    let source = r#"
        module Main; import Debug;

        type A = box struct {x: I64, y: Bool};
        type B = box struct {x: Bool, y: I64};
            
        main : IO ();
        main = (
            let a = A {x: 3, y: true};
            let b = B {x: true, y: 5};
            let ans = add(if a.@y { a.@x } else { 0 }, if b.@x { b.@y } else { 0 });
            let u = assert_eq(|_|"", ans, 8);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test36() {
    // Test modifier composition.
    let source = r#"
        module Main; import Debug;

        type A = box struct {x: B};
        type B = box struct {x: I64};
            
        main : IO ();
        main = (
            let a = A{x: B{x: 16}};
            let a = a.(mod_x $ mod_x $ |x| x + 15);
            let ans = a . @x . @x;
            let u = assert_eq(|_|"", ans, 31);
            pure ()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test37() {
    // Test unique modField.
    let source = r#"
        module Main; import Debug;

        type A = box struct {x: B};
        type B = box struct {x: I64};

        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let b = a . (mod_x! $ mod_x! $ |x| x + 15);
            let ans = b . @x . @x;
            let u = assert_eq(|_|"", ans, 31);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test37_5() {
    // Test shared modField.
    let source = r#"
        module Main; import Debug;

        type A = box struct {x: B};
        type B = box struct {x: I64};

        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let b = a.(mod_x $ mod_x $ |x| x + 15);
            let ans = a.@x.@x + b.@x.@x;
            let u = assert_eq(|_|"", ans, (16 + 15) + 16);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test38() {
    // Test type annotation.
    let source = r#"
        module Main; import Debug;

        type A = box struct {x: B};
        type B = box struct {x: I64};

        main : IO ();
        main = (    
            let a = A {x: B {x: 16}};
            let f = |a| (a : A) . (mod_x! $ mod_x! $ |x| x + 15);
            let a = a.f;
            let ans = a.@x.@x;
            let u = assert_eq(|_|"", ans, 31);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test39() {
    // Test type annotation.
    let source = r#"
        module Main; import Debug;

        type A = box struct {x: B};
        type B = box struct {x: I64};
        
        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let f = |a| a . ((mod_x! : (B -> B) -> A -> A) $ mod_x! $ |x| x + 15);
            let a = a.f;
            let ans = a.@x.@x;
            let u = assert_eq(|_|"", ans, 31);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test40() {
    // Test type annotation at let-binding.
    let source = r#"
        module Main; import Debug;

        type A = box struct {x: B};
        type B = box struct {x: I64};
        
        main : IO ();
        main = (
            let a = A {x: B {x: 16}};
            let f: A -> A = |a| a.(mod_x! $ mod_x! $ |x| x + 15);
            let a = a .f;
            let ans = a .@x .@x;
            let u = assert_eq(|_|"", ans, 31);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test41() {
    // Test type annotation at let-binding.
    let source = r#"
        module Main; import Debug;
        
        main : IO ();
        main = (
            let x: I64 -> I64 = |x| x;
            let ans = x(42);
            let u = assert_eq(|_|"", ans, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test41_5() {
    // Test type annotation at lambda
    let source = r#"
        module Main; import Debug;
        
        main : IO ();
        main = (
            let x = |x: I64| x;
            let ans = x(42);
            let u = assert_eq(|_|"", ans, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test42() {
    // Recursion function using global variable (not tail call).
    let n = 10000;
    let source = format!(
        r#"
            module Main; import Debug;
            
            loop : I64 -> I64;
            loop = |x| if x == 0 {{ 0 }} else {{ add(x) $ loop $ add(x, -1) }};
    
            main : IO ();
            main = (
                let ans = Main::loop({});
                let u = assert_eq(|_|"", ans, {});
                pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), Configuration::develop_compiler());
}

#[test]
pub fn test43() {
    // Recursion function using global variable (tail call).
    let n: i64 = 10000000;
    let source = format!(
        r#"
            module Main; import Debug;
            
            my_loop : I64 -> I64 -> I64;
            my_loop = |x, acc| if x == 0 {{ acc }} else {{ my_loop(x + -1, acc + x) }};
    
            main : IO ();
            main = (
                let ans = my_loop({}, 0);
                let u = assert_eq(|_|"", ans, {});
                pure()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), Configuration::release());
}

#[test]
pub fn test44() {
    // Test basic use of traits.
    let source = r#"
        module Main; import Debug;

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
            let arr0 = arr0.set!(0, true);
            let x = add_head_and_next(arr0);

            let arr1 = Array::fill(2, 3);
            let arr1 = arr1.set!(1, 5);
            let z = add_head_and_next(arr1);

            let y = toI64(5) + toI64(false);
            let ans = x + y + z;
            let u = assert_eq(|_|"", ans, 11);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test44_5() {
    // Test Array::from_map.
    let source = r#"
        module Main; import Debug;

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
            let u = assert_eq(|_|"", ans, 285);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test45() {
    // Test HKT.
    let source = r#"
        module Main; import Debug;

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
            let u = assert_eq(|_|"", ans, 285);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test46() {
    // Test confliction of global name and local name.
    let source = r#"
        module Main; import Debug;

        x : I64;
        x = 5;

        y : I64;
        y = 7;

        main : IO ();
        main = (
            let ans = (let x = 3 in let y = 2 in add(x, Main::y)) + x;
            let u = assert_eq(|_|"", ans, 15);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test47() {
    // Basic use of union.
    let source = r#"
        module Main; import Debug;

        type I64OrBool = union {int : I64, bool: Bool};

        main : IO ();
        main = (
            let int_union = int(2).mod_bool(not).mod_int(add(1));
            let bool_union = bool(false).mod_bool(not).mod_int(add(1));
            let int_val = if int_union.is_int { int_union.as_int } else { 0 };
            let bool_val = if bool_union.is_bool { bool_union.as_bool } else { false };
            let ans = if bool_val { int_val } else { 0 };
            let u = assert_eq(|_|"", ans, 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test47_2() {
    // Basic use of boxed union.
    let source = r#"
        module Main; import Debug;

        type I64OrBool = box union {int : I64, bool: Bool};

        main : IO ();
        main = (
            let int_union = int(2).mod_bool(not).mod_int(add(1));
            let bool_union = bool(false).mod_bool(not).mod_int(add(1));
            let int_val = if int_union.is_int { int_union.as_int } else { 0 };
            let bool_val = if bool_union.is_bool { bool_union.as_bool } else { false };
            let ans = if bool_val { int_val } else { 0 };
            let u = assert_eq(|_|"", ans, 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test47_5() {
    // Test union of closure object
    let source = r#"
        module Main; import Debug;

        type Union = union {val: I64, func: I64 -> I64};

        main : IO ();
        main = (
            let five = 5;
            let val = Union::val(3);
            let func = Union::func(|x| x + five).mod_func(|f||x|f(x)+2); // x -> x + 5 + 2
            let ans = func.as_func $ val.as_val;
            let u = assert_eq(|_|"", ans, 7 + 3);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test47_6() {
    // Test union of boxed object
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let uni = Option::some([1,2,3]).mod_some(
                |lhs| lhs.force_unique!.append([4,5,6])
            );
            let arr = uni.as_some;
            eval assert_eq(|_|"", arr.@(0), 1);
            eval assert_eq(|_|"", arr.@(1), 2);
            eval assert_eq(|_|"", arr.@(2), 3);
            eval assert_eq(|_|"", arr.@(3), 4);
            eval assert_eq(|_|"", arr.@(4), 5);
            eval assert_eq(|_|"", arr.@(5), 6);
            eval assert_eq(|_|"", arr.get_size, 6);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test48() {
    // Parametrised struct.
    let source = r#"
        module Main; import Debug;

        type Vec a = box struct {data: Array a};

        main : IO ();
        main = (
            let int_vec = Vec {data: Array::fill(2, 5)};
            let int_vec = int_vec.mod_data!(|arr| arr.set(0, 3));
            let head = int_vec.@data.@(0);
            let next = int_vec.@data.@(1);
            let ans = add(head, next);
            let u = assert_eq(|_|"", ans, 8);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test49() {
    // Parametrised union.
    let source = r#"
        module Main; import Debug;

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
            let u = assert_eq(|_|"", ans, 5);
            pure()
        );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test50() {
    // test loop.
    let n = 100;
    let source = format!(
        r#"
            module Main; import Debug;
    
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
                let u = assert_eq(|_|"", ans, {});
                pure()
            );
        "#,
        n,
        (n * (n - 1)) / 2
    );
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test50_3() {
    // test loop_iter, loop_iter_m.
    let source = r#"
        module Main; import Debug;
        
        main : IO ();
        main = (
            let sum = Iterator::count_up(0).loop_iter(0, |sum, n| (
                if n > 100 { break $ sum };
                continue $ sum + n
            ));
            eval assert_eq(|_|"case-loop", sum, 100 * 101 / 2);

            let io_sum : IO I64 = Iterator::count_up(0).loop_iter_m(0, |sum, n| (
                if n > 5 { break_m $ sum };
                eval *(print $ n.to_string + " ");
                continue_m $ sum + n
            ));
            eval *println("");
            eval assert_eq(|_|"case-loop_m", io_sum._unsafe_perform, 5 * 6 / 2);

            pure()
        );
            "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test51() {
    // test trait bounds.
    let source = r#"
    module Main; import Debug;
    
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
        let u = assert_eq(|_|"", ans, 2);
        pure()
    );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test52() {
    // Test loop with boxed state / break.
    let source = r#"
    module Main; import Debug;

    type SieveState = box struct {i: I64, arr: Array Bool};
    
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
            let next_arr = if arr.@(i) {
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
            let sum = sum + (if arr.@(i) == elem {1} else {0});
            continue $ (i+1, sum)
        )
    );
    
    main : IO ();
    main = (
        let ans = (is_prime $ 100).count(true);
        let u = assert_eq(|_|"", ans, 25);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test53() {
    // Test mutation of unique unboxed struct (e.g., tuple).
    let source = r#"
    module Main; import Debug;
    
    main : IO ();
    main = (
        let pair = (13, Array::fill(1, 0));
        let pair = pair.mod_0!(|x| x + 3);
        let pair = pair.mod_1!(|arr| arr.set!(0, 5));
        let x = pair.@0;
        let y = pair.@1.@(0);
        let ans = x + y;
        let u = assert_eq(|_|"", ans, 13 + 3 + 5);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test54() {
    // Test mutation of shared unboxed struct (e.g., tuple).
    let source = r#"
    module Main; import Debug;
    
    main : IO ();
    main = (
        let pair0 = (13, Array::fill(1, 0));
        let pair1 = pair0.mod_1(|arr| arr.set(0, 5));
        let pair2 = pair0.mod_0!(|x| x + 3);
        let x = pair1.@1.@(0);
        let y = pair2.@0;
        let ans = x + y;
        let u = assert_eq(|_|"", ans, 13 + 3 + 5);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test55() {
    // Test <= operator
    let source = r#"
    module Main; import Debug;
    
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
        let u = assert_eq(|_|"", ans, 1);
        pure ()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test56() {
    // Test && and || operator
    let source = r#"
    module Main; import Debug;
    
    main : IO ();
    main = (
        let ans = (
            if false || false == false 
            && false || true == true 
            && true || false == true 
            && true || true == true 
            {1} else {0}
        );
        let u = assert_eq(|_|"", ans, 1);
        pure ()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test57() {
    // Test ! operator
    let source = r#"
    module Main; import Debug;
    
    main : IO ();
    main = (
        let ans = (
            if !false == true && !true == false {
                1
            } else {
                0
            }
        );
        let u = assert_eq(|_|"", ans, 1);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test58() {
    // Test != operator
    let source = r#"
    module Main; import Debug;
    
    main : IO ();
    main = (
        let ans = (
            if false != true && true != false && !(true != true) && !(false != false) {
                1
            } else {
                0
            }
        );
        let u = assert_eq(|_|"", ans, 1);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test59() {
    // Test namespace definition
    let source = r#"
    module Main; import Debug;
    
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
        let u = assert_eq(|_|"", ans, 9);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test60() {
    // Test unit.
    let source = r"
    module Main; import Debug;
    
    unit : ();
    unit = ();

    main : IO ();
    main = let u = unit; pure ();
    ";
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test61() {
    // Test Hello world.
    let source = r#"
    module Main; import Debug;

    main_loop : I64 -> IO ();
    main_loop = |counter| (
        if counter == 0 {
            pure()
        } else {
            eval *println("Hello World! (" + counter.to_string + ")");
            main_loop(counter - 1)
        }
    );

    main : IO ();
    main = main_loop(3);
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test61_5() {
    // Test Hello world.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        loop_m(0, |i| (
            if i == 3 { break_m $ () };
            eval *println("Hello World! (" + i.to_string + ")");
            continue_m $ i + 1
        ))
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test62() {
    // Test String length.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let len = "Hello World!".get_size;
        let u = assert_eq(|_|"", len, 12);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test63() {
    // Test I64 ToString.
    // See also test98.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let min = -9223372036854775808;
        eval assert_eq(|_|"", min.to_string, "-9223372036854775808");
        println $ min.to_string
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test64() {
    // Test escape sequence.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        println $ "\u2764"
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test65() {
    // Test tuple pattern matching.
    let source = r#"
    module Main; import Debug;

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
        let u = assert_eq(|_|"", sum, 45);
        pure ()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test66() {
    // Test unboxed struct pattern matching.
    let source = r#"
    module Main; import Debug;

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
        let u = assert_eq(|_|"", sum, 45);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test67() {
    // Test boxed struct pattern matching.
    let source = r#"
    module Main; import Debug;

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
        let u = assert_eq(|_|"", sum, 45);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test72() {
    // Test pattern matching on argment.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let sum = loop((0, 0), |(i, sum)|
            if i == 10 {
                break $ sum
            } else {
                continue $ (i + 1, sum + i)
            }
        );
        let u = assert_eq(|_|"", sum, 45);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test73() {
    // Test pattern matching on argment.
    let source = r#"
    module Main; import Debug;

    type I64Bool = box struct {x: I64, y: Bool};

    main : IO ();
    main = (
        let int_bool = I64Bool { y: true, x: 42 };
        let u = assert_eq(|_|"", int_bool.@x, 42);
        let u = assert_eq(|_|"", int_bool.@y, true);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test74() {
    // Test setter function of struct / tuple.
    let source = r#"
    module Main; import Debug;

    type UnboxStr = unbox struct {x: I64, y: Bool};
    type BoxStr = box struct {x: I64, y: Bool};

    main : IO ();
    main = (
        // Setter / getter of unboxed struct.
        let int_bool = UnboxStr { y: false, x: 0 };
        let int_bool = int_bool.set_x(3);
        let u = assert_eq(|_|"case 0", int_bool.@x, 3);
        let int_bool = int_bool.set_x!(5);
        let u = assert_eq(|_|"case 1", int_bool.@x, 5);

        // Setter / getter of pair.
        let pair = (false, 0);
        let pair = pair.set_0(true);
        let u = assert_eq(|_|"case 2", pair.@0, true);
        let pair = pair.set_0!(false);
        let u = assert_eq(|_|"case 3", pair.@0, false);

        // Setter / getter of boxed struct.
        let int_bool = BoxStr { y: false, x: 0 };
        let int_bool = int_bool.set_y(true);
        let u = assert_eq(|_|"case 4", int_bool.@y, true);
        let int_bool = int_bool.set_y!(false);
        let u = assert_eq(|_|"case 5", int_bool.@y, false);

        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test75() {
    // Test iterator.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let iter = Iterator::from_map(|i| i*i );
        let (n, iter) = iter.advance.as_some;
        eval assert_eq(|_|"", n, 0*0);
        let (n, iter) = iter.advance.as_some;
        eval assert_eq(|_|"", n, 1*1);
        let (n, iter) = iter.advance.as_some;
        eval assert_eq(|_|"", n, 2*2);
        let (n, iter) = iter.advance.as_some;
        eval assert_eq(|_|"", n, 3*3);
        let (n, iter) = iter.advance.as_some;
        eval assert_eq(|_|"", n, 4*4);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test76() {
    // Test array modifier.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let array = Array::from_map(3, |_i| Array::from_map(3, |_j| 0));
        let array = array.mod!(1, Array::set!(1, 9));
        eval assert_eq(|_|"", array.@(1).@(1), 9);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test77() {
    // Test Iterator::zip / map / take / fold / subsequences.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let iter0 = Iterator::count_up(5);
        let iter1 = Iterator::from_map(|i| 2*i);
        let iter = iter0.zip(iter1);
        let iter = iter.map(|(a,b)| a+b).take(3);
        let res = iter.fold(0, add);
        eval assert_eq(|_|"case 1", res, (5+2*0) + (6+2*1) + (7+2*2));

        let subs = (Iterator::empty : Iterator I64).subsequences;
        eval assert_eq(|_|"subsequences 1", subs.get_size, 1);
        eval assert_eq(|_|"subsequences 2", subs.advance.as_some.@0.get_size, 0);

        let subs = [1,2,3].to_iter.subsequences;
        eval assert_eq(|_|"subsequences 3", subs.map(to_array).to_array, [[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]]);
        // eval debug_println $ subs.to_iter.map(to_iter).map(map(to_string) >> join(", ") >> |s| "[" + s + "]").join(", ");

        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test78() {
    // Test Iterator::filter
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let iter = Iterator::count_up(1).take(100);
        let iter = iter.filter(|n| n%3 == 0 || n%5 == 0);
        let count = iter.map(|_|1).fold(0, add);
        eval assert_eq(|_|"", count, 100/3 + 100/5 - 100/15);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test79() {
    // Test Iterator::push_front
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let ls = Iterator::empty;
        let ls = ls.push_front(1).push_front(2);
        let (e, ls) = ls.advance.as_some;
        eval assert_eq(|_|"", 2, e);
        let (e, ls) = ls.advance.as_some;
        eval assert_eq(|_|"", 1, e);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test80() {
    // Test Iterator::last
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let iter = Iterator::empty.push_front(4).push_front(3).push_front(2).push_front(1);
        let last = iter.find_last.as_some;
        eval assert_eq(|_|"", last, 4);
        let last: Option Bool = Iterator::empty.find_last;
        eval assert(|_|"", last.is_none);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test81() {
    // Test array literal.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let arr = [1,2,3,4];
        eval assert_eq(|_|"", arr.get_size, 4);
        let arr: Array Bool = [];
        eval assert_eq(|_|"", arr.get_size, 0);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test82() {
    // Test Array::append.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (

        // Test 0+2
        let v1 = [];
        let v2 = [3,4];
        let v = v1.append(v2);
        eval assert_eq(|_|"wrong reserved length (0+2)", v.get_capacity, 2);
        eval assert_eq(|_|"wrong length (0+2)", v.get_size, 2);
        eval assert_eq(|_|"wrong element (0+2)", v.@(0), 3);
        eval assert_eq(|_|"wrong element (0+2)", v.@(1), 4);

        // Test 2+0
        let v1 = [1,2];
        let v2 = [];
        let v = v1.append(v2);
        eval assert_eq(|_|"wrong reserved length (2+0)", v.get_capacity, 2);
        eval assert_eq(|_|"wrong length (2+0)", v.get_size, 2);
        eval assert_eq(|_|"wrong element (2+0)", v.@(0), 1);
        eval assert_eq(|_|"wrong element (2+0)", v.@(1), 2);

        // Test 0+0
        let v1: Array (I64 -> Bool) = [];
        let v2 = [];
        let v = v1.append(v2);
        eval assert_eq(|_|"wrong capacity (0+0)", v.get_capacity, 0);
        eval assert_eq(|_|"wrong length (0+0)", v.get_size, 0);

        // Test boxed elements.
        let v1 = [add(1), add(2)];
        let v2 = [add(3), add(4)];
        let v = v1.append(v2);
        let x = 0;
        let x = v.@(0) $ x;
        eval assert_eq(|_|"wrong value (boxed) 0+1", x, 0+1);
        let x = v.@(1) $ x;
        eval assert_eq(|_|"wrong value (boxed) 0+1+2", x, 0+1+2);
        let x = v.@(2) $ x;
        eval assert_eq(|_|"wrong value (boxed) 0+1+2+3", x, 0+1+2+3);
        let x = v.@(3) $ x;
        eval assert_eq(|_|"wrong value (boxed) 0+1+2+3+4", x, 0+1+2+3+4);

        // Test appending shared array.
        let v1 = [add(1), add(2)].reserve(4);
        let v2 = [add(3), add(4)];
        let v = v1.append(v2);
        let w = v2.append(v1);
        let x = 0;
        let x = v.@(0) $ x; // += 1
        let x = w.@(3) $ x; // += 2
        eval assert_eq(|_|"", x, 3);

        let res = Array::empty(3);
        let v = [[1], [2], [3]].to_iter.fold(res, |res, v| (
            res.append!(v)
        ));
        eval assert_eq(|_|"", v, [1, 2, 3]);

        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test83() {
    // Test Array::push_back, pop_back
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        // Unboxed element
        let v = [];
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.push_back(idx);
            continue $ (idx+1, v)
        ));
        eval loop(0, |idx|(
            if idx == 100 { break $ () };
            eval assert_eq(|_|"wrong element", idx, v.@(idx));
            continue $ idx + 1
        ));
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.pop_back;
            continue $ (idx+1, v)
        ));
        eval assert_eq(|_|"wrong length after pop", 0, v.get_size);
        eval assert(|_|"wrong reserved length after pop", v.get_capacity >= 100);
    
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
        eval assert_eq(|_|"wrong value (boxed)", x, 99 * 100 / 2);
        let v = loop((0, v), |(idx, v)|(
            if idx == 100 { break $ v };
            let v = v.pop_back;
            continue $ (idx+1, v)
        ));
        eval assert_eq(|_|"wrong length after pop (boxed)", 0, v.get_size);
        eval assert(|_|"wrong reserved length after pop (boxed)", v.get_capacity >= 100);
    
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test84() {
    // Test Eq for Array
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let v1 = [1,2,3];
        let v2 = [1,2,3];
        eval assert(|_|"", v1 == v2);
    
        let v1 = [1,2,3];
        let v2 = [0,2,3];
        eval assert(|_|"", v1 != v2);
    
        let v1 = [];
        let v2 = [0];
        eval assert(|_|"", v1 != v2);
    
        let v1: Array I64 = [];
        let v2 = [];
        eval assert(|_|"", v1 == v2);
    
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test85() {
    // Test concat string, compare string.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let s1 = "Hello";
        let s2 = " ";
        let s3 = "World!";
        eval assert_eq(|_|"", s1.concat(s2).concat(s3), "Hello World!");
    
        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test86() {
    // Test concat_iter
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let iter = Iterator::from_array(["Hello", " ", "World", "!"]);
        eval assert_eq(|_|"", iter.concat_iter, "Hello World!");
        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test87() {
    // Test iterator comparison
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([1,2,3]);
        eval assert_eq(|_|"", lhs, rhs);

        let lhs: Iterator Bool = Iterator::from_array([]);
        let rhs = Iterator::from_array([]);
        eval assert_eq(|_|"", lhs, rhs);

        let lhs = Iterator::from_array([]);
        let rhs = Iterator::from_array([1,2]);
        eval assert(|_|"", lhs != rhs);

        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test88() {
    // Test iterator comparison
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let iter = Iterator::from_array([1,2,3]);
        let iter = iter.intersperse(0);
        eval assert_eq(|_|"", iter, Iterator::from_array([1,0,2,0,3]));
    
        let iter = Iterator::from_array([1]);
        let iter = iter.intersperse(0);
        eval assert_eq(|_|"", iter, Iterator::from_array([1]));
    
        let iter = Iterator::from_array([]);
        let iter = iter.intersperse(0);
        eval assert_eq(|_|"", iter, Iterator::from_array([]));
    
        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test89() {
    // Test Iterator::append
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([4,5,6]);
        eval assert_eq(|_|"", lhs + rhs, Iterator::from_array([1,2,3,4,5,6]));
    
        let lhs = Iterator::from_array([]);
        let rhs = Iterator::from_array([4,5,6]);
        eval assert_eq(|_|"", lhs + rhs, Iterator::from_array([4,5,6]));

        let lhs = Iterator::from_array([1,2,3]);
        let rhs = Iterator::from_array([]);
        eval assert_eq(|_|"", lhs + rhs, Iterator::from_array([1,2,3]));

        let lhs: Iterator I64 = Iterator::from_array([]);
        let rhs = Iterator::from_array([]);
        eval assert_eq(|_|"", lhs + rhs, Iterator::from_array([]));
    
        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test90() {
    // Test Array::sort_by.
    let source = r#"
    module Main; import Debug;

    main : IO ();
    main = (
        let vec = [5,3,1,7,4,6,9,8,2];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        eval assert_eq(|_|"wrong result 9", vec, [1,2,3,4,5,6,7,8,9]);

        let vec = [1];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        eval assert_eq(|_|"wrong result 1", vec, [1]);

        let vec: Array I64 = [];
        let vec = vec.sort_by(|(lhs, rhs)| lhs < rhs);
        eval assert_eq(|_|"wrong result 0", vec, []);

        pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test92() {
    let source = r#"
    module Main; import Debug;

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
    run_source(source, Configuration::develop_compiler());
}

#[test]
pub fn test_call_c() {
    // Test FFI
    let source = r#"
            module Main; import Debug;
    
            main : IO ();
            main = (
                eval "Hello C function! Number = %d\n".borrow_c_str(|ptr|
                    let _ = CALL_C[I32 printf(Ptr, ...), ptr, 42];
                    ()
                );
                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test95() {
    // Test Std::unsafe_is_unique, Debug::assert_unique!
    let source = r#"
            module Main; 
            import Debug;
    
            main : IO ();
            main = (
                // For unboxed value, it returns true even if the value is used later.
                let int_val = 42;
                let (unique, _) = int_val.unsafe_is_unique;
                let use = int_val + 1;
                eval assert_eq(|_|"fail: int_val is shared", unique, true);

                // For boxed value, it returns true if the value isn't used later.
                let arr = Array::fill(10, 10);
                let (unique, arr) = arr.unsafe_is_unique;
                let use = arr.@(0); // This `arr` is not the one passed to `is_unique`, but the one returned by `is_unique`.
                eval assert_eq(|_|"fail: arr is shared", unique, true);

                // Fox boxed value, it returns false if the value will be used later.
                let arr = Array::fill(10, 10);
                let (unique, _) = arr.unsafe_is_unique;
                let use = arr.@(0);
                eval assert_eq(|_|"fail: arr is unique", unique, false);

                let int_val = 42;
                let _ = int_val.assert_unique!(|_|"fail: int_val is shared (2)");
                let use = int_val + 1;

                let arr = Array::fill(10, 10);
                let arr = arr.assert_unique!(|_|"fail: arr is shared (2)");
                let use = arr.@(0);

                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test96() {
    // Test U8 literal
    let source = r#"
            module Main; import Debug;
            
            main : IO ();
            main = (
                eval assert_eq(|_|"", 255_U8, 255_U8);
                eval assert_eq(|_|"", 'A', 65_U8);
                eval assert_eq(|_|"", '\0', 0_U8);
                eval assert_eq(|_|"", '\t', 9_U8);
                eval assert_eq(|_|"", '\r', 13_U8);
                eval assert_eq(|_|"", '\n', 10_U8);
                eval assert_eq(|_|"", '\\', 92_U8);
                eval assert_eq(|_|"", '\'', 39_U8);
                eval assert_eq(|_|"", '\x7f', 127_U8);
                pure()
            );
        "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test97() {
    // Test arithmetic operation of U8, I32
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"1", -(1_U8), 255_U8);
            eval assert_eq(|_|"2", 255_U8 + 3_U8, 2_U8);
            eval assert_eq(|_|"3", 1_U8 - 3_U8, 254_U8);
            eval assert_eq(|_|"4", 20_U8 * 30_U8, 88_U8);
            eval assert_eq(|_|"5", 10_U8 / 3_U8, 3_U8);
            eval assert_eq(|_|"6", 10_U8 % 3_U8, 1_U8);
            eval assert_eq(|_|"7", 255_U8 > 0_U8, true);
            eval assert_eq(|_|"8", 255_U8 >= 0_U8, true);

            eval assert_eq(|_|"9", 2147483647_I32 + 2_I32, -2147483647_I32);
            eval assert_eq(|_|"10", -2147483647_I32 - 2_I32, 2147483647_I32);
            eval assert_eq(|_|"11", 2147483647_I32 * 2_I32, -2_I32);
            eval assert_eq(|_|"12", 10_I32 / -3_I32, -3_I32);
            eval assert_eq(|_|"13", 10_I32 % -3_I32, 1_I32);
            eval assert_eq(|_|"14", -1_I32 < 0_I32, true);
            
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test98() {
    // Test to_string, from_string for integrals
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            // I8
            eval assert_eq(|_|"I8 1", -128_I8.to_string, "-128");
            eval assert_eq(|_|"I8 2", 127_I8.to_string, "127");
            eval assert_eq(|_|"I8 3", -128_I8, "-128".from_string.as_ok);

            // U8
            eval assert_eq(|_|"", 0_U8.to_string, "0");
            eval assert_eq(|_|"", 255_U8.to_string, "255");
            eval assert_eq(|_|"", 255_U8, "255".from_string.as_ok);

            // I16
            eval assert_eq(|_|"I16 1", -32768_I16.to_string, "-32768");
            eval assert_eq(|_|"I16 2", 32767_I16.to_string, "32767");
            eval assert_eq(|_|"I16 3", -32768_I16, "-32768".from_string.as_ok);

            // U16
            eval assert_eq(|_|"", 0_U16.to_string, "0");
            eval assert_eq(|_|"", 65535_U16.to_string, "65535");
            eval assert_eq(|_|"", 65535_U16, "65535".from_string.as_ok);

            // I32
            eval assert_eq(|_|"", -2147483648_I32.to_string, "-2147483648");
            eval assert_eq(|_|"", 2147483647_I32.to_string, "2147483647");
            eval assert_eq(|_|"", -2147483648_I32, "-2147483648".from_string.as_ok);

            // U32
            eval assert_eq(|_|"", 0_U32.to_string, "0");
            eval assert_eq(|_|"", 4294967295_U32.to_string, "4294967295");
            eval assert_eq(|_|"", 4294967295_U32, "4294967295".from_string.as_ok);

            // I64
            eval assert_eq(|_|"", -9223372036854775808_I64.to_string, "-9223372036854775808");
            eval assert_eq(|_|"", 9223372036854775807_I64.to_string, "9223372036854775807");
            eval assert_eq(|_|"", -9223372036854775808_I64, "-9223372036854775808".from_string.as_ok);

            // U64
            eval assert_eq(|_|"", 0_U64.to_string, "0");
            eval assert_eq(|_|"", 18446744073709551615_U64.to_string, "18446744073709551615");
            eval assert_eq(|_|"", 18446744073709551615_U64, "18446744073709551615".from_string.as_ok);

            // Cases from_string fails.

            let res: Result ErrMsg I64 = "Hello World!".from_string;
            eval assert(|_|"Case: from_string invalid format", res.is_err);

            let res: Result ErrMsg I64 = " 42".from_string;
            eval assert(|_|"Case: from_string invalid format (whitespace)", res.is_err);

            let res: Result ErrMsg I64 = "1844674407370955161518446744073709551615".from_string;
            eval assert(|_|"Case: from_string out of range", res.is_err);
            
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
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
            let case = format!("eval assert_eq(|_|{}, {}, {});", msg, lhs, rhs);
            cases.push(case);
        }
    }
    let source = format!(
        r#"
            module Main; import Debug;
    
            main : IO ();
            main = (
                {}
                pure()
            );
        "#,
        cases.join("\n")
    );
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test99_5() {
    // Test cast float to integral types.
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"", -3.14_F32.to_I8, -3_I8);
            eval assert_eq(|_|"", 3.14_F32.to_U8, 3_U8);
            eval assert_eq(|_|"", -3.14_F32.to_I16, -3_I16);
            eval assert_eq(|_|"", 3.14_F32.to_U16, 3_U16);
            eval assert_eq(|_|"", -3.14_F32.to_I32, -3_I32);
            eval assert_eq(|_|"", 3.14_F32.to_U32, 3_U32);
            eval assert_eq(|_|"", -3.14_F32.to_I64, -3_I64);
            eval assert_eq(|_|"", 3.14_F32.to_U64, 3_U64);

            eval assert_eq(|_|"", -3.14_F64.to_I8, -3_I8);
            eval assert_eq(|_|"", 3.14_F64.to_U8, 3_U8);
            eval assert_eq(|_|"", -3.14_F64.to_I16, -3_I16);
            eval assert_eq(|_|"", 3.14_F64.to_U16, 3_U16);
            eval assert_eq(|_|"", -3.14_F64.to_I32, -3_I32);
            eval assert_eq(|_|"", 3.14_F64.to_U32, 3_U32);
            eval assert_eq(|_|"", -3.14_F64.to_I64, -3_I64);
            eval assert_eq(|_|"", 3.14_F64.to_U64, 3_U64);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test99_51() {
    // Test cast integral to float types.
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"", -123_I8.to_F32, -123.0_F32);
            eval assert_eq(|_|"", 123_U8.to_F32, 123.0_F32);
            eval assert_eq(|_|"", -123_I16.to_F32, -123.0_F32);
            eval assert_eq(|_|"", 123_U16.to_F32, 123.0_F32);
            eval assert_eq(|_|"", -123_I32.to_F32, -123.0_F32);
            eval assert_eq(|_|"", 123_U32.to_F32, 123.0_F32);
            eval assert_eq(|_|"", -123_I64.to_F32, -123.0_F32);
            eval assert_eq(|_|"", 123_U64.to_F32, 123.0_F32);

            eval assert_eq(|_|"", -123_I8.to_F64, -123.0);
            eval assert_eq(|_|"", 123_U8.to_F64, 123.0);
            eval assert_eq(|_|"", -123_I16.to_F64, -123.0);
            eval assert_eq(|_|"", 123_U16.to_F64, 123.0);
            eval assert_eq(|_|"", -123_I32.to_F64, -123.0);
            eval assert_eq(|_|"", 123_U32.to_F64, 123.0);
            eval assert_eq(|_|"", -123_I64.to_F64, -123.0);
            eval assert_eq(|_|"", 123_U64.to_F64, 123.0);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test100() {
    // Test u8 literal
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"case 1", 'A', 65_U8);
            eval assert_eq(|_|"case 2", '0', 48_U8);
            eval assert_eq(|_|"case 3", '\n', 10_U8);
            eval assert_eq(|_|"case 3", '\x7f', 127_U8);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test101() {
    // Test Array::is_empty, get_first, get_last.
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let cap = 42;
            let arr: Array (() -> I64) = [];
            eval assert_eq(|_|"case 1", arr.is_empty, true);
            eval assert_eq(|_|"case 2", arr.get_first.is_none, true);
            eval assert_eq(|_|"case 3", arr.get_last.is_none, true);

            let cap = 42;
            let arr: Array (() -> I64) = [|_|cap];
            eval assert_eq(|_|"case 4", arr.is_empty, false);
            eval assert_eq(|_|"case 5", arr.get_first.as_some $ (), 42);
            eval assert_eq(|_|"case 6", arr.get_last.as_some $ (), 42);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test102() {
    // Test I64 : Eq, LessThan, LessThanEq
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"case 1", 0 == 0, true);
            eval assert_eq(|_|"case 2", 0 == 1, false);
            eval assert_eq(|_|"case 3", 0 != 0, false);
            eval assert_eq(|_|"case 4", 0 != 1, true);

            eval assert_eq(|_|"case 5", 0 < 0, false);
            eval assert_eq(|_|"case 6", 0 > 0, false);
            eval assert_eq(|_|"case 7", 0 < 1, true);
            eval assert_eq(|_|"case 8", 0 > 1, false);

            eval assert_eq(|_|"case 9", 0 <= 0, true);
            eval assert_eq(|_|"case 10", 0 >= 0, true);
            eval assert_eq(|_|"case 11", 0 <= 1, true);
            eval assert_eq(|_|"case 12", 0 >= 1, false);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test103() {
    // Test Bool : Eq
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"case 1", false == false, true);
            eval assert_eq(|_|"case 2", false == true, false);
            eval assert_eq(|_|"case 3", true == false, false);
            eval assert_eq(|_|"case 4", true == true, true);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test104() {
    // Test Bool : ToString
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"case 1", true.to_string, "true");
            eval assert_eq(|_|"case 2", false.to_string, "false");

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test105() {
    // Test String::get_first_byte, get_last_byte, is_empty
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"case 1", "".is_empty, true);
            eval assert_eq(|_|"case 2", "".get_first_byte.is_none, true);
            eval assert_eq(|_|"case 3", "".get_last_byte.is_none, true);
            eval assert_eq(|_|"case 4", "abc".is_empty, false);
            eval assert_eq(|_|"case 5", "abc".get_first_byte.as_some, 'a');
            eval assert_eq(|_|"case 6", "abc".get_last_byte.as_some, 'c');

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test106() {
    // Test [a : Eq] Option a : Eq
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let lhs: Option I64 = Option::none();
            let rhs: Option I64 = Option::none();
            eval assert(|_|"case 1", lhs == rhs);

            let lhs: Option I64 = Option::none();
            let rhs: Option I64 = Option::some(42);
            eval assert(|_|"case 2", lhs != rhs);

            let lhs: Option I64 = Option::some(84);
            let rhs: Option I64 = Option::some(42);
            eval assert(|_|"case 3", lhs != rhs);

            let lhs: Option I64 = Option::some(42);
            let rhs: Option I64 = Option::some(42);
            eval assert(|_|"case 4", lhs == rhs);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test107() {
    // Test String::pop_back_byte, strip_last_bytes, strip_last_newlines.
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"case 1", "".pop_back_byte, "");
            eval assert_eq(|_|"case 2", "a".pop_back_byte, "");

            eval assert_eq(|_|"case 3", "".strip_last_bytes(|c|c == 'x'), "");
            eval assert_eq(|_|"case 4", "abc".strip_last_bytes(|_|true), "");
            eval assert_eq(|_|"case 5", "".strip_last_bytes(|_|true), "");
            eval assert_eq(|_|"case 6", "x".strip_last_bytes(|c|c == 'x'), "");
            eval assert_eq(|_|"case 7", "y".strip_last_bytes(|c|c == 'x'), "y");
            eval assert_eq(|_|"case 8", "yx".strip_last_bytes(|c|c == 'x'), "y");
            eval assert_eq(|_|"case 9", "yxz".strip_last_bytes(|c|c == 'x'), "yxz");

            eval assert_eq(|_|"case 10", "abc\n\r".strip_last_newlines, "abc");

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test108() {
    // Test write_file_string!, read_file_string!, read_line.
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let file_path = Path::parse("test_uAfQDPwJ7sS6.txt").as_some;
            let lines = ["Hello", "World!"];
            let content = Iterator::from_array(lines).intersperse("\n").concat_iter;
            do {
                eval *write_file_string(file_path, content);

                let read_content = *read_file_string(file_path);
                eval assert_eq(|_|"case 1", content, read_content);

                let read_lines = *with_file(file_path, "r", |file| (
                    pure $ [*read_line(file), *read_line(file)]
                ));
                eval assert_eq(|_|"case 2", read_lines.@(0), lines.@(0) + "\n");
                eval assert_eq(|_|"case 3", read_lines.@(1), lines.@(1));

                pure()
            }.try(exit_with_msg(1))
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
    remove_file("test_uAfQDPwJ7sS6.txt").unwrap();
}

#[test]
pub fn test_is_eof() {
    let source = r#"
        module Main; 
        import Debug;

        main : IO ();
        main = (
            let file_path = Path::parse("test_bUeW9baGGZmE.txt").as_some;
            let content = "Hello World!";
            do {
                eval *write_file_string(file_path, content);

                let read_content = *with_file(file_path, "r", |file| (
                    let content = *read_string(file);
                    let is_eof = *is_eof(file).lift;
                    eval assert(|_|"file had not reached to EOF!", is_eof);
                    pure $ content
                ));
            
                eval assert_eq(|_|"read_content != content", content, read_content);

                pure()
            }.try(exit_with_msg(1))
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
    remove_file("test_bUeW9baGGZmE.txt").unwrap();
}

#[test]
pub fn test108_5() {
    // Test write_file_bytes, read_file_bytes.
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let file_path = Path::parse("test_vgZNhmj4gPbF.dat").as_some;
            let data = Array::from_map(1024 + 512, |n| n.to_U8);
            do {
                eval *write_file_bytes(file_path, data);

                let read = *read_file_bytes(file_path);
                eval assert_eq(|_|"case 1", data, read);

                pure()
            }.try(exit_with_msg(1))
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
    remove_file("test_vgZNhmj4gPbF.dat").unwrap();
}

#[test]
pub fn test109() {
    // Test monad syntax.
    let source = r#"
        module Main; import Debug;

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

            eval assert_eq(|_|"case 1", add_opt_int(one, two), three);
            eval assert_eq(|_|"case 2", add_opt_int(none, two), none);
            eval assert_eq(|_|"case 3", add_opt_int(one, none), none);
            eval assert_eq(|_|"case 4", add_opt_int(none, none), none);

            let res0 = Result::ok(0) : Result String I64;
            let res1 = Result::ok(1);
            let res2 = Result::ok(2);
            let res3 = Result::ok(3);
            let res_iter = Iterator::from_array([res0, res1, res2, res3]).sequence;
            eval assert_eq(|_|"case 5", res_iter.is_ok, true);
            eval assert_eq(|_|"case 6", res_iter.as_ok, Iterator::from_array([0, 1, 2, 3]));

            let res0 = Result::ok(0) : Result String I64;
            let res1 = Result::ok(1);
            let res2 = Result::err("Error 2");
            let res3 = Result::err("Error 3");
            let res_iter = Iterator::from_array([res0, res1, res2, res3]).sequence;
            eval assert_eq(|_|"case 5", res_iter.is_err, true);
            eval assert_eq(|_|"case 6", res_iter.as_err, "Error 2");

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test110a() {
    // Test basic float operations, cast between floats, to_string, from_string, to_string_with_precision
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let x = -3.1415_F32;
            let y = 3.1415_F32;
            eval assert(|_|"case 1", x.abs == y);
            eval assert(|_|"case 2", y.abs == y);

            let x = -3.1415;
            let y = 3.1415;
            eval assert(|_|"case 3", x.abs == y);
            eval assert(|_|"case 4", y.abs == y);

            let x = 3.1415_F32;
            let y = 3.1415_F32;
            eval assert(|_|"case 5", x == y);

            let x = 3.1415;
            let y = 3.1415;
            eval assert(|_|"case 6", x == y);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            eval assert(|_|"case 7", x != y);

            let x = 3.1415;
            let y = 2.7183;
            eval assert(|_|"case 8", x != y);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 5.8598_F32;
            eval assert(|_|"case 9", (x + y - z).abs < 1.0e-4_F32);

            let x = 3.1415;
            let y = 2.7183;
            let z = 5.8598;
            eval assert(|_|"case 10", (x + y - z).abs < 1.0e-4);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 8.5395_F32;
            eval assert(|_|"case 11", (x * y - z).abs < 1.0e-4_F32);

            let x = 3.1415;
            let y = 2.7183;
            let z = 8.5395;
            eval assert(|_|"case 12", (x * y - z).abs < 1.0e-4);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            let z = 1.1557_F32;
            eval assert(|_|"case 13", (x / y - z).abs < 1.0e-4_F32);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test110b() {
    // Test basic float operations, cast between floats, to_string, from_string, to_string_with_precision
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let x = 3.1415;
            let y = 2.7183;
            let z = 1.1557;
            eval assert(|_|"case 14", (x / y - z).abs < 1.0e-4);

            let x = 3.1415_F32;
            let y = 2.7183_F32;
            eval assert(|_|"case 15", x > y);

            let x = 3.1415;
            let y = 2.7183;
            eval assert(|_|"case 16", x > y);

            let x = 3.1415_F32;
            let y = 3.1415_F32;
            eval assert(|_|"case 17", x >= y);

            let x = 3.1415;
            let y = 3.1415;
            eval assert(|_|"case 18", x >= y);

            let x = 3.1415_F32;
            let y = 3.1415_F32;
            eval assert(|_|"case 19.1", x.to_F32 == y);

            let x = 3.1415;
            let y = 3.1415;
            eval assert(|_|"case 19.1", x.to_F64 == y);

            let x = 3.1415_F32;
            let y = 3.1415;
            eval assert(|_|"case 19.3", (x.to_F64 - y) < 1.0e-4);

            let x = 3.1415;
            let y = 3.1415_F32;
            eval assert(|_|"case 19.4", (x.to_F32 - y) < 1.0e-4_F32);

            let x = 3141;
            let y = 3141.0;
            eval assert(|_|"case 20", x.to_F64 == y);

            let x = 3141.0;
            let y = 3141;            
            eval assert(|_|"case 21", x.to_I64 == y);

            let x = 3.14;
            eval assert_eq(|_|"case 22", x, x.to_string.from_string.as_ok);

            let x = 3.14_F32;
            eval assert_eq(|_|"case 23", x, x.to_string.from_string.as_ok);

            // Cases from_string fails.
            
            let res: Result ErrMsg F64 = "Hello World!".from_string;
            eval assert(|_|"Case: from_string invalid format", res.is_err);

            let res: Result ErrMsg F64 = " 3.14".from_string;
            eval assert(|_|"Case: from_string invalid format (whitespace)", res.is_err);

            let res: Result ErrMsg I64 = "9999999999999999999999999999999999999999999999999999".from_string;
            eval assert(|_|"Case: from_string out of range", res.is_err);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test111() {
    // Test function composition operators
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let f = |x| x + 2;
            let g = |x| 3*x + 4;

            let f_g = f << g;
            let g_f = f >> g;

            eval assert_eq(|_|"case 1", f_g(0), 6);
            eval assert_eq(|_|"case 2", g_f(0), 10);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test112() {
    // Test Iterator::generate
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let iter = Iterator::generate(0, |_| Option::none());
            let ans = [] : Array I64;
            eval assert_eq(|_|"case 1", iter.to_array, ans);

            let iter = Iterator::generate(0, |i| if i == 3 { Option::none() } else { Option::some $ (i, i+1) });
            let ans = [0, 1, 2];
            eval assert_eq(|_|"case 1", iter.to_array, ans);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test113() {
    // Test bit operations.
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            // Shift
            let x = 10_U8.shift_right(2_U8);
            eval assert_eq(|_|"case 1", x, 2_U8);

            let x = -10_I32.shift_right(2_I32);
            eval assert_eq(|_|"case 1", x, -3_I32);

            let x = 10_U8.shift_left(2_U8);
            eval assert_eq(|_|"case 1", x, 40_U8);

            // Xor, Or, And
            let x = 10.bit_xor(12);
            eval assert_eq(|_|"case 1", x, 6);

            let x = 10.bit_or(12);
            eval assert_eq(|_|"case 1", x, 14);

            let x = 10.bit_and(12);
            eval assert_eq(|_|"case 1", x, 8);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test114() {
    // Test Array::find_by
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let arr = [0,1,2,3];

            let res = arr.find_by(|x| x % 5 == 2);
            eval assert_eq(|_|"case 1", res, Option::some(2));

            let res = arr.find_by(|x| x % 5 == 4);
            eval assert_eq(|_|"case 1", res, Option::none());

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test115() {
    // Test HashMap
    let source = r#"
        module Main; 

        import Debug;
        import HashMap;

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
        
            eval assert_eq(|_|"case 0", mp.find(0), Option::some(0));
            eval assert_eq(|_|"case 1", mp.find(1), Option::some(1));
            eval assert_eq(|_|"case 2", mp.find(2), Option::some(2));
            eval assert_eq(|_|"case 3", mp.find(3), Option::some(3));
            eval assert_eq(|_|"case 4", mp.find(4), Option::none());
            eval assert_eq(|_|"case 5", mp.find(5), Option::none());
            eval assert_eq(|_|"case 6", mp.find(6), Option::none());
            eval assert_eq(|_|"case 7", mp.find(7), Option::some(7));

            eval assert_eq(|_|"case 0.5", mp.contains_key(0), true);
            eval assert_eq(|_|"case 1.5", mp.contains_key(1), true);
            eval assert_eq(|_|"case 2.5", mp.contains_key(2), true);
            eval assert_eq(|_|"case 3.5", mp.contains_key(3), true);
            eval assert_eq(|_|"case 4.5", mp.contains_key(4), false);
            eval assert_eq(|_|"case 5.5", mp.contains_key(5), false);
            eval assert_eq(|_|"case 6.5", mp.contains_key(6), false);
            eval assert_eq(|_|"case 7.5", mp.contains_key(7), true);

            eval assert_eq(|_|"case size", mp.get_size, 5);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test115_5() {
    // Test HashSet
    let source = r#"
        module Main; 

        import Debug;
        import HashSet;

        main : IO ();
        main = (
            let set = HashSet::empty(0)
                            .reserve(3)
                            .insert(0)
                            .insert(1)
                            .insert(2)
                            .erase(2).insert(2)
                            .insert(3)
                            .insert(4).erase(4)
                            .erase(5)
                            // Do nothing for 6
                            .insert(7).insert(7);
    
            eval assert_eq(|_|"case A-0", set.contains(0), true);
            eval assert_eq(|_|"case A-1", set.contains(1), true);
            eval assert_eq(|_|"case A-2", set.contains(2), true);
            eval assert_eq(|_|"case A-3", set.contains(3), true);
            eval assert_eq(|_|"case A-4", set.contains(4), false);
            eval assert_eq(|_|"case A-5", set.contains(5), false);
            eval assert_eq(|_|"case A-6", set.contains(6), false);
            eval assert_eq(|_|"case A-7", set.contains(7), true);

            eval assert_eq(|_|"case B", set.get_size, 5);
            
            let set = HashSet::from_iter([1, 1, 2, 3].to_iter);

            eval assert_eq(|_|"case C-0", set.contains(0), false);
            eval assert_eq(|_|"case c-1", set.contains(1), true);
            eval assert_eq(|_|"case C-2", set.contains(2), true);
            eval assert_eq(|_|"case C-3", set.contains(3), true);
            eval assert_eq(|_|"case C-4", set.contains(4), false);

            eval assert_eq(|_|"case D", set.get_size, 3);

            let set: HashSet I64 = HashSet::from_iter([].to_iter);
            eval assert_eq(|_|"case E", set.get_size, 0);

            let set0 = HashSet::from_iter([1, 2, 3].to_iter);
            let set1 = HashSet::from_iter([3, 4, 5].to_iter);
            let set = set0.intersect(set1);
            eval assert_eq(|_|"case F", set.to_iter, [3].to_iter);

            let set0 = HashSet::from_iter([1, 2, 3].to_iter);
            let set1 = HashSet::from_iter([4, 5, 6].to_iter);
            let set = set0.intersect(set1);
            eval assert_eq(|_|"case G", set.to_iter, [].to_iter);

            let set0 = HashSet::from_iter([1, 2, 3].to_iter);
            let set1 = HashSet::from_iter([].to_iter);
            let set = set0.intersect(set1);
            eval assert_eq(|_|"case H", set.to_iter, [].to_iter);

            let set0: HashSet I64 = HashSet::from_iter([].to_iter);
            let set1 = HashSet::from_iter([].to_iter);
            let set = set0.intersect(set1);
            eval assert_eq(|_|"case I", set.to_iter, [].to_iter);

            let set0 = HashSet::from_iter([1, 2, 3].to_iter);
            let set1 = HashSet::from_iter([3, 4, 5].to_iter);
            let set = set0.merge(set1);
            eval assert_eq(|_|"case J", set.to_iter.to_array.sort_by(|(lhs, rhs)| lhs < rhs), [1, 2, 3, 4, 5]);

            let set0 = HashSet::from_iter([1, 2, 3].to_iter);
            let set1 = HashSet::from_iter([].to_iter);
            let set = set0.merge(set1);
            eval assert_eq(|_|"case K", set.to_iter.to_array.sort_by(|(lhs, rhs)| lhs < rhs), [1, 2, 3]);

            let set0: HashSet I64 = HashSet::from_iter([].to_iter);
            let set1 = HashSet::from_iter([].to_iter);
            let set = set0.merge(set1);
            eval assert_eq(|_|"case L", set.to_iter, [].to_iter);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test116() {
    // Test Std::Destructor
    let source = r#"
        module Main; import Debug;

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
            let dtor3 = Destructor { 
                value : 2, 
                dtor : |val| (
                    debug_println("dtor3 destructed. val: " + val.to_string)
                )
            };
            let dtor2 = Destructor { 
                value : dtor3, 
                dtor : |val| (
                    debug_println("dtor2 destructed. val.@value: " + val.@value.to_string)
                )
            };

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test117() {
    // Test String::from_c_str
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let str = String::_unsafe_from_c_str([65_U8, 66_U8, 67_U8, 0_U8, 0_U8]);
            eval assert_eq(|_|"case 1", str, "ABC");
            eval assert_eq(|_|"case 2", str.get_size, 3);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test118() {
    // Test fold_m
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval *count_up(0).take(10).fold_m(0, |s, i| (
                let s = s + i;
                eval *print("Sum upto " + i.to_string + " is " + s.to_string + ". ");
                pure $ s
            )).forget;
            eval *println("");
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test119() {
    // Test namespace and MakeStruct, Pattern.
    let source = r#"
        module Main; import Debug;

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
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test120() {
    // Test abort of type Array or function.
    let source = r#"
        module Main; 
        import Debug;

        main : IO ();
        main = (
            let x = 3;
            let a = if true { Array::fill(1, |_| x) } else { abort() };
            eval assert_eq(|_|"case 1", (a.@(0))(1), x);
            let a = if true { |_| x } else { abort() };
            eval assert_eq(|_|"case 1", a(1), x);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test121() {
    // Test Math module
    let source = r#"
        module Main; 

        import Debug;
        import Math;

        main : IO ();
        main = (
            // gcd
            eval assert_eq(|_|"case gcd-0", gcd(16, 6), 2);
            eval assert_eq(|_|"case gcd-1", gcd(544, 119), 17);
            eval assert_eq(|_|"case gcd-2", gcd(2089, 3571), 1);
            eval assert_eq(|_|"case gcd-3", gcd(-16, 6), 2);
            eval assert_eq(|_|"case gcd-4", gcd(544, -119), 17);
            eval assert_eq(|_|"case gcd-5", gcd(-2089, 3571), 1);
            eval assert_eq(|_|"case gcd-6", gcd(0, 0), 0);
            eval assert_eq(|_|"case gcd-7", gcd(0, 1), 1);
            eval assert_eq(|_|"case gcd-8", gcd(-1, 0), 1);

            // binomial_coefficients
            let binom = [
                [1],
                [1, 1],
                [1, 2, 1],
                [1, 3, 3, 1],
                [1, 4, 6, 4, 1],
                [1, 5, 10, 10, 5, 1],
                [1, 6, 15, 20, 15, 6, 1],
                [1, 7, 21, 35, 35, 21, 7, 1],
                [1, 8, 28, 56, 70, 56, 28, 8, 1],
                [1, 9, 36, 84, 126, 126, 84, 36, 9, 1],
                [1, 10, 45, 120, 210, 252, 210, 120, 45, 10, 1]
            ];
            eval assert_eq(|_|"case binomial_coefficients-0", binomial_coefficients(10), binom);

            let binom = [
                [1]
            ];
            eval assert_eq(|_|"case binomial_coefficients-1", binomial_coefficients(0), binom);

            let binom = [
                [1],
                [1, 1]
            ];
            eval assert_eq(|_|"case binomial_coefficients-2", binomial_coefficients(1), binom);

            // libm functions
            eval assert(|_|"case acos", (acos(0.2) - 1.369438406004566).abs < 1.0e-8);
            eval assert(|_|"case asin", (asin(0.2) - 0.2013579207903308).abs < 1.0e-8);
            eval assert(|_|"case atan", (atan(0.2) - 0.19739555984988078).abs < 1.0e-8);            
            eval assert(|_|"case atan2", (atan2(0.2, 0.5) - 0.3805063771123649).abs < 1.0e-8);
            eval assert(|_|"case ceil", (ceil(1.9) - 2.0).abs < 1.0e-8);
            eval assert(|_|"case cos", (cos(0.2) - 0.9800665778412416).abs < 1.0e-8);
            eval assert(|_|"case cosh", (cosh(0.2) - 1.020066755619076).abs < 1.0e-8);
            eval assert(|_|"case exp", (exp(0.2) - 1.2214027581601699).abs < 1.0e-8);
            eval assert(|_|"case floor", (floor(1.2) - 1.0).abs < 1.0e-8);
            eval assert(|_|"case fmod", (2.0.fmod(1.2) - 0.8).abs < 1.0e-8);
            eval assert(|_|"case frexp 1", (frexp(2560.0).@0 - 0.625).abs < 1.0e-8);
            eval assert(|_|"case frexp 2", (frexp(2560.0).@1 - 12_I32).abs == 0_I32);
            eval assert(|_|"case ldexp", (3.14.ldexp(2_I32) - 12.56).abs < 1.0e-8);
            eval assert(|_|"case log", (log(0.2) - -1.6094379124341003).abs < 1.0e-8);
            eval assert(|_|"case log10", (log10(0.2) - -0.6989700043360187).abs < 1.0e-8);
            eval assert(|_|"case modf 1", (modf(3.14).@0 - 0.14).abs < 1.0e-8);
            eval assert(|_|"case modf 2", (modf(3.14).@1 - 3.0).abs < 1.0e-8);
            eval assert(|_|"case pow", (3.14.pow(2.72) - 22.472357891492628).abs < 1.0e-8);
            eval assert(|_|"case sin", (sin(0.2) - 0.19866933079506122).abs < 1.0e-8);
            eval assert(|_|"case sinh", (sinh(0.2) - 0.20133600254109402).abs < 1.0e-8);
            eval assert(|_|"case sqrt", (sqrt(0.2) - 0.4472135954999579).abs < 1.0e-8);
            eval assert(|_|"case tan", (tan(0.2) - 0.2027100355086725).abs < 1.0e-8);
            eval assert(|_|"case tanh", (tanh(0.2) - 0.197375320224904).abs < 1.0e-8);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test122() {
    // Test PunchedArray.
    let source = r#"
        module Main; 

        import Debug;

        type MyBoxed = box struct { x : I64 };

        main : IO ();
        main = (
            // Case 1-1: Punch an array of two boxed values and release parray.
            let arr = [MyBoxed { x : 5 }, MyBoxed { x : 7 }];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 1-1", five.@x, 5);

            // Case 1-2: Punch an array of two boxed values and plug-in the same element.
            let arr = [MyBoxed { x : 5 }, MyBoxed { x : 7 }];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 1-2-a", five.@x, 5);
            let arr = parr.plug_in!(five);
            eval assert_eq(|_|"case 1-2-b", arr.@(0).@x + arr.@(1).@x, 5 + 7);

            // Case 1-3: Punch an array of two boxed values and plug-in the other element.
            let seven = MyBoxed { x : 7 };
            let arr = [MyBoxed { x : 5 }, seven];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 1-3-a", five.@x, 5);
            let arr = parr.plug_in!(seven);
            eval assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 7);

            // Case 1-4: Punch an array of two boxed values and plug-in another value.
            let arr = [MyBoxed { x : 5 }, MyBoxed { x : 7 }];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 1-3-a", five.@x, 5);
            let arr = parr.plug_in!(MyBoxed { x : 11 });
            eval assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 11);

            // Case 2-1: Punch an array of two shared boxed values and release parray.
            let five = MyBoxed { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 2-1", five.@x, 5);

            // Case 2-2: Punch an array of two shared boxed values and plug-in the same element.
            let five = MyBoxed { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 2-2-a", five.@x, 5);
            let arr = parr.plug_in!(five);
            eval assert_eq(|_|"case 2-2-b", arr.@(0).@x + arr.@(1).@x, 5 + 5);

            // Case 2-3: Punch an array of two shared boxed values and plug-in the value again.
            let five = MyBoxed { x : 5 };
            let arr = [five, five];
            let (parr, five1) = arr.punch!(0);
            eval assert_eq(|_|"case 2-3-a", five1.@x, 5);
            let arr = parr.plug_in!(five);
            eval assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 5 + 5);

            // Case 2-4: Punch an array of two shared boxed values and plug-in another value.
            let five = MyBoxed { x : 5 };
            let arr = [five, five];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 2-3-a", five.@x, 5);
            let arr = parr.plug_in!(MyBoxed { x : 7 });
            eval assert_eq(|_|"case 1-3-b", arr.@(0).@x + arr.@(1).@x, 7 + 5);

            // Case 3-1: Punch an array of one boxed values and release parray.
            let arr = [MyBoxed { x : 5 }];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 3-1", five.@x, 5);

            // Case 3-2: Punch an array of two boxed values and plug-in the same element.
            let arr = [MyBoxed { x : 5 }];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 3-2-a", five.@x, 5);
            let arr = parr.plug_in!(five);
            eval assert_eq(|_|"case 3-2-b", arr.@(0).@x, 5);

            // Case 4-1: Punch an array of two unboxed values and release parray.
            let arr = [5, 7];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 1-1", five, 5);

            // Case 4-2: Punch an array of two boxed values and plug-in a value.
            let arr = [5, 7];
            let (parr, five) = arr.punch!(0);
            eval assert_eq(|_|"case 4-2-a", five, 5);
            let arr = parr.plug_in!(13);
            eval assert_eq(|_|"case 4-2-b", arr.@(0) + arr.@(1), 13 + 7);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test123() {
    // Test Array::act and Array::act!
    let source = r#"
        module Main; 

        import Debug;

        type MyBoxed = box struct { x : I64 };

        main : IO ();
        main = (
            let act0: MyBoxed -> Option MyBoxed = |v| (
                if v.@x == 0 { Option::some $ v.mod_x!(add(5)) } else { Option::none() }
            );
            let act01: MyBoxed -> Option MyBoxed = |v| (
                if v.@x == 0 { Option::some $ v.mod_x(add(5)) } else { Option::none() }
            );
            let act1: MyBoxed -> Option MyBoxed = |v| (
                if v.@x == 0 { Option::some $ MyBoxed { x : 5 } } else { Option::none() }
            );

            // Case 0-0-0-0: Boxed element, unique array, act0 succeeds.
            let case = "0-0-0-0";
            let arr = [MyBoxed { x : 0 }, MyBoxed { x : 3 }];
            let opt_arr = arr.act!(0, act0);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_some);
            eval assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);
            eval assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0).@x, 5);
            eval assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1).@x, 3);

            // Case 0-0-0-1: Boxed element, unique array, act0 fails.
            let case = "0-0-0-1";
            let arr = [MyBoxed { x : 1 }, MyBoxed { x : 3 }];
            let opt_arr = arr.act!(0, act0);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_none);

            // Case 0-0-1-0: Boxed element, unique array, act1 succeeds.
            let case = "0-0-1-0";
            let arr = [MyBoxed { x : 0 }, MyBoxed { x : 3 }];
            let opt_arr = arr.act!(0, act1);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_some);
            eval assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);
            eval assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0).@x, 5);
            eval assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1).@x, 3);

            // Case 0-0-1-1: Boxed element, unique array, act1 fails.
            let case = "0-0-1-1";
            let arr = [MyBoxed { x : 1 }, MyBoxed { x : 3 }];
            let opt_arr = arr.act!(0, act1);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_none);

            // Case 0-1-0-0: Boxed element, shared array, act01 succeeds.
            let case = "0-1-0-0";
            let arr = [MyBoxed { x : 0 }, MyBoxed { x : 3 }];
            let opt_arr = arr.act(0, act01);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_some);
            eval assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);
            eval assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0).@x, 5);
            eval assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1).@x, 3);
            eval assert_eq(|_|"Case " + case + "-e", arr.@(0).@x + arr.@(1).@x, 3);

            // Case 0-1-0-1: Boxed element, shared array, act0 fails.
            let case = "0-1-0-1";
            let arr = [MyBoxed { x : 1 }, MyBoxed { x : 3 }];
            let opt_arr = arr.act(0, act0);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_none);
            eval assert_eq(|_|"Case " + case + "-e", arr.@(0).@x + arr.@(1).@x, 4);

            // Case 0-1-1-0: Boxed element, shared array, act1 succeeds.
            let case = "0-1-1-0";
            let arr = [MyBoxed { x : 0 }, MyBoxed { x : 3 }];
            let opt_arr = arr.act(0, act1);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_some);
            eval assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);
            eval assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0).@x, 5);
            eval assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1).@x, 3);
            eval assert_eq(|_|"Case " + case + "-e", arr.@(0).@x + arr.@(1).@x, 3);

            // Case 0-1-1-1: Boxed element, shared array, act1 fails.
            let case = "0-1-1-1";
            let arr = [MyBoxed { x : 1 }, MyBoxed { x : 3 }];
            let opt_arr = arr.act(0, act1);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_none);
            eval assert_eq(|_|"Case " + case + "-e", arr.@(0).@x + arr.@(1).@x, 4);

            let act2: I64 -> Option I64 = |v| (
                if v == 0 { Option::some $ v + 5 } else { Option::none() }
            );

            // Case 1-0-0-0: Unboxed element, unique array, act2 succeeds.
            let case = "1-0-0-0";
            let arr = [0, 3];
            let opt_arr = arr.act!(0, act2);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_some);
            eval assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);
            eval assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0), 5);
            eval assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1), 3);

            // Case 1-0-0-1: Unboxed element, unique array, act2 fails.
            let case = "1-0-0-1";
            let arr = [1, 3];
            let opt_arr = arr.act!(0, act2);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_none);

            // Case 1-1-0-0: Unboxed element, shared array, act2 succeeds.
            let case = "1-1-0-0";
            let arr = [0, 3];
            let opt_arr = arr.act(0, act2);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_some);
            eval assert_eq(|_|"Case " + case + "-b", opt_arr.as_some.get_size, 2);
            eval assert_eq(|_|"Case " + case + "-c", opt_arr.as_some.@(0), 5);
            eval assert_eq(|_|"Case " + case + "-d", opt_arr.as_some.@(1), 3);
            eval assert_eq(|_|"Case " + case + "-e", arr.@(0) + arr.@(1), 3);

            // Case 1-1-0-1: Unboxed element, shared array, act2 fails.
            let case = "1-1-0-1";
            let arr = [1, 3];
            let opt_arr = arr.act(0, act2);
            eval assert(|_|"Case " + case + "-a", opt_arr.is_none);
            eval assert_eq(|_|"Case " + case + "-e", arr.@(0) + arr.@(1), 4);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test123_5() {
    // Test Array::act and Array::act! (case 2)
    let source = r#"
        module Main; 

        import Debug;

        main : IO ();
        main = (
            let act2: I64 -> Option I64 = |v| (
                if v == 0 { Option::some $ v + 5 } else { Option::none() }
            );

            let case = "2-0-0";
            let arr = [[1, 2, 3], [4, 0, 6], [7, 8, 9]];
            let opt_arr = arr.act!(1, act!(1, act2));
            eval assert(|_|"Case " + case + "-a", opt_arr.is_some);
            eval assert_eq(|_|"Case " + case + "-b", opt_arr.as_some, [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);

            // Case 2-0-1: Fails updating an element of unique two-dimensional array by act2.
            let case = "2-0-1";
            let arr = [[1, 2, 3], [4, 1, 6], [7, 8, 9]];
            let opt_arr = arr.act!(1, act!(1, act2));
            eval assert(|_|"Case " + case + "-a", opt_arr.is_none);

            // Case 2-1-0: Succeeds updating an element of shared two-dimensional array by act2.
            let case = "2-1-0";
            let arr = [[1, 2, 3], [4, 0, 6], [7, 8, 9]];
            let opt_arr = arr.act(1, act(1, act2));
            eval assert(|_|"Case " + case + "-a", opt_arr.is_some);
            eval assert_eq(|_|"Case " + case + "-b", opt_arr.as_some, [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
            eval assert_eq(|_|"Case " + case + "-c", arr, [[1, 2, 3], [4, 0, 6], [7, 8, 9]]);

            // Case 2-1-1: Fails updating an element of shared two-dimensional array by act2.
            let case = "2-1-1";
            let arr = [[1, 2, 3], [4, 1, 6], [7, 8, 9]];
            let opt_arr = arr.act(1, act(1, act2));
            eval assert(|_|"Case " + case + "-a", opt_arr.is_none);
            eval assert_eq(|_|"Case " + case + "-c", arr, [[1, 2, 3], [4, 1, 6], [7, 8, 9]]);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test124() {
    // Test Array : Functor, Array : Monad
    let source = r#"
        module Main; 

        import Debug;

        main : IO ();
        main = (
            // flatten
            eval assert_eq(|_|"case 1", [[1,2,3], [], [4, 5, 6]].flatten, [1, 2, 3, 4, 5, 6]);
            eval assert_eq(|_|"case 2", [[]].flatten, []: Array I64);
            eval assert_eq(|_|"case 3", [].flatten, []: Array I64);

            // bind
            let arr = do {
                let x = *[1,2,3];
                let y = *['a','b','c'];
                pure $ (x, y)
            };
            eval assert_eq(|_|"case 4", arr, [(1, 'a'), (1, 'b'), (1, 'c'), (2, 'a'), (2, 'b'), (2, 'c'), (3, 'a'), (3, 'b'), (3, 'c')]);

            let arr = do {
                let x = *[1,2,3];
                [x, x]
            };
            eval assert_eq(|_|"case 5", arr, [1, 1, 2, 2, 3, 3]);

            let arr = do {
                let x = *[1,2,3];
                []
            };
            eval assert_eq(|_|"case 6", arr, [] : Array I64);

            let arr = do {
                let x = *[];
                [x]
            };
            eval assert_eq(|_|"case 7", arr, [] : Array I64);

            // map
            eval assert_eq(|_|"case 8", [1, 2, 3].map(|i| i*i), [1, 4, 9]);
            eval assert_eq(|_|"case 9", [].map(|i| i*i), [] : Array I64);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test125() {
    // Test () : Eq
    let source = r#"
        module Main; 

        import Debug;

        main : IO ();
        main = (
            let arr = [(), ()];
            eval assert_eq(|_|"", arr.@(0), arr.@(1));

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test126() {
    // Test Iterator::sum.
    let source = r#"
        module Main; 

        import Debug;

        main : IO ();
        main = (
            let n = 100;
            let v = Iterator::range(0, n+1).sum;
            eval assert_eq(|_|"", v, n*(n+1)/2);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test127() {
    // Test trait alias.
    // Basic example are Additive and Iterator::sum, which are tested in other tests.
    let source = r#"
        module Main; 
        import Debug;

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
            eval assert_eq(|_|"case 1", sum_vec.@x, 4);
            eval assert_eq(|_|"case 2", sum_vec.@y, 6);

            let opts = [Option::some(1), Option::some(2)].to_iter;
            let opt_sum = opts.my_msum;
            eval assert_eq(|_|"case 3", opt_sum.as_some, 1);

            let opts = [Option::none(), Option::some(2)].to_iter;
            let opt_sum = opts.my_msum;
            eval assert_eq(|_|"case 4", opt_sum.as_some, 2);

            let opts = [Option::none(), Option::none()].to_iter;
            let opt_sum : Option I64 = opts.my_msum;
            eval assert_eq(|_|"case 5", opt_sum.is_none, true);

            let opts = [].to_iter;
            let opt_sum : Option I64 = opts.my_msum;
            eval assert_eq(|_|"case 6", opt_sum.is_none, true);

            eval assert_eq(|_|"case 7", [1,2,3,4,5].my_sum, 1+2+3+4+5);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test128() {
    // Test type alias.
    let source = r#"
        module Main; 
        import Debug;

        // Test of higher kinded trait alias is covered by Std::Lazy.

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
            eval assert_eq(|_|"", "John".greet + " " + get_name(Person { name : "Smith" }), "My name is John Smith");

            // Type alias in type annotation.
            let names : Array Name = ["John Smith"];
            eval assert_eq(|_|"", names.@(0).MyToString::to_string, "John Smith");

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test129() {
    // Test ToBytes/FromBytes
    let source = r#"
        module Main; 
        import Debug;

        main : IO ();
        main = (
            // U8
            let case = "U8";
            let n = 1;
            let x = 127_U8;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            let y : Result ErrMsg U8 = Array::fill(n-1, 127_U8).from_bytes;
            eval assert(|_|case + " 2", y.is_err);

            // I8
            let case = "I8";
            let n = 1;
            let x = 127_U8;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            let y : Result ErrMsg I8 = Array::fill(n-1, 127_U8).from_bytes;
            eval assert(|_|case + " 2", y.is_err);

            // U16
            let case = "U16";
            let n = 2;
            let x = 65535_U16;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            // let y : Result ErrMsg U16 = Array::fill(n-1, 127_U8).from_bytes;
            // eval assert(|_|case + " 2", y.is_err);

            // I16
            // let case = "I16";
            // let n = 2;
            // let x = -32768_I16;
            // eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            // let y : Result ErrMsg I16 = Array::fill(n-1, 127_U8).from_bytes;
            // eval assert(|_|case + " 2", y.is_err);

            // U32
            let case = "U32";
            let n = 4;
            let x = 90123456_U32;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            let y : Result ErrMsg U32 = Array::fill(n-1, 127_U8).from_bytes;
            eval assert(|_|case + " 2", y.is_err);

            // I32
            let case = "I32";
            let n = 4;
            let x = -12345678_I32;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            let y : Result ErrMsg I32 = Array::fill(n-1, 127_U8).from_bytes;
            eval assert(|_|case + " 2", y.is_err);

            // U64
            let case = "U64";
            let n = 8;
            let x = 123456789012345678_U64;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            let y : Result ErrMsg U64 = Array::fill(n-1, 127_U8).from_bytes;
            eval assert(|_|case + " 2", y.is_err);

            // I64
            let case = "I64";
            let n = 8;
            let x = 123456789012345678_I64;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            let y : Result ErrMsg I64 = Array::fill(n-1, 127_U8).from_bytes;
            eval assert(|_|case + " 2", y.is_err);

            // F32
            let case = "F32";
            let n = 4;
            let x = 3.14_F32;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            let y : Result ErrMsg F32 = Array::fill(n-1, 127_U8).from_bytes;
            eval assert(|_|case + " 2", y.is_err);

            // F64
            let case = "F64";
            let n = 8;
            let x = 3.14_F64;
            eval assert_eq(|_|case + " 1", x, x.to_bytes.from_bytes.as_ok);
            let y : Result ErrMsg F64 = Array::fill(n-1, 127_U8).from_bytes;
            eval assert(|_|case + " 2", y.is_err);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test130() {
    // Test Time module
    let source = r#"
        module Main; 
        import Debug;
        import Time;

        weeks : Array String;
        weeks = ["Sun", "Mon", "Tue", "Wed", "Thr", "Fri", "Sat"];

        dt_to_string : DateTime -> String;
        dt_to_string = |dt| (
            dt.@year.to_string + "/" + dt.@month.to_string + "/" + dt.@day_in_month.to_string + " (" + weeks.@(dt.@day_in_week.to_I64) + ") " + 
            dt.@hour.to_string + ":" + dt.@min.to_string + ":" + 
            (dt.@sec.to_F64 + 1.0e-9 * dt.@nanosec.to_F64).to_string + 
            ", dst = " + 
            if dt.@is_dst.is_none { "none" } else { dt.@is_dst.as_some.to_string }
        );

        main : IO ();
        main = (
            let now = *get_now;
            eval *(println $ "now.sec = " + now.@sec.to_string + ", now.nanosec = " + now.@nanosec.to_string);
            let utc = now.to_utc.as_ok;
            eval *(println $ "UTC: " + dt_to_string(utc));
            let loc = *now.to_local.try(exit_with_msg(1));
            eval *(println $ "Loc: " + dt_to_string(loc));
            let now_from_utc = Time::from_utc(utc).as_ok;
            let now_from_loc = *Time::from_local(loc).try(exit_with_msg(1));
            eval assert(|_|"diff utc", (now.to_F64 - now_from_utc.to_F64).abs < 0.1);
            eval assert(|_|"diff loc", (now.to_F64 - now_from_loc.to_F64).abs < 0.1);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test131() {
    // Test Character module
    let source = r#"
        module Main; 
        import Debug;
        import Character;

        main : IO ();
        main = (
            // is_alnum
            eval assert_eq(|_|"is_alnum", is_alnum('A'), true);
            eval assert_eq(|_|"is_alnum", is_alnum('0'), true);
            eval assert_eq(|_|"is_alnum", is_alnum('+'), false);
        
            // is_alpha
            eval assert_eq(|_|"is_alpha", is_alpha('A'), true);
            eval assert_eq(|_|"is_alpha", is_alpha('0'), false);
            eval assert_eq(|_|"is_alpha", is_alpha('+'), false);
        
            // is_blank
            eval assert_eq(|_|"is_blank 1", is_blank('A'), false);
            eval assert_eq(|_|"is_blank 2", is_blank('0'), false);
            eval assert_eq(|_|"is_blank 3", is_blank(' '), true);
            eval assert_eq(|_|"is_blank 4", is_blank('\t'), true);
            eval assert_eq(|_|"is_blank 5", is_blank('\n'), false);
            eval assert_eq(|_|"is_blank 6", is_blank('\r'), false);
        
            // is_cntrl
            eval assert_eq(|_|"is_cntrl", is_cntrl('A'), false);
            eval assert_eq(|_|"is_cntrl", is_cntrl('0'), false);
            eval assert_eq(|_|"is_cntrl", is_cntrl('\0'), true);
            eval assert_eq(|_|"is_cntrl", is_cntrl('\t'), true);
            eval assert_eq(|_|"is_cntrl", is_cntrl('\n'), true);
            eval assert_eq(|_|"is_cntrl", is_cntrl('\r'), true);
        
            // is_digit
            eval assert_eq(|_|"is_digit", is_digit('0'), true);
            eval assert_eq(|_|"is_digit", is_digit('9'), true);
            eval assert_eq(|_|"is_digit", is_digit('+'), false);
        
            // is_graph
            eval assert_eq(|_|"is_graph", is_graph('A'), true);
            eval assert_eq(|_|"is_graph", is_graph('0'), true);
            eval assert_eq(|_|"is_graph", is_graph(' '), false);
            eval assert_eq(|_|"is_graph", is_graph('\n'), false);
            eval assert_eq(|_|"is_graph", is_graph('\t'), false);
            eval assert_eq(|_|"is_graph", is_graph('\0'), false);
        
            // is_lower
            eval assert_eq(|_|"is_lower", is_lower('a'), true);
            eval assert_eq(|_|"is_lower", is_lower('A'), false);
            eval assert_eq(|_|"is_lower", is_lower('+'), false);
        
            // is_print
            eval assert_eq(|_|"is_print", is_print('A'), true);
            eval assert_eq(|_|"is_print", is_print('0'), true);
            eval assert_eq(|_|"is_print", is_print(' '), true);
            eval assert_eq(|_|"is_print", is_print('\n'), false);
            eval assert_eq(|_|"is_print", is_print('\t'), false);
            eval assert_eq(|_|"is_print", is_print('\0'), false);
        
            // is_punct
            eval assert_eq(|_|"is_punct 1", is_punct('.'), true);
            eval assert_eq(|_|"is_punct 2", is_punct(','), true);
            eval assert_eq(|_|"is_punct 3", is_punct('+'), true);
            eval assert_eq(|_|"is_punct 4", is_punct('A'), false);
            eval assert_eq(|_|"is_punct 5", is_punct('0'), false);
        
            // is_space
            eval assert_eq(|_|"is_space", is_space(' '), true);
            eval assert_eq(|_|"is_space", is_space('\t'), true);
            eval assert_eq(|_|"is_space", is_space('\n'), true);
            eval assert_eq(|_|"is_space", is_space('\r'), true);
            eval assert_eq(|_|"is_space", is_space('A'), false);
            eval assert_eq(|_|"is_space", is_space('0'), false);
        
            // is_upper
            eval assert_eq(|_|"is_upper", is_upper('A'), true);
            eval assert_eq(|_|"is_upper", is_upper('a'), false);
            eval assert_eq(|_|"is_upper", is_upper('+'), false);
        
            // is_xdigit
            eval assert_eq(|_|"is_xdigit 1", is_xdigit('A'), true);
            eval assert_eq(|_|"is_xdigit 2", is_xdigit('a'), true);
            eval assert_eq(|_|"is_xdigit 3", is_xdigit('0'), true);
            eval assert_eq(|_|"is_xdigit 4", is_xdigit('x'), false);
            eval assert_eq(|_|"is_xdigit 5", is_xdigit('+'), false);

            // to_lower
            eval assert_eq(|_|"to_lower 1", to_lower('A'), 'a');
            eval assert_eq(|_|"to_lower 2", to_lower('a'), 'a');
            eval assert_eq(|_|"to_lower 3", to_lower('+'), '+');

            // to_upper
            eval assert_eq(|_|"to_upper 1", to_upper('A'), 'A');
            eval assert_eq(|_|"to_upper 2", to_upper('a'), 'A');
            eval assert_eq(|_|"to_upper 3", to_upper('+'), '+');

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test132() {
    // Test Debug module
    let source = r#"
        module Main; 
        import Debug;

        main : IO ();
        main = (
            let (r, t) = consumed_time_while_lazy(|_| (
                loop((0, 0), |(i, sum)| if i == 1000000000 { break $ sum } else { continue $ (i + 1, sum + i) })
            ));
            eval debug_println("loop time : " + t.to_string + ", sum : " + r.to_string);

            let (_, t) = *consumed_time_while_io(
                let file_path = Path::parse("test_tMB3iCfTeeES.txt").as_some;
                eval *write_file_string(file_path, "Hello World!").try(exit_with_msg(1));
                let read_content = *read_file_string(file_path).try(exit_with_msg(1));
                println $ read_content
            );
            eval debug_println("write/read/println time : " + t.to_string);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
    remove_file("test_tMB3iCfTeeES.txt").unwrap();
}

#[test]
pub fn test_signed_integral_abs() {
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            eval assert_eq(|_|"", -123_I8.abs, 123_I8);
            eval assert_eq(|_|"", 123_I8.abs, 123_I8);

            eval assert_eq(|_|"", -123_I16.abs, 123_I16);
            eval assert_eq(|_|"", 123_I16.abs, 123_I16);

            eval assert_eq(|_|"", -123_I32.abs, 123_I32);
            eval assert_eq(|_|"", 123_I32.abs, 123_I32);

            eval assert_eq(|_|"", -123.abs, 123);
            eval assert_eq(|_|"", 123.abs, 123);

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_float_to_string_precision() {
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let x = 3.14_F32;
            eval assert_eq(|_|"case to_string_precision F32 0", x.to_string_precision(0_U8), "3");
            eval assert_eq(|_|"case to_string_precision F32 255", x.to_string_precision(255_U8), "3.140000104904174804687500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

            let x = -3.14;
            eval assert_eq(|_|"case to_string_precision F64 0", x.to_string_precision(0_U8), "-3");
            eval assert_eq(|_|"case to_string_precision F64 255", x.to_string_precision(255_U8), "-3.140000000000000124344978758017532527446746826171875000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_float_to_string_exp() {
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let x = 123.45_F32;
            eval assert_eq(|_|"case to_string_exp F32", x.to_string_exp, "1.234500e+02");

            let x = -123.45_F64;
            eval assert_eq(|_|"case to_string_exp F64", x.to_string_exp, "-1.234500e+02");
        
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_float_to_string_exp_precision() {
    let source = r#"
        module Main; import Debug;

        main : IO ();
        main = (
            let x = 123.45_F32;
            eval assert_eq(|_|"", x.to_string_exp_precision(0_U8), "1e+02");
            eval assert_eq(|_|"", x.to_string_exp_precision(255_U8), "1.234499969482421875000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e+02");

            let x = -123.45_F64;
            eval assert_eq(|_|"", x.to_string_exp_precision(0_U8), "-1e+02");
            eval assert_eq(|_|"", x.to_string_exp_precision(255_U8), "-1.234500000000000028421709430404007434844970703125000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e+02");
        
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_string_unsafe_from_c_str_ptr() {
    let source = r#"
        module Main;
        import Debug;

        main : IO ();
        main = (
            let src = "Hello World!";
            let cpy = src.borrow_c_str(String::_unsafe_from_c_str_ptr);
            eval assert_eq(|_|"", src, cpy);
            pure()
        );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_subprocess_run_stream() {
    let source = r#"
    module Main;
    import Debug;
    import Subprocess;
    
    main : IO ();
    main = (
        eval *println("Run \"ls -l -r\".");
        let (_, exit_status) = *run_with_stream("ls", ["ls", "-l", "-r"], |(stdin, stdout, stderr)| (
            let output = *read_string(stdout); // Read standard output of the command.
            println(output).lift
        )).try(exit_with_msg(1));
        eval assert_eq(|_|"", exit_status.as_exit, 0_U8);
    
        eval *println("Run \"sed s/w/W/\" and write \"Hello world!\" to the standard input.");
        let (_, exit_status) = *run_with_stream("sed", ["/usr/bin/sed", "s/w/W/"], |(stdin, stdout, stderr)| (
            eval *write_string(stdin, "Hello world!");
            eval *close_file(stdin).lift; // Send EOF.
            let output = *read_string(stdout); // Read standard output of the command.
            println(output).lift
        )).try(exit_with_msg(1));
        eval assert_eq(|_|"", exit_status.as_exit, 0_U8);
     
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_loop_lines() {
    let source = r#"
    module Main;
    import Debug;

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
            eval *write_file_string(file_path, ["0", "1", "2", "X", "3", "4"].to_iter.join("\n"));
            eval assert_eq(|_|"", *sum_up_while(file_path), 0 + 1 + 2);

            eval *write_file_string(file_path, ["0", "1", "2", "3", "4"].to_iter.join("\n"));
            eval assert_eq(|_|"", *sum_up_while(file_path), 0 + 1 + 2 + 3 + 4);

            eval *write_file_string(file_path, [].to_iter.join("\n"));
            eval assert_eq(|_|"", *sum_up_while(file_path), 0);

            pure()
        }.try(exit_with_msg(1))
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
    remove_file("test_GndeZP399tLX.txt").unwrap();
}

#[test]
pub fn test_array_get_sub() {
    let source = r#"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        // Unboxed case
        let arr = [0, 1, 2, 3, 4];
        eval assert_eq(|_|"", arr.get_sub(2, 4), [2, 3]);
        eval assert_eq(|_|"", arr.get_sub(0, 0), []);
        eval assert_eq(|_|"", arr.get_sub(3, 1), [3, 4, 0]);
        eval assert_eq(|_|"", arr.get_sub(1, -1), [1, 2, 3]);
    
        let arr : Array I64 = [];
        eval assert_eq(|_|"", arr.get_sub(2, 4), []);
    
        // Boxed case
        let arr = [[0], [1], [2], [3], [4]];
        eval assert_eq(|_|"", arr.get_sub(2, 4), [[2], [3]]);
        eval assert_eq(|_|"", arr.get_sub(0, 0), []);
        eval assert_eq(|_|"", arr.get_sub(3, 1), [[3], [4], [0]]);
        eval assert_eq(|_|"", arr.get_sub(1, -1), [[1], [2], [3]]);
    
        let arr : Array (Array I64) = [];
        eval assert_eq(|_|"", arr.get_sub(2, 4), []);
    
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_string_get_sub() {
    let source = r#"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        let str = "Hello";
        eval assert_eq(|_|"", str.get_sub(2, 4), "ll");
        eval assert_eq(|_|"", str.get_sub(0, 0), "");
        eval assert_eq(|_|"", str.get_sub(3, 1), "loH");
        eval assert_eq(|_|"", str.get_sub(1, -1), "ell");
    
        eval assert_eq(|_|"", "".get_sub(2, 4), "");
    
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_string_strip_first_spaces() {
    let source = r#"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"", "".strip_first_spaces, "");
        eval assert_eq(|_|"", "Hello".strip_first_spaces, "Hello");
        eval assert_eq(|_|"", " Hello".strip_first_spaces, "Hello");
        eval assert_eq(|_|"", " \tHello".strip_first_spaces, "Hello");
        eval assert_eq(|_|"", " ".strip_first_spaces, "");
        eval assert_eq(|_|"", "  ".strip_first_spaces, "");
    
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_loop_lines_io() {
    let source = r#"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        let content1 = "Hello\nWorld!";
        let file1 = Path::parse("test_MsuHh3QEXKYN.txt").as_some;
        let file2 = Path::parse("test_9A5bu4U57xTd.txt").as_some;
        do {
            eval *write_file_string(file1, content1);

            eval *with_file(file1, "r", |file1| (
                with_file(file2, "w", |file2| (
                    loop_lines_io(file1, (), |_, line| (
                        continue_m $ *write_string(file2, line)
                    ))
                ))
            ));

            let content2 = *read_file_string(file2);

            eval assert_eq(|_|"", content2, content1);

            pure()
        }.try(exit_with_msg(1))
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
    remove_file("test_MsuHh3QEXKYN.txt").unwrap();
    remove_file("test_9A5bu4U57xTd.txt").unwrap();
}

#[test]
pub fn test_string_find() {
    let source = r#"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"1", "abcdef".find("ab", 0), Option::some(0));
        eval assert_eq(|_|"2", "abcdef".find("bc", 0), Option::some(1));
        eval assert_eq(|_|"3", "abcdef".find("ef", 0), Option::some(4));
        eval assert_eq(|_|"4", "abcdef".find("xyz", 0), Option::none());
        eval assert_eq(|_|"5", "abcdef".find("", 0), Option::some(0));
        eval assert_eq(|_|"6", "".find("xyz", 0), Option::none());
        eval assert_eq(|_|"7", "".find("", 0), Option::some(0));

        eval assert_eq(|_|"8", "abcdef".find("ab", 1), Option::none());
        eval assert_eq(|_|"9", "abcdef".find("bc", 1), Option::some(1));
        eval assert_eq(|_|"10", "abcdef".find("ef", 1), Option::some(4));
        eval assert_eq(|_|"11", "abcdef".find("xyz", 1), Option::none());
        eval assert_eq(|_|"12", "abcdef".find("", 1), Option::some(1));
        eval assert_eq(|_|"13", "".find("xyz", 1), Option::none());
        eval assert_eq(|_|"14", "".find("", 1), Option::some(0));

        eval assert_eq(|_|"15", "abcdef".find("ab", 7), Option::none());
        eval assert_eq(|_|"16", "abcdef".find("bc", 7), Option::none());
        eval assert_eq(|_|"17", "abcdef".find("ef", 7), Option::none());
        eval assert_eq(|_|"18", "abcdef".find("xyz", 7), Option::none());
        eval assert_eq(|_|"19", "abcdef".find("", 7), Option::some(6));
        eval assert_eq(|_|"20", "".find("xyz", 7), Option::none());
        eval assert_eq(|_|"21", "".find("", 7), Option::some(0));

        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_names_literal_prefix() {
    let source = r#"
    module Main;
    import Debug;

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

        eval assert_eq(|_|"", true_global_val + false_global_val + nullptr_global_val + true_local_num + false_local_num + nullptr_local_num, 42 + 42);

        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_string_split() {
    let source = r#"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"1", "--ab---cde----".split("--").to_array, ["", "ab", "-cde", "", ""]);
        eval assert_eq(|_|"2", "ab---cde----".split("--").to_array, ["ab", "-cde", "", ""]);
        eval assert_eq(|_|"3", "--ab---cde".split("--").to_array, ["", "ab", "-cde"]);
        eval assert_eq(|_|"3", "ab---cde".split("--").to_array, ["ab", "-cde"]);
        eval assert_eq(|_|"4", "--".split("--").to_array, ["", ""]);
        eval assert_eq(|_|"5", "".split("--").to_array, [""]);

        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_ptr_to_string() {
    let source = r#"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"", nullptr.add_offset(3134905646).to_string, "00000000badadd2e");
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_async_task_fib() {
    let source = r#"
    module Main;
    import Debug;
    import AsyncTask;

    fib_async : I64 -> I64;
    fib_async = |n| (
        if n <= 1 {
            let _ = AsyncTask::make(|_| n + 1); // A task which is not waited.
            AsyncTask::make(|_| n).get // A task which is waited soon.
        } else {
            let minus_one_task = AsyncTask::make(|_| n-1); // A task which is captured by another task.
            let minus_two_task = AsyncTask::make(|_| minus_one_task.get - 1); // A task which is captured by another task.
            let minus_three_task = AsyncTask::make(|_| minus_two_task.get - 1); // A task which is captured by another task but not waited.
            let one_task = AsyncTask::make(|_| let _ = minus_three_task; fib_async(minus_one_task.get));
            let two_task = AsyncTask::make(|_| let _ = minus_three_task; fib_async(minus_two_task.get));
            one_task.get + two_task.get
        }
    );

    main : IO ();
    main = (
        let x = fib_async(10);
        eval assert_eq(|_|"", x, 55);
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_async_shared_array() {
    // This test shares array between multiple threads, try to mutate it from one thread while read it from multiple threads simultaneously.
    let source = r#"
    module Main;
    import Debug;
    import AsyncTask;

    main : IO ();
    main = (
        let n = 100000;
        let arr = Iterator::range(0, n).to_array;
        let sum_task_0 = AsyncTask::make(|_| arr.to_iter.fold(0, Add::add));
        let sum_task_1 = AsyncTask::make(|_| arr.to_iter.reverse.fold(0, Add::add));
        let sum_task_2 = AsyncTask::make(|_| (
            loop((0, 0), |(i, sum)| 
                if i == arr.get_size { 
                    break $ sum
                } else {
                    continue $ (i + 1, sum + arr.@(i))
                }
            )
        ));
        let sum_task_3 = AsyncTask::make(|_| (
            let half = arr.get_size / 2;
            let sum = loop((0, 0), |(i, sum)| 
                if i == half { 
                    break $ sum
                } else {
                    continue $ (i + 1, sum + arr.@(i))
                }
            );
            let arr = loop((arr, 0), |(arr, i)| (
                if i == 1000 { 
                    break $ arr
                } else {
                    let arr = arr.push_back(i).push_back(-i);
                    continue $ (arr, i + 1)
                }
            ));
            loop((half, sum), |(i, sum)| 
                if i == arr.get_size { 
                    break $ sum
                } else {
                    continue $ (i + 1, sum + arr.@(i))
                }
            )
        ));
        let sum_task_4 = loop((0, 0), |(i, sum)| (
            if i == arr.get_size { break $ sum };
            continue $ (i + 1, sum + arr.@(i))
        ));
        let ans = n * (n - 1) / 2;
        eval assert_eq(|_|"", sum_task_0.get, ans);
        eval assert_eq(|_|"", sum_task_1.get, ans);
        eval assert_eq(|_|"", sum_task_2.get, ans);
        eval assert_eq(|_|"", sum_task_3.get, ans);
        eval assert_eq(|_|"", sum_task_4, ans);
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_async_task_array_result() {
    let source = r#"
    module Main;
    import Debug;
    import AsyncTask;

    main : IO ();
    main = (
        let n = 1000000;
        let task_0 = AsyncTask::make(|_| Iterator::range(0, n).to_array);
        let task_1 = AsyncTask::make(|_| Iterator::range(n, 2*n).to_array);
        let task_2 = AsyncTask::make(|_| Iterator::range(0, 2*n).to_array);
        eval assert_eq(|_|"", task_0.get.append(task_1.get), task_2.get);
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_async_task_io() {
    let source = r##"
    module Main;
    import AsyncTask;
    
    main : IO ();
    main = (
        eval *AsyncIOTask::make(println $ "Thread 1").forget;
        eval *AsyncIOTask::make(println $ "Thread 2").forget;
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_tarai() {
    let source = r#"
    module Main;
    import Debug;

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
        eval assert_eq(|_|"", n, 12);
        pure()
    );
    "#;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_number_of_processors() {
    let source = r##"
    module Main;
    import AsyncTask;

    main : IO ();
    main = ("Number of processors: " + AsyncTask::number_of_processors.to_string).println;
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_async_task_dedicated_thread() {
    let source = r##"
    module Main;
    import AsyncTask;

    main : IO ();
    main = (
        let num_threads = 2;
        Iterator::range(0, num_threads).fold_m((), |_, i| (
            eval *AsyncIOTask::make(println $ "Thread " + i.to_string).forget;
            pure()
        ))
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_mvar() {
    let source = r##"
    module Main;
    import AsyncTask;

    main : IO ();
    main = (
        let logger = *Var::make([]); // A mutable array of strings.

        // Launch multiple threads, and log in which order each thread is executed.
        let num_threads = number_of_processors * 2;
        eval *Iterator::range(0, num_threads).fold_m((), |_, i| (
            eval *AsyncIOTask::make(
                logger.lock(|logs| (
                    let count = logs.get_size;
                    let msg = "Thread " + i.to_string + " is running at " + count.to_string + 
                        if count % 10 == 1 { "st" } else if count % 10 == 2 { "nd" } else if count % 10 == 3 { "rd" } else { "th" };
                    let msg = msg + if i == count { "." } else { "!" };
                    logger.set(logs.push_back(msg))
                ))
            ).forget;
            pure()
        ));

        // Wait until all threads are finished.
        eval *logger.wait(|logs| logs.get_size == num_threads);

        println $ (*logger.get).to_iter.join("\n")
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_mvar_of_shared_object() {
    // Share an array between multiple threads using Var.
    // This test is to check whether the refcnt of the array is not corrupted.
    let source = r##"
    module Main;
    import Debug;
    import AsyncTask;

    main : IO ();
    main = (
        let n = 100000;
        let var = *Var::make([]);
        let th0 = *AsyncIOTask::make((
            eval *(println $ "Thread is running."); // This line makes the `arr` is created on `th0` and not in the main thread.
            let arr = Iterator::range(0, n).to_array;
            eval *var.Var::set(arr);
            // NOTE: we need `Var::` in the above line, because:
            // - When the compiler try to infer namespace of `set`, it knows that it has type `set : a -> Var (Array b) -> c`.
            // - `Array::set` has type `I64 -> d -> Array d -> Array d`, which is unifiable to `a -> Var (Array b) -> c` 
            //   by `a = I64`, `d = Var (Array b)` and `c = Array d -> Array d`.
            pure $ arr.to_iter.fold(0, Add::add)
        ));
        let th1 = *AsyncIOTask::make((
            let arr = *var.wait_and_lock(|arr| !arr.is_empty, |arr| pure $ arr);
            pure $ arr.to_iter.fold(0, Add::add)
        ));
        eval assert_eq(|_|"", th0.get, n * (n - 1) / 2);
        eval assert_eq(|_|"", th1.get, n * (n - 1) / 2);
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
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
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_regression_issue_14() {
    let source = r##"
    module Main;
    
    main: IO ();
    main = (
        let _ = *(eprintln ("started"));
        let str = "abc";
        let buf = Array::fill(256, 0_U8);
        let res = str.borrow_c_str(|p_str|
            buf.borrow_ptr(|p_ret|
                0_I32
            )
        );
        pure()
    );
    "##;
    let mut config = Configuration::develop_compiler();
    config.set_threaded();
    config.sanitize_memory = false;
    run_source(&source, config);
}

#[test]
pub fn test_random() {
    let source = r##"
    module Main;
    import Random;
    
    main : IO ();
    main = (
        let init = [/* 0x12345 = */ 74565_U64, /* 0x23456 = */ 144470_U64, /* 0x34567 = */ 214375_U64, /* 0x45678 = */ 284280_U64];
        let random = init_by_array(init);
        let n = 100;
        eval *println(n.to_string + " outputs of generate_U64()");
        let random : Random = *loop_m (
            (random, 0), |(random, i)|
            if i >= n {
                break_m $ random
            };
            let (x, random) = generate_U64(random);
            eval *print(x.to_string + " ");
            eval *(if (i%5==4) { println("") } else { pure() });
            continue_m $ (random, i + 1)
        );
        eval *println("");
        eval *println(n.to_string + " outputs of generate_F64_2()");
        let random : Random = *loop_m (
            (random, 0), |(random, i)|
            if i >= n {
                break_m $ random
            };
            let (x, random) = generate_F64_2(random);
            eval *print (x.to_string + " ");
            eval *(if (i%5==4) { println("") } else { pure() });
            continue_m $ (random, i + 1)
        );
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_float_inf_nan() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"", F32::infinity.to_string, "inf");

        eval assert_eq(|_|"", F64::infinity.to_string, "inf");

        eval assert_eq(|_|"", (-F32::infinity).to_string, "-inf");

        eval assert_eq(|_|"", (-F64::infinity).to_string, "-inf");

        eval assert_eq(|_|"", F32::quiet_nan.to_bytes, [255_U8, 255_U8, 255_U8, 127_U8]);

        eval assert_eq(|_|"", F64::quiet_nan.to_bytes, [255_U8, 255_U8, 255_U8, 255_U8, 255_U8, 255_U8, 255_U8, 127_U8]);

        eval assert_eq(|_|"", F32::quiet_nan.to_string, "nan");

        eval assert_eq(|_|"", F64::quiet_nan.to_string, "nan");

        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_large_tuple() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        let x = (1,2,3,4,5,6,7,8,9,10);
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
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
        let mut config = Configuration::develop_compiler();
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
        let display = path.display();
        println!("[{}]:", display);
        run_file(config);
        remove_file("test_process_text_file.txt").unwrap_or(());
    }
}

#[test]
pub fn test_comment_0() {
    // block comment
    let source = r"/* head */ module Main; import Debug; 
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
pub fn test_comment_1() {
    // ilne comment
    let source = r"
        module Main; import Debug; //// /* */
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

#[test]
pub fn test_hex_oct_bin_lit() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"", 0x0, 0);
        eval assert_eq(|_|"", 0o0, 0);
        eval assert_eq(|_|"", 0b0, 0);
        eval assert_eq(|_|"", -0x0, 0);
        eval assert_eq(|_|"", -0o0, 0);
        eval assert_eq(|_|"", -0b0, 0);
        eval assert_eq(|_|"", 0x0123456789abcdef, 81985529216486895);
        eval assert_eq(|_|"", 0x0123456789ABCDEF, 81985529216486895);
        eval assert_eq(|_|"", 0o01234567, 342391);
        eval assert_eq(|_|"", 0b01, 1);
        eval assert_eq(|_|"", -0x0123456789abcdef, -81985529216486895);
        eval assert_eq(|_|"", -0x0123456789ABCDEF, -81985529216486895);
        eval assert_eq(|_|"", -0o01234567, -342391);
        eval assert_eq(|_|"", -0b01, -1);
        eval assert_eq(|_|"", 0xdeadbeef, 3735928559);
        eval assert_eq(|_|"", 0o33653337357, 3735928559);
        eval assert_eq(|_|"", 0b11011110101011011011111011101111, 3735928559);
        eval assert_eq(|_|"", 0x7FFFFFFFFFFFFFFF, 9223372036854775807);
        eval assert_eq(|_|"", -0x8000000000000000, -9223372036854775808);
        eval assert_eq(|_|"", 0o0777777777777777777777, 9223372036854775807);
        eval assert_eq(|_|"", -0o1000000000000000000000, -9223372036854775808);
        eval assert_eq(|_|"", 0b0111111111111111111111111111111111111111111111111111111111111111, 9223372036854775807);
        eval assert_eq(|_|"", -0b1000000000000000000000000000000000000000000000000000000000000000, -9223372036854775808);
        eval assert_eq(|_|"", 0xFFFFFFFFFFFFFFFF_U64, 18446744073709551615_U64);
        eval assert_eq(|_|"", 0o1777777777777777777777_U64, 18446744073709551615_U64);
        eval assert_eq(|_|"", 0b1111111111111111111111111111111111111111111111111111111111111111_U64, 18446744073709551615_U64);
        eval assert_eq(|_|"", 0x7FFFFFFF_I32, 2147483647_I32);
        eval assert_eq(|_|"", -0x80000000_I32, -2147483648_I32);
        eval assert_eq(|_|"", 0xFFFFFFFF_U32, 4294967295_U32);
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_regexp() {
    let source = r##"
    module Main;
    import Debug;
    import RegExp;
    
    main : IO ();
    main = (
        let regexp = RegExp::compile("[a-z]+([0-9]+)", "").as_ok;
        let groups = regexp.match("abc012 def345").as_ok;
        eval assert_eq(|_|"", groups, ["abc012", "012"]);

        let regexp = RegExp::compile("[a-z]+([0-9]+)", "g").as_ok;
        let groups = regexp.match("abc012 def345").as_ok;
        eval assert_eq(|_|"", groups, ["abc012", "def345"]);

        let regexp = RegExp::compile("(\\w\\w)(\\w)", "").as_ok;
        let result = regexp.replace_all("abc def ijk", "$2$1");
        eval assert_eq(|_|"", result, "cab fde kij");

        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_array_to_string() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"", ([] : Array Bool).to_string, "[]");
        eval assert_eq(|_|"", [1, 2, 3].to_string, "[1, 2, 3]");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_option_to_string() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"", (Option::none() : Option Bool).to_string, "none()");
        eval assert_eq(|_|"", (Option::some(42) : Option I64).to_string, "some(42)");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_unit_tuple_to_string() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"", ().to_string, "()");
        eval assert_eq(|_|"", (42, true).to_string, "(42, true)");
        eval assert_eq(|_|"", (42, true, "truth").to_string, "(42, true, truth)");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_unit_tuple_eq() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert(|_|"", () == ());
        eval assert(|_|"", (42, true) == (42, true));
        eval assert(|_|"", (0, true) != (42, true));
        eval assert(|_|"", (42, false) != (42, true));

        eval assert(|_|"", (42, true, "truth") == (42, true, "truth"));
        eval assert(|_|"", (0, true, "truth") != (42, true, "truth"));
        eval assert(|_|"", (42, false, "truth") != (42, true, "truth"));
        eval assert(|_|"", (42, false, "falsy") != (42, true, "truth"));

        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_tuple_less_than_and_less_than_or_eq() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        eval assert_eq(|_|"", (1, 2) < (2, 1), true);
        eval assert_eq(|_|"", (2, 1) < (1, 2), false);
        eval assert_eq(|_|"", (1, 2) < (1, 1), false);
        eval assert_eq(|_|"", (1, 1) < (1, 2), true);
        eval assert_eq(|_|"", (1, 1) < (1, 1), false);

        eval assert_eq(|_|"", (1, 2) <= (2, 1), true);
        eval assert_eq(|_|"", (2, 1) <= (1, 2), false);
        eval assert_eq(|_|"", (1, 2) <= (1, 1), false);
        eval assert_eq(|_|"", (1, 1) <= (1, 2), true);
        eval assert_eq(|_|"", (1, 1) <= (1, 1), true);

        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_result_to_string() {
    let source = r##"
    module Main;
    import Debug;
    
    main : IO ();
    main = (
        let res : Result String Bool = Result::ok(true);
        eval assert_eq(|_|"", res.to_string, "ok(true)");
        let res : Result String Bool = Result::err("error");
        eval assert_eq(|_|"", res.to_string, "err(error)");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_result_eq() {
    let source = r##"
    module Main;
    import Debug;
    
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
        eval *indices.loop_iter_m((), |_, (i, j)| (
            eval if i == j {
                assert_eq(|_|"", ress.@(i) == ress.@(j), true)
            } else {
                assert_eq(|_|"", ress.@(i) == ress.@(j), false)
            };
            continue_m $ ()
        ));

        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_array_less_than_and_less_than_or_eq() {
    let source = r##"
    module Main;
    import Debug;
    
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
        eval *indices.loop_iter_m((), |_, (i, j)| (
            eval assert_eq(|_|"", arrs.@(i) < arrs.@(j), i < j);
            eval assert_eq(|_|"", arrs.@(i) <= arrs.@(j), i <= j);
            continue_m $ ()
        ));

        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_string_less_than_and_less_than_or_eq() {
    let source = r##"
    module Main;
    import Debug;
    
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
        eval *indices.loop_iter_m((), |_, (i, j)| (
            eval assert_eq(|_|"", ss.@(i) < ss.@(j), i < j);
            eval assert_eq(|_|"", ss.@(i) <= ss.@(j), i <= j);
            continue_m $ ()
        ));

        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_overlapping_trait_and_function() {
    let source = r##"
    module Main;
    import Debug;
    
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
        eval assert_eq(|_|"", Main::show(42), "(function) 42");
        eval assert_eq(|_|"", Show::show(42), "(trait) 42");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
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
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
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
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
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
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_implement_trait_on_arrow_1() {
    let source = r##"
    module Main;
    import Debug;

    trait a : MyToString {
        to_string : a -> String;
    }

    impl [b : ToString] I64 -> b : MyToString {
        to_string = |f| "f(0) = " + f(0).ToString::to_string + ", f(1) = " + f(1).ToString::to_string;
    }
    
    main : IO ();
    main = (
        eval assert_eq(|_|"fail", (|x| x + 1).to_string, "f(0) = 1, f(1) = 2");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
pub fn test_implement_trait_on_arrow_2() {
    let source = r##"
    module Main;
    import Debug;

    trait a : MyToString {
        to_string : a -> String;
    }

    impl a -> b : MyToString {
        to_string = |f| "arrow";
    }
    
    main : IO ();
    main = (
        eval assert_eq(|_|"fail", (|x| x + 1).to_string, "arrow");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
pub fn test_overlapping_instances_1() {
    let source = r##"
    module Main;
    import Debug;

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
        eval assert_eq(|_|"fail", [1,2,3].to_string, "array");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
pub fn test_overlapping_instances_2() {
    let source = r##"
    module Main;
    import Debug;

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
        eval assert_eq(|_|"fail", [1,2,3].to_string, "array");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
pub fn test_overlapping_instances_3() {
    let source = r##"
    module Main;
    import Debug;

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
        eval assert_eq(|_|"fail", [1,2,3].to_string, "array");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
pub fn test_overlapping_instances_4() {
    let source = r##"
    module Main;
    import Debug;

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
        eval assert_eq(|_|"fail", Result::ok(1), "result");
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
pub fn test_eval_non_unit() {
    let source = r##"
    module Main;
    
    main : IO ();
    main = (
        eval 1;
        pure()
    );
    "##;
    run_source(&source, Configuration::develop_compiler());
}

#[test]
#[should_panic]
pub fn test_unrelated_trait_method() {
    let source = r##"
    module Main;

    trait a : MyTrait {
        value : I64;
    }
    
    main : IO ();
    main = pure();
    "##;
    run_source(&source, Configuration::develop_compiler());
}
