// A global whose struct argument is split into its fields hands those fields over under the names
// the destructuring pattern gave them. Where a recursive call passes one of those names as another
// argument, that argument has to reach the call holding the value it had before the new struct was
// taken apart; where the body reads the struct itself, the twin has to rebuild it. Every function
// below is self-recursive, so `inline` leaves it in place and the split stands in the program that
// runs.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::test_source;

    /// Verifies that splitting a struct argument leaves every value where it was, over the shapes
    /// that make a field arrive under a name the body binds elsewhere.
    #[test]
    fn test_split_struct_argument_computes_the_same_values() {
        let source = r#"
module Main;

type P = struct { x : I64, y : I64 };
type Q = struct { u : I64, v : Array I64 };
type Deep = struct { inner : P, z : I64 };

// A recursive call handing over an argument named exactly as one of the field binders.
f : I64 -> P -> I64 -> I64;
f = |n, p, acc| (
    let P { x : x, y : y } = p;
    if n <= 0 { x * 1000 + y * 10 + acc };
    f(n - 1, P { x : x + 1, y : y * 2 }, x)
);

// The struct argument first, and two other arguments naming field binders.
g : P -> I64 -> I64 -> I64 -> I64;
g = |p, n, a, b| (
    let P { x : x, y : y } = p;
    if n <= 0 { x * 100 + y * 10 + a + b };
    g(P { x : y, y : x + 1 }, n - 1, y, x)
);

// A field holding a boxed value, so that a field handed over at the wrong index shows as a wrong
// element too.
k : Q -> I64 -> I64 -> I64;
k = |q, n, prev| (
    let Q { u : u, v : v } = q;
    if n <= 0 { u * 100 + v.@(0) * 10 + prev };
    k(Q { u : u + 1, v : v.set(0, u) }, n - 1, u)
);

// A struct whose field is a struct, which one round of splitting flattens and the next splits again.
d : Deep -> I64 -> I64 -> I64;
d = |s, n, carry| (
    let Deep { inner : inner, z : z } = s;
    let P { x : x, y : y } = inner;
    if n <= 0 { x * 1000 + y * 100 + z * 10 + carry };
    d(Deep { inner : P { x : y, y : x + 1 }, z : z + 1 }, n - 1, x)
);

// An argument that is an expression naming a field binder rather than a bare name.
e : I64 -> P -> I64 -> I64;
e = |n, p, acc| (
    let P { x : x, y : y } = p;
    if n <= 0 { x + y * 10 + acc * 100 };
    e(n - 1, P { x : y, y : x }, acc + x * y)
);

main : IO () = (
    assert_eq(|_|"f", f(4, P { x : 5, y : 1 }, 0), 9168);;
    assert_eq(|_|"g", g(P { x : 1, y : 3 }, 4, 0, 0), 357);;
    assert_eq(|_|"k", k(Q { u : 2, v : [7, 8] }, 3, 0), 544);;
    assert_eq(|_|"d", d(Deep { inner : P { x : 1, y : 2 }, z : 0 }, 3, 0), 3332);;
    assert_eq(|_|"e", e(4, P { x : 1, y : 2 }, 0), 821);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }
}
