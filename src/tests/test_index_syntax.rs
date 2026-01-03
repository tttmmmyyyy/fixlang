use crate::{
    tests::util::{test_source, test_source_fail},
    Configuration,
};

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
    test_source_fail(
        source,
        Configuration::compiler_develop_mode(),
        "Unknown name `act_foo`",
    );
}

#[test]
pub fn test_index_syntax_and_dot_operator() {
    let source = r##"
module Main;

type A a = struct { a : a };

main : IO ();
main = (
    let x = A { a : [7] };

    let v = x.@a[0].iget;
    assert_eq(|_|"", v, 7);;

    let v = x.@a[0].imod(|n| n * 6);
    assert_eq(|_|"", v, [42]);;
    
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_index_syntax_and_function_call() {
    let source = r##"
module Main;

type A a = struct { a : a };

main : IO ();
main = (
    let x = A { a : [0] };

    let v = x.mod_a(set(0, 42))[^a][0].iget;
    assert_eq(|_|"", v, 42);;

    let v = Indexable::iget $ x.mod_a(set(0, 42))[^a][0];
    assert_eq(|_|"", v, 42);;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
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
