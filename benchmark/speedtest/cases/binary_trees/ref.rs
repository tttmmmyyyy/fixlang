// The Rust counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

// Allocation and reference-counting stress: build a full binary tree and count its nodes.
//
// Every node is heap-allocated, leaves included, so all three languages allocate 2^(n+1)-1
// times. A `Leaf` as its own enum variant would sit inside the parent's `Box` and allocate
// only for the internal nodes, which is half the work the other two do.
struct Tree {
    children: Option<(Box<Tree>, Box<Tree>)>,
}

fn make(d: i64) -> Box<Tree> {
    if d == 0 {
        Box::new(Tree { children: None })
    } else {
        Box::new(Tree {
            children: Some((make(d - 1), make(d - 1))),
        })
    }
}

fn check(t: &Tree) -> i64 {
    match &t.children {
        None => 1,
        Some((l, r)) => 1 + check(l) + check(r),
    }
}

fn main() {
    let n: i64 = 20;
    let t = make(n);
    assert_eq!(check(&t), 2_097_151);
}
