use super::*;

pub fn test_run_source(source: &str, answer: i64, opt_level: OptimizationLevel) {
    assert_eq!(run_source(source, opt_level), answer)
}

// Tests should run sequentially, since OBJECT_TABLE in libfixsanitizer.so is shared between tests and check_leak() asserts OBJECT_TABLE is empty.
#[test]
#[serial]
pub fn test0() {
    let source = r"5";
    let answer = 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test1() {
    let source = r"let x = 5 in x";
    let answer = 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test2() {
    let source = r"let x = 5 in 3";
    let answer = 3;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test3() {
    let source = r"let n = -5 in let p = 5 in n";
    let answer = -5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test4() {
    let source = r"let n = -5 in let p = 5 in p";
    let answer = 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test5() {
    let source = r"let x = -5 in let x = 5 in x";
    let answer = 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test6() {
    let source = r"let x = let y = 3 in y in x";
    let answer = 3;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test7() {
    let source = r"false 10";
    let answer = 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test8() {
    let source = r"(\x: Int -> x) 6";
    let answer = 6;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test9() {
    let source = r"add 3 5";
    let answer = 8;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test10() {
    let source = r"let x = 5 in add 2 x";
    let answer = 7;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test11() {
    let source = r"
            let x = 5 in 
            let y = -3 in
            add x y
        ";
    let answer = 2;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test12() {
    let source = r"
            let x = 5 in 
            let y = -3 in
            let z = 12 in
            let xy = add x y in
            add xy z
        ";
    let answer = 14;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test13() {
    let source = r"
            let f = add 5 in
            f 3
        ";
    let answer = 5 + 3;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test13_5() {
    let source = r"
            let f = add 5 in
            add (f -3) (f 12)
        ";
    let answer = 5 - 3 + 5 + 12;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test14() {
    let source = r"
            let x = 3 in 
            let y = 5 in
            let f = add x in
            f y
        ";
    let answer = 3 + 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test15() {
    let source = r"
            let f = \x: Int -> add 3 x in
            f 5
        ";
    let answer = 3 + 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test15_5() {
    let source = r"
            let x = 3;
            let f = \y: Int -> x;
            f 5
        ";
    let answer = 3;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test16() {
    let source = r"
            let f = \x: Int -> add x 3 in
            f 5
        ";
    let answer = 3 + 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test17() {
    let source = r"if true then 3 else 5";
    let answer = 3;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test18() {
    let source = r"if false then 3 else 5";
    let answer = 5;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test19() {
    let source = r"if eq<Int> 3 3 then 1 else 0";
    let answer = 1;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test20() {
    let source = r"if eq<Int> 3 5 then 1 else 0";
    let answer = 0;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test20_5() {
    let source = r"         
        if eq<Int> 2 0 then 
            0 
        else if eq<Int> 2 1 then 
            1
        else 2
    ";
    let answer = 2;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test21() {
    let n = 10000;
    let source = format!(
        r"
                let g = fix<Int,Int> \f:Int=>Int -> \x: Int -> if eq<Int> x 0 then 0 else add x (f (add x -1));
                g {}
        ",
        n
    );
    let answer = (n * (n + 1)) / 2;
    test_run_source(source.as_str(), answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test22() {
    let n = 100000;
    let source = format!(
        r"
                let g = fix<Int,Int=>Int> \f:Int=>Int=>Int -> \a:Int -> \x:Int -> 
                            if eq<Int> x 0 then 
                                a 
                            else
                                let a2 = add a x;
                                let x2 = add x -1;
                                f a2 x2
                in g 0 {}
        ",
        n
    );
    let answer = (n * (n + 1)) / 2;
    test_run_source(source.as_str(), answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test22_5() {
    let source = r"
        let fib = fix<Int,Int> \f:Int=>Int -> \n:Int ->
                    if eq<Int> n 0 then
                        0
                    else if eq<Int> n 1 then
                        1
                    else
                        add (f (add n -1)) (f (add n -2))
        in fib 10
    ";
    let answer = 55;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test23() {
    // Test newArray of size 0.
    let source = r"
            let arr = newArray<Int> 0 42;
            32
        ";
    let answer = 32;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test24() {
    // Test newArray of size > 0.
    let source = r"
            let arr = newArray<Int> 100 42;
            32
        ";
    let answer = 32;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test25() {
    // Test readArray.
    let source = r"
            let arr = newArray<Int> 100 42;
            let elem = readArray<Int> arr 50;
            elem
        ";
    let answer = 42;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test26() {
    // Test writeArray.
    let source = r"
            let arr = newArray<Int> 100 42;
            let arr = writeArray<Int> arr 50 21;
            readArray<Int> arr 50
        ";
    let answer = 21;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test28() {
    // Calculate Fibonacci sequence using array.
    let source = r"
            let arr = newArray<Int> 31 0;
            let arr = writeArray!<Int> arr 0 0;
            let arr = writeArray!<Int> arr 1 1;
            let loop = fix<Array<Int>,Int=>Array<Int>> \f:Array<Int>=>Int=>Array<Int> -> \arr:Array<Int> -> \n:Int ->
                if eq<Int> n 31 then
                    arr
                else
                    let x = readArray<Int> arr (add n (-1));
                    let y = readArray<Int> arr (add n (-2));
                    let arr = writeArray!<Int> arr n (add x y);
                    f arr (add n 1);
            let fib = loop arr 2;
            readArray<Int> fib 30
        ";
    let answer = 832040;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test29() {
    let source = r"
            let id = for<a> \x: a -> x;
            if id<Bool> true then id<Int> 100 else 30
        ";
    let answer = 100;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test_comment_0() {
    // block comment
    let source = r"{- head -}
            let x = 5 in 
            let y = -3 in
            {- If the closing symbol is put on the end of this line, g will evaluate.
            let g = fix \f -> \x -> if eq x 0 then 0 else add x (f (add x -1));
            g 100
            {--}
            {- 
            multiple line 
            block comment
            -}
            {- sub 1 -}add x{- This comment is parsed as a separater -}y{- comment -}


        {-tail-}";
    let answer = 2;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
#[serial]
pub fn test_comment_1() {
    // ilne comment
    let source = r"////
            let x = 5 in
            // let x = 3 in
// some excellent and brilliant comment
            let y = -3 in// comment
            add x y
        //";
    let answer = 2;
    test_run_source(source, answer, OptimizationLevel::Default);
}

#[test]
pub fn test_print_type_0() {
    let source = r"5";
    let answer = "Int";
    assert_eq!(type_of_source(source).to_string(), answer);
}

#[test]
pub fn test_print_type_1() {
    let source = r"true";
    let answer = "Bool";
    assert_eq!(type_of_source(source).to_string(), answer);
}

#[test]
pub fn test_print_type_2() {
    let source = r"\x: Int -> x";
    let answer = "Int => Int";
    assert_eq!(type_of_source(source).to_string(), answer);
}

#[test]
pub fn test_print_type_3() {
    let source = r"for<a> \x: a -> x";
    let answer = "for<a> a => a";
    assert_eq!(type_of_source(source).to_string(), answer);
}

#[test]
pub fn test_print_type_4() {
    let source = r"\x: Int -> \y: Int -> add x y";
    let answer = "Int => Int => Int";
    assert_eq!(type_of_source(source).to_string(), answer);
}

#[test]
pub fn test_print_type_5() {
    let source = r"\f: Int=>Int -> \g: Int=>Int -> \x:Int -> f (g x)";
    let answer = "(Int => Int) => (Int => Int) => Int => Int";
    assert_eq!(type_of_source(source).to_string(), answer);
}

#[test]
pub fn test_print_type_6() {
    let source = r"\f: for<a> a=>a -> \g: for<b> b=>b -> for<c> \x:c -> f<c> (g<c> x)";
    let answer = "(for<a> a => a) => (for<b> b => b) => for<c> c => c";
    assert_eq!(type_of_source(source).to_string(), answer);
}

#[test]
pub fn test_print_type_7() {
    let source = r"newArray<Int> 30 0";
    let answer = "Array<Int>";
    assert_eq!(type_of_source(source).to_string(), answer);
}

#[test]
pub fn test_print_type_8() {
    let source = r"newArray<Array<Int=>Bool>> 30 (newArray<Int=>Bool> 30 \x: Int -> eq<Int> x 0)";
    let answer = "Array<Array<Int => Bool>>";
    assert_eq!(type_of_source(source).to_string(), answer);
}
