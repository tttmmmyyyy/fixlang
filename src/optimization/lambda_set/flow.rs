// What a builtin operation tells the inference about the functions its operands and its result hold.

use super::{
    slots::{AType, SlotLayout},
    solver::{DynamicKind, Solver},
};

/// The operands and the result of one builtin operation, for its `BuiltinOp::lambda_set_flow` to
/// say how the functions they hold flow between them.
///
/// A rule declares a flow by making two places the same (`same`), and declares a place whose
/// values are closures the operation builds or calls at run time by making it Dynamic
/// (`make_own_dynamic`, `make_all_dynamic`). A place the rule says nothing about holds nothing the
/// operation put there, which is right for an operation that moves no function into it.
pub struct BuiltinFlow<'a, 'e> {
    layout: &'a SlotLayout<'e>,
    solver: &'a mut Solver,
    operands: Vec<AType>,
    result: AType,
    /// The name of the operation, recorded in the places it makes Dynamic.
    op_name: String,
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
        }
    }

    /// The `i`-th operand, in the order `BuiltinOp::free_vars` lists them.
    pub fn operand(&self, i: usize) -> AType {
        self.operands[i].clone()
    }

    /// The result.
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
    }

    /// Declares the own slots of `ty` (for a function, the closure itself, leaving what it takes
    /// and returns) places of closures built at run time.
    pub fn make_own_dynamic(&mut self, ty: &AType) {
        let kind = DynamicKind::Builtin(self.op_name.clone());
        for v in self.layout.own_slots(ty) {
            self.solver.make_dynamic(*v, kind.clone());
        }
    }

    /// Declares every slot of every operand and of the result a place of closures built at run time.
    /// This is the rule of an operation that declares none.
    pub fn make_all_dynamic(&mut self) {
        let kind = DynamicKind::Builtin(self.op_name.clone());
        for operand in &self.operands {
            self.solver.make_all_dynamic(operand, &kind);
        }
        self.solver.make_all_dynamic(&self.result, &kind);
    }
}
