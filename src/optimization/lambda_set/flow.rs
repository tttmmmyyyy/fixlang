// What a builtin operation tells the inference about the functions its operands and its result hold.

use super::{
    slots::{AType, SlotLayout},
    solver::{DynamicKind, Member, SetVar, Solver},
};
use crate::misc::Set;

/// The operands and the result of one builtin operation, for its `BuiltinOp::lambda_set_flow` to
/// say how the functions they hold flow between them.
///
/// A rule declares a flow by making two places the same (`same`), and declares a place whose
/// values are closures the operation builds or calls at run time by making it Dynamic
/// (`make_own_dynamic`, `make_all_dynamic`). With `no_other_flow` it declares the places no
/// function flows through: where the operation leaves or drops the functions of an operand, and
/// where it puts no function in the result.
///
/// A rule accounts for every slot of every operand and of the result in one of these ways, and
/// `describe_unaccounted_slot` names a slot it left out. A function that flows through such a slot
/// at run time is missing from the lambda set of the place it reaches.
pub struct BuiltinFlow<'a, 'e> {
    /// The slots of the program's types.
    layout: &'a SlotLayout<'e>,
    /// The lambda set variables the rule makes equal or Dynamic.
    solver: &'a mut Solver,
    /// The annotated type of each operand, in the order `BuiltinOp::free_vars` lists them.
    operands: Vec<AType>,
    /// The annotated type of the result.
    result: AType,
    /// The name of the operation, recorded in the places it makes Dynamic.
    op_name: String,
    /// The slots the rule has accounted for so far.
    accounted: Set<SetVar>,
}

impl<'a, 'e> BuiltinFlow<'a, 'e> {
    /// The flow of an operation named `op_name` from `operands`, in the order
    /// `BuiltinOp::free_vars` lists them, into `result`.
    pub fn new(
        layout: &'a SlotLayout<'e>,
        solver: &'a mut Solver,
        operands: Vec<AType>,
        result: AType,
        op_name: String,
    ) -> Self {
        BuiltinFlow {
            layout,
            solver,
            operands,
            result,
            op_name,
            accounted: Set::default(),
        }
    }

    /// The annotated type of the `i`-th operand, in the order `BuiltinOp::free_vars` lists them.
    pub fn operand(&self, i: usize) -> AType {
        self.operands[i].clone()
    }

    /// The annotated type of the result.
    pub fn result(&self) -> AType {
        self.result.clone()
    }

    /// The field `i` of the struct, the variant `i` of the union, or (with `i` 0) the element of
    /// the array `ty`.
    pub fn field(&mut self, ty: &AType, i: usize) -> AType {
        self.layout.field(ty, i, self.solver)
    }

    /// The `i`-th type argument of `ty`.
    pub fn type_arg(&self, ty: &AType, i: usize) -> AType {
        self.layout.type_arg(ty, i)
    }

    /// Declares that the values at `a` and at `b` are the same values: what flows into one flows
    /// out of the other.
    pub fn same(&mut self, a: &AType, b: &AType) {
        self.solver.unify(a, b);
        self.account(a);
        self.account(b);
    }

    /// Declares that no function flows through the slots of `ty` beyond the flows `same` declares
    /// there: the operation leaves the functions an operand holds there where they are or drops
    /// them, or puts no function in the result there.
    pub fn no_other_flow(&mut self, ty: &AType) {
        self.account(ty);
    }

    /// Declares the own slots of `ty` (for a function, the closure itself, leaving what it takes
    /// and returns) places of closures built at run time.
    pub fn make_own_dynamic(&mut self, ty: &AType) {
        let kind = DynamicKind::Builtin(self.op_name.clone());
        for v in self.layout.own_slots(ty) {
            self.solver.make_dynamic(*v, kind.clone());
            self.accounted.insert(*v);
        }
    }

    /// Whether an own slot of some operand holds `Member::Dynamic`. The rule of an operation that
    /// applies a function operand makes the own slot of that operand Dynamic.
    pub fn has_dynamic_operand_own_slot(&mut self) -> bool {
        self.operands.iter().any(|operand| {
            self.layout.own_slots(operand).iter().any(|v| {
                self.solver
                    .members(*v)
                    .iter()
                    .any(|m| matches!(m, Member::Dynamic(_)))
            })
        })
    }

    /// Declares every slot of every operand and of the result a place of closures built at run time.
    /// This is the rule of an operation that declares none.
    pub fn make_all_dynamic(&mut self) {
        let kind = DynamicKind::Builtin(self.op_name.clone());
        for ty in self.operands.iter().chain([&self.result]) {
            self.solver.make_all_dynamic(ty, &kind);
            self.accounted.extend(ty.slots.iter().copied());
        }
    }

    /// Records that the rule has accounted for every slot of `ty`.
    fn account(&mut self, ty: &AType) {
        self.accounted.extend(ty.slots.iter().copied());
    }

    /// A slot of an operand or of the result that the rule has not accounted for, named by its
    /// index, the operand or result it belongs to, and the type there.
    ///
    /// # Returns
    /// `None` where the rule accounted for every slot.
    ///
    /// # Examples
    /// A rule that leaves out the second slot of its first operand, of type `Main::P`, gives
    /// ``slot 1 of operand 0, of type `Main::P` ``.
    pub fn describe_unaccounted_slot(&self) -> Option<String> {
        let (place_idx, ty, slot_idx) = self
            .operands
            .iter()
            .chain([&self.result])
            .enumerate()
            .find_map(|(place_idx, ty)| {
                let slot_idx = ty.slots.iter().position(|v| !self.accounted.contains(v))?;
                Some((place_idx, ty, slot_idx))
            })?;
        let place = if place_idx < self.operands.len() {
            format!("operand {}", place_idx)
        } else {
            "the result".to_string()
        };
        Some(format!(
            "slot {} of {}, of type `{}`",
            slot_idx,
            place,
            ty.ty.to_string()
        ))
    }
}
