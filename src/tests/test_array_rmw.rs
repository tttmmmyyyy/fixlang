// Tests for the array read-modify-write operations `mod`, `act`, and `pop_back`. `mod` and `act`
// are built on the PunchedArray punch/plug builtins; `pop_back` is built on the
// `_unsafe_truncate_bounds_unchecked` builtin, which drops the tail in place. With boxed elements
// the moved-out / dropped element must be neither leaked nor double-freed, which the memory-safety
// test checks under valgrind. These operations force-unique the array, so applying them to a
// shared array must clone it and leave the original intact.

#[cfg(test)]
mod array_rmw_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    #[test]
    pub fn test_mod_act_pop_correctness() {
        let source = r#"
module Main;

// A functor whose `map` applies the function twice with no loop or closure in between — the most
// inlinable shape a multi-run `act` can take, and so the sharpest test that each run plugs into an
// array of its own.
type Twice a = unbox struct { fst : a, snd : a };

impl Twice : Functor {
    map = |f, t| Twice { fst : f(t.@fst), snd : f(t.@snd) };
}

main : IO () = (
    // `mod` on unboxed and boxed arrays.
    assert_eq(|_|"mod unboxed", [1, 2, 3].mod(1, |x| x + 10), [1, 12, 3]);;
    assert_eq(|_|"mod boxed", [[1], [2]].mod(0, |x| x.push_back(9)), [[1, 9], [2]]);;

    // `mod` on a shared array clones it, leaving the original intact.
    let a = [1, 2, 3];
    let b = a.mod(0, |x| x + 100);
    assert_eq(|_|"mod shared original", a, [1, 2, 3]);;
    assert_eq(|_|"mod shared result", b, [101, 2, 3]);;

    // `act` with the Tuple2 functor and with the Option monad.
    let (picked, arr) = [1, 2, 3].act(2, |x| (x, x * 5));
    assert_eq(|_|"act tuple2 picked", picked, 3);;
    assert_eq(|_|"act tuple2 array", arr, [1, 2, 15]);;
    assert_eq(|_|"act option", [1, 2, 3].act(0, |x| some(x + 7)), some([8, 2, 3]));;

    // A functor whose `map` runs its function more than once puts each result into the punched
    // array separately, so every run must get an array of its own.
    assert_eq(|_|"act array functor", [1, 2, 3].act(0, |x| [x + 10, x + 20]), [[11, 2, 3], [21, 2, 3]]);;
    let t = [1, 2, 3].act(0, |x| Twice { fst : x + 10, snd : x + 20 });
    assert_eq(|_|"act twice fst", t.@fst, [11, 2, 3]);;
    assert_eq(|_|"act twice snd", t.@snd, [21, 2, 3]);;

    // `act` on a shared array leaves the original intact.
    let e = [1, 2, 3];
    let f = e.act(0, |x| some(x + 100)).as_some;
    assert_eq(|_|"act shared original", e, [1, 2, 3]);;
    assert_eq(|_|"act shared result", f, [101, 2, 3]);;

    // The element `act` hands to the action is moved out of the array without being retained, so the
    // shared array it was cloned from still holds it: updating that element has to clone it. The
    // action uses the element once, so nothing else forces the clone.
    let base = [[1, 2], [3]];
    let (_, updated) = base.act(0, |x| ((), x.set(0, 99)));
    assert_eq(|_|"act moved-out element original", base.@(0), [1, 2]);;
    assert_eq(|_|"act moved-out element result", updated.@(0), [99, 2]);;

    // `pop_back` on unboxed / empty / boxed arrays.
    assert_eq(|_|"pop_back", [1, 2, 3].pop_back, [1, 2]);;
    assert_eq(|_|"pop_back empty", ([] : Array I64).pop_back, []);;
    assert_eq(|_|"pop_back boxed", [[1], [2], [3]].pop_back, [[1], [2]]);;

    // `pop_back` on a shared array clones it, leaving the original intact.
    let c = [1, 2, 3];
    let d = c.pop_back;
    assert_eq(|_|"pop_back shared original", c, [1, 2, 3]);;
    assert_eq(|_|"pop_back shared result", d, [1, 2]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_mod_act_pop_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let source = r#"
module Main;

type Twice a = unbox struct { fst : a, snd : a };

impl Twice : Functor {
    map = |f, t| Twice { fst : f(t.@fst), snd : f(t.@snd) };
}

main : IO () = (
    // `pop_back` drops the last (boxed) element exactly once.
    eval [[1], [2], [3]].pop_back;
    // Popping a boxed array all the way down releases every element.
    let arr = [[1], [2], [3], [4]];
    eval loop(arr, |a| if a.@size == 0 { break $ a }; continue $ a.pop_back);

    // `mod` / `act` / `pop_back` on a shared boxed array clone it, so both the original and the
    // result stay valid and are released independently.
    let base = [[1], [2], [3]];
    let m = base.mod(1, |x| x.push_back(9));
    let (_, a) = base.act(0, |x| (x, x.push_back(8)));
    let p = base.pop_back;
    assert_eq(|_|"shared base intact", base, [[1], [2], [3]]);;
    assert_eq(|_|"mod result", m, [[1], [2, 9], [3]]);;
    assert_eq(|_|"act result", a, [[1, 8], [2], [3]]);;
    assert_eq(|_|"pop result", p, [[1], [2]]);;

    // An `act` whose functor runs the action's result more than once holds the array with a hole
    // across the runs: each run must fill a hole of its own, and the element it moves out must be
    // released once per array it ends up in.
    let multi = [[1], [2]].act(0, |x| [x.push_back(7), x.push_back(8)]);
    assert_eq(|_|"act multi-run", multi, [[[1, 7], [2]], [[1, 8], [2]]]);;
    let tw = [[1], [2]].act(0, |x| Twice { fst : x.push_back(7), snd : x.push_back(8) });
    assert_eq(|_|"act twice fst", tw.@fst, [[1, 7], [2]]);;
    assert_eq(|_|"act twice snd", tw.@snd, [[1, 8], [2]]);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(source, config);
    }
}
