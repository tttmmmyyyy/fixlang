use super::*;

// Tests should run sequentially, since OBJECT_TABLE in libfixsanitizer.so is shared between tests and check_leak() asserts OBJECT_TABLE is empty.
#[test]
#[serial]
pub fn test0() {
    let source = r#"
            module Main;
    
            main : IOState -> ((), IOState);
            main = (
                let u = assert_eq "" (5 + 3 * 8 / 5 + 7 % 3) 10;
                pure ()
            );
        "#;
    run_source(&source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test1() {
    let source = r#"
            module Main;
            
            main : IOState -> ((), IOState);
            main = (
                let u = assert_eq "" (let x = 5 in -x) (-5);
                pure ()
            );
        "#;
    run_source(&source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test2() {
    let source = r#"
            module Main;
            main : IOState -> ((), IOState);
            main = (
                let u = assert_eq "" (let x = 5 in 3) 3;
                pure ()
            );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test3() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (let n = -5 in let p = 5 in n) (-5);
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test4() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (let n = -5 in let p = 5 in p) 5;
            pure ()
            
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test5() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (let x = -5 in let x = 5 in x) 5;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test6() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (let x = let y = 3 in y in x) 3;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test7() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" ((\x -> 5) 10) 5;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test8() {
    let source = r#"
        module Main; 

        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" ((\x -> x) 6) 6;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test9() {
    let source = r#"
        module Main;
        
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (3 + 5) 8;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
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
            let u = assert_eq "" (x - y) -2;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test10() {
    let source = r#"
        module Main;

        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (let x = 5 in 2 + x) 7;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
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
            let u = assert_eq "" (x + y) 2;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
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
            let u = assert_eq "" (xy + z) 14;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test13() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let f = add 5 in
            let u = assert_eq "" (f 3) (5+3);
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test13_5() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let f = add 5 in
            let u = assert_eq "" (f -3 + f 12) (5 - 3 + 5 + 12);
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
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
            let f = add x in
            let u = assert_eq "" (f y) (3 + 5);
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test15() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let f = \x -> 3 + x in
            let u = assert_eq "" (f 5) (3 + 5);
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test15_5() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let x = 3;
            let f = \y -> x;
            let u = assert_eq "" (f 5) 3;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test16() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let f = \x -> x + 3 in
            let u = assert_eq "" (f 5) (3 + 5);
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test17() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (if true then 3 else 5) 3;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test18() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (if false then 3 else 5) 5;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test19() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (if 3 == 3 then 1 else 0) 1;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test20() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (if 3 == 5 then 1 else 0) 0;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test20_5() {
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let ans = (
                if 2 == 0 then
                    0 
                else if 2 == 1 then 
                    1
                else 2
            );
            let u = assert_eq "" ans 2;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test21() {
    let source = r#"
            module Main;

            main : IOState -> ((), IOState);
            main = (
                let fact = fix \loop -> \n -> if n == 0 then 1 else n * loop (n-1);
                let u = assert_eq "" (fact 5) (5 * 4 * 3 * 2 * 1);
                pure ()
            );
        "#;
    run_source(source, OptimizationLevel::Default);
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
                let g = fix \loop -> \a -> \x -> 
                            if x == 0 then 
                                a 
                            else
                                let a2 = a + x;
                                let x2 = x + -1;
                                loop a2 x2
                in 
                    let u = assert_eq "" (g 0 {}) {};
                    pure ()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test22_5() {
    // Test recursion function defined by fix that is not tail-call.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let fib = fix \f -> \n ->
                        if n == 0 then
                            0
                        else if n == 1 then
                            1
                        else
                            f (n+-1) + f (n+-2)
            in 
                let u = assert_eq "" (fib 10) 55;
                pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test22_7() {
    // Test global recursion function
    let source = r#"
        module Main;

        fib : Int -> Int;
        fib = \n -> (
            if n == 0 then
                0
            else if n == 1 then
                1
            else
                fib (n-1) + fib (n-2)
        );
        
        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (fib 30) 832040;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test23() {
    // Test Array.new of size 0.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new 0 42;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test24() {
    // Test Array.new of size > 0.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new 100 42;
            let u = assert_eq "" (arr.len) 100;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test25() {
    // Test Array.read.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new 100 42;
            let elem = arr.get 50;
            let u = assert_eq "" elem 42;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test26() {
    // Test Array.set (unique case).
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new 100 42;
            let arr = arr.set 50 21;
            let u = assert_eq "" (arr.get 50) 21;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test27() {
    // Test Array.set (shared case).
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr0 = Array.new 100 42;
            let arr1 = arr0.set 50 21;
            let u = assert_eq "" (arr0.get 50 + arr1.get 50) 63;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test27_5() {
    // Test Array of boxed object.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.from_map 100 $ \i -> add i;
            let arr = arr.set 99 \x -> x - 100;
            let u = assert_eq "" (arr.get 99 $ arr.get 50 $ 1) (1 + 50 - 100);
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test28() {
    // Calculate Fibonacci sequence using array.
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let arr = Array.new 31 0;
            let arr = arr.set! 0 0;
            let arr = arr.set! 1 1;
            let loop = fix \f -> \arr -> \n ->
                if n == 31 then
                    arr
                else
                    let x = arr.get (add n (-1));
                    let y = arr.get (add n (-2));
                    let arr = arr.set! n (x+y);
                    f arr (n+1);
            let fib = loop arr 2;
            let u = assert_eq "" (fib.get 30) 832040;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test29() {
    let source = r#"
        module Main;

        id : a -> a;
        id = \x -> x;

        main : IOState -> ((), IOState);
        main = (
            let u = assert_eq "" (if id true then id 100 else 30) 100;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test30() {
    // Test dollar combinator
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let f = \x -> x + 3;
            let g = \x -> x == 8;
            let ans = g $ f $ 5;
            let u = assert_eq "" (if ans then 1 else 0) 1;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test31() {
    // Test . combinator
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let f = \x -> x + 3;
            let g = \x -> x == 8;
            let ans = 5 .f. g;
            let u = assert_eq "" (if ans then 1 else 0) 1;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test32() {
    // Test . and $ combinator
    let source = r#"
        module Main;
        main : IOState -> ((), IOState);
        main = (
            let f = \x -> x + 10;
            let u = assert_eq "" (5.add $ 3.f) 18;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test33() {
    // Test struct declaration and new, mod.
    let source = r#"
        module Main;
        type IntBool = struct (x: Int, y: Bool);

        main : IOState -> ((), IOState);
        main = (
            let obj = IntBool.new 18 false;
            let obj = IntBool.mod_x (\x -> x + 42) obj;
            let u = assert_eq "" (IntBool.get_x obj) 60;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test34_5() {
    // Test unboxed struct declaration and new, mod.
    let source = r#"
        module Main;
        type IntBool = unbox struct (x: Int, y: Bool);

        main : IOState -> ((), IOState);
        main = (
            let obj = IntBool.new 18 false;
            let obj = IntBool.mod_x (\x -> x + 42) obj;
            let u = assert_eq "" (IntBool.get_x obj) 60;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test34() {
    // Test namespace inference.
    let source = r#"
        module Main;        
        
        type OtherStruct = struct (y: Int, x: Bool);
        type IntBool = struct (x: Int, y: Bool);

        main : IOState -> ((), IOState);
        main = (
            let obj = IntBool.new 18 false;
            let obj = obj . mod_x (\x -> x + 42);
            let u = assert_eq "" (obj . get_x) 60;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test35() {
    // Test overloading resolution.
    let source = r#"
        module Main;

        type A = struct (x: Int, y: Bool);
        type B = struct (x: Bool, y: Int);
            
        main : IOState -> ((), IOState);
        main = (
            let a = A.new 3 true;
            let b = B.new true 5;
            let ans = add (if a.get_y then a.get_x else 0) (if b.get_x then b.get_y else 0);
            let u = assert_eq "" ans 8;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test36() {
    // Test modifier composition.
    let source = r#"
        module Main;

        type A = struct (x: B);
        type B = struct (x: Int);
            
        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new 16);
            let a = a.(mod_x $ mod_x $ \x -> x + 15);
            let ans = a . get_x . get_x;
            let u = assert_eq "" ans 31;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test37() {
    // Test unique modField.
    let source = r#"
        module Main;

        type A = struct (x: B);
        type B = struct (x: Int);

        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new 16);
            let b = a . (mod_x! $ mod_x! $ \x -> x + 15);
            let ans = b . get_x . get_x;
            let u = assert_eq "" ans 31;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test37_5() {
    // Test shared modField.
    let source = r#"
        module Main;

        type A = struct (x: B);
        type B = struct (x: Int);

        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new 16);
            let b = a.(mod_x $ mod_x $ \x -> x + 15);
            let ans = a.get_x.get_x + b.get_x.get_x;
            let u = assert_eq "" ans ((16 + 15) + 16);
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test38() {
    // Test type annotation.
    let source = r#"
        module Main;

        type A = struct (x: B);
        type B = struct (x: Int);

        main : IOState -> ((), IOState);
        main = (    
            let a = A.new (B.new 16);
            let f = \a -> (a : A) . (mod_x! $ mod_x! $ \x -> x + 15);
            let a = a.f;
            let ans = a.get_x.get_x;
            let u = assert_eq "" ans 31;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test39() {
    // Test type annotation.
    let source = r#"
        module Main;

        type A = struct (x: B);
        type B = struct (x: Int);
        
        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new 16);
            let f = \a -> a . ((mod_x! : (B -> B) -> A -> A) $ mod_x! $ \x -> x + 15);
            let a = a.f;
            let ans = a.get_x.get_x;
            let u = assert_eq "" ans 31;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test40() {
    // Test type annotation at let-binding.
    let source = r#"
        module Main;

        type A = struct (x: B);
        type B = struct (x: Int);
        
        main : IOState -> ((), IOState);
        main = (
            let a = A.new (B.new 16);
            let f: A -> A = \a -> a.(mod_x! $ mod_x! $ \x -> x + 15);
            let a = a .f;
            let ans = a .get_x .get_x;
            let u = assert_eq "" ans 31;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test41() {
    // Test type annotation at let-binding.
    let source = r#"
        module Main;
        
        main : IOState -> ((), IOState);
        main = (
            let x: Int -> Int = \x -> x;
            let ans = x 42;
            let u = assert_eq "" ans 42;
            pure ()
        );
        "#;
    run_source(source, OptimizationLevel::Default);
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
            loop = \x -> if x == 0 then 0 else add x $ loop $ add x -1;
    
            main : IOState -> ((), IOState);
            main = (
                let ans = Main.loop {};
                let u = assert_eq "" ans {};
                pure ()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test43() {
    // Recursion function using global variable (tail call).
    let n: i64 = 1000000;
    let source = format!(
        r#"
            module Main;
            
            my_loop : Int -> Int -> Int;
            my_loop = \x -> \acc -> if x == 0 then acc else my_loop (x + -1) (acc + x);
    
            main : IOState -> ((), IOState);
            main = (
                let ans = my_loop {} 0;
                let u = assert_eq "" ans {};
                pure ()
            );
        "#,
        n,
        (n * (n + 1)) / 2
    );
    run_source(source.as_str(), OptimizationLevel::Default);
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
            toInt = \x -> x;
        }

        impl Bool : ToInt {
            toInt = \b -> if b then 0 else -1;
        }

        add_head_and_next : [a: ToInt] Array a -> Int; 
        add_head_and_next = \arr -> (
            let head = arr.get 0.toInt;
            let next = arr.get 1.toInt;
            add head next
        );

        main : IOState -> ((), IOState);
        main = (
            let arr0 = Array.new 2 false;
            let arr0 = arr0.set! 0 true;
            let x = add_head_and_next arr0;

            let arr1 = Array.new 2 3;
            let arr1 = arr1.set! 1 5;
            let z = add_head_and_next arr1;

            let y = toInt 5 + toInt false;
            let ans = x + y + z;
            let u = assert_eq "" ans 11;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test44_5() {
    // Test Array.from_map.
    let source = r#"
        module Main;

        sum : Array Int -> Int;
        sum = \arr -> (
            let loop = fix \loop -> \idx -> \sum -> (
                if idx == arr.len 
                then sum
                else loop (idx + 1) (sum + arr.get idx)
            );
            loop 0 0
        );

        main : IOState -> ((), IOState);
        main = (
            let arr = Array.from_map 10 \x -> x * x;
            let ans = sum arr;
            let u = assert_eq "" ans 285;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
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
            map = \f -> \arr -> (
                Array.from_map (arr.len) \idx -> f (arr.get idx)
            );
        }

        sum : Array Int -> Int;
        sum = \arr -> (
            let loop = fix \loop -> \idx -> \sum -> (
                if idx == arr.len 
                then sum
                else loop (idx + 1) (sum + arr.get idx)
            );
            loop 0 0
        );

        main : IOState -> ((), IOState);
        main = (
            let arr = Array.from_map 10 \x -> x;
            let arr = arr.map \x -> x * x;
            let ans = arr.sum;
            let u = assert_eq "" ans 285;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
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
            let ans = (let x = 3 in let y = 2 in add x Main.y) + x;
            let u = assert_eq "" ans 15;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test47() {
    // Basic use of union.
    let source = r#"
        module Main;

        type IntOrBool = union (int : Int, bool: Bool);

        main : IOState -> ((), IOState);
        main = (
            let int_union = int 3;
            let bool_union = bool true;
            let int_val = if int_union.is_int then int_union.as_int else 0;
            let bool_val = if bool_union.is_bool then bool_union.as_bool else false;
            let ans = if bool_val then int_val else 0;
            let u = assert_eq "" ans 3;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test47_2() {
    // Basic use of boxed union.
    let source = r#"
        module Main;

        type IntOrBool = box union (int : Int, bool: Bool);

        main : IOState -> ((), IOState);
        main = (
            let int_union = int 3;
            let bool_union = bool true;
            let int_val = if int_union.is_int then int_union.as_int else 0;
            let bool_val = if bool_union.is_bool then bool_union.as_bool else false;
            let ans = if bool_val then int_val else 0;
            let u = assert_eq "" ans 3;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test47_5() {
    // Test union of boxed object
    let source = r#"
        module Main;

        type Union = union (val: Int, func: Int -> Int);

        main : IOState -> ((), IOState);
        main = (
            let val = Union.val 3;
            let func = Union.func \x -> x + 5;
            let ans = func.as_func $ val.as_val;
            let u = assert_eq "" ans (5 + 3);
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test48() {
    // Parametrised struct.
    let source = r#"
        module Main;

        type Vec a = struct (data: Array a);

        main : IOState -> ((), IOState);
        main = (
            let int_vec = Vec.new $ Array.new 2 5;
            let int_vec = int_vec.mod_data! \arr -> arr.set 0 3;
            let head = int_vec.get_data.get 0;
            let next = int_vec.get_data.get 1;
            let ans = add head next;
            let u = assert_eq "" ans 8;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test49() {
    // Parametrised union.
    let source = r#"
        module Main;

        type Either a b = union (left: a, right: b);

        main : IOState -> ((), IOState);
        main = (
            let int_left = Either.left 5;
            let ans = (
                if int_left.is_left 
                    then int_left.as_left 
                    else if int_left.as_right then 1 else 0
            );
            let u = assert_eq "" ans 5;
            pure ()
        );
    "#;
    run_source(source, OptimizationLevel::Default);
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
                    loop (0, 0) \state -> 
                        let i = state.get_0;
                        let sum = state.get_1;
                        if i == {} then break sum else continue $ (i+1, sum+i)
                );
                let u = assert_eq "" ans {};
                pure ()
            );
        "#,
        n,
        (n * (n - 1)) / 2
    );
    run_source(&source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test51() {
    // test trait bounds.
    let source = r#"
    module Main;
    
    impl [a: Eq, b: Eq] (a, b) : Eq {
        eq = \lhs -> \rhs -> (
            lhs.get_0 == rhs.get_0 && lhs.get_1 == rhs.get_1
        );
    }

    search : [a: Eq] a -> Array a -> Int;
    search = \elem -> \arr -> loop 0 \idx -> (
        if idx == arr.len then break -1
        else if arr.get idx == elem then break idx
        else continue $ idx + 1
    );
    
    main : IOState -> ((), IOState);
    main = (
        let arr = Array.new 4 (0, false);
        let arr = arr.set 0 (0, false);
        let arr = arr.set 1 (0, true);
        let arr = arr.set 2 (1, false);
        let arr = arr.set 3 (1, true);
        let ans = arr.search (1, false); // evaluates to 2
        let u = assert_eq "" ans 2;
        pure ()
    );
        "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test52() {
    // Test loop with boxed state / break.
    let source = r#"
    module Main;

    type SieveState = struct (i: Int, arr: Array Bool);
    
    // Calculate a Bool array whose element is true iff idx is prime.
    is_prime : Int -> Array Bool;
    is_prime = \n -> (
        let arr = Array.new n true;
        let arr = arr.set! 0 false;
        let arr = arr.set! 1 false;
        loop (SieveState.new 2 arr) \state -> (
            let i = state.get_i;
            let arr = state.get_arr;
            if i*i > n then break arr else 
            let next_arr = if arr.get i then (
                loop (SieveState.new (i+i) arr) \state -> (
                    let q = state.get_i;
                    let arr = state.get_arr;
                    if n-1 < q then 
                        break arr
                    else 
                        continue $ SieveState.new (q + i) $ arr.set! q false
                )
            ) else arr;
            continue $ SieveState.new (i + 1) next_arr
        )
    );

    // Count the appearance of a value in an array.
    count : [a: Eq] a -> Array a -> Int;
    count = \elem -> \arr -> (
        loop (0, 0) \state -> (
            let i = state.get_0;
            let sum = state.get_1;
            if arr.len == i then break sum 
            else 
                let sum = sum + (if arr.get i == elem then 1 else 0);
                continue $ (i+1, sum)
        )
    );
    
    main : IOState -> ((), IOState);
    main = (
        let ans = (is_prime 100).count true;
        let u = assert_eq "" ans 25;
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test53() {
    // Test mutation of unique unboxed struct (e.g., tuple).
    let source = r#"
    module Main;
    
    main : IOState -> ((), IOState);
    main = (
        let pair = (13, Array.new 1 0);
        let pair = pair.mod_0! \x -> x + 3;
        let pair = pair.mod_1! \arr -> arr.set! 0 5;
        let x = pair.get_0;
        let y = pair.get_1.get 0;
        let ans = x + y;
        let u = assert_eq "" ans (13 + 3 + 5);
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test54() {
    // Test mutation of shared unboxed struct (e.g., tuple).
    let source = r#"
    module Main;
    
    main : IOState -> ((), IOState);
    main = (
        let pair0 = (13, Array.new 1 0);
        let pair1 = pair0.mod_1 \arr -> arr.set 0 5;
        let pair2 = pair0.mod_0! \x -> x + 3;
        let x = pair1.get_1.get 0;
        let y = pair2.get_0;
        let ans = x + y;
        let u = assert_eq "" ans (13 + 3 + 5);
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
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
            if 0 <= -1 && -1 >= 0 then
                0
            else if 0 <= 0 && 0 <= 1 && 0 >= 0 && 1 >= 0 then
                1
            else 
                2
        );
        let u = assert_eq "" ans 1;
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
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
            then 1 else 0
        );
        let u = assert_eq "" ans 1;
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
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
            if !false == true && !true == false
            then 1 else 0
        );
        let u = assert_eq "" ans 1;
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
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
            if false != true && true != false && !(true != true) && !(false != false)
            then 1 else 0
        );
        let u = assert_eq "" ans 1;
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
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
        let ans = (if y then A.x + B.x + A.y else 0);
        let u = assert_eq "" ans 9;
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
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
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test61() {
    // Test Hello world.
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = \io -> (
        loop (0, io) \state -> (
            let counter = state.get_0;
            let io = state.get_1;
            if counter == 3 then
                break ((), io)
            else
                let ret = io.print "Hello World! ";
                let io = ret.get_1;
                continue (counter + 1, io)
        )
    );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test62() {
    // Test String length.
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = (
        let len = "Hello World!".get_len;
        let u = assert_eq "" len 12;
        pure ()
    );
    "#;
    run_source(source, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test63() {
    // Test Int ToString.
    let source = r#"
    module Main;

    main : IOState -> ((), IOState);
    main = print "Neko";
    "#;
    run_source(source, OptimizationLevel::Default);
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
            let g = fix \f -> \x -> if x == 0 then 0 else add x (f (add x -1));
            g 100
            /* */
            //
            /*
            multiple line 
            block comment
            */
            let z = /* sub 1 */add x/* This comment is parsed as a separater */y/* comment */;
            pure ()
        );
        /*tail*/";
    run_source(source, OptimizationLevel::Default);
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
            let z = add x y;
            pure ()
        //
        );";
    run_source(source, OptimizationLevel::Default);
}
