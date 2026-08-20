//! Bind a field directly where a destructuring meets the construction that made the struct.
//!
//! `let T { g : a, h : b } = s;` where `s` was built as `T { g : x, h : y }` asks at run time for
//! what the program already says: `a` is `x` and `b` is `y`. Binding them outright leaves the
//! construction with one fewer reader, and it hands the stages that follow a name they can trace to
//! the value it holds — which a name bound by a pattern is not.
//!
//! Only a construction whose every field is a name is collapsed. Binding a field to an expression
//! would move that expression to where the field is read, which is a different program when the
//! struct is read more than once or not at all.
//!
//! The two meet only when they stand in one chain of `let`s, so the pass floats the `let`s of each
//! global outward first and gives every local a name of its own. Floating is what brings a
//! construction built inside a `let` out to the chain the destructuring reads from, and unique names
//! are what let one name stand for one value across that chain.

use crate::{
    ast::{
        expr::{expr_let_typed, ExprNode},
        name::{FullName, Name},
        pattern::Pattern,
        program::Program,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::TyCon,
    },
    misc::{Map, Set},
    optimization::{pull_let, unique_local_names},
};
use std::sync::Arc;

/// Collapse every destructuring of a struct whose construction is in scope, over every global.
pub fn run(prg: &mut Program) {
    for (_name, sym) in prg.symbols.iter_mut() {
        let mut expr = pull_let::run_on_expr(sym.expr.as_ref().unwrap());
        expr = unique_local_names::run_on_expr(&expr, Set::default());
        loop {
            let mut collapser = Collapser {
                built: Map::default(),
            };
            let res = collapser.traverse(&expr);
            expr = res.expr;
            if !res.changed {
                break;
            }
        }
        sym.expr = Some(expr);
    }
}

/// The walk, carrying what each local in scope was built as.
struct Collapser {
    /// The struct each local name holds, where it was built here out of names. A name bound to
    /// another such name carries the same answer.
    built: Map<FullName, (Arc<TyCon>, Vec<(Name, FullName)>)>,
}

impl Collapser {
    /// What `expr` was built as, where it is a name holding a struct built out of names.
    fn built_as(&self, expr: &Arc<ExprNode>) -> Option<&(Arc<TyCon>, Vec<(Name, FullName)>)> {
        expr.is_var()
            .then(|| self.built.get(&expr.get_var().name))
            .flatten()
    }

    /// The struct `expr` builds, where it builds one out of names.
    fn builds(&self, expr: &Arc<ExprNode>) -> Option<(Arc<TyCon>, Vec<(Name, FullName)>)> {
        let (tycon, fields) = expr.destructure_make_struct()?;
        let fields = fields
            .iter()
            .map(|(name, _, value)| {
                value
                    .is_var()
                    .then(|| (name.clone(), value.get_var().name.clone()))
            })
            .collect::<Option<Vec<_>>>()?;
        Some((tycon, fields))
    }
}

impl ExprVisitor for Collapser {
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let pat = expr.get_let_pat();
        let bound = expr.get_let_bound();

        // A name bound to such a struct, or to a name already holding one, carries what it holds.
        if pat.is_var() {
            let holds = self
                .builds(&bound)
                .or_else(|| self.built_as(&bound).cloned());
            if let Some(holds) = holds {
                self.built.insert(pat.get_var().name.clone(), holds);
            }
            return StartVisitResult::VisitChildren;
        }

        let Pattern::Struct(pat_tycon, field_to_pat) = &pat.pattern else {
            return StartVisitResult::VisitChildren;
        };
        let Some((tycon, fields)) = self.built_as(&bound) else {
            return StartVisitResult::VisitChildren;
        };
        if tycon != pat_tycon || !field_to_pat.iter().all(|(_, _, pat)| pat.is_var()) {
            return StartVisitResult::VisitChildren;
        }

        // Each field the pattern names is bound to the name the construction put there.
        let fields = fields.clone();
        let mut collapsed = expr.get_let_value();
        for (field, _, field_pat) in field_to_pat.iter().rev() {
            let Some((_, held)) = fields.iter().find(|(name, _)| name == field) else {
                return StartVisitResult::VisitChildren;
            };
            let ty = field_pat.info.type_.as_ref().unwrap().clone();
            collapsed = expr_let_typed(
                field_pat.clone(),
                crate::ast::expr::expr_var(held.clone(), None).set_type(ty),
                collapsed,
            );
        }
        StartVisitResult::ReplaceAndRevisit(collapsed)
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_tyanno(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
