// The lambda set variables, and the union-find that makes two of them one.

use super::slots::AType;
use crate::{
    ast::{name::FullName, types::TyCon},
    constants::MAX_LS_VARIABLE_LAMBDAS,
};
use std::{mem, rc::Rc};

/// A lambda set variable: the set of lambdas a value of function type can be at one place of the
/// program. Two places whose values flow into each other share one variable.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct SetVar(u32);

impl SetVar {
    /// The index of the variable, which a dump writes it by.
    pub fn index(self) -> u32 {
        self.0
    }
}

/// A lambda of the program, as its index into `Analysis::lambdas`.
pub type LamId = u32;

/// One thing a lambda set variable is known to hold. A variable holding `Member::Dynamic` holds no
/// `Member::Lam`: its values are closures whatever lambdas build them.
#[derive(Clone)]
pub enum Member {
    /// The values the lambda `lam` builds, inside the copy of its symbol that `owner` names.
    Lam { lam: LamId, owner: Owner },
    /// A value only a closure built at run time can stand for.
    Dynamic(Rc<DynamicReason>),
}

/// Which copy of the symbol holding a lambda builds the values a `Member::Lam` stands for.
#[derive(Clone)]
pub enum Owner {
    /// The copy being inferred: the lambda is written in the component under inference. Inference
    /// replaces it by `Instance` once the component is solved, with the variables the lambda's
    /// body depends on.
    Current,
    /// The copy given by these variables, one per component parameter the lambda the member names
    /// depends on, in the order `Lambda::param_deps` lists them.
    Instance(Vec<SetVar>),
}

/// Why a lambda set holds `Member::Dynamic`.
#[derive(Clone, PartialEq, Eq)]
pub struct DynamicReason {
    /// The kind of place whose values have to be closures.
    pub kind: DynamicKind,
    /// The symbol whose body the reason was found in. `None` for the program's entry and the
    /// values it exports.
    pub found_in: Option<FullName>,
}

/// The kinds of place whose values have to be closures built at run time.
#[derive(Clone, PartialEq, Eq)]
pub enum DynamicKind {
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
    /// A parameter the bodies of a component depend on (`Component::body_params`), where the
    /// program asks the component for more bodies than its copy budget (`LS_COPY_BUDGET_BASE`).
    PastCopyBudget,
    /// A variable that more than `MAX_LS_VARIABLE_LAMBDAS` lambdas were recorded in.
    TooManyLambdas,
}

impl DynamicReason {
    /// How the reason reads in a dump.
    pub fn to_string(&self) -> String {
        let kind = match &self.kind {
            DynamicKind::Entry => "the entry value".to_string(),
            DynamicKind::Export => "an exported value".to_string(),
            DynamicKind::Builtin(op) => format!("builtin {}", op),
            DynamicKind::CalledByRuntime(tc) => {
                format!("called by the runtime in {}", tc.to_string())
            }
            DynamicKind::TypeConstructorArgument(tc) => {
                format!("inside a type argument of {}", tc.to_string())
            }
            DynamicKind::ReadsOwnCapture => "a lambda reading its own capture".to_string(),
            DynamicKind::PastCopyBudget => "given to a component past its copy budget".to_string(),
            DynamicKind::TooManyLambdas => {
                format!("holding more than {} lambdas", MAX_LS_VARIABLE_LAMBDAS)
            }
        };
        match &self.found_in {
            Some(at) => format!("{} (in {})", kind, at.to_string()),
            None => kind,
        }
    }
}

/// The lambda set variables of the program and what each is known to hold. Unifying two variables
/// makes them one, holding what both held.
///
/// A variable holds either lambdas, at most `MAX_LS_VARIABLE_LAMBDAS` of them, or Dynamic alone.
/// The lambdas recorded in a variable are counted one per `add`, and those of two variables
/// unified are added up. A variable whose count goes past the bound is made Dynamic
/// (`DynamicKind::TooManyLambdas`), and a variable made Dynamic gives up its lambdas and takes no
/// more. The count of a variable only grows and Dynamic stays, so which variables end up Dynamic
/// does not depend on the order the lambdas are recorded and the variables unified in.
pub struct Solver {
    /// The parent of each variable in the union-find forest; a root is its own parent.
    parent: Vec<u32>,
    /// The number of variables in the tree of each root, which decides which root a union keeps.
    size: Vec<u32>,
    /// What each root holds: `Member::Lam`s alone, at most `MAX_LS_VARIABLE_LAMBDAS` of them, or
    /// `Member::Dynamic`s alone. The entry of a variable that is not a root is empty.
    members: Vec<Vec<Member>>,
    /// Whether each root is pinned: every reference to its component, and every copy, shares this
    /// one variable, whose members the components inferred later may add to.
    pinned: Vec<bool>,
    /// The symbol whose body is being inferred, recorded in the reasons for Dynamic found there.
    current_symbol: Option<FullName>,
}

impl Solver {
    /// A solver holding no variable.
    pub fn new() -> Self {
        Solver {
            parent: vec![],
            size: vec![],
            members: vec![],
            pinned: vec![],
            current_symbol: None,
        }
    }

    /// Records `symbol` as the one whose body the reasons for Dynamic found from now on are in.
    pub fn set_current_symbol(&mut self, symbol: Option<FullName>) {
        self.current_symbol = symbol;
    }

    /// The symbol whose body is being inferred, `None` while the roots of the program are.
    pub fn current_symbol(&self) -> Option<&FullName> {
        self.current_symbol.as_ref()
    }

    /// A new variable, holding nothing.
    pub fn fresh(&mut self) -> SetVar {
        let v = self.parent.len() as u32;
        self.parent.push(v);
        self.size.push(1);
        self.members.push(vec![]);
        self.pinned.push(false);
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

    /// Whether the root `root` holds Dynamic, and so no lambda.
    fn root_is_dynamic(&self, root: SetVar) -> bool {
        matches!(
            self.members[root.0 as usize].first(),
            Some(Member::Dynamic(_))
        )
    }

    /// Makes the root `root`, which holds lambdas alone, Dynamic where it holds more of them than
    /// `MAX_LS_VARIABLE_LAMBDAS`. The reason names the current symbol.
    fn bound_lambdas(&mut self, root: SetVar) {
        if self.members[root.0 as usize].len() > MAX_LS_VARIABLE_LAMBDAS {
            self.make_dynamic(root, DynamicKind::TooManyLambdas);
        }
    }

    /// Makes `a` and `b` one variable, holding what both held: the reasons of both where either
    /// holds Dynamic, and the lambdas of both otherwise, unless they are more than
    /// `MAX_LS_VARIABLE_LAMBDAS`.
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
        self.pinned[keep.0 as usize] |= self.pinned[drop.0 as usize];
        let moved = mem::take(&mut self.members[drop.0 as usize]);
        let moved_dynamic = matches!(moved.first(), Some(Member::Dynamic(_)));
        match (self.root_is_dynamic(keep), moved_dynamic) {
            (true, true) => self.members[keep.0 as usize].extend(moved),
            (true, false) => {}
            (false, true) => self.members[keep.0 as usize] = moved,
            (false, false) => {
                self.members[keep.0 as usize].extend(moved);
                self.bound_lambdas(keep);
            }
        }
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

    /// Records that `v` holds the lambda `lam` of the copy `owner`. A variable holding Dynamic
    /// stays as it is, and one this lambda takes past `MAX_LS_VARIABLE_LAMBDAS` becomes Dynamic.
    pub fn add_lambda(&mut self, v: SetVar, lam: LamId, owner: Owner) {
        let root = self.find(v);
        if self.root_is_dynamic(root) {
            return;
        }
        self.members[root.0 as usize].push(Member::Lam { lam, owner });
        self.bound_lambdas(root);
    }

    /// Records that the values at `v` are closures built at run time, for the reason `kind`, found
    /// in the current symbol, as `add_reason` does.
    pub fn make_dynamic(&mut self, v: SetVar, kind: DynamicKind) {
        self.make_dynamic_found_in(v, kind, self.current_symbol.clone());
    }

    /// Records that the values at `v` are closures built at run time, for the reason `kind`, found
    /// in `found_in` (`DynamicReason::found_in`), as `add_reason` does.
    pub fn make_dynamic_found_in(
        &mut self,
        v: SetVar,
        kind: DynamicKind,
        found_in: Option<FullName>,
    ) {
        self.add_reason(v, Rc::new(DynamicReason { kind, found_in }));
    }

    /// Records that the values at `v` are closures built at run time, for `reason`. Each reason is
    /// recorded once per variable. The lambdas `v` held are dropped.
    pub fn add_reason(&mut self, v: SetVar, reason: Rc<DynamicReason>) {
        let root = self.find(v);
        if !self.root_is_dynamic(root) {
            self.members[root.0 as usize].clear();
        }
        let members = &mut self.members[root.0 as usize];
        let known = members.iter().any(|m| match m {
            Member::Dynamic(known) => *known == reason,
            Member::Lam { .. } => false,
        });
        if !known {
            members.push(Member::Dynamic(reason));
        }
    }

    /// Records that every slot of `ty` is a closure built at run time, for the reason `kind`.
    pub fn make_all_dynamic(&mut self, ty: &AType, kind: &DynamicKind) {
        for v in &ty.slots {
            self.make_dynamic(*v, kind.clone());
        }
    }

    /// Pins `v`, and every variable unified with it now or later.
    pub fn pin(&mut self, v: SetVar) {
        let root = self.find(v);
        self.pinned[root.0 as usize] = true;
    }

    /// Whether `v` is pinned.
    pub fn is_pinned(&mut self, v: SetVar) -> bool {
        let root = self.find(v);
        self.pinned[root.0 as usize]
    }

    /// What the root of `v` holds.
    pub fn members(&mut self, v: SetVar) -> &[Member] {
        let root = self.find(v);
        &self.members[root.0 as usize]
    }

    /// What the root of `v` holds, each member for the caller to change in place.
    pub fn members_mut(&mut self, v: SetVar) -> &mut [Member] {
        let root = self.find(v);
        &mut self.members[root.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::{DynamicKind, Member, Owner, SetVar, Solver};
    use crate::constants::MAX_LS_VARIABLE_LAMBDAS;

    /// A new variable of `solver` that `count` lambdas were recorded in.
    fn var_of_lambdas(solver: &mut Solver, count: usize) -> SetVar {
        let v = solver.fresh();
        for lam in 0..count {
            solver.add_lambda(v, lam as u32, Owner::Current);
        }
        v
    }

    /// Whether `v` holds Dynamic alone, for the one reason of holding too many lambdas.
    fn holds_too_many_lambdas(solver: &mut Solver, v: SetVar) -> bool {
        match solver.members(v) {
            [Member::Dynamic(reason)] => reason.kind == DynamicKind::TooManyLambdas,
            _ => false,
        }
    }

    /// A variable keeps the lambdas recorded in it up to `MAX_LS_VARIABLE_LAMBDAS`, and the next
    /// one makes it Dynamic, holding no lambda.
    #[test]
    fn test_a_variable_keeps_its_lambdas_up_to_the_bound_and_is_dynamic_past_it() {
        let mut solver = Solver::new();
        let v = var_of_lambdas(&mut solver, MAX_LS_VARIABLE_LAMBDAS);
        assert_eq!(solver.members(v).len(), MAX_LS_VARIABLE_LAMBDAS);
        assert!(solver
            .members(v)
            .iter()
            .all(|m| matches!(m, Member::Lam { .. })));
        solver.add_lambda(v, MAX_LS_VARIABLE_LAMBDAS as u32, Owner::Current);
        assert!(holds_too_many_lambdas(&mut solver, v));
    }

    /// Two variables unified keep the lambdas of both where these are `MAX_LS_VARIABLE_LAMBDAS`
    /// between them, and are Dynamic where they are one more, whichever of the two holds more.
    #[test]
    fn test_unified_variables_keep_their_lambdas_up_to_the_bound_and_are_dynamic_past_it() {
        let few = 3;
        for (a_count, b_count, past) in [
            (MAX_LS_VARIABLE_LAMBDAS - few, few, false),
            (few, MAX_LS_VARIABLE_LAMBDAS - few, false),
            (MAX_LS_VARIABLE_LAMBDAS - few, few + 1, true),
            (few + 1, MAX_LS_VARIABLE_LAMBDAS - few, true),
        ] {
            let mut solver = Solver::new();
            let a = var_of_lambdas(&mut solver, a_count);
            let b = var_of_lambdas(&mut solver, b_count);
            solver.union(a, b);
            assert_eq!(holds_too_many_lambdas(&mut solver, a), past);
            if !past {
                assert_eq!(solver.members(a).len(), MAX_LS_VARIABLE_LAMBDAS);
            }
        }
    }

    /// A variable holding Dynamic holds no lambda: making a variable Dynamic drops its lambdas, a
    /// lambda recorded in it afterwards is left out, and unifying it with a variable of lambdas,
    /// from either side, leaves the reason alone.
    #[test]
    fn test_a_dynamic_variable_holds_no_lambda() {
        let is_entry_alone = |solver: &mut Solver, v: SetVar| match solver.members(v) {
            [Member::Dynamic(reason)] => reason.kind == DynamicKind::Entry,
            _ => false,
        };
        let mut solver = Solver::new();
        let v = var_of_lambdas(&mut solver, 2);
        solver.make_dynamic(v, DynamicKind::Entry);
        assert!(is_entry_alone(&mut solver, v));
        solver.add_lambda(v, 7, Owner::Current);
        assert!(is_entry_alone(&mut solver, v));
        // The variable of more variables is the root a union keeps, so each side is tried as it.
        for dynamic_is_larger in [false, true] {
            let mut solver = Solver::new();
            let dynamic = solver.fresh();
            solver.make_dynamic(dynamic, DynamicKind::Entry);
            let lambdas = var_of_lambdas(&mut solver, 2);
            let extra = solver.fresh();
            solver.union(if dynamic_is_larger { dynamic } else { lambdas }, extra);
            solver.union(dynamic, lambdas);
            assert!(is_entry_alone(&mut solver, lambdas));
        }
    }
}
