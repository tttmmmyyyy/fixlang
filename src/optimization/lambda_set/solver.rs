// The lambda set variables, and the union-find that makes two of them one.

use super::slots::AType;
use crate::ast::{name::FullName, types::TyCon};
use std::rc::Rc;

/// A lambda set variable: the set of lambdas a value of function type can be at one place of the
/// program. Two places whose values flow into each other share one variable.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct SetVar(u32);

/// A lambda of the program, as its index into `Analysis::lambdas`.
pub type LamId = u32;

/// One thing a lambda set variable is known to hold.
#[derive(Clone)]
pub enum Member {
    /// The values the lambda `lam` builds, inside the copy of its symbol that `owner` names.
    Lam { lam: LamId, owner: Owner },
    /// A value only a closure built at run time can stand for.
    Erased(Rc<Erasure>),
}

/// Which copy of the symbol holding a lambda builds the values a `Member::Lam` stands for.
#[derive(Clone)]
pub enum Owner {
    /// The copy being inferred: the lambda is written in the component under inference. Inference
    /// replaces it by `Instance` once the component is solved, with the variables the lambda's
    /// body depends on.
    Current,
    /// The copy given by these variables, one per parameter of the lambda the member names, in the
    /// order `Analysis::lambda_params` lists them.
    Instance(Vec<SetVar>),
}

/// Why a lambda set holds `Member::Erased`.
#[derive(Clone, PartialEq, Eq)]
pub struct Erasure {
    pub kind: ErasureKind,
    /// The symbol whose body the erasure was found in. `None` for the program's entry and the
    /// values it exports.
    pub at: Option<FullName>,
}

/// The kinds of place whose values have to be closures built at run time.
#[derive(Clone, PartialEq, Eq)]
pub enum ErasureKind {
    /// The type of the program's entry `IO` value, which the runtime runs.
    Entry,
    /// The type of a value exported to C.
    Export,
    /// An operand or the result of a builtin operation that declares no rule for lambda sets, or
    /// one whose rule says it calls the closure there. Holds the operation's name.
    Builtin(String),
    /// A field of a type whose functions the runtime calls, such as `Std::FFI::Destructor`.
    CalledByRuntime(TyCon),
    /// A place inside the type a type constructor makes of a type argument given without its own
    /// arguments, such as the `IO ()` that `f ()` becomes at `f = IO`.
    TypeConstructorArgument(TyCon),
    /// A lambda whose body reads its own capture as a whole, which only a closure object holds.
    ReadsOwnCapture,
}

impl Erasure {
    /// How the erasure reads in a dump.
    pub fn to_string(&self) -> String {
        let kind = match &self.kind {
            ErasureKind::Entry => "the entry value".to_string(),
            ErasureKind::Export => "an exported value".to_string(),
            ErasureKind::Builtin(op) => format!("builtin {}", op),
            ErasureKind::CalledByRuntime(tc) => {
                format!("called by the runtime in {}", tc.to_string())
            }
            ErasureKind::TypeConstructorArgument(tc) => {
                format!("inside a type argument of {}", tc.to_string())
            }
            ErasureKind::ReadsOwnCapture => "a lambda reading its own capture".to_string(),
        };
        match &self.at {
            Some(at) => format!("{} (in {})", kind, at.to_string()),
            None => kind,
        }
    }
}

/// The lambda set variables of the program and what each is known to hold. Unifying two variables
/// makes them one, holding what both held.
pub struct Solver {
    /// The parent of each variable in the union-find forest; a root is its own parent.
    parent: Vec<u32>,
    /// The number of variables in the tree of each root, which decides which root a union keeps.
    size: Vec<u32>,
    /// What each root holds. A variable that is not a root holds nothing here.
    members: Vec<Vec<Member>>,
    /// The symbol whose body is being inferred, recorded in the erasures found there.
    current_symbol: Option<FullName>,
}

impl Solver {
    /// A solver holding no variable.
    pub fn new() -> Self {
        Solver {
            parent: vec![],
            size: vec![],
            members: vec![],
            current_symbol: None,
        }
    }

    /// Records `symbol` as the one whose body the erasures found from now on are in.
    pub fn set_current_symbol(&mut self, symbol: Option<FullName>) {
        self.current_symbol = symbol;
    }

    /// A new variable, holding nothing.
    pub fn fresh(&mut self) -> SetVar {
        let v = self.parent.len() as u32;
        self.parent.push(v);
        self.size.push(1);
        self.members.push(vec![]);
        SetVar(v)
    }

    /// The root standing for `v` and every variable unified with it.
    pub fn find(&mut self, v: SetVar) -> SetVar {
        let mut root = v.0;
        while self.parent[root as usize] != root {
            root = self.parent[root as usize];
        }
        let mut cur = v.0;
        while self.parent[cur as usize] != root {
            let next = self.parent[cur as usize];
            self.parent[cur as usize] = root;
            cur = next;
        }
        SetVar(root)
    }

    /// Makes `a` and `b` one variable, holding what both held.
    pub fn union(&mut self, a: SetVar, b: SetVar) {
        let a = self.find(a);
        let b = self.find(b);
        if a == b {
            return;
        }
        let (keep, drop) = if self.size[a.0 as usize] >= self.size[b.0 as usize] {
            (a, b)
        } else {
            (b, a)
        };
        self.parent[drop.0 as usize] = keep.0;
        self.size[keep.0 as usize] += self.size[drop.0 as usize];
        let moved = std::mem::take(&mut self.members[drop.0 as usize]);
        self.members[keep.0 as usize].extend(moved);
    }

    /// Makes each slot of `a` one variable with the slot of `b` at the same position. The two are
    /// annotations of one Fix type, which is what makes their slots line up.
    pub fn unify(&mut self, a: &AType, b: &AType) {
        assert_eq!(
            a.slots.len(),
            b.slots.len(),
            "the annotated types `{}` and `{}` have different numbers of slots",
            a.ty.to_string(),
            b.ty.to_string()
        );
        for (x, y) in a.slots.iter().zip(b.slots.iter()) {
            self.union(*x, *y);
        }
    }

    /// Records that `v` holds `member`.
    pub fn add(&mut self, v: SetVar, member: Member) {
        let root = self.find(v);
        self.members[root.0 as usize].push(member);
    }

    /// Records that the values at `v` are closures built at run time, for the reason `kind`, found
    /// in the current symbol. A reason the variable already holds is not recorded twice.
    pub fn erase(&mut self, v: SetVar, kind: ErasureKind) {
        let erasure = Erasure {
            kind,
            at: self.current_symbol.clone(),
        };
        let root = self.find(v);
        let members = &mut self.members[root.0 as usize];
        let known = members.iter().any(|m| match m {
            Member::Erased(e) => **e == erasure,
            _ => false,
        });
        if !known {
            members.push(Member::Erased(Rc::new(erasure)));
        }
    }

    /// Records that every slot of `ty` is a closure built at run time, for the reason `kind`.
    pub fn erase_all(&mut self, ty: &AType, kind: &ErasureKind) {
        for v in &ty.slots {
            self.erase(*v, kind.clone());
        }
    }

    /// What the root of `v` holds.
    pub fn members(&mut self, v: SetVar) -> &[Member] {
        let root = self.find(v);
        &self.members[root.0 as usize]
    }

    /// What the root of `v` holds, for the caller to change.
    pub fn members_mut(&mut self, v: SetVar) -> &mut Vec<Member> {
        let root = self.find(v);
        &mut self.members[root.0 as usize]
    }

    /// The index of `v`, which a dump writes it by.
    pub fn index(v: SetVar) -> u32 {
        v.0
    }
}
