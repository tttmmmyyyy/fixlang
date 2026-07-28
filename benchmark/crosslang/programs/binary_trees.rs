enum Tree {
    Leaf,
    Node(Box<Tree>, Box<Tree>),
}

fn make(d: i64) -> Tree {
    if d == 0 {
        Tree::Leaf
    } else {
        Tree::Node(Box::new(make(d - 1)), Box::new(make(d - 1)))
    }
}

fn check(t: &Tree) -> i64 {
    match t {
        Tree::Leaf => 1,
        Tree::Node(l, r) => 1 + check(l) + check(r),
    }
}

fn main() {
    let n: i64 = std::env::args().last().unwrap().parse().unwrap();
    let t = make(n);
    println!("{}", check(&t));
}
