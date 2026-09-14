use crate::configuration::Configuration;
use crate::tests::test_util::{
    run_source_assert_failed, test_source, test_source_fail, test_source_fail_excludes,
    test_sources,
};

// ============================================================
// Basic use cases
// ============================================================

/// An opaque return type stands for the iterator a chain of combinators builds, and the value is
/// called at two element types.
#[test]
pub fn test_opaque_repeat() {
    let source = r#"
        module Main;

        repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
        repeat = |x, n| Iterator::range(0, n).map(|_| x);

        main : IO ();
        main = (
            let arr = repeat("hello", 3).to_array;
            assert_eq(|_|"repeat str", arr, ["hello", "hello", "hello"]);;
            let arr = repeat(42, 5).to_array;
            assert_eq(|_|"repeat int", arr, [42, 42, 42, 42, 42]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// Three combinators chained behind one opaque type, so the signature names none of the iterator
/// types the chain builds.
#[test]
pub fn test_opaque_doubled_evens() {
    let source = r#"
        module Main;

        doubled_evens : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        doubled_evens = |n| Iterator::range(0, n).filter(|x| x % 2 == 0).map(|x| x * 2);

        main : IO ();
        main = (
            let arr = doubled_evens(6).to_array;
            assert_eq(|_|"doubled_evens", arr, [0, 4, 8]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A trait member returns an opaque type whose items are the trait's associated type, and the
/// implementation for `Array a` gives it the iterator `Array::to_iter` returns.
#[test]
pub fn test_opaque_to_iter() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : ToIter {
            type Elem c;
            to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it;
        }

        impl Array a : ToIter {
            type Elem (Array a) = a;
            to_iter = Array::to_iter;
        }

        main : IO ();
        main = (
            let arr = [1, 2, 3].ToIter::to_iter.to_array;
            assert_eq(|_|"to_iter", arr, [1, 2, 3]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// Two implementations of one member with an opaque return type each hide an iterator type of
/// their own, and one program reaches both.
#[test]
pub fn test_opaque_to_iter_multiple_impls() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : ToIter {
            type Elem c;
            to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it;
        }

        impl Array a : ToIter {
            type Elem (Array a) = a;
            to_iter = Array::to_iter;
        }

        type MyRange = box struct { start : I64, end_ : I64 };

        impl MyRange : ToIter {
            type Elem MyRange = I64;
            to_iter = |r| Iterator::range(r.@start, r.@end_);
        }

        main : IO ();
        main = (
            let arr_result = [10, 20, 30].ToIter::to_iter.to_array;
            assert_eq(|_|"array to_iter", arr_result, [10, 20, 30]);;
            let range_result = (MyRange { start : 0, end_ : 4 }).ToIter::to_iter.to_array;
            assert_eq(|_|"myrange to_iter", range_result, [0, 1, 2, 3]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// An opaque type of kind `* -> *` constrained by `Monad`: the body returns `Option I64`, and the
/// use site chains the calls through `bind`.
#[test]
pub fn test_opaque_higher_kinded() {
    let source = r#"
        module Main;

        safe_div : [?m : * -> *, ?m : Monad] I64 -> I64 -> ?m I64;
        safe_div = |x, y| if y == 0 { none() } else { some(x / y) };

        main : IO ();
        main = (
            // Chain safe_div through bind to verify Monad interface
            let result = safe_div(100, 10).bind(|x| safe_div(x, 2));
            let result = result.bind(|x| safe_div(x, 0));
            let _ = result;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// The kind of an opaque type is taken from the trait it is constrained by, so a higher-kinded
/// opaque type needs no kind signature of its own.
#[test]
pub fn test_opaque_higher_kinded_without_a_kind_signature() {
    let source = r#"
        module Main;

        safe_div : [?m : Monad] I64 -> I64 -> ?m I64;
        safe_div = |x, y| if y == 0 { none() } else { some(x / y) };

        main : IO ();
        main = (
            let result = safe_div(100, 10).bind(|x| safe_div(x, 2));
            let _ = result;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A signature mixing an ordinary type variable constrained by `Iterator` with an opaque one: the
/// iterator the value takes is the caller's choice and the one it returns is the body's.
#[test]
pub fn test_opaque_zip_with_index() {
    let source = r#"
        module Main;

        zip_with_index : [it_in : Iterator, Item it_in = a, ?it_out : Iterator, Item ?it_out = (I64, a)] it_in -> ?it_out;
        zip_with_index = |iter| iter.enumerate;

        main : IO ();
        main = (
            let arr = zip_with_index(Iterator::range(0, 3).map(|x| x * 10)).to_array;
            assert_eq(|_|"zip_with_index", arr, [(0, 0), (1, 10), (2, 20)]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// Two opaque types under the same constraints stand for the two iterators one value returns, and
/// each is collected on its own.
#[test]
pub fn test_opaque_partition() {
    let source = r#"
        module Main;

        partition : [?evens : Iterator, Item ?evens = a, ?odds : Iterator, Item ?odds = a]
                    (a -> Bool) -> Array a -> (?evens, ?odds);
        partition = |pred, arr| (arr.to_iter.filter(pred), arr.to_iter.filter(|x| pred(x).not));

        main : IO ();
        main = (
            let (evens, odds) = partition(|x| x % 2 == 0, [1, 2, 3, 4, 5]);
            assert_eq(|_|"evens", evens.to_array, [2, 4]);;
            assert_eq(|_|"odds", odds.to_array, [1, 3, 5]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// An opaque type carrying a trait constraint alone: the use site reaches the value behind it
/// through `ToString`.
#[test]
pub fn test_opaque_predicate_only() {
    let source = r#"
        module Main;

        to_string_opaque : [?s : ToString] I64 -> ?s;
        to_string_opaque = |n| n.to_string;

        main : IO ();
        main = (
            let s = to_string_opaque(42);
            // Use through ToString interface to get a concrete String
            let result = s.to_string;
            assert_eq(|_|"predicate only", result, "42");;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

// ============================================================
// A trait member's type has to fix the trait's type variable
// ============================================================

/// A member whose declared type leaves the trait's type variable to a constraint on an opaque type
/// variable. What stands behind the opaque type is the implementation's choice, so a use site has
/// nothing to pick the implementation by.
#[test]
pub fn test_opaque_constraint_alone_does_not_fix_the_trait_variable() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = c] I64 -> ?it;
        }

        impl I64 : Make {
            make = |n| Iterator::range(0, n);
        }

        main : IO ();
        main = (
            let is : Array I64 = Make::make(3).to_array;
            println(is.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type variable `c` is not fixed by this type signature",
    );
}

/// The trait's type variable stands in an argument of the member's type, and an opaque constraint
/// names it as well. The argument is what a use site picks the implementation by, so the member is
/// accepted and each implementation hides a concrete type of its own.
#[test]
pub fn test_opaque_constraint_beside_the_trait_variable_in_the_type() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : ToIter {
            type Elem c;
            to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it;
        }

        impl Array a : ToIter {
            type Elem (Array a) = a;
            to_iter = Array::to_iter;
        }

        type MyRange = box struct { start : I64, end_ : I64 };

        impl MyRange : ToIter {
            type Elem MyRange = I64;
            to_iter = |r| Iterator::range(r.@start, r.@end_);
        }

        main : IO ();
        main = (
            assert_eq(|_|"array", [10, 20].ToIter::to_iter.to_array, [10, 20]);;
            assert_eq(|_|"range", (MyRange { start : 0, end_ : 3 }).ToIter::to_iter.to_array, [0, 1, 2]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// A member that returns the type the trait is implemented for. The trait's type variable stands in
/// the result, which is what a use site picks the implementation by.
#[test]
pub fn test_the_trait_variable_in_the_result_fixes_it() {
    let source = r##"
        module Main;

        trait c : FromCount {
            from_count : I64 -> c;
        }

        impl Array I64 : FromCount {
            from_count = |n| Iterator::range(0, n).to_array;
        }

        main : IO ();
        main = (
            let xs : Array I64 = FromCount::from_count(3);
            assert_eq(|_|"from_count", xs, [0, 1, 2]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// A member whose type names the trait's type variable only as an argument of an associated type
/// application. `Ele c` at a use site does not say which type `c` stands for, so the member's type
/// does not fix it, although the equality's right side names it as well.
#[test]
pub fn test_associated_type_application_does_not_fix_the_trait_variable() {
    let source = r##"
        module Main;

        trait c : Make {
            type Ele c;
            make : [?it : Iterator, Item ?it = c] Ele c -> ?it;
        }

        impl I64 : Make {
            type Ele I64 = I64;
            make = |_| Iterator::range(0, 1);
        }

        main : IO ();
        main = println("ok");
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type variable `c` is not fixed by this type signature",
    );
}

/// Each member that leaves the trait's type variable out is reported, in every trait the program
/// declares, in one compilation.
#[test]
pub fn test_every_member_leaving_the_trait_variable_out_is_reported() {
    let source = r##"
        module Main;

        trait c : ZapA {
            zap_a : I64 -> I64;
        }

        trait c : ZapB {
            zap_b : I64 -> I64;
        }

        impl I64 : ZapA { zap_a = |x| x; }
        impl I64 : ZapB { zap_b = |x| x; }

        main : IO ();
        main = println("ok");
    "##;
    let errmsg = run_source_assert_failed(&source, Configuration::develop_mode());
    assert_eq!(
        errmsg
            .matches("Type variable `c` is not fixed by this type signature")
            .count(),
        2,
        "both members are expected to be reported.\nActual message:\n{}",
        errmsg
    );
}

/// Two implementations of a trait whose member names the trait's type variable in an argument and
/// constrains the opaque result by that same variable. Each implementation stands for a concrete
/// type of its own, so one program reaches both.
#[test]
pub fn test_opaque_constraint_naming_the_trait_variable_directly() {
    let source = r##"
        module Main;

        trait c : ToI {
            to_i : [?it : Iterator, Item ?it = c] Array c -> ?it;
        }

        impl I64 : ToI {
            to_i = |arr| arr.to_iter;
        }

        impl Bool : ToI {
            to_i = |arr| arr.to_iter;
        }

        main : IO ();
        main = (
            assert_eq(|_|"i64", [1, 2, 3].to_i.to_array, [1, 2, 3]);;
            assert_eq(|_|"bool", [true, false].to_i.to_array, [true, false]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

// ============================================================
// An opaque type annotating an expression of an impl method that writes no type signature
// ============================================================

/// An implementation that writes no type signature annotates an expression of its body with the
/// opaque type the declaration writes. The names a declaration writes stand in the declaration
/// alone, so the annotation names an unknown type variable.
#[test]
pub fn test_opaque_in_impl_annotation() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : ToIter {
            type Elem c;
            to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it;
        }

        impl Array a : ToIter {
            type Elem (Array a) = a;
            to_iter = |x| (x.Array::to_iter : ?it);
        }

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Unknown type variable `?it`",
    );
}

// ============================================================
// An opaque type annotating an expression of an impl method that writes a type signature
// ============================================================

/// An implementation that writes a type signature of its own annotates an expression of its body
/// with the opaque type that signature writes. The annotation names an unknown type variable.
#[test]
pub fn test_opaque_in_impl_annotation_with_sig() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : ToIter {
            type Elem c;
            to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it;
        }

        impl Array a : ToIter {
            type Elem (Array a) = a;
            to_iter : [?iter : Iterator, Item ?iter = a] Array a -> ?iter;
            to_iter = |x| (x.Array::to_iter : ?iter);
        }

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Unknown type variable `?iter`",
    );
}

// ============================================================
// A type signature on an impl method that agrees with the declaration
// ============================================================

/// An implementation writes the member's type under type variable names of its own, `?iter` where
/// the declaration writes `?it`, and the program compiles and runs.
#[test]
pub fn test_opaque_impl_method_type_sig() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : ToIter {
            type Elem c;
            to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it;
        }

        impl Array a : ToIter {
            type Elem (Array a) = a;
            to_iter : [?iter : Iterator, Item ?iter = a] Array a -> ?iter;
            to_iter = Array::to_iter;
        }

        main : IO ();
        main = (
            let arr = [1, 2, 3].ToIter::to_iter.to_array;
            assert_eq(|_|"impl method sig", arr, [1, 2, 3]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// The member's type carries a type variable beside the trait's, and the implementation writes
/// that variable under a name of its own in both places it stands: the function the member takes,
/// and the equality on the items of the opaque type.
#[test]
pub fn test_opaque_impl_method_type_sig_renamed_vars() {
    let source = r##"
        module Main;

        trait c : MyTrait {
            my_map : [?it : Iterator, Item ?it = b] (c -> b) -> Array c -> ?it;
        }

        impl I64 : MyTrait {
            my_map : [?out : Iterator, Item ?out = d] (I64 -> d) -> Array I64 -> ?out;
            my_map = |f, arr| arr.Array::to_iter.map(f);
        }

        main : IO ();
        main = (
            let arr = [1, 2, 3].my_map(|x| x.to_string).to_array;
            assert_eq(|_|"renamed vars", arr, ["1", "2", "3"]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// A type variable of kind `* -> *` standing beside an opaque type becomes a type argument of the
/// opaque type constructor, which then has kind `(* -> *) -> *`.
#[test]
pub fn test_opaque_tycon_takes_a_higher_kinded_type_argument() {
    let source = r#"
        module Main;

        wrap_each : [m : Monad, ?it : Iterator, Item ?it = m I64] I64 -> ?it;
        wrap_each = |n| Iterator::range(0, n).map(|x| pure(x));

        main : IO ();
        main = (
            let opts : Array (Option I64) = wrap_each(3).to_array;
            assert_eq(|_|"wrapped in Option", opts, [some(0), some(1), some(2)]);;
            let arrs : Array (Array I64) = wrap_each(2).to_array;
            assert_eq(|_|"wrapped in Array", arrs, [[0], [1]]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A trait whose member has an opaque result, declared in one module, implemented in another and
/// called from a third. Each implementation's concrete type is found where that implementation is
/// written, and the call site reaches it across the module boundary.
#[test]
pub fn test_opaque_member_implemented_in_another_module() {
    let lib = r#"
        module Lib;

        trait c : Coll {
            type Ele c;
            items : [?it : Iterator, Iterator::Item ?it = Ele c] c -> ?it;
        }

        repeat_n : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
        repeat_n = |x, n| Iterator::range(0, n).map(|_| x);
    "#;
    let impls = r#"
        module Impls;

        import Lib;

        type Pair a = struct { x : a, y : a };
        type Trip a = unbox struct { a : a, b : a, c : a };

        impl Pair a : Lib::Coll {
            type Ele (Pair a) = a;
            items = |p| [p.@x, p.@y].to_iter;
        }

        impl Trip a : Lib::Coll {
            type Ele (Trip a) = a;
            items = |t| [t.@a, t.@b, t.@c].to_iter;
        }
    "#;
    let main = r#"
        module Main;

        import Lib;
        import Impls;

        main : IO ();
        main = (
            assert_eq(|_|"pair", Impls::Pair { x : 1, y : 2 }.Coll::items.to_array, [1, 2]);;
            assert_eq(|_|"trip", Impls::Trip { a : 3, b : 4, c : 5 }.Coll::items.to_array, [3, 4, 5]);;
            assert_eq(|_|"repeat_n", Lib::repeat_n("z", 2).to_array, ["z", "z"]);;
            pure()
        );
    "#;
    test_sources(&[lib, impls, main], Configuration::develop_mode());
}

/// Two members of one trait, each hiding a type of its own behind an opaque type. One
/// implementation gives the two different concrete types, and a use of either reaches its own.
#[test]
pub fn test_opaque_two_members_of_one_trait_resolve_to_two_types() {
    let source = r##"
        module Main;

        trait c : Two {
            fst : [?a : ToString] c -> ?a;
            snd : [?b : ToString] c -> ?b;
        }

        impl I64 : Two {
            fst = |n| n;
            snd = |n| n > 0;
        }

        main : IO ();
        main = (
            assert_eq(|_|"two members", 3.fst.to_string, "3");;
            assert_eq(|_|"two members", 3.snd.to_string, "true");;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// The implementation writes the opaque type under the name the declaration writes it under, so
/// that the comparison of the two signatures has one name standing in both of them.
#[test]
pub fn test_opaque_impl_method_type_sig_same_opaque_name() {
    let source = r##"
        module Main;

        trait c : ToIter {
            to_iter : [?it : Iterator, Item ?it = I64] c -> ?it;
        }

        type Odd = box struct { n : I64 };

        impl Odd : ToIter {
            to_iter : [?it : Iterator, Item ?it = I64] Odd -> ?it;
            to_iter = |o| Iterator::range(0, o.@n);
        }

        main : IO ();
        main = (
            let arr = Odd { n : 3 }.to_iter.to_array;
            assert_eq(|_|"same opaque name", arr, [0, 1, 2]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// The `e` of the declaration stands in its constraints and nowhere in the member's type, so the
/// implementation writes no type for it. An implementation that writes no type signature has
/// nothing to disagree with, so what is reported is the type the body leaves undetermined.
#[test]
pub fn test_opaque_member_with_a_type_variable_of_its_constraints_alone() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = e, e : ToString] c -> ?it;
        }

        impl Bool : Make {
            make = |_| Iterator::range(0, 3);
        }

        main : IO ();
        main = (
            let is : Array I64 = Make::make(true).to_array;
            println(is.to_string)
        );
    "##;
    test_source_fail_excludes(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation",
    );
}

// ============================================================
// A type signature on an impl method that disagrees with the declaration
// ============================================================

/// The implementation writes the type of the iterator it returns where the declaration writes
/// the opaque type, which the desugaring of opaque types has no reading for.
#[test]
pub fn test_opaque_impl_method_type_sig_writes_a_concrete_type() {
    let source = r##"
        module Main;

        trait c : ToIter {
            to_iter : [?it : Iterator, Item ?it = I64] c -> ?it;
        }

        type Odd = box struct { n : I64 };

        impl Odd : ToIter {
            to_iter : Odd -> RangeIterator;
            to_iter = |o| Iterator::range(0, o.@n);
        }

        main : IO ();
        main = println(Odd { n : 3 }.to_iter.to_array.to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation writes `Std::Iterator::RangeIterator` where the trait definition writes the opaque type `?it`.",
    );
}

/// The implementation writes the type of the iterator it returns where the declaration writes the
/// opaque type, in a trait whose opaque type is constrained by an associated type of its own and
/// whose implementation is for a type constructor applied to a type variable.
#[test]
pub fn test_opaque_impl_method_type_sig_writes_a_concrete_type_under_an_associated_type() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : ToIter {
            type Elem c;
            to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it;
        }

        impl Array a : ToIter {
            type Elem (Array a) = a;
            to_iter : Array a -> ArrayIterator a;
            to_iter = Array::to_iter;
        }

        main : IO ();
        main = (
            let arr = [1, 2, 3].ToIter::to_iter.to_array;
            println(arr.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation writes `Std::Iterator::ArrayIterator a` where the trait definition writes the opaque type `?it`.",
    );
}

/// The declaration hides two types behind two opaque types, and the implementation writes one
/// opaque type for both. Each opaque type of a declaration stands for one type an
/// implementation returns, so one for two is a statement the declaration does not make.
#[test]
pub fn test_opaque_impl_method_type_sig_writes_one_opaque_type_for_two() {
    let source = r##"
        module Main;

        trait c : Two {
            two : [?a : ToString, ?b : ToString] c -> (?a, ?b);
        }

        impl I64 : Two {
            two : [?x : ToString] I64 -> (?x, ?x);
            two = |n| (n.to_string, n.to_string);
        }

        main : IO ();
        main = (
            let (a, b) = 3.two;
            println(a.to_string + b.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation writes one opaque type `?x` for two opaque types of the trait definition, `?a` and `?b`.",
    );
}

/// The implementation writes a type variable where the declaration writes `I64`. The report
/// prints the two signatures as they are written, opaque types and all.
#[test]
pub fn test_opaque_impl_method_type_sig_is_more_general_than_the_declaration() {
    let source = r##"
        module Main;

        trait c : ToIter {
            to_iter : [?it : Iterator, Item ?it = I64] c -> I64 -> ?it;
        }

        type Odd = box struct { n : I64 };

        impl Odd : ToIter {
            to_iter : [?j : Iterator, Item ?j = I64] Odd -> a -> ?j;
            to_iter = |o, _| Iterator::range(0, o.@n);
        }

        main : IO ();
        main = println(Odd { n : 3 }.to_iter(0).to_array.to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation does not match trait definition.\nExpected: `[?it : Std::Iterator, Std::Iterator::Item ?it = Std::I64] Main::Odd -> Std::I64 -> ?it`\nFound: `[?j : Std::Iterator, Std::Iterator::Item ?j = Std::I64] Main::Odd -> a -> ?j`",
    );
}

/// The implementation states that the iterator it returns has `Bool` elements where the
/// declaration states `I64`, and returns an iterator of `I64`. The constraints an
/// implementation writes on an opaque type are compared with the declaration's.
#[test]
pub fn test_opaque_impl_method_type_sig_constrains_item_to_another_type() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = I64] c -> ?it;
        }

        impl Bool : Make {
            make : [?jt : Iterator, Item ?jt = Bool] Bool -> ?jt;
            make = |_| Iterator::range(0, 3);
        }

        main : IO ();
        main = (
            let is : Array I64 = Make::make(true).to_array;
            println(is.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation does not match trait definition.\nExpected: `[?it : Std::Iterator, Std::Iterator::Item ?it = Std::I64] Std::Bool -> ?it`\nFound: `[?jt : Std::Iterator, Std::Iterator::Item ?jt = Std::Bool] Std::Bool -> ?jt`",
    );
}

/// The implementation writes an opaque type and none of the constraints the declaration puts on
/// it, which says less about the type it returns than the declaration does.
#[test]
pub fn test_opaque_impl_method_type_sig_omits_the_constraints() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = I64] c -> ?it;
        }

        impl Bool : Make {
            make : Bool -> ?jt;
            make = |_| Iterator::range(0, 3);
        }

        main : IO ();
        main = (
            let is : Array I64 = Make::make(true).to_array;
            println(is.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation does not match trait definition.\nExpected: `[?it : Std::Iterator, Std::Iterator::Item ?it = Std::I64] Std::Bool -> ?it`\nFound: `Std::Bool -> ?jt`",
    );
}

/// The implementation writes an ordinary type variable where the declaration writes the opaque
/// type. What the declaration hides is the implementation's own type, and an implementation that
/// writes a variable of its own there would let a caller choose that type instead.
#[test]
pub fn test_opaque_impl_method_type_sig_writes_a_type_variable_of_its_own() {
    let source = r##"
        module Main;

        trait c : Two {
            two : [?a : ToString] c -> ?a;
        }

        impl I64 : Two {
            two : [x : ToString] I64 -> x;
            two = |n| n.to_string;
        }

        main : IO ();
        main = println(3.two.to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation writes `x` where the trait definition writes the opaque type `?a`.",
    );
}

/// The declaration says what the iterator's elements are through a second opaque type, and the
/// implementation states `Bool` of them. An opaque type of the declaration stands for one type
/// the implementation chooses, so a signature stating something else about it is reported, as it
/// is where the declaration names the element type itself.
#[test]
pub fn test_opaque_impl_method_type_sig_constrains_item_to_another_type_through_an_opaque_type() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = ?jt, ?jt : ToString] c -> ?it;
        }

        impl Bool : Make {
            make : [?p : Iterator, Item ?p = Bool] Bool -> ?p;
            make = |_| Iterator::range(0, 3);
        }

        main : IO ();
        main = println(Make::make(true).to_array.to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation does not match trait definition.",
    );
}

/// Two implementations of one member, each writing a signature that disagrees with the
/// declaration in its own way. Each implementation is read on its own, and one compilation
/// reports both.
#[test]
pub fn test_opaque_impl_method_type_sigs_are_all_reported_in_one_compilation() {
    let source = r##"
        module Main;

        trait c : ToIter {
            to_iter : [?it : Iterator, Item ?it = I64] c -> ?it;
        }

        type Odd = box struct { n : I64 };
        type Even = box struct { n : I64 };

        impl Odd : ToIter {
            to_iter : Odd -> RangeIterator;
            to_iter = |o| Iterator::range(0, o.@n);
        }

        impl Even : ToIter {
            to_iter : [?j : Iterator, Item ?j = Bool] Even -> ?j;
            to_iter = |e| Iterator::range(0, e.@n);
        }

        main : IO ();
        main = println(Odd { n : 3 }.to_iter.to_array.to_string);
    "##;
    let errmsg = run_source_assert_failed(&source, Configuration::develop_mode());
    assert!(
        errmsg.contains("writes `Std::Iterator::RangeIterator` where the trait definition writes the opaque type `?it`"),
        "the implementation for `Odd` is reported, but the message is:\n{}",
        errmsg
    );
    assert!(
        errmsg.contains("Main::Even -> ?j`"),
        "the implementation for `Even` is reported, but the message is:\n{}",
        errmsg
    );
}

/// The declaration hides two types behind two opaque types, and the implementation writes an
/// opaque type variable of its own for each, naming them in the other order. Which opaque type
/// of the declaration each one stands for is read off the type the signature writes.
#[test]
pub fn test_opaque_impl_method_type_sig_writes_two_opaque_types_in_the_other_order() {
    let source = r##"
        module Main;

        trait c : Two {
            two : [?a : ToString, ?b : Iterator, Item ?b = I64] c -> (?a, ?b);
        }

        impl I64 : Two {
            two : [?q : Iterator, Item ?q = I64, ?p : ToString] I64 -> (?p, ?q);
            two = |n| (n.to_string, Iterator::range(0, n));
        }

        main : IO ();
        main = (
            let (s, it) = 3.two;
            assert_eq(|_|"two opaque types", s.to_string, "3");;
            assert_eq(|_|"two opaque types", it.to_array, [0, 1, 2]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// The implementation writes `I64` where the declaration writes `Elem c`, which the
/// implementation of `Elem` gives that type. The two signatures describe the same values, and
/// the desugaring of opaque types reads one as the other written with its type variables
/// replaced, which this is not.
#[test]
pub fn test_opaque_impl_method_type_sig_reduces_an_associated_type() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : Coll {
            type Elem c;
            iter_with : [?it : Iterator, Item ?it = Elem c] Elem c -> c -> ?it;
        }

        type MyArr = box struct { xs : Array I64 };

        impl MyArr : Coll {
            type Elem MyArr = I64;
            iter_with : [?jt : Iterator, Item ?jt = I64] I64 -> MyArr -> ?jt;
            iter_with = |x, m| m.@xs.to_iter.push_front(x);
        }

        main : IO ();
        main = (
            let m = MyArr { xs : [1, 2] };
            println(Coll::iter_with(0, m).to_array.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Type signature in implementation is not the type of the trait definition with its type variables replaced.\nExpected: `Main::Coll::Elem Main::MyArr -> Main::MyArr -> ?it`\nFound: `Std::I64 -> Main::MyArr -> ?jt`",
    );
}

// ============================================================
// Higher-kinded opaque types
// ============================================================

/// An opaque type of kind `* -> *` constrained by `Functor`: the use site calls `map` on what the
/// value returns.
#[test]
pub fn test_opaque_higher_kinded_functor() {
    let source = r#"
        module Main;

        make_singleton : [?f : * -> *, ?f : Functor] a -> ?f a;
        make_singleton = |x| [x];

        main : IO ();
        main = (
            let xs = make_singleton(42);
            let ys = xs.map(|x| x * 2);
            let zs = ys.map(|x| x + 1);
            // Verify the computation runs (can't compare opaque with concrete)
            let _ = zs;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

// ============================================================
// Associated types behind an opaque type
// ============================================================

/// The equality `Item ?it = a` carries the element type to a use site that sums what the value
/// returns.
#[test]
pub fn test_opaque_with_associated_type_basic() {
    let source = r#"
        module Main;

        repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
        repeat = |x, n| Iterator::range(0, n).map(|_| x);

        sum_repeat : [a : Additive] a -> I64 -> a;
        sum_repeat = |x, n| repeat(x, n).sum;

        main : IO ();
        main = (
            assert_eq(|_|"sum_repeat", sum_repeat(3, 4), 12);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A use site reduces `Item ?it` to `I64` to give the closure `fold` takes its argument types.
#[test]
pub fn test_opaque_associated_type_reduction() {
    let source = r#"
        module Main;

        make_range : [?it : Iterator, Item ?it = I64] I64 -> I64 -> ?it;
        make_range = |start, end_| Iterator::range(start, end_);

        main : IO ();
        main = (
            let iter = make_range(0, 5);
            // fold uses Item ?it = I64 to determine closure arg types
            let result = iter.fold(0, |item, acc| acc + item);
            assert_eq(|_|"reduction", result, 10);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A use site reduces an associated type of two arguments, `Rebuild ?c String`, to `Array String`
/// through the equality the opaque type carries.
#[test]
pub fn test_opaque_with_higher_arity_assoc_type() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : Rebuildable {
            type Elem c;
            type Rebuild c a;
            rebuild : (Elem c -> a) -> c -> Rebuild c a;
        }

        impl Array a : Rebuildable {
            type Elem (Array a) = a;
            type Rebuild (Array a) b = Array b;
            rebuild = |f, arr| arr.map(f);
        }

        from_array : [?c : Rebuildable, Elem ?c = a, Rebuild ?c b = Array b] Array a -> ?c;
        from_array = |arr| arr;

        main : IO ();
        main = (
            let c = Main::from_array([1, 2, 3]);
            // Rebuild (?c I64) String should reduce to Array String
            let result = c.rebuild(|x| x.to_string);
            assert_eq(|_|"higher arity", result, ["1", "2", "3"]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// An associated type of kind `* -> *` behind an opaque type: `Repr ?fmt` is applied to `I64` and
/// to `String`, and the equality `Repr ?fmt = Array` reduces both.
#[test]
pub fn test_opaque_with_higher_kinded_assoc_type() {
    let source = r##"
        module Main;

        trait fmt : Format {
            type Repr fmt : * -> *;
            format_value : a -> fmt -> Repr fmt a;
        }

        impl () : Format {
            type Repr () = Array;
            format_value = |x, _| [x];
        }

        default_format : [?fmt : Format, Repr ?fmt = Array] () -> ?fmt;
        default_format = |_| ();

        wrap_pair : [fmt : Format] fmt -> a -> b -> (Repr fmt a, Repr fmt b);
        wrap_pair = |fmt, x, y| (format_value(x, fmt), format_value(y, fmt));

        main : IO ();
        main = (
            let fmt = default_format();
            let (xs, ys) = wrap_pair(fmt, 42, "hello");
            // Repr ?fmt I64 = Array I64, Repr ?fmt String = Array String
            assert_eq(|_|"hk int", xs, [42]);;
            assert_eq(|_|"hk str", ys, ["hello"]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// Two opaque types whose equalities name one element type stand for the two iterators one value
/// returns.
#[test]
pub fn test_opaque_multi_opaque_with_shared_assoc_type() {
    let source = r#"
        module Main;

        merge_iters : [?it1 : Iterator, Item ?it1 = a, ?it2 : Iterator, Item ?it2 = a]
                      Array a -> Array a -> (?it1, ?it2);
        merge_iters = |arr1, arr2| (arr1.to_iter, arr2.to_iter);

        main : IO ();
        main = (
            let (it1, it2) = merge_iters([1, 2], [3, 4]);
            let sum1 = it1.fold(0, |item, acc| acc + item);
            let sum2 = it2.fold(0, |item, acc| acc + item);
            assert_eq(|_|"shared assoc 1", sum1, 3);;
            assert_eq(|_|"shared assoc 2", sum2, 7);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

// ============================================================
// Multiple calls of one value with an opaque return type
// ============================================================

/// One value with an opaque return type is called at two element types, and each call collects
/// what it returns.
#[test]
pub fn test_opaque_multiple_calls_different_type_args() {
    let source = r#"
        module Main;

        repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
        repeat = |x, n| Iterator::range(0, n).map(|_| x);

        main : IO ();
        main = (
            let str_arr = repeat("hello", 3).to_array;
            let int_arr = repeat(42, 5).to_array;
            assert_eq(|_|"str repeat", str_arr, ["hello", "hello", "hello"]);;
            assert_eq(|_|"int repeat", int_arr, [42, 42, 42, 42, 42]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// Two calls of one value at the same type arguments have one type, so what they return goes into
/// one array.
#[test]
pub fn test_opaque_multiple_calls_same_type_args() {
    let source = r#"
        module Main;

        repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
        repeat = |x, n| Iterator::range(0, n).map(|_| x);

        main : IO ();
        main = (
            let a = repeat("a", 3);
            let b = repeat("b", 2);
            // Both have the same opaque type; put them in an array and collect sizes
            let count = [a, b].map(|it| it.to_array.get_size).to_iter.sum;
            assert_eq(|_|"same type args", count, 5);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

// ============================================================
// Where an opaque type variable may stand
// ============================================================

/// An opaque type variable as a parameter of a type definition is rejected.
#[test]
pub fn test_opaque_in_type_defn() {
    let source = r#"
        module Main;

        type Foo ?a = box struct { val : ?a };

        main : IO ();
        main = pure();
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "is not allowed in a type definition",
    );
}

/// An opaque type variable as the type variable of a trait definition is rejected.
#[test]
pub fn test_opaque_in_trait_defn() {
    let source = r#"
        module Main;

        trait ?a : Foo {
            bar : ?a -> ?a;
        }

        main : IO ();
        main = pure();
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "is not allowed in a trait definition",
    );
}

/// An opaque type variable as the type a trait is implemented for is rejected.
#[test]
pub fn test_opaque_in_impl_type_param() {
    let source = r#"
        module Main;

        trait a : Foo {
            bar : a -> a;
        }

        impl ?x : Foo {
            bar = |x| x;
        }

        main : IO ();
        main = pure();
    "#;
    // The exact error message may vary; we expect some kind of rejection
    test_source_fail(&source, Configuration::develop_mode(), "is not allowed");
}

/// An opaque type variable standing as an extra argument of an equality on another type is
/// rejected. `Rebuild c ?s = Array I64` is on `c`, so meeting it falls to the use site, which
/// never learns what `?s` stands for.
#[test]
pub fn test_opaque_tyvar_in_extra_argument_of_equality_on_another_type() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : Rebuildable {
            type Elem c;
            type Rebuild c a;
            rebuild : (Elem c -> a) -> c -> Rebuild c a;
        }

        impl Array a : Rebuildable {
            type Elem (Array a) = a;
            type Rebuild (Array a) b = Array b;
            rebuild = |f, arr| arr.map(f);
        }

        // `?s` sits in an extra argument of `Rebuild c ?s = Array I64`, an equality on `c`
        foo : [?s : ToString, c : Rebuildable, Elem c = I64, Rebuild c ?s = Array I64] c -> ?s;
        foo = |x| (
            let rebuilt = x.rebuild(|n| n == 0);
            let arr : Array I64 = rebuilt;
            arr.@(0).to_string
        );

        main : IO ();
        main = println(foo([1, 2, 3]).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "The first argument of the left side of an equality constraint involving an opaque type \
         should be an opaque type.",
    );
}

/// An opaque type variable inside an extra argument of an equality on another type is rejected.
/// `Rebuild c (Array ?s) = Array I64` is on `c`, and `?s` stands one level down, inside
/// `Array ?s`.
#[test]
pub fn test_opaque_tyvar_nested_in_extra_argument_of_equality_on_another_type() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : Rebuildable {
            type Elem c;
            type Rebuild c a;
            rebuild : (Elem c -> a) -> c -> Rebuild c a;
        }

        impl Array a : Rebuildable {
            type Elem (Array a) = a;
            type Rebuild (Array a) b = Array b;
            rebuild = |f, arr| arr.map(f);
        }

        // `?s` sits inside an extra argument of `Rebuild c (Array ?s) = Array I64`, an equality on `c`
        foo : [?s : ToString, c : Rebuildable, Elem c = I64, Rebuild c (Array ?s) = Array I64] c -> ?s;
        foo = |x| (
            let rebuilt = x.rebuild(|n| [n.to_string]);
            let arr : Array I64 = rebuilt;
            arr.@(0).to_string
        );

        main : IO ();
        main = println(foo([1, 2, 3]).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "The first argument of the left side of an equality constraint involving an opaque type \
         should be an opaque type.",
    );
}

/// An opaque type variable on the right side of an equality on another type is rejected.
/// `Rebuild c Bool = ?s` is on `c`, and `?s` stands as the type the equality says
/// `Rebuild c Bool` is.
#[test]
pub fn test_opaque_tyvar_on_right_side_of_equality_on_another_type() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : Rebuildable {
            type Elem c;
            type Rebuild c a;
            rebuild : (Elem c -> a) -> c -> Rebuild c a;
        }

        impl Array a : Rebuildable {
            type Elem (Array a) = a;
            type Rebuild (Array a) b = Array b;
            rebuild = |f, arr| arr.map(f);
        }

        // `?s` stands on the right side of `Rebuild c Bool = ?s`, an equality on `c`
        foo : [?s : ToString, c : Rebuildable, Elem c = I64, Rebuild c Bool = ?s] c -> ?s;
        foo = |x| x.rebuild(|n| n == 0);

        main : IO ();
        main = println(foo([1, 2, 3]).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "The first argument of the left side of an equality constraint involving an opaque type \
         should be an opaque type.",
    );
}

// ============================================================
// The formal parameters of an equality constraint
// ============================================================

/// The extra arguments on the left side of an equality constraint are its formal parameters, and a
/// concrete type written in one of those places is rejected.
#[test]
pub fn test_opaque_equality_non_tyvar_formal_param() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : Rebuildable {
            type Elem c;
            type Rebuild c a;
            rebuild : (Elem c -> a) -> c -> Rebuild c a;
        }

        // I64 is a concrete type in the extra argument position
        foo : [?c : Rebuildable, Elem ?c = a, Rebuild ?c I64 = Array I64] Array a -> ?c;
        foo = |arr| arr;

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "must be type variables",
    );
}

/// A formal parameter of an equality constraint that also stands in the type the signature writes
/// is rejected.
#[test]
pub fn test_opaque_equality_formal_param_in_ty_body() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : Rebuildable {
            type Elem c;
            type Rebuild c a;
            rebuild : (Elem c -> a) -> c -> Rebuild c a;
        }

        // 'b' appears in both the equality and the type body
        foo : [?c : Rebuildable, Elem ?c = a, Rebuild ?c b = Array b] Array a -> b -> ?c;
        foo = |arr, x| arr;

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "must not appear elsewhere in the type signature",
    );
}

// ============================================================
// The concrete type behind an opaque type is left undetermined
// ============================================================

/// An opaque type standing nowhere in the value's type leaves the body nothing to determine it by,
/// and is rejected.
#[test]
pub fn test_opaque_unused_cannot_determine() {
    let source = r#"
        module Main;

        useless : [?t : ToString] I64;
        useless = 42;

        main : IO ();
        main = pure();
    "#;
    test_source_fail(&source, Configuration::develop_mode(), "");
}

/// An opaque type named by the constraints alone, standing nowhere in the type `a -> I64` the
/// signature writes, is rejected.
#[test]
pub fn test_opaque_not_in_return_type() {
    let source = r#"
        module Main;

        foo : [?it : Iterator, Item ?it = a] a -> I64;
        foo = |x| 42;

        main : IO ();
        main = pure();
    "#;
    test_source_fail(&source, Configuration::develop_mode(), "");
}

/// A body whose branches return `RangeIterator` and `TakeIterator CountUpIterator` gives the opaque
/// type two concrete types, and is rejected.
#[test]
pub fn test_opaque_branch_type_mismatch() {
    let source = r#"
        module Main;

        choose_iter : [?it : Iterator, Item ?it = I64] Bool -> ?it;
        choose_iter = |flag| (
            if flag { Iterator::range(0, 10) }
            else { Iterator::count_up(0).take(10) }
        );

        main : IO ();
        main = pure();
    "#;
    test_source_fail(&source, Configuration::develop_mode(), "");
}

/// A definition that returns a value of the very opaque type it is declared to return leaves the
/// concrete type undetermined, and is reported.
#[test]
pub fn test_opaque_concrete_type_is_the_opaque_type_itself() {
    let source = r#"
        module Main;

        f : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        f = |n| f(n + 1);

        main : IO ();
        main = (
            let it = f(0);
            println("ok")
        );
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "`Main::f::?it` cannot be determined, because the definition gives it a type which contains that opaque type itself",
    );
}

/// The concrete type is undetermined just as well when it carries the opaque type inside another
/// type rather than being it.
#[test]
pub fn test_opaque_concrete_type_contains_the_opaque_type_itself() {
    let source = r#"
        module Main;

        f : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        f = |n| f(n + 1).map(|x| x);

        main : IO ();
        main = (
            let it = f(0);
            println("ok")
        );
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "`Main::f::?it` cannot be determined, because the definition gives it a type which contains that opaque type itself",
    );
}

/// Two values whose concrete types are each written in terms of the other's determine neither, and
/// the report names both.
#[test]
pub fn test_opaque_concrete_types_of_two_values_contain_each_other() {
    let source = r#"
        module Main;

        f : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        f = |n| g(n);

        g : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        g = |n| f(n);

        main : IO ();
        main = (
            let it = f(0);
            println("ok")
        );
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "`Main::f::?it`, `Main::g::?it` cannot be determined, because they are written in terms of each other",
    );
}

/// One implementation of a trait member may return what another implementation of the same member
/// returns, since the type the second one gives the opaque type is the concrete type of the first.
///
/// Both implementations give their concrete type to one opaque type constructor, so a check that
/// asked whether a constructor's concrete type names that constructor would reject this program.
#[test]
pub fn test_opaque_impl_returns_what_another_impl_returns() {
    let source = r##"
        module Main;

        import Std::* hiding Indexable::Elem;

        trait c : ToIter {
            type Elem c;
            to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it;
        }

        impl Array a : ToIter {
            type Elem (Array a) = a;
            to_iter = Array::to_iter;
        }

        type Wrap = box struct { v : Array I64 };

        impl Wrap : ToIter {
            type Elem Wrap = I64;
            to_iter = |w| w.@v.ToIter::to_iter;
        }

        main : IO ();
        main = (
            let arr = Wrap { v : [1, 2, 3] }.ToIter::to_iter.to_array;
            assert_eq(|_|"delegating impl", arr, [1, 2, 3]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// Two implementations of one trait member that each return what the other returns determine no
/// concrete type, and are reported on the implementations.
#[test]
pub fn test_opaque_two_impls_return_what_each_other_returns() {
    let source = r##"
        module Main;

        trait c : ToIter {
            to_iter : [?it : Iterator, Item ?it = I64] c -> ?it;
        }

        type Odd = box struct { n : I64 };
        type Even = box struct { n : I64 };

        impl Odd : ToIter {
            to_iter = |o| Even { n : o.@n }.to_iter;
        }

        impl Even : ToIter {
            to_iter = |e| Odd { n : e.@n }.to_iter;
        }

        main : IO ();
        main = println(Odd { n : 1 }.to_iter.to_array.to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be determined, because they are written in terms of each other",
    );
}

/// An implementation that returns what itself returns is reported on that implementation, and the
/// implementation of the same member for another type, which does determine a concrete type, is
/// left out of the report.
#[test]
pub fn test_opaque_one_impl_returns_what_itself_returns() {
    let source = r##"
        module Main;

        trait c : ToIter {
            to_iter : [?it : Iterator, Item ?it = I64] c -> ?it;
        }

        type Odd = box struct { n : I64 };
        type Even = box struct { n : I64 };

        impl Odd : ToIter {
            to_iter = |o| Odd { n : o.@n }.to_iter;
        }

        impl Even : ToIter {
            to_iter = |e| Iterator::range(0, e.@n);
        }

        main : IO ();
        main = println(Even { n : 2 }.to_iter.to_array.to_string);
    "##;
    let errmsg = run_source_assert_failed(&source, Configuration::develop_mode());
    assert!(
        errmsg.contains("Main::Odd") && errmsg.contains("to_iter = |o|"),
        "the implementation for `Odd` determines no concrete type, and the report is:\n{}",
        errmsg
    );
    assert!(
        !errmsg.contains("Even"),
        "the implementation for `Even` determines a concrete type, and the report is:\n{}",
        errmsg
    );
}

/// An implementation for a type of one parameter that returns what the member returns for that
/// parameter is reported.
///
/// Each step of such a resolution reaches a smaller type and the chain ends, but instantiation
/// resolves the member's type as the implementation writes it, where the parameter is a type
/// variable and no implementation matches, so the concrete type is one the compiler cannot use.
#[test]
pub fn test_opaque_impl_returns_what_the_member_returns_for_its_parameter() {
    let source = r##"
        module Main;

        trait c : ToIter {
            to_iter : [?it : Iterator, Item ?it = I64] c -> ?it;
        }

        type Leaf = box struct { n : I64 };

        impl Leaf : ToIter {
            to_iter = |l| Iterator::range(0, l.@n);
        }

        type Wrap a = box struct { inner : a };

        impl [a : ToIter] Wrap a : ToIter {
            to_iter = |w| w.@inner.to_iter;
        }

        main : IO ();
        main = println(Wrap { inner : Leaf { n : 3 } }.to_iter.to_array.to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be determined, because the definition gives it a type which contains that opaque type itself",
    );
}

/// A definition may call itself and use what the call returns; what leaves the concrete type
/// undetermined is giving the opaque return type back as the result.
#[test]
pub fn test_opaque_recursive_definition_that_returns_a_concrete_type() {
    let source = r#"
        module Main;

        f : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        f = |n| (
            let size = if n <= 0 { 0 } else { f(n - 1).to_array.@size };
            Iterator::range(0, size + 1)
        );

        main : IO ();
        main = (
            assert_eq(|_|"recursive opaque", f(2).to_array, [0, 1, 2]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A concrete type that carries the opaque type it stands for at a larger type argument
/// determines no type either: each step of the replacement reaches a bigger type.
#[test]
pub fn test_opaque_concrete_type_grows_the_opaque_type_it_stands_for() {
    let source = r#"
        module Main;

        trait a : Any {
            any : a -> I64;
        }

        impl I64 : Any {
            any = |_| 1;
        }

        impl [a : Any] Array a : Any {
            any = |_| 0;
        }

        f : [?t : Any] a -> ?t;
        f = |x| f([x]);

        main : IO ();
        main = println(f(0).any.to_string);
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be determined, because the definition gives it a type which contains that opaque type itself",
    );
}

/// An opaque type constructor that takes no type arguments stands where a type variable of kind
/// `* -> *` is expected, and the concrete type behind it is put in that place.
#[test]
pub fn test_opaque_type_constructor_of_no_arguments_as_a_higher_kinded_argument() {
    let source = r#"
        module Main;

        trait [f : *->*] f : Extract {
            extract : f a -> a;
        }

        impl Option : Extract {
            extract = |o| o.as_some;
        }

        mk : [?m : * -> *, ?m : Extract] I64 -> ?m I64;
        mk = |x| some(x);

        type [f : *->*] Holder f = box struct { v : f I64 };

        main : IO ();
        main = (
            let h = Holder { v : mk(3) };
            assert_eq(|_|"held value", h.@v.extract, 3);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// An implementation of a trait member with an opaque return type owes the constraint the member
/// declares at the implementing type, and an iterator whose items are of another type is reported.
#[test]
pub fn test_opaque_member_implementation_owes_the_constraint_at_its_own_type() {
    let source = r#"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = c] c -> I64 -> ?it;
        }

        impl I64 : Make {
            make = |_, n| Iterator::range(0, n);
        }

        impl Bool : Make {
            make = |_, n| Iterator::range(0, n);
        }

        main : IO ();
        main = (
            assert_eq(|_|"the implementation for I64", Make::make(0, 3).to_array, [0, 1, 2]);;
            pure()
        );
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Std::I64 = Std::Bool",
    );
}

// ============================================================
// A use site asks more of an opaque type than its constraints say
// ============================================================

/// A use site that calls `to_string` on a value of an opaque type constrained by `Iterator` alone
/// is rejected.
#[test]
pub fn test_opaque_trait_not_satisfied_at_use_site() {
    let source = r#"
        module Main;

        repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it;
        repeat = |x, n| Iterator::range(0, n).map(|_| x);

        main : IO ();
        main = (
            let it = repeat(42, 3);
            // ?it only has Iterator constraint, not ToString
            let _ = it.to_string;
            pure()
        );
    "#;
    test_source_fail(&source, Configuration::develop_mode(), "");
}

// ============================================================
// An opaque type on the right side of an equality constraint
// ============================================================

/// An equality whose right side is a second opaque type, `Item ?it = ?e` with `?e : ToString`: a
/// use site folds over what the value returns and reaches the items through `ToString`.
#[test]
pub fn test_opaque_in_equality_rhs() {
    let source = r#"
        module Main;

        opaque_elem_iter : [?it : Iterator, ?e : ToString, Item ?it = ?e] Array I64 -> ?it;
        opaque_elem_iter = |arr| arr.to_iter.map(|x| x.to_string);

        main : IO ();
        main = (
            let iter = opaque_elem_iter([10, 20, 30]);
            // Use fold: item has type Item ?it = ?e, and ?e : ToString
            let result = iter.fold("", |item, acc| acc + item.to_string + ",");
            assert_eq(|_|"opaque rhs fold", result, "10,20,30,");;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// `map` over a value whose items are given by an equality on a second opaque type,
/// `Item ?it = ?e`: the element type the closure takes comes to the use site through that
/// equality.
#[test]
pub fn test_opaque_in_equality_rhs_map() {
    let source = r#"
        module Main;

        opaque_elem_iter : [?it : Iterator, ?e : ToString, Item ?it = ?e] Array I64 -> ?it;
        opaque_elem_iter = |arr| arr.to_iter.map(|x| x.to_string);

        main : IO ();
        main = (
            let iter = opaque_elem_iter([10, 20, 30]);
            let strs = iter.map(|e| e.to_string).to_array;
            assert_eq(|_|"opaque rhs map", strs, ["10", "20", "30"]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// Two opaque types linked by an equality both stand in the value's result, and the use site
/// reaches each of them through `ToString`.
#[test]
pub fn test_opaque_in_equality_rhs_both_in_return() {
    let source = r#"
        module Main;

        iter_and_first : [?it : Iterator, ?e : ToString, Item ?it = ?e] Array I64 -> (?it, ?e);
        iter_and_first = |arr| (
            arr.to_iter.map(|x| x.to_string),
            arr.get_size.to_string
        );

        main : IO ();
        main = (
            let (iter, first) = iter_and_first([10, 20, 30]);
            // iter : ?it, Item ?it = ?e where ?e : ToString
            // Convert elements to String via to_string to avoid needing Eq on ?e
            let strs = iter.map(|e| e.to_string).to_array;
            assert_eq(|_|"iter part", strs, ["10", "20", "30"]);;
            // first : ?e where ?e : ToString
            let s = first.to_string;
            assert_eq(|_|"first part", s, "3");;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A chain of trait members with opaque return types, `c.baz.bar.foo`, whose opaque type
/// constructors stand inside the type arguments of one another.
#[test]
pub fn test_opaque_nested_trait_chain() {
    let source = r#"
        module Main;

        trait a: FooTrait { foo: a -> I64; }
        type Foo = unbox struct { val: I64 };
        impl Foo: FooTrait { foo = |a| a.@val; }

        trait b: BarTrait { bar: [?a: FooTrait] b -> ?a; }
        type Bar = unbox struct { foo: Foo };
        impl Bar: BarTrait { bar = |b| b.@foo; }

        trait c: BazTrait { baz: [?b: BarTrait] c -> ?b; }
        type Baz = unbox struct { bar: Bar };
        impl Baz: BazTrait { baz = |c| c.@bar; }

        main : IO ();
        main = (
            let a = Foo { val: 42 };
            let b = Bar { foo: a };
            let c = Baz { bar: b };
            let val = c.baz.bar.foo;
            assert_eq(|_|"nested opaque chain", val, 42);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A trait alias written as the constraint on an opaque type, `?v : Additive`, constrains it by the
/// traits the alias stands for, `Add` and `Zero`.
#[test]
pub fn test_opaque_trait_alias_in_constraint() {
    let source = r#"
        module Main;

        trait [f: *->*] f: Extract {
            extract: f a -> a;
        }

        impl Array: Extract {
            extract = |arr| arr.@(0);
        }

        impl Option: Extract {
            extract = |opt| opt.as_some;
        }

        trait a: FooTrait {
            foo: [?v: ToString, ?v: Additive] a -> ?v;
        }

        type Foo = unbox struct {
            val: I64
        };

        impl Foo: FooTrait {
            foo = |a| a.@val;
        }

        trait b: BarTrait {
            bar: [?a1: FooTrait, ?a2: FooTrait, ?f1: Extract, ?f2: Extract] b -> (?f1 ?a1, ?f2 ?a2);
        }

        type Bar = unbox struct {
            foo1: Foo,
            foo2: Foo,
        };

        impl Bar: BarTrait {
            bar = |b| ([b.@foo1], some $ b.@foo2);
        }

        main: IO ();
        main = (
            let a1 = Foo { val: 42 };
            let a2 = Foo { val: 123 };
            let b = Bar { foo1: a1, foo2: a2 };
            let (fa1, fa2) = b.bar;
            let v1 = fa1.extract.foo;
            let v2 = fa2.extract.foo;
            let result = (zero + v1 + v1, zero + v2 + v2).to_string;
            assert_eq(|_|"trait alias in opaque constraint", result, "(84, 246)");;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A trait member with an opaque return type implemented for two concrete types, reached through a
/// value whose type is constrained by the trait.
#[test]
pub fn test_opaque_trait_method_returning_opaque() {
    let source = r#"
        module Main;

        trait a: FooTrait {
            foo: [?s: ToString] a -> ?s;
        }

        impl I64: FooTrait {
            foo = |a| "I64";
        }

        impl U64: FooTrait {
            foo = |a| "U64";
        }

        print_foo: [a: FooTrait] a -> IO ();
        print_foo = |a| (
            a.foo.to_string.println
        );

        main: IO ();
        main = (
            print_foo(1);;
            print_foo(2_U64);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

// ============================================================
// A body that fails the constraints the signature writes on the opaque type
// ============================================================

/// A global value declared to return an opaque type constrained by `Iterator` whose body returns
/// `String` is reported, naming the constraint the body owes.
#[test]
pub fn test_opaque_impl_trait_constraint_not_satisfied_global() {
    let source = r#"
        module Main;

        wrong_iter : [?it : Iterator] I64 -> ?it;
        wrong_iter = |n| n.to_string;

        main : IO ();
        main = pure();
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "String : Std::Iterator",
    );
}

/// A global value declared to return an iterator of `I64` whose body returns an iterator of
/// `String` is reported, naming the equality the body owes.
#[test]
pub fn test_opaque_impl_assoc_type_mismatch_global() {
    let source = r#"
        module Main;

        wrong_item : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        wrong_item = |n| Iterator::range(0, n).map(|x| x.to_string);

        main : IO ();
        main = pure();
    "#;
    test_source_fail(&source, Configuration::develop_mode(), "String = Std::I64");
}

/// An implementation of a member declared to return an opaque type constrained by `Iterator` whose
/// body returns `String` is reported.
#[test]
pub fn test_opaque_impl_trait_constraint_not_satisfied_method() {
    let source = r#"
        module Main;

        trait a : MakeIter {
            make_iter : [?it : Iterator] a -> ?it;
        }

        impl I64 : MakeIter {
            make_iter = |n| n.to_string;
        }

        main : IO ();
        main = pure();
    "#;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "String : Std::Iterator",
    );
}

/// An implementation of a member declared to return an iterator of `I64` whose body returns an
/// iterator of `String` is reported.
#[test]
pub fn test_opaque_impl_assoc_type_mismatch_method() {
    let source = r#"
        module Main;

        trait a : MakeIntIter {
            make_ints : [?it : Iterator, Item ?it = I64] a -> ?it;
        }

        impl I64 : MakeIntIter {
            make_ints = |n| Iterator::range(0, n).map(|x| x.to_string);
        }

        main : IO ();
        main = pure();
    "#;
    test_source_fail(&source, Configuration::develop_mode(), "String = Std::I64");
}

/// An opaque return type in a module whose `import Std::{...}` names one by one what it uses: the
/// value the desugaring writes into the body resolves although that list leaves it out.
#[test]
pub fn test_opaque_regression_unknown_name_undefined_internal() {
    let source = r#"
        module Main;

        import Std::{IO, Monad::pure, I64, Iterator, Iterator::range};
        import Std::Iterator::Item;

        f : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        f = |n| Iterator::range(0, n);

        main : IO ();
        main = pure();
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// A global value whose body fails the constraints its opaque return type carries is reported at
/// the source span of that body, in the error saying the constraint cannot be deduced.
#[test]
pub fn test_opaque_error_carries_source_location() {
    let source = r#"
        module Main;

        pairs : [it : Iterator, ?out : Iterator, Item ?out = (Item it, Item it)] it -> ?out;
        pairs = |it| ();

        main : IO ();
        main = pure();
    "#;
    let errmsg = run_source_assert_failed(source, Configuration::develop_mode());
    assert!(
        errmsg.contains("is required in the type inference"),
        "Expected the predicate-not-deduced error, got: {}",
        errmsg
    );
    // The body `|it| ()` must be cited; the rendered span output
    // contains the line marker `pairs = |it| ()`.
    assert!(
        errmsg.contains("pairs = |it| ()"),
        "Error did not include the source location of the offending body, got: {}",
        errmsg
    );
}

/// Two values with opaque return types compose: resolving the opaque type constructor at
/// instantiation exposes an `AssocTy`, which is reduced before optimization runs.
#[test]
pub fn test_opaque_regression_assoc_ty_in_resolved_rhs() {
    let source = r#"
        module Main;

        wrap : [it : Iterator, ?out : Iterator, Item ?out = Item it] it -> ?out;
        wrap = |it| Iterator::generate(it, |_| Option::none());

        vals : [?it : Iterator, Item ?it = I64] ?it = Iterator::range(0, 10).wrap;

        main : IO ();
        main = (
            let arr = vals.to_array;
            assert_eq(|_|"empty wrapped iter", arr, []);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// Resolving one opaque type constructor whose concrete type is another opaque type constructor
/// reaches the concrete type behind the second one, so a chain of such values compiles and runs.
#[test]
pub fn test_opaque_concrete_type_is_another_opaque_type() {
    let source = r#"
        module Main;

        inner : [?b : Iterator, Item ?b = I64] I64 -> ?b;
        inner = |n| Iterator::range(0, n);

        outer : [?a : Iterator, Item ?a = I64] I64 -> ?a;
        outer = |n| inner(n);

        main : IO ();
        main = (
            assert_eq(|_|"chained opaque", outer(4).to_array, [0, 1, 2, 3]);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}
