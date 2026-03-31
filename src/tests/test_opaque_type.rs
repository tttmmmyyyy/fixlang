use crate::configuration::Configuration;
use crate::tests::test_util::{test_source, test_source_fail};

// ============================================================
// 1-1. Basic use case tests
// ============================================================

#[test]
pub fn test_opaque_repeat() {
    // Use case 1: Iterator combinator return type simplification
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

#[test]
pub fn test_opaque_doubled_evens() {
    // Use case 2: Multiple combinator chaining (type explosion avoidance)
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

#[test]
pub fn test_opaque_to_iter() {
    // Use case 3: Trait method with opaque return type
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

#[test]
pub fn test_opaque_to_iter_multiple_impls() {
    // Use case 3 extended: Multiple types implementing the same trait with opaque return
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

#[test]
pub fn test_opaque_higher_kinded() {
    // Use case 4: Higher-kinded opaque type (Monad)
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

#[test]
pub fn test_opaque_zip_with_index() {
    // Use case 5: Opaque type with normal type variable mixed in signature
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

#[test]
pub fn test_opaque_partition() {
    // Use case 6: Multiple opaque types with the same constraints
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

#[test]
pub fn test_opaque_predicate_only() {
    // Use case 7: Opaque type with predicate only (no equality constraint)
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

#[test]
pub fn test_opaque_higher_arity_associated_type() {
    // Use case 8: Higher-arity associated type (Rebuildable pattern)
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
// 1-2. Opaque type in impl annotation without type signature should be rejected
// ============================================================

#[test]
pub fn test_opaque_in_impl_annotation() {
    // Using an opaque type variable in a type annotation inside an impl method
    // without a type signature should be rejected, because the opaque type variable
    // is a trait-definition-derived variable not visible in the impl context.
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
    test_source_fail(&source, Configuration::develop_mode(), "Unknown type variable `?it`");
}

// ============================================================
// 1-2b. Opaque type annotation in impl expression WITH type signature (currently unsupported)
// ============================================================

#[test]
pub fn test_opaque_in_impl_annotation_with_sig() {
    // Using an opaque type variable in a type annotation inside an impl method body
    // is not yet supported, even when the user provides an explicit type signature.
    //
    // Supporting this would require mapping the impl's opaque tyvar name (e.g., `?iter`)
    // to the trait definition's name (e.g., `?it`) so that the type-checker can look up
    // the corresponding #wrap_opaque instantiation. A prototype was implemented using
    // expression-level renaming in desugar_opaque.rs, but was reverted as too ad-hoc.
    // This may be revisited in the future with a cleaner approach.
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
    test_source_fail(&source, Configuration::develop_mode(), "Unknown type variable `?iter`");
}

// ============================================================
// 1-2c. Opaque type with user type signature on impl method
// ============================================================

#[test]
pub fn test_opaque_impl_method_type_sig() {
    // User provides a type signature on the impl method with different variable names
    // than the trait definition. This tests that defn_to_impl substitution correctly
    // uses the impl scheme's variable names (not scm_via_defn's).
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

#[test]
pub fn test_opaque_impl_method_type_sig_renamed_vars() {
    // The trait method `my_map` has an extra free type variable `b` beyond the trait's
    // type variable `c`. The user impl renames `b` to `d` in the type signature.
    // This detects whether defn_to_impl maps via impl_.scm.ty (correct: lhs uses `d`)
    // vs impl_.scm_via_defn.ty (wrong: lhs uses `b`, mismatching rhs which uses `d`).
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
// 1-3. Higher-kinded opaque type additional cases
// ============================================================

#[test]
pub fn test_opaque_higher_kinded_functor() {
    // Higher-kinded opaque with Functor constraint
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
// 1-4. Associated type tests
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
// 1-5. Multiple calls of the same opaque function
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
// 2-1. V-1: Opaque type variable usage restriction
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
    test_source_fail(&source, Configuration::develop_mode(), "is not allowed in a type definition");
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
    test_source_fail(&source, Configuration::develop_mode(), "is not allowed in a trait definition");
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
// 2-2. V-3: Equality constraint formal parameter checks
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
    test_source_fail(&source, Configuration::develop_mode(), "must be type variables");
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
    test_source_fail(&source, Configuration::develop_mode(), "must not appear elsewhere in the type signature");
}

// ============================================================
// 2-3. Opaque type concrete type determination failures
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

// ============================================================
// 2-4. Opaque type trait constraint not satisfied at use site
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
// 2-5. Opaque type on equality RHS
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

