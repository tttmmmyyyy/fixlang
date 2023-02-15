use std::fs;

use super::*;

// Tests should run sequentially, since OBJECT_TABLE in libfixsanitizer.so is shared between tests and check_leak() asserts OBJECT_TABLE is empty.
#[test]
#[serial]
pub fn test0() {
    let source = r#"
            module Main;
    
            main : IOState -> ((), IOState);
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
            
            main : IOState -> ((), IOState);
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
            main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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

        main : IOState -> ((), IOState);
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
        
        main : IOState -> ((), IOState);
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

        main : IOState -> ((), IOState);
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

        main : IOState -> ((), IOState);
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

        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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

            main : IOState -> ((), IOState);
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
            main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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

        fib : Int -> Int;
        fib = |n| (
            if n == 0 {
                0
            } else if n == 1 {
                1
            } else {
                fib(n-1) + fib(n-2)
            }
        );
        
        main : IOState -> ((), IOState);
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
    // Test Array.new of size 0.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new(0, 42);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test24() {
    // Test Array.new of size > 0.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new(100, 42);
            let u = assert_eq("", arr.get_length, 100);
            pure()
        );
        "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test25() {
    // Test Array.get.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new(100, 42);
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
    // Test Array.set (unique case).
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new(100, 42);
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
    // Test Array.set (shared case).
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr0 = Array.new(100, 42);
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
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.from_map(100) $ |i| add(i);
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
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new(31, 0);
            let arr = arr.set!(0, 0);
            let arr = arr.set!(1, 1);
            let loop = fix $ |f, arr: Array Int, n| (
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

        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
        type IntBool = struct {x: Int, y: Bool};

        main : IOState -> ((), IOState);
        main = (
            let obj = IntBool.new(18, false);
            let obj = IntBool.mod_x(|x| x + 42, obj);
            let u = assert_eq("", IntBool.@x(obj), 60);
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
        type IntBool = unbox struct {x: Int, y: Bool};

        main : IOState -> ((), IOState);
        main = (
            let obj = IntBool.new(18, false);
            let obj = IntBool.mod_x(|x| x + 42, obj);
            let u = assert_eq("", IntBool.@x(obj), 60);
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
        
        type OtherStruct = struct {y: Int, x: Bool};
        type IntBool = struct {x: Int, y: Bool};

        main : IOState -> ((), IOState);
        main = (
            let obj = IntBool.new(18, false);
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

        type A = struct {x: Int, y: Bool};
        type B = struct {x: Bool, y: Int};
            
        main : IOState -> ((), IOState);
        main = (
            let a = A.new(3, true);
            let b = B.new(true, 5);
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
        type B = struct {x: Int};
            
        main : IOState -> ((), IOState);
        main = (
            let a = A.new(B.new(16));
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
        type B = struct {x: Int};

        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new $ 16);
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
        type B = struct {x: Int};

        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new $ 16);
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
        type B = struct {x: Int};

        main : IOState -> ((), IOState);
        main = (    
            let a = A.new (B.new $ 16);
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
        type B = struct {x: Int};
        
        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new (16));
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
        type B = struct {x: Int};
        
        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new $ 16);
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
        
        main : IOState -> ((), IOState);
        main = (
            let x: Int -> Int = |x| x;
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
        
        main : IOState -> ((), IOState);
        main = (
            let x = |x: Int| x;
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
            
            loop : Int -> Int;
            loop = |x| if x == 0 {{ 0 }} else {{ add(x) $ loop $ add(x, -1) }};
    
            main : IOState -> ((), IOState);
            main = (
                let ans = Main.loop({});
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
            
            my_loop : Int -> Int -> Int;
            my_loop = |x, acc| if x == 0 {{ acc }} else {{ my_loop(x + -1, acc + x) }};
    
            main : IOState -> ((), IOState);
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

        trait a : ToInt {
            toInt : a -> Int;
        }

        impl Int : ToInt {
            toInt = |x| x;
        }

        impl Bool : ToInt {
            toInt = |b| if b { 0 } else { -1 };
        }

        add_head_and_next : [a: ToInt] Array a -> Int; 
        add_head_and_next = |arr| (
            let head = arr.get(0).toInt;
            let next = arr.get(1).toInt;
            add(head, next)
        );

        main : IOState -> ((), IOState);
        main = (
            let arr0 = Array.new(2, false);
            let arr0 = arr0.set!(0, true);
            let x = add_head_and_next(arr0);

            let arr1 = Array.new(2, 3);
            let arr1 = arr1.set!(1, 5);
            let z = add_head_and_next(arr1);

            let y = toInt(5) + toInt(false);
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
    // Test Array.from_map.
    let source = r#"
        module Main;

        sum : Array Int -> Int;
        sum = |arr| (
            let loop = fix $ |loop, idx, sum| (
                if idx == arr.get_length { sum };
                loop(idx + 1, sum + arr.get(idx))
            );
            loop(0, 0)
        );

        main : IOState -> ((), IOState);
        main = (
            let arr = Array.from_map(10, |x| x * x);
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

        trait [f:*->*] f : Functor {
            map : (a -> b) -> f a -> f b;
        }

        impl Array : Functor {
            map = |f, arr| (
                Array.from_map(arr.get_length, |idx| f $ arr.get(idx))
            );
        }

        sum : Array Int -> Int;
        sum = |arr| (
            let loop = fix $ |loop, idx, sum| (
                if idx == arr.get_length { sum };
                loop(idx + 1, sum + arr.get(idx))
            );
            loop(0, 0)
        );

        main : IOState -> ((), IOState);
        main = (
            let arr = Array.from_map(10, |x| x);
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

        x : Int;
        x = 5;

        y : Int;
        y = 7;

        main : IOState -> ((), IOState);
        main = (
            let ans = (let x = 3 in let y = 2 in add(x, Main.y)) + x;
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

        type IntOrBool = union {int : Int, bool: Bool};

        main : IOState -> ((), IOState);
        main = (
            let int_union = int(3);
            let bool_union = bool(true);
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

        type IntOrBool = box union {int : Int, bool: Bool};

        main : IOState -> ((), IOState);
        main = (
            let int_union = int(3);
            let bool_union = bool(true);
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
    // Test union of boxed object
    let source = r#"
        module Main;

        type Union = union {val: Int, func: Int -> Int};

        main : IOState -> ((), IOState);
        main = (
            let val = Union.val(3);
            let func = Union.func(|x| x + 5);
            let ans = func.as_func $ val.as_val;
            let u = assert_eq("", ans, 5 + 3);
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

        main : IOState -> ((), IOState);
        main = (
            let int_vec = Vec.new $ Array.new(2, 5);
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

        main : IOState -> ((), IOState);
        main = (
            let int_left = Either.left(5);
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
    
            main : IOState -> ((), IOState);
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
    
    impl [a: Eq, b: Eq] (a, b) : Eq {
        eq = |lhs, rhs| (
            lhs.@0 == rhs.@0 && lhs.@1 == rhs.@1
        );
    }

    search : [a: Eq] a -> Array a -> Int;
    search = |elem, arr| loop(0) $ |idx| (
        if idx == arr.get_length {
            break $ -1
        } else if arr.get(idx) == elem { 
            break $ idx
        } else { 
            continue $ idx + 1 
        } 
    );
    
    main : IOState -> ((), IOState);
    main = (
        let arr = Array.new(4, (0, false));
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

    type SieveState = struct {i: Int, arr: Array Bool};
    
    // Calculate a Bool array whose element is true iff idx is prime.
    is_prime : Int -> Array Bool;
    is_prime = |n| (
        let arr = Array.new(n, true);
        let arr = arr.set!(0, false);
        let arr = arr.set!(1, false);
        loop(SieveState.new(2, arr)) $ |state| (
            let i = state.@i;
            let arr = state.@arr;
            if i*i > n { break $ arr };
            let next_arr = if arr.get(i) {
                loop(SieveState.new(i+i, arr)) $ |state| (
                    let q = state.@i;
                    let arr = state.@arr;
                    if n-1 < q { 
                        break $ arr
                    } else {
                        continue $ SieveState.new (q + i) $ arr.set!(q, false)
                    }
                )
            } else {
                arr
            };
            continue $ SieveState.new((i + 1), next_arr)
        )
    );

    // Count the appearance of a value in an array.
    count : [a: Eq] a -> Array a -> Int;
    count = |elem, arr| (
        loop((0, 0)) $ |state| (
            let i = state.@0;
            let sum = state.@1;
            if arr.get_length == i { break $ sum };
            let sum = sum + (if arr.get(i) == elem {1} else {0});
            continue $ (i+1, sum)
        )
    );
    
    main : IOState -> ((), IOState);
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
    
    main : IOState -> ((), IOState);
    main = (
        let pair = (13, Array.new(1, 0));
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
    
    main : IOState -> ((), IOState);
    main = (
        let pair0 = (13, Array.new(1, 0));
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
    
    main : IOState -> ((), IOState);
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
    
    main : IOState -> ((), IOState);
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
    
    main : IOState -> ((), IOState);
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
    
    main : IOState -> ((), IOState);
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
        x : Int;
        x = 3;

        y : Int;
        y = 1;
    }

    namespace B {
        x : Int;
        x = 5;

        y : Bool;
        y = true;
    }

    main : IOState -> ((), IOState);
    main = (
        let ans = (if y {A.x + B.x + A.y} else {0});
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

    main : IOState -> ((), IOState);
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

    main : IOState -> ((), IOState);
    main = |io| (
        loop((0, io)) $ |(counter, io)| (
            if counter == 3 {
                break $ ((), io)
            } else {
                let ret = io.println!("Hello World! ");
                let io = ret.@1;
                continue $ (counter + 1, io)
            }
        )
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

    main : IOState -> ((), IOState);
    main = (
        let len = "Hello World!".get_length;
        let u = assert_eq("", len, 12);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test63() {
    // Test Int ToString.
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = (
        let min = -9223372036854775808;
        println! $ min.to_string
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

    main : IOState -> ((), IOState);
    main = (
        println! $ "\u2764"
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

    main : IOState -> ((), IOState);
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

    type State = unbox struct {idx: Int, sum: Int};

    main : IOState -> ((), IOState);
    main = (
        let sum = loop(State.new(0, 0), |state|
            let State {idx: i, sum: sum} = state;
            if i == 10 {
                break $ sum
            } else {
                continue $ State.new(i+1, sum+i)
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

    type State = box struct {idx: Int, sum: Int};

    main : IOState -> ((), IOState);
    main = (
        let sum = loop(State.new(0, 0), |state|
            let State {idx: i, sum: sum} = state;
            if i == 10 {
                break $ sum
            } else {
                continue $ State.new(i+1, sum+i)
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

    type IntOrBool = unbox union {int: Int, bool: Bool};

    main : IOState -> ((), IOState);
    main = (
        let u = IntOrBool.int(42);
        let IntOrBool.int(x) = u;
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

    type IntOrBool = box union {int: Int, bool: Bool};

    main : IOState -> ((), IOState);
    main = (
        let u = IntOrBool.bool(true);
        let IntOrBool.bool(x) = u;
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

    type Union = union {left: (Int, String), right: Bool};

    main : IOState -> ((), IOState);
    main = (
        let u = Union.left((42, "truth"));
        let Union.left((x, y)) = u;
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

    type Struct = struct {uni: Union, value: Int};
    type Union = union {left: (Int, String), right: Bool};

    main : IOState -> ((), IOState);
    main = (
        let u = Struct.new(Union.left((42, "truth")), 13);
        let Struct { uni: Union.left((truth, string)), value: val } = u;
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

    main : IOState -> ((), IOState);
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

    type IntBool = struct {x: Int, y: Bool};

    main : IOState -> ((), IOState);
    main = (
        let int_bool = IntBool { y: true, x: 42 };
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

    type IntBool = struct {x: Int, y: Bool};

    main : IOState -> ((), IOState);
    main = (
        let int_bool = IntBool { y: false, x: 0 };
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

    main : IOState -> ((), IOState);
    main = (
        let iter = Iterator.from_map(|i| i*i );
        let Option.some((n, iter)) = iter.next;
        let _ = assert_eq("", n, 0*0);
        let Option.some((n, iter)) = iter.next;
        let _ = assert_eq("", n, 1*1);
        let Option.some((n, iter)) = iter.next;
        let _ = assert_eq("", n, 2*2);
        let Option.some((n, iter)) = iter.next;
        let _ = assert_eq("", n, 3*3);
        let Option.some((n, iter)) = iter.next;
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

    main : IOState -> ((), IOState);
    main = (
        let array = Array.from_map(3, |_i| Array.from_map(3, |_j| 0));
        let array = array.mod!(1, Array.set!(1, 9));
        let _ = assert_eq("", array.get(1).get(1), 9);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test77() {
    // Test Iterator.zip / map / take / fold.
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = (
        let iter0 = Iterator.count_up(5);
        let iter1 = Iterator.from_map(|i| 2*i);
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
    // Test Iterator.filter
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = (
        let iter = Iterator.count_up(1).take(100);
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
    // Test Iterator.append
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = (
        let ls = Iterator.make_empty;
        let ls = ls.push_front(1).push_front(2);
        let (e, ls) = ls.next.unwrap;
        let _ = assert_eq("", 2, e);
        let (e, ls) = ls.next.unwrap;
        let _ = assert_eq("", 1, e);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test80() {
    // Test Iterator.last
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = (
        let iter = Iterator.make_empty.push_front(4).push_front(3).push_front(2).push_front(1);
        let last = iter.take_last.unwrap;
        let _ = assert_eq("", last, 4);
        let last: Option Bool = Iterator.make_empty.take_last;
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

    main : IOState -> ((), IOState);
    main = (
        let arr = [1,2,3,4];
        let _ = assert_eq("", arr.get_length, 4);
        let arr: Array Bool = [];
        let _ = assert_eq("", arr.get_length, 0);
        pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test82() {
    // Test Vector.append.
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = (
        // Test 2+2
        let v1 = Vector.from_array([1,2]);
        let v2 = Vector.from_array([3,4]);
        let v = v1.append(v2);
        let _ = assert_eq("wrong reserved length (2+2)", v.get_reserved_length, 4);
        let _ = assert_eq("wrong length (2+2)", v.get_length, 4);
        let _ = assert_eq("wrong element (2+2)", v.get(0), 1);
        let _ = assert_eq("wrong element (2+2)", v.get(1), 2);
        let _ = assert_eq("wrong element (2+2)", v.get(2), 3);
        let _ = assert_eq("wrong element (2+2)", v.get(3), 4);

        // Test 0+2
        let v1 = Vector.from_array([]);
        let v2 = Vector.from_array([3,4]);
        let v = v1.append(v2);
        let _ = assert_eq("wrong reserved length (0+2)", v.get_reserved_length, 2);
        let _ = assert_eq("wrong length (0+2)", v.get_length, 2);
        let _ = assert_eq("wrong element (0+2)", v.get(0), 3);
        let _ = assert_eq("wrong element (0+2)", v.get(1), 4);

        // Test 2+0
        let v1 = Vector.from_array([1,2]);
        let v2 = Vector.from_array([]);
        let v = v1.append(v2);
        let _ = assert_eq("wrong reserved length (2+0)", v.get_reserved_length, 2);
        let _ = assert_eq("wrong length (2+0)", v.get_length, 2);
        let _ = assert_eq("wrong element (2+0)", v.get(0), 1);
        let _ = assert_eq("wrong element (2+0)", v.get(1), 2);

        // Test 0+0
        let v1: Vector (Int -> Bool) = Vector.from_array([]);
        let v2 = Vector.from_array([]);
        let v = v1.append(v2);
        let _ = assert_eq("wrong reserved length (0+0)", v.get_reserved_length, 0);
        let _ = assert_eq("wrong length (0+0)", v.get_length, 0);

        // Test boxed elements.
        let v1 = Vector.from_array([add(1), add(2)]);
        let v2 = Vector.from_array([add(3), add(4)]);
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
        let v1 = Vector.from_array([add(1), add(2)]).reserve(4);
        let v2 = Vector.from_array([add(3), add(4)]);
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
    // Test Vector.push_back, pop_back
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = |io| (
        // Unboxed element
        let v = Vector.from_array([]);
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
        let _ = assert_eq("wrong length after pop", 0, v.get_length);
        let _ = assert("wrong reserved length after pop", v.get_reserved_length >= 100);
    
        // Boxed element
        let v = Vector.from_array([]);
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
        let _ = assert_eq("wrong length after pop (boxed)", 0, v.get_length);
        let _ = assert("wrong reserved length after pop (boxed)", v.get_reserved_length >= 100);
    
        io.pure()
    );
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test84() {
    // Test Eq for Vector
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = |io| (
        let v1 = Vector.from_array([1,2,3]);
        let v2 = Vector.from_array([1,2,3]);
        let _ = assert("", v1 == v2);
    
        let v1 = Vector.from_array([1,2,3]);
        let v2 = Vector.from_array([0,2,3]);
        let _ = assert("", v1 != v2);
    
        let v1 = Vector.from_array([]);
        let v2 = Vector.from_array([0]);
        let _ = assert("", v1 != v2);
    
        let v1: Vector Int = Vector.from_array([]);
        let v2 = Vector.from_array([]);
        let _ = assert("", v1 == v2);
    
        io.pure()
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

    main : IOState -> ((), IOState);
    main = |io| (
        let s1 = "Hello";
        let s2 = " ";
        let s3 = "World!";
        let _ = assert_eq("", s1.concat(s2).concat(s3) == "Hello World!");
    
        io.pure()
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

    main : IOState -> ((), IOState);
    main = |io| (
        let iter = Iterator.from_array(["Hello", " ", "World", "!"]);
        let _ = assert_eq("", iter.concat_iter, "Hello World!");
        io.pure()
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

    main : IOState -> ((), IOState);
    main = |io| (
        let lhs = Iterator.from_array([1,2,3]);
        let rhs = Iterator.from_array([1,2,3]);
        let _ = assert_eq("", lhs, rhs);

        let lhs: Iterator Bool = Iterator.from_array([]);
        let rhs = Iterator.from_array([]);
        let _ = assert_eq("", lhs, rhs);

        let lhs = Iterator.from_array([]);
        let rhs = Iterator.from_array([1,2]);
        let _ = assert("", lhs != rhs);

        io.pure()
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

    main : IOState -> ((), IOState);
    main = |io| (
        let iter = Iterator.from_array([1,2,3]);
        let iter = iter.intersperse(0);
        let _ = assert_eq("", iter, Iterator.from_array([1,0,2,0,3]));
    
        let iter = Iterator.from_array([1]);
        let iter = iter.intersperse(0);
        let _ = assert_eq("", iter, Iterator.from_array([1]));
    
        let iter = Iterator.from_array([]);
        let iter = iter.intersperse(0);
        let _ = assert_eq("", iter, Iterator.from_array([]));
    
        io.pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test89() {
    // Test Iterator.append
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = |io| (
        let lhs = Iterator.from_array([1,2,3]);
        let rhs = Iterator.from_array([4,5,6]);
        let _ = assert_eq("", lhs + rhs, Iterator.from_array([1,2,3,4,5,6]));
    
        let lhs = Iterator.from_array([]);
        let rhs = Iterator.from_array([4,5,6]);
        let _ = assert_eq("", lhs + rhs, Iterator.from_array([4,5,6]));

        let lhs = Iterator.from_array([1,2,3]);
        let rhs = Iterator.from_array([]);
        let _ = assert_eq("", lhs + rhs, Iterator.from_array([1,2,3]));

        let lhs: Iterator Int = Iterator.from_array([]);
        let rhs = Iterator.from_array([]);
        let _ = assert_eq("", lhs + rhs, Iterator.from_array([]));
    
        io.pure()
    );
    
    "#;
    run_source(source, Configuration::develop_compiler());
}

#[test]
#[serial]
pub fn test_run_examples() {
    let paths = fs::read_dir("./examples").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let display = path.display();
        if path.extension().unwrap() != "fix" {
            continue;
        }
        println!("[run_examples] {}:", display);

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => {}
        }

        // Since examples may include heavy computation, perform optimization.
        run_source(&s, Configuration::release());
    }
}

#[test]
#[serial]
pub fn test_comment_0() {
    // block comment
    let source = r"/* head */ module Main; 
        main : IOState -> ((), IOState);
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
        main : IOState -> ((), IOState);
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
