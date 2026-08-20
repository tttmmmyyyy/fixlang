//! Marking a value's object graph: `mark_global`, which the initialization of a global value runs
//! over its result, and `Std::mark_threaded`, which a program runs over a value it is about to hand
//! to another thread.
//!
//! A value whose subgraphs are shared reaches one object by more than one path, and the paths
//! multiply along the depth of the sharing: the values here hold `LEVELS + 1` objects and reach the
//! bottom one by `2^LEVELS` paths. Marking each object once is what keeps the work proportional to
//! the objects, and the deadline on each run is what measures it: walking the paths instead takes
//! hours on a value this shape.

use crate::configuration::Configuration;
use crate::tests::test_util::{build_and_run_within, test_source};
use std::time::Duration;

/// The levels of sharing the values here are built from. Each level holds two references to the
/// level below, so the value holds `LEVELS + 1` objects and offers `2^LEVELS` paths from its top to
/// its bottom.
const LEVELS: usize = 40;

/// How long each program gets to build its value, mark it and walk it.
const TIMEOUT: Duration = Duration::from_secs(30);

/// The optimization level the programs are built at.
const OPT_LEVEL: &str = "max";

/// The type the values are built from, the recursion that builds one of `levels` levels, and the
/// function that counts the levels by walking one path from the top of a value to its bottom.
const PRELUDE: &str = r#"
module Main;

type Tree = box struct { children : Array Tree };

tree : I64 -> Tree;
tree = |levels| (
    if levels == 0 { Tree { children : Array::empty(0) } };
    let below = tree(levels - 1);
    Tree { children : [below, below] }
);

depth : Tree -> I64;
depth = |tree| if tree.@children.@size == 0 { 0 } else { 1 + tree.@children.@(0).depth };
"#;

/// A program that holds the value in a global value, whose initialization marks it global.
fn global_source(levels: usize) -> String {
    format!(
        r#"{prelude}
shared : Tree;
shared = tree({levels});

main : IO ();
main = println(shared.depth.to_string);
"#,
        prelude = PRELUDE,
        levels = levels,
    )
}

/// A program that hands the value to `Std::mark_threaded`, as one does with a value it is about to
/// send to a thread.
fn threaded_source(levels: usize) -> String {
    format!(
        r#"{prelude}
main : IO ();
main = (
    let shared = tree({levels}).mark_threaded;
    println(shared.depth.to_string)
);
"#,
        prelude = PRELUDE,
        levels = levels,
    )
}

/// Verifies that initializing a global value marks each object of its result once.
#[test]
fn test_marking_a_global_value_visits_each_object_once() {
    let printed = build_and_run_within(
        &global_source(LEVELS),
        OPT_LEVEL,
        &[],
        TIMEOUT,
        &format!("a global value whose sharing is {} levels deep", LEVELS),
    );
    assert_eq!(
        printed,
        LEVELS.to_string(),
        "the program should walk its global value to the bottom"
    );
}

/// Verifies that `Std::mark_threaded` marks each object of the value it is given once.
#[test]
fn test_marking_a_threaded_value_visits_each_object_once() {
    let printed = build_and_run_within(
        &threaded_source(LEVELS),
        OPT_LEVEL,
        &["--threaded"],
        TIMEOUT,
        &format!(
            "a value whose sharing is {} levels deep, handed to `Std::mark_threaded`",
            LEVELS
        ),
    );
    assert_eq!(
        printed,
        LEVELS.to_string(),
        "the program should walk the value it marked to the bottom"
    );
}

/// Verifies that `Std::mark_threaded` leaves an object already in the global state where it is.
///
/// A global object is exempt from reference counting, so its count stays at what its initialization
/// left, and the references a value takes to it are never counted either. If `Std::mark_threaded`
/// put such an object into the threaded state, dropping the value that holds it would release a
/// count no retain had raised, destroying an object the global value still names. The run is under
/// memcheck, which `Configuration::develop_mode` asks for, and that is what reports the
/// destruction.
#[test]
fn test_marking_a_threaded_value_leaves_a_global_object_global() {
    let source = r#"
module Main;

type Node = box struct { tag : I64, children : Array Node };

leaf : Node;
leaf = Node { tag : 42, children : Array::empty(0) };

main : IO ();
main = (
    let shared = Node { tag : 0, children : [leaf, leaf] }.mark_threaded;
    assert_eq(|_|"the global object is reached along both paths",
        shared.@children.@(0).@tag + shared.@children.@(1).@tag, 84);;

    // `shared` is dropped here, releasing what it holds.
    assert_eq(|_|"the global object outlives the value that held it", leaf.@tag, 42);;
    pure()
);
"#;
    let mut config = Configuration::develop_mode();
    config.set_threaded();
    test_source(source, config);
}
