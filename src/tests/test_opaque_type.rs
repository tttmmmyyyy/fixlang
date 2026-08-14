use crate::configuration::Configuration;
use crate::tests::test_util::{run_source_assert_failed, test_source, test_source_fail};

// ============================================================
// A declared return type that is opaque, hiding the concrete type the definition gives it
// ============================================================

/// A chain of iterator combinators is returned under an opaque type, so the caller writes none of
/// the combinator types. The value is called at two element types, and each call gets the elements
/// it put in.
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

/// Several combinators applied in turn build a type that names every one of them; the opaque
/// return type stands for all of it, and the elements the caller receives are the ones the chain
/// produces.
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

/// A trait member declares an opaque return type, constrained by an equality that ties its element
/// type to the trait's associated type. The one implementation hides its iterator behind it.
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

/// Two types implement one member whose return type is opaque, each hiding an iterator of its own,
/// and a call on a value of either type reaches that type's implementation.
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

/// An opaque type of kind `* -> *` stands for the monad the definition returns, and the caller
/// reaches it through the `Monad` interface the signature promises, binding one call into the next.
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

/// One signature carries an ordinary type variable for the iterator it takes and an opaque one for
/// the iterator it returns, and the element type of the result is written in terms of the element
/// type of the argument.
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

/// Two opaque types under the same constraints appear in one signature, and each stands for the
/// concrete type of the component of the returned pair it is written for.
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

/// An opaque type carrying a trait constraint alone is enough to use the value through that
/// trait's interface, which is all the caller learns about it.
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

/// The constraints on an opaque type may name an associated type of more than one argument, and
/// the concrete type the definition gives it has to answer them all.
#[test]
pub fn test_opaque_higher_arity_associated_type() {
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
            let result = c.rebuild(|x| x.to_string);
            assert_eq(|_|"higher arity assoc", result, ["1", "2", "3"]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

// ============================================================
// A trait member whose declared type fixes the trait's type variable through a constraint alone
// ============================================================

/// The declared type of `make` is `I64 -> ?it`, and the equality `Item ?it = c` alone fixes the
/// trait's type variable `c`. Each implementation hides its own concrete iterator behind `?it`,
/// and the caller reaches the one it asks for.
#[test]
pub fn test_opaque_trait_variable_fixed_by_a_constraint_alone() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = c] I64 -> ?it;
        }

        impl I64 : Make {
            make = |n| Iterator::range(0, n);
        }

        impl Bool : Make {
            make = |n| Iterator::range(0, n).map(|x| x % 2 == 0);
        }

        main : IO ();
        main = (
            let is : Array I64 = Make::make(3).to_array;
            assert_eq(|_|"make for I64", is, [0, 1, 2]);;
            let bs : Array Bool = Make::make(3).to_array;
            assert_eq(|_|"make for Bool", bs, [true, false, true]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// `impl Bool : Make` owes an iterator of `Bool`, since `Item ?it = c` holds `c` to the type the
/// implementation is for. The iterator this one returns has `I64` elements, and the constraint it
/// breaks is the implementation's own.
#[test]
pub fn test_opaque_member_implementation_owes_the_constraint_that_fixes_the_trait_variable() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = c] I64 -> ?it;
        }

        impl Bool : Make {
            make = |n| Iterator::range(0, n);
        }

        main : IO ();
        main = (
            let bs : Array Bool = Make::make(3).to_array;
            println(bs.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "`Std::I64 = Std::Bool` cannot be deduced",
    );
}

/// The type `Wrap f` this implementation is for takes a type variable of kind `* -> *`. That kind
/// reaches the opaque type constructor's argument through the trait's type variable, which a
/// constraint of `make`'s declared type fixes alone.
#[test]
pub fn test_opaque_implementing_type_with_a_higher_kinded_parameter() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = c] I64 -> ?it;
        }

        type [f : *->*] Wrap f = box struct { x : f I64 };

        impl [f : *->*, f : Monad] Wrap f : Make {
            make = |n| Iterator::range(0, n).map(|i| Wrap { x : pure(i) });
        }

        main : IO ();
        main = (
            let ws : Array (Wrap Option) = Make::make(3).to_array;
            assert_eq(|_|"make for Wrap Option", ws.map(|w| w.@x.as_some), [0, 1, 2]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// The trait's type variable `c` is of kind `* -> *`, and the equality `Item ?it = c I64` alone
/// fixes it in the declared type of `make`. The type each implementation is for is a type
/// constructor, and the opaque type constructor takes it as its argument, so each implementation
/// hides an iterator of its own.
#[test]
pub fn test_opaque_trait_variable_of_a_higher_kind() {
    let source = r##"
        module Main;

        trait [c : *->*] c : Make {
            make : [?it : Iterator, Item ?it = c I64] I64 -> ?it;
        }

        impl Array : Make {
            make = |n| Iterator::range(0, n).map(|i| [i]);
        }

        impl Option : Make {
            make = |n| Iterator::range(0, n).map(|i| some(i));
        }

        main : IO ();
        main = (
            let xs : Array (Array I64) = Make::make(2).to_array;
            assert_eq(|_|"make for Array", xs, [[0], [1]]);;
            let ys : Array (Option I64) = Make::make(2).to_array;
            assert_eq(|_|"make for Option", ys.map(|o| o.as_some), [0, 1]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// A caller generic in the trait's type variable calls the member at that variable. Each
/// instantiation of the caller reaches the implementation for the type it is instantiated at.
#[test]
pub fn test_opaque_trait_variable_fixed_by_a_constraint_alone_reached_through_a_caller() {
    let source = r##"
        module Main;

        trait c : Make {
            make : [?it : Iterator, Item ?it = c] I64 -> ?it;
        }

        impl I64 : Make {
            make = |n| Iterator::range(0, n);
        }

        impl Bool : Make {
            make = |n| Iterator::range(0, n).map(|x| x % 2 == 0);
        }

        collect : [c : Make] I64 -> Array c;
        collect = |n| Make::make(n).to_array;

        main : IO ();
        main = (
            assert_eq(|_|"collect at I64", collect(3), [0, 1, 2]);;
            assert_eq(|_|"collect at Bool", collect(3), [true, false, true]);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

// ============================================================
// An opaque type variable named in a type annotation inside a trait member implementation
// ============================================================

/// The opaque type variable of a member's declared type belongs to the trait definition, and an
/// implementation that names it in a type annotation of its body is reported as writing a type
/// variable no scope of its own gives a meaning to.
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
// An opaque type variable named in a type annotation of an implementation that writes its own
// type signature
// ============================================================

/// An implementation that declares its own opaque type variable, such as `?iter`, may name it in
/// its type signature, and naming it in a type annotation inside the body is reported.
///
/// Accepting the annotation asks for the implementation's name for the variable to be carried to
/// the trait definition's name for it, `?it`, under which the `#wrap_opaque` instantiation that
/// settles the concrete type is held.
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
// A type signature written on the implementation of a member whose return type is opaque
// ============================================================

/// The implementation writes its own type signature, naming the opaque type variable `?iter` where
/// the trait definition names it `?it`. The resolution recorded for the opaque type constructor is
/// written in the names of the signature the body is checked against, so the concrete type the
/// body determines is the one the resolution carries.
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

/// The member `my_map` carries a type variable `b` of its own beyond the trait's type variable
/// `c`, and the implementation's signature names that variable `d`. The left hand side of the
/// opaque type constructor's resolution is written in `d`, as is the concrete type the body
/// determines, so the two meet.
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

// ============================================================
// An opaque type of a higher kind
// ============================================================

/// An opaque type of kind `* -> *` constrained by `Functor` is applied to a type argument in the
/// return type, and the caller maps over what it receives, twice in a row.
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
// An opaque type constrained by an equality on an associated type
// ============================================================

#[test]
pub fn test_opaque_with_associated_type_basic() {
    // Item ?it = a propagates through sum
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

#[test]
pub fn test_opaque_associated_type_reduction() {
    // Verify that Item ?it reduces correctly at use site via fold
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

#[test]
pub fn test_opaque_with_higher_arity_assoc_type() {
    // Higher-arity associated type: Rebuild ?c b = Array b
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

#[test]
pub fn test_opaque_with_higher_kinded_assoc_type() {
    // Higher-kinded associated type (kind * -> *).
    // The same Repr is applied to different element types (I64 and String),
    // and both reduce to Array via the opaque equality Repr ?fmt = Array.
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

#[test]
pub fn test_opaque_multi_opaque_with_shared_assoc_type() {
    // Multiple opaque types sharing the same associated type constraint
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
// Several calls of one value whose return type is opaque
// ============================================================

#[test]
pub fn test_opaque_multiple_calls_different_type_args() {
    // Same opaque function called with different type arguments
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

#[test]
pub fn test_opaque_multiple_calls_same_type_args() {
    // Same opaque function called multiple times with the same type args.
    // Results should have the same opaque type and can be placed in an Array.
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
// Where an opaque type variable may be written
// ============================================================

#[test]
pub fn test_opaque_in_type_defn() {
    // Opaque type variable in struct definition should be rejected
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

#[test]
pub fn test_opaque_in_trait_defn() {
    // Opaque type variable in trait definition should be rejected
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

#[test]
pub fn test_opaque_in_impl_type_param() {
    // Opaque type variable as the implementing type in a trait impl should be rejected
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

// ============================================================
// The formal parameters of an equality constraint on an opaque type
// ============================================================

#[test]
pub fn test_opaque_equality_non_tyvar_formal_param() {
    // Extra arguments on the left side of equality must be type variables, not concrete types
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

#[test]
pub fn test_opaque_equality_formal_param_in_ty_body() {
    // Extra argument on the left side of equality must not appear elsewhere in the type signature
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
// An opaque type whose concrete type is left undetermined
// ============================================================

#[test]
pub fn test_opaque_unused_cannot_determine() {
    // Opaque type that doesn't affect the function's type: ?t can't be determined
    let source = r#"
        module Main;

        useless : [?t : ToString] I64;
        useless = 42;

        main : IO ();
        main = pure();
    "#;
    // ?t is unconstrained in the body, leading to an ambiguous type variable
    test_source_fail(&source, Configuration::develop_mode(), "");
}

#[test]
pub fn test_opaque_not_in_return_type() {
    // Opaque type is in constraints but not in the function's return type
    let source = r#"
        module Main;

        foo : [?it : Iterator, Item ?it = a] a -> I64;
        foo = |x| 42;

        main : IO ();
        main = pure();
    "#;
    // ?it has no effect on the type, leading to an undetermined type variable
    test_source_fail(&source, Configuration::develop_mode(), "");
}

#[test]
pub fn test_opaque_branch_type_mismatch() {
    // if-then-else branches return different concrete types
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
    // RangeIterator and TakeIterator CountUpIterator don't unify
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
// A use of an opaque type that asks for more than the constraints it carries
// ============================================================

#[test]
pub fn test_opaque_trait_not_satisfied_at_use_site() {
    // Calling a method not available on the opaque type's constraints
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
// An opaque type standing on the right hand side of an equality constraint
// ============================================================

#[test]
pub fn test_opaque_in_equality_rhs() {
    // An opaque type appears on the RHS of an equality constraint:
    //   Item ?it = ?e, where ?e is itself opaque with ToString constraint.
    // This tests that the desugaring correctly handles opaque-to-opaque equality.
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

#[test]
pub fn test_opaque_in_equality_rhs_map() {
    // Same setup but using .map() which requires the element type to be inferred
    // through the equality chain: Item ?it -> ?e (via EqualityScheme)
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

#[test]
pub fn test_opaque_in_equality_rhs_both_in_return() {
    // Both opaque types appear in the return type, with an equality linking them.
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

#[test]
pub fn test_opaque_nested_trait_chain() {
    // Regression test: chaining trait methods with opaque return types.
    // `c.baz.bar.foo` requires resolving nested opaque tycons in type arguments
    // before matching resolutions.
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

#[test]
pub fn test_opaque_trait_alias_in_constraint() {
    // Regression test: using a trait alias (e.g., `Additive`) in an opaque type constraint
    // should behave the same as writing its expansion (`Add + Zero`).
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

#[test]
pub fn test_opaque_trait_method_returning_opaque() {
    // Regression test: compiler crashed when a trait method returns an opaque type
    // and the trait is implemented for multiple concrete types (e.g., I64, U64).
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
// A definition whose body breaks a constraint declared on the opaque return type
// ============================================================

#[test]
pub fn test_opaque_impl_trait_constraint_not_satisfied_global() {
    // The declared signature requires `?it : Iterator`, but the implementation
    // returns `String`, which does not implement `Iterator`.
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

#[test]
pub fn test_opaque_impl_assoc_type_mismatch_global() {
    // The declared signature requires `Item ?it = I64`, but the implementation
    // returns an iterator whose item type is `String`.
    let source = r#"
        module Main;

        wrong_item : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        wrong_item = |n| Iterator::range(0, n).map(|x| x.to_string);

        main : IO ();
        main = pure();
    "#;
    test_source_fail(&source, Configuration::develop_mode(), "String = Std::I64");
}

#[test]
pub fn test_opaque_impl_trait_constraint_not_satisfied_method() {
    // The trait declares `?it : Iterator`, but the impl for `I64` returns
    // `String`, which does not implement `Iterator`.
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

#[test]
pub fn test_opaque_impl_assoc_type_mismatch_method() {
    // The trait declares `Item ?it = I64`, but the impl for `I64` returns
    // an iterator whose item type is `String`.
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

#[test]
pub fn test_opaque_regression_unknown_name_undefined_internal() {
    // Regression: with an explicit `import Std::{...}` that excludes
    // `_undefined_internal`, the generated #wrap_opaque placeholder
    // caused "unknown name `Std::_undefined_internal`".
    let source = r#"
        module Main;

        import Std::{IO, Monad::pure, I64, Iterator, Iterator::range};
        import Std::Iterator::Item;

        f : [?it : Iterator, Item ?it = I64] I64 -> ?it;
        f = |n| Iterator::range(0, n);

        main : IO ();
        main = (
            eval f(3);
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

/// Verifies that when a global value with an opaque-return-type signature
/// has a body that violates its scheme's constraints, the resulting
/// "X is required ... but cannot be deduced" error cites the source span
/// of the user-written body rather than appearing without a location.
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

/// Verifies that an `AssocTy` exposed by opaque-tycon resolution at
/// instantiation is reduced before optimization, so composing two
/// opaque-returning functions does not leave an unresolved `Item T`
/// in the program.
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

/// Verifies that resolving one opaque type constructor whose concrete type is
/// another opaque type constructor reaches the concrete type behind the second
/// one, so a chain of opaque-returning functions compiles and runs.
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
