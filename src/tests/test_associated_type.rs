use crate::{
    configuration::Configuration,
    tests::test_util::{test_source, test_source_fail},
};

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
    test_source(&source, Configuration::develop_mode());
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
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_associated_type_equality_in_impl_context() {
    let source = r##"
module Main;

trait a : MyTraitA {
    type MyTypeA a;
    to_mytype : a -> MyTypeA a;
}

trait a : MyTraitB {
    to_i64 : a -> I64;
}

type MyWrapper a = unbox struct {
    value: a
};

impl [a : MyTraitA, MyTypeA a = I64] MyWrapper a : MyTraitB {
    to_i64 = |w: MyWrapper a| w.@value.to_mytype;
}

impl I64 : MyTraitA {
    type MyTypeA I64 = I64;
    to_mytype = |x: I64| x;
}

main: IO () = (
    let x = MyWrapper { value : 42 }.to_i64;
    assert_eq(|_|"", x, 42);;
    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_associated_type_equality_in_impl_context_unsatisfied() {
    let source = r##"
module Main;

trait a : MyTraitA {
    type MyTypeA a;
    to_mytype : a -> MyTypeA a;
}

trait a : MyTraitB {
    to_i64 : a -> I64;
}

type MyWrapper a = unbox struct {
    value: a
};

impl [a : MyTraitA, MyTypeA a = I64] MyWrapper a : MyTraitB {
    to_i64 = |w: MyWrapper a| w.@value.to_mytype;
}

impl U64 : MyTraitA {
    type MyTypeA U64 = U64;
    to_mytype = |x: U64| x;
}

main: IO () = (
    let x = MyWrapper { value : 42.u64 }.to_i64;
    assert_eq(|_|"", x, 42);;
    pure()
);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "since `Std::U64 = Std::I64` cannot be deduced",
    );
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
        Configuration::develop_mode(),
        "All appearance of associated type has to be saturated.",
    );
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
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_associated_type_in_type_sign_lacking_assumption() {
    let source = r#"
module Main;

trait a: MyTrait {
    type MyAssoc a;
}

impl I64: MyTrait {
    type MyAssoc I64 = ();
}

func : a -> MyAssoc a;
func = undefined("");

main: IO ();
main = (
    let x = func(0);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`a : Main::MyTrait` cannot be deduced",
    );
}

#[test]
pub fn test_associated_type_use_unknown_type_variable_in_associated_type_implementation() {
    let source = r#"
module Main;

trait a : MyTrait {
    type MyType a;
}

impl I64 : MyTrait {
    type MyType I64 = a;
}

main: IO () = (
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Unknown type variable `a`",
    );
}

#[test]
pub fn test_regression_issue_70() {
    let source = r#"
module Main;

trait a: MyTrait {
    type MyAssoc a;
    type MyAssoc2 a b;
}

impl I64: MyTrait {
    type MyAssoc I64 = ();
    type MyAssoc2 I64 b = MyAssoc b;
}

main: IO ();
main = (
    let f: MyAssoc2 I64 String -> () = |_| ();
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`Std::String : Main::MyTrait` cannot be deduced",
    );
}

#[test]
pub fn test_higher_kinded_associated_type() {
    let source = r#"
module Main;
 
trait [m : * -> *] m : MyTrait {
   type MyResult m : * -> *;
   some_method : m a -> IO (MyResult m a);
}
 
impl IOFail: MyTrait {
    type MyResult IOFail = Result ErrMsg;
    some_method = |iofail| iofail.to_result;
}

some_func : [m : MyTrait, f : Functor, MyResult m = f] (a -> b) -> m a -> IO (MyResult m b) = |f, ma| (
   ma.some_method.map(map(f))
);
 
main: IO () = do {
    let iof: IOFail I64 = pure(42);
    let io: IO (Result ErrMsg String) = iof.some_func(to_string);
    let expected = ok("42");
    let actual = *io.lift;
    println(actual.to_string).lift;;
    assert_eq(|_| "not eq", expected, actual).lift
}.try(exit_with_msg(1));
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_unsaturated_associated_type_in_global_function_signature() {
    let source = r#"
module Main;

trait a : MyTrait {
    type MyAssoc a;
}

func : [a : MyTrait] a -> MyAssoc;
func = |x| undefined("");

main: IO ();
main = (
    eval func(0);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "All appearance of associated type has to be saturated.",
    );
}

#[test]
pub fn test_unsaturated_associated_type_in_equality_constraint() {
    let source = r#"
module Main;

trait a : MyTrait {
    type MyAssoc a b;
}

func : [a : MyTrait, MyAssoc a = I64] a -> I64;
func = |x| undefined("");

main: IO ();
main = (
    eval func(0);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Invalid number of arguments for associated type",
    );
}

#[test]
pub fn test_unsaturated_associated_type_in_impl_rhs() {
    let source = r#"
module Main;

trait a : MyTraitA {
    type Assoc1 a;
}

trait a : MyTraitB {
    type Assoc2 a;
}

impl I64 : MyTraitA {
    type Assoc1 I64 = Assoc2;
}

impl I64 : MyTraitB {
    type Assoc2 I64 = String;
}

main: IO ();
main = pure();
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "All appearance of associated type has to be saturated.",
    );
}

#[test]
pub fn test_unsaturated_associated_type_in_type_annotation() {
    let source = r#"
module Main;

trait a : MyTrait {
    type MyAssoc a;
}

impl I64 : MyTrait {
    type MyAssoc I64 = String;
}

main: IO () = (
    let x : MyAssoc = "hello";
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "All appearance of associated type has to be saturated.",
    );
}

#[test]
pub fn test_unsaturated_multi_param_associated_type() {
    let source = r#"
module Main;

trait a : MyTrait {
    type MyAssoc a b;
}

impl I64 : MyTrait {
    type MyAssoc I64 b = b;
}

func : [a : MyTrait] a -> MyAssoc a;
func = |x| undefined("");

main: IO ();
main = (
    eval func(0);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "All appearance of associated type has to be saturated.",
    );
}

#[test]
pub fn test_unsaturated_associated_type_in_impl_lhs() {
    // Associated type `MyAssoc` is defined with 1 extra parameter (arity 2 including impl type),
    // but the implementation provides 0 extra parameters on the LHS.
    let source = r#"
    module Main;
    
    trait [a : *->*] a : MyTrait {
        type MyAssoc a b;
    }

    impl Array : MyTrait {
        type MyAssoc Array = I64;
    }

    main : IO ();
    main = pure();
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Invalid number of parameters for associated type",
    );
}

#[test]
pub fn test_higher_kinded_second_param_of_associated_type() {
    // Test that constraint syntax [f : *->*] works for 2nd+ params of associated types.
    let source = r#"
module Main;

trait a : MyTrait {
    type [f : *->*] MyAssoc a f;
}

impl I64 : MyTrait {
    type MyAssoc I64 f = f I64;
}

main: IO ();
main = (
    let x : MyAssoc I64 Array = Array::empty(0);
    let x = x.push_back(42);
    eval x.@(0);
    pure()
);
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_higher_kinded_second_param_kind_mismatch() {
    // Test that passing a *-kinded type where *->* is expected gives an error.
    let source = r#"
module Main;

trait a : MyTrait {
    type [f : *->*] MyAssoc a f;
}

impl I64 : MyTrait {
    type MyAssoc I64 f = f I64;
}

// String has kind *, but MyAssoc expects *->* for the 2nd param.
func : [a : MyTrait] MyAssoc a String -> I64;
func = |x| 0;

main: IO ();
main = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), "Kind mismatch");
}

// Tests below use opaque type syntax (`?`-prefixed type variables) and are expected
// to fail until opaque types are implemented. They verify associated type saturation
// in opaque type contexts (TODO 11 from plan1.md).

#[test]
pub fn test_opaque_unsaturated_associated_type_in_equality_lhs() {
    // `Item` requires 1 arg but is given 0 in the equality constraint.
    let source = r#"
module Main;

repeat : [?it : Iterator, Item = a] a -> I64 -> ?it;
repeat = |x, n| Iterator::range(0, n).map(|_| x);

main: IO ();
main = (
    eval repeat("hello", 3);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "The left side of an equality constraint should be the application of an associated type",
    );
}

#[test]
pub fn test_opaque_unsaturated_associated_type_in_equality_rhs() {
    // Unsaturated associated type on RHS of equality constraint with opaque type.
    let source = r#"
module Main;

trait a : MyTraitA {
    type AssocA a;
}

trait a : MyTraitB {
    type AssocB a b;
}

impl I64 : MyTraitA {
    type AssocA I64 = String;
}

impl I64 : MyTraitB {
    type AssocB I64 b = b;
}

// AssocB takes 2 args but only 1 is provided on the RHS.
func : [?t : MyTraitA, ?t : MyTraitB, AssocA ?t = AssocB ?t] ?t -> I64;
func = |x| 0;

main: IO ();
main = (
    eval func(42);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "associated type has to be saturated",
    );
}

#[test]
pub fn test_opaque_saturated_associated_type_in_equality() {
    // Properly saturated associated type with opaque type — should compile and run.
    let source = r#"
module Main;

repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
repeat = |x, n| Iterator::range(0, n).map(|_| x);

main: IO ();
main = (
    let iter = repeat("hello", 3);
    let result = iter.fold("", |s, acc| acc + s);
    assert_eq(|_|"", result, "hellohellohello");;
    pure()
);
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_opaque_multiple_associated_types_in_equality() {
    // Multiple associated types used in equality constraints with opaque type.
    // The opaque type variable ?c appears in return position (natural for opaque types),
    // and the caller uses get_elem / container_size through the trait interface.
    let source = r##"
module Main;

import Std::* hiding Indexable::Elem;

trait a : Container {
    type Elem a;
    type Size a;
    get_elem : a -> Elem a;
    container_size : a -> Size a;
}

impl Array a : Container {
    type Elem (Array a) = a;
    type Size (Array a) = I64;
    get_elem = |arr| arr.@(0);
    container_size = |arr| arr.get_size;
}

make_container : [?c : Container, Elem ?c = I64, Size ?c = I64] () -> ?c;
make_container = |_| [42];

main: IO ();
main = (
    let c = make_container();
    let elem : I64 = c.get_elem;
    let size : I64 = c.container_size;
    assert_eq(|_|"elem", elem, 42);;
    assert_eq(|_|"size", size, 1);;
    pure()
);
    "##;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_opaque_higher_arity_associated_type_in_equality() {
    // Higher-arity associated type (Rebuild c b = Array b) with opaque type.
    // The opaque type variable ?c appears in return position, and the caller
    // uses to_array through the trait interface to verify the Rebuild constraint.
    let source = r#"
module Main;

import Std::* hiding Indexable::Elem;

trait c : Rebuildable {
    type Elem c;
    type Rebuild c a;
    to_array : c -> Array (Elem c);
}

impl Array a : Rebuildable {
    type Elem (Array a) = a;
    type Rebuild (Array a) b = Array b;
    to_array = |arr| arr;
}

make_rebuildable : [?c : Rebuildable, Elem ?c = I64, Rebuild ?c b = Array b] () -> ?c;
make_rebuildable = |_| [1, 2, 3];

main: IO ();
main = (
    let c = make_rebuildable();
    let result : Array I64 = c.Rebuildable::to_array;
    assert_eq(|_|"", result, [1, 2, 3]);;
    pure()
);
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_opaque_unsaturated_higher_arity_associated_type_in_equality() {
    // Higher-arity associated type with missing argument in opaque context.
    let source = r#"
module Main;

trait c : Rebuildable {
    type Rebuild c a;
}

impl Array a : Rebuildable {
    type Rebuild (Array a) b = Array b;
}

// Rebuild takes 2 args (c + a) but only 1 is provided — unsaturated.
func : [?c : Rebuildable, Rebuild ?c = Array] ?c -> I64;
func = |x| 0;

main: IO ();
main = (
    eval func([1, 2, 3]);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Invalid number of arguments for associated type",
    );
}

#[test]
pub fn test_associated_type_namespace_qualified_impl_type() {
    let source = r#"
module Main;

type MyType = struct { data: I64 };

trait a : MyTrait {
    type MyElem a;
    get_elem : a -> MyElem a;
}

impl MyType : MyTrait {
    type MyElem Main::MyType = I64;
    get_elem = |mt| mt.@data;
}

main : IO ();
main = (
    let mt = MyType { data: 42 };
    let item: I64 = mt.get_elem;
    eval assert_eq(|_| "", item, 42);
    pure()
);
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_associated_type_wrong_namespace_impl_type() {
    let source = r#"
module Main;

type MyType = struct { data: I64 };

trait a : MyTrait {
    type MyElem a;
}

impl MyType : MyTrait {
    type MyElem Wrong::MyType = I64;
}

main : IO ();
main = pure();
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Unknown type or associated type name `Wrong::MyType`",
    );
}

#[test]
pub fn test_associated_type_namespace_qualified_wrong_impl_type() {
    // `Main::MyType2` is a real type, but the impl is for `MyType1`.
    // The namespace-qualified name resolves successfully, but the post-name-resolution
    // check in `validate_trait_impl` should catch the mismatch.
    let source = r#"
module Main;

type MyType1 = struct { data: I64 };
type MyType2 = struct { data: I64 };

trait a : MyTrait {
    type MyElem a;
}

impl MyType1 : MyTrait {
    type MyElem Main::MyType2 = I64;
}

main : IO ();
main = pure();
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "{impl_type} is `Main::MyType1`",
    );
}

#[test]
pub fn test_associated_type_mismatched_impl_type() {
    let source = r#"
module Main;

type MyType1 = struct { data: I64 };
type MyType2 = struct { data: I64 };

trait a : MyTrait {
    type MyElem a;
}

impl MyType1 : MyTrait {
    type MyElem MyType2 = I64;
}

main : IO ();
main = pure();
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "{impl_type} is `Main::MyType1`",
    );
}
