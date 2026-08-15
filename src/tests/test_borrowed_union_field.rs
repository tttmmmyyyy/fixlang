// Memory-safety tests for a union read out of an aggregate the reading function only borrows.
//
// An unboxed union is one reference-counting unit: an operation on it dispatches on the tag rather
// than naming a variant, so its reference count is kept at the union's root. Its provenance,
// though, is recorded one level down, on the leaves of its variants. A reader that asks where the
// union at that root came from therefore finds nothing recorded and would take it for a value
// produced on the spot — its own to release — when it is really the caller's, read out of a
// borrowed parameter. The release then falls twice: once in the callee that never took a
// reference, once in the caller that did.
//
// The program below has that shape: a node of an array carries an unboxed union with boxed
// variants, a function reads the union out of a node it is handed and only looks at it, and the
// walk that hands the nodes over is a loop whose state carries an array. Two of the union's
// variants carry a boxed value: one carries a single one, and one carries a pair of them, so that
// the root is resolved both where a single leaf lies beneath it and where several have to agree on
// the object they come from.

#[cfg(test)]
mod borrowed_union_field_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const BORROWED_UNION_FIELD_SOURCE: &str = r#"
module Main;

// A boxed value a variant of the union carries, so that the union holds a reference count.
type Guard = box struct { allowed : Array U8 };

// Two boxed values in one variant, so that the union's root has several leaves beneath it.
type Pair = unbox struct { first : Guard, second : Guard };

// The union the node carries: one variant holds a boxed value, one a pair of them, and one only a
// number.
type Action = unbox union { wait : Guard, pair : Pair, mark : I64 };

type Node = unbox struct { action : Action, next : I64 };

// A thread of the walk. Its array is what keeps the walk going through a borrowing clone of the
// function that reads the nodes.
type State = unbox struct { at : I64, marks : Array I64 };

// Which nodes the walk has reached in this round.
type Seen = unbox struct { at_step : Array I64, step : I64 };

// The threads gathered so far, the record of the nodes they hold, and those still to be followed.
type Gathered = (Array State, Seen, Array State);

namespace Seen {
    claim : State -> Seen -> (Bool, Seen);
    claim = |state, seen| (
        let node = state.@at;
        let step = seen.@step;
        if seen.@at_step.@(node) == step { (false, seen) };
        (true, seen.mod_at_step(set(node, step)))
    );
}

namespace Walk {
    // Adds a thread unless the node it stands at is already held by another.
    _offer : State -> Gathered -> Gathered;
    _offer = |state, (threads, seen, pending)| (
        if state.@at < 0 { (threads, seen, pending) };
        let (fresh, seen) = seen.claim(state);
        if !fresh { (threads, seen, pending) };
        (threads.push_back(state), seen, pending.push_back(state))
    );

    // Reads the union out of the node and follows it, unless it is a variant that waits. The node
    // and the array it came out of are only read here, so this function is given both borrowed,
    // which is the shape the reference counting has to get right.
    _act : I64 -> Node -> State -> Gathered -> Array Node -> Gathered;
    _act = |at, node, state, gathered, nodes| (
        let action = node.@action;
        if action.is_wait || action.is_pair { gathered };
        let next = state.set_at(node.@next);
        _offer(next.mod_marks(mod(0, |_| action.as_mark + at)), gathered)
    );

    // Adds a thread and every thread it leads to without reading input.
    _close : I64 -> State -> (Array State, Seen) -> Array Node -> (Array State, Seen);
    _close = |at, state, (threads, seen), nodes| (
        let gathered = _offer(state, (threads, seen, Array::empty(4)));
        let (threads, seen, _) = loop(gathered, |(threads, seen, pending)|
            if pending.@size == 0 { break $ (threads, seen, pending) };
            let state = pending.@(pending.@size - 1);
            let pending = pending.pop_back;
            let node = nodes.@(state.@at);
            continue $ nodes._act(at, node, state, (threads, seen, pending))
        );
        (threads, seen)
    );

    // Reads the boxed values of every waiting thread. This is where a freed union is read back.
    _step : I64 -> Array State -> (Array State, Seen) -> Array Node -> (Array State, Seen);
    _step = |at, threads, frontier, nodes| (
        Iterator::range(0, threads.@size).fold(frontier, |i, frontier|
            let state = threads.@(i);
            let node = nodes.@(state.@at);
            let action = node.@action;
            if action.is_mark { frontier };
            let opened = if action.is_wait {
                action.as_wait.@allowed.@(0) == 'a'
            } else {
                action.as_pair.@first.@allowed.@(0) == 'b'
                    && action.as_pair.@second.@allowed.@(0) == 'c'
            };
            if !opened { frontier };
            nodes._close(at + 1, state.set_at(node.@next), frontier)
        )
    );

    // Walks the nodes for a number of rounds and counts the threads gathered.
    run : I64 -> Array Node -> I64;
    run = |rounds, nodes| (
        let node_count = nodes.@size;
        let start = State { at : 0, marks : fill(1, 0) };
        let (total, _, _, _) = loop(
            (0, Array::empty(node_count), Seen { at_step : fill(node_count, -1), step : 0 }, 0),
            |(total, threads, seen, at)|
            let (threads, seen) = nodes._close(at, start, (threads, seen));
            let total = total + threads.@size;
            if at >= rounds { break $ (total, threads, seen, at) };
            let seen = seen.mod_step(|step| step + 1);
            let (threads, seen) = nodes._step(at, threads, (Array::empty(node_count), seen));
            continue $ (total, threads, seen, at + 1)
        );
        total
    );
}

main : IO ();
main = (
    let nodes = [
        Node { action : mark(1), next : 1 },
        Node { action : wait(Guard { allowed : ['a'] }), next : 2 },
        Node { action : pair(Pair {
            first : Guard { allowed : ['b'] }, second : Guard { allowed : ['c'] }
        }), next : 3 },
        Node { action : mark(2), next : 4 },
        Node { action : mark(3), next : 4 }
    ];
    assert_eq(|_|"the walk gathers the same threads it does with the optimizations off", run(6, nodes), 30);;
    pure()
);
"#;

    /// The walk reaches the same answer it does with the optimizations off. A union released once
    /// too often leaves the walk reading a freed value, which this catches without Valgrind.
    #[test]
    pub fn test_borrowed_union_field_correctness() {
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::None);
        test_source(BORROWED_UNION_FIELD_SOURCE, config);
    }

    /// The boxed values the borrowed nodes carry are freed exactly once and none of them leaks,
    /// checked under Valgrind MemCheck.
    #[test]
    pub fn test_borrowed_union_field_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let config = Configuration::develop_mode();
        test_source(BORROWED_UNION_FIELD_SOURCE, config);
    }

    const DROPPED_UNION_FIELD_SOURCE: &str = r#"
module Main;

// A boxed value a variant of the union carries, so that the union holds a reference count.
type Guard = box struct { allowed : Array U8 };

// Two boxed values in one variant, so that the union's root has several leaves beneath it.
type Pair = unbox struct { first : Guard, second : Guard };

// One variant holds a boxed value, one a pair of them, and one only a number, so that the union's
// root is resolved with one leaf beneath it, with several, and with none.
type Action = unbox union { wait : Guard, pair : Pair, mark : I64 };

type Node = unbox struct { action : Action, n : I64 };

// Reads the union out of a node it is handed and drops it without consuming it. This function only
// reads the node, so it is given the node borrowed, and the union read out of it has to be
// recognized as the caller's.
glance : I64 -> Node -> I64;
glance = |k, node| (
    let action = node.@action;
    eval action;
    if k <= 0 { node.@n };
    node.@n + glance(k - 1, node)
);

// The same, with the borrowing getter `is_mark` reading the union before it is dropped.
sniff : I64 -> Node -> I64;
sniff = |k, node| (
    let action = node.@action;
    if action.is_mark { node.@n };
    if k <= 0 { 100 };
    1 + sniff(k - 1, node)
);

main : IO ();
main = (
    let single = Node { action : Action::wait(Guard { allowed : ['a'] }), n : 5 };
    let several = Node { action : Action::pair(Pair {
        first : Guard { allowed : ['b'] }, second : Guard { allowed : ['c'] }
    }), n : 2 };
    let scalar = Node { action : Action::mark(9), n : 1 };
    assert_eq(|_|"one leaf beneath the union's root", glance(3, single), 20);;
    assert_eq(|_|"several leaves beneath the union's root", glance(3, several), 8);;
    assert_eq(|_|"no leaf beneath the union's root", glance(3, scalar), 4);;
    assert_eq(|_|"dropped after a borrowing getter", sniff(3, several), 103);;
    assert_eq(|_|"dropped by the getter's own arm", sniff(3, scalar), 1);;
    pure()
);
"#;

    /// The boxed values a borrowed node carries survive a reader that reads the union out of it and
    /// drops it without consuming it, and none of them leaks. This runs under Valgrind MemCheck
    /// because a double release here frees a value nothing reads afterwards, so the assertions on
    /// their own still pass.
    #[test]
    pub fn test_dropped_union_field_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let config = Configuration::develop_mode();
        test_source(DROPPED_UNION_FIELD_SOURCE, config);
    }
}
