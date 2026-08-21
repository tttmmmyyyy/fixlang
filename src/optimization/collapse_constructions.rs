//! Read a construction where the code that takes it apart can see it.
//!
//! A value that is built and then taken apart asks at run time for what the program already says.
//! `let T { g : a } = s`, where `s` was built as `T { g : x }`, binds `a` to `x`; a `match` on a
//! union variant the walk has seen built runs the arm that variant selects. Doing that here leaves
//! the construction with one fewer reader, and it hands the stages below a name they can trace to
//! the value it holds — which a name bound by a pattern is not, and which is what decides whether
//! `closure_specialization` recognizes the function an iterator carries.
//!
//! Three shapes stand between a construction and its reader, and the pass takes each away:
//!
//! - The construction sits under `let`s, so the pass floats those outward. That is `pull_let`, run
//!   between rounds, and it is what brings a construction out to the chain its reader stands in.
//! - A field holds an expression rather than a name, so the pass binds it to one first. A name is
//!   what a reader can be given without the expression being evaluated a second time.
//! - The construction is chosen by a case — every arm of a `match`, or both branches of an `if`,
//!   builds a variant — so the pass moves the reading `match` into those arms, where each meets a
//!   construction it can read. It does that only when the arms select pairwise distinct arms of the
//!   reading `match`, so that no arm's body is ever copied and every round makes the program
//!   smaller.
//!
//! A construction left with no reader goes with it, which is what keeps the reference it holds from
//! being counted alongside the one the reader now has.

use crate::{
    ast::{
        expr::{expr_let_typed, expr_var, var_var, Expr, ExprNode},
        name::{FullName, Name},
        pattern::{Pattern, PatternNode},
        program::{Program, TypeEnv},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::{TyCon, TypeNode},
    },
    constants::BOUND_FIELD_PREFIX,
    fixstd::builtin::InlineLLVMMakeUnionBody,
    misc::{Map, Set},
    optimization::{pull_let, unique_local_names},
};
use std::sync::Arc;

/// Read every construction the code taking it apart can see, over every global.
pub fn run(prg: &mut Program) {
    let type_env = prg.type_env.clone();
    for (_name, sym) in prg.symbols.iter_mut() {
        let mut expr = with_lets_pulled_out(sym.expr.as_ref().unwrap());
        let mut bound_field_count = 0;
        let mut read_any = false;
        loop {
            let mut collapser = Collapser {
                type_env: &type_env,
                constructions: Map::default(),
                bound_fields: &mut bound_field_count,
            };
            let res = collapser.traverse(&expr);
            if !res.changed {
                break;
            }
            read_any = true;
            expr = with_lets_pulled_out(&res.expr);
        }
        // Floating the `let`s outward and renaming the locals is what lets a construction be read,
        // so a global holding none of them keeps the body it came with. The stages below read the
        // shape of a body as well as its meaning, and one this pass has nothing to say about is one
        // they should meet as it was written.
        if read_any {
            sym.expr = Some(expr);
        }
    }
}

/// `expr` with its `let`s floated outward and every local under a name of its own. Floating is what
/// puts a construction and its reader in one chain, and unique names are what let one name stand
/// for one value across that chain.
fn with_lets_pulled_out(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    let expr = pull_let::run_on_expr(expr);
    unique_local_names::run_on_expr(&expr, Set::default())
}

/// A value whose construction this walk has seen.
#[derive(Clone)]
enum Construction {
    /// A struct built out of names: its type constructor, and the name each field holds.
    Struct(Arc<TyCon>, Vec<(Name, FullName)>),
    /// A union variant built out of a name: the variant's index, and the name holding its payload.
    Union(usize, FullName),
}

/// The walk, carrying what each local in scope was built as.
struct Collapser<'a> {
    /// The declarations a type's layout and a union's variants are read out of.
    type_env: &'a TypeEnv,
    /// What each local in scope was built as, keyed by its name.
    constructions: Map<FullName, Construction>,
    /// How many fields this global has had bound to a name of their own, which is what the next
    /// such name is numbered by. Counting across the rounds is what keeps two rounds from choosing
    /// one name for two values.
    bound_fields: &'a mut usize,
}

impl<'a> Collapser<'a> {
    /// Whether a value of `ty` is a struct or a union laid out where it stands rather than behind a
    /// reference.
    ///
    /// A boxed value is an object of its own, and the reader this pass takes away is what its
    /// construction is there for, so reading it here would leave an allocation nobody looks at. A
    /// type the environment does not declare — a closure, a type variable — is nothing this pass
    /// reads either way.
    fn is_unboxed_datatype(&self, ty: &Arc<TypeNode>) -> bool {
        ty.toplevel_tycon()
            .and_then(|tycon| {
                self.type_env
                    .tycons()
                    .get(tycon.as_ref())
                    .map(|ti| ti.is_unbox)
            })
            .unwrap_or(false)
    }

    /// A name for a field value, which nothing else in the global carries.
    fn fresh_field_name(&mut self) -> FullName {
        let name = FullName::local(&format!("{}{}", BOUND_FIELD_PREFIX, self.bound_fields));
        *self.bound_fields += 1;
        name
    }

    /// What `expr` was built as: what it builds itself, or what the name it is holds.
    fn construction_of(&self, expr: &Arc<ExprNode>) -> Option<Construction> {
        if !self.is_unboxed_datatype(expr.type_.as_ref().unwrap()) {
            return None;
        }
        if expr.is_var() {
            return self.constructions.get(&expr.get_var().name).cloned();
        }
        built_by(expr)
    }

    /// The index of the arm of `arms` that the variant `variant` selects. A union pattern selects
    /// its own variant, and a variable pattern takes whatever the arms above it leave.
    fn arm_for_variant(
        &self,
        arms: &[(Arc<PatternNode>, Arc<ExprNode>)],
        variant: usize,
    ) -> Option<usize> {
        arms.iter().position(|(pat, _)| match &pat.pattern {
            Pattern::Union(name, _, _) => {
                Pattern::resolve_union_variant(name, self.type_env).map(|(idx, _)| idx)
                    == Some(variant)
            }
            Pattern::Var(_, _) => true,
            Pattern::Struct(_, _) => false,
        })
    }

    /// The payload each of `bodies` builds and the arm of `arms` that the variant it builds selects,
    /// where every one of them ends in a construction laid out where it stands and no two select one
    /// arm.
    ///
    /// Two bodies selecting one arm would put that arm's body in the program twice, so the move this
    /// answers for waits for a shape where nothing is copied.
    fn payload_and_arm_for_each_body(
        &self,
        arms: &[(Arc<PatternNode>, Arc<ExprNode>)],
        bodies: &[Arc<ExprNode>],
    ) -> Option<Vec<(FullName, usize)>> {
        let mut selected: Vec<(FullName, usize)> = Vec::with_capacity(bodies.len());
        for body in bodies {
            let built = tail(body);
            if !self.is_unboxed_datatype(built.type_.as_ref().unwrap()) {
                return None;
            }
            let (variant, payload) = union_built_by(&built)?;
            let arm = self.arm_for_variant(arms, variant)?;
            if selected.iter().any(|(_, taken)| *taken == arm) {
                return None;
            }
            selected.push((payload, arm));
        }
        Some(selected)
    }

    /// `body` under the binding the arm pattern `pat` makes. A union pattern binds its sub-pattern
    /// to the payload the construction holds, and a variable pattern binds the constructed value
    /// itself, which `built` is.
    fn bound_arm(
        pat: &Arc<PatternNode>,
        built: &Arc<ExprNode>,
        payload: &FullName,
        body: &Arc<ExprNode>,
    ) -> Arc<ExprNode> {
        match &pat.pattern {
            Pattern::Union(_, _, sub) => {
                let ty = sub.info.type_.as_ref().unwrap().clone();
                expr_let_typed(
                    sub.clone(),
                    expr_var(payload.clone(), None).set_type(ty),
                    body.clone(),
                )
            }
            Pattern::Var(_, _) => expr_let_typed(pat.clone(), built.clone(), body.clone()),
            Pattern::Struct(_, _) => {
                unreachable!("`arm_for_variant` never selects an arm whose pattern is a struct")
            }
        }
    }
}

/// The variant `expr` constructs and the name holding its payload, where it constructs one.
fn union_built_by(expr: &Arc<ExprNode>) -> Option<(usize, FullName)> {
    let Expr::LLVM(llvm) = &*expr.expr else {
        return None;
    };
    let body = llvm
        .generator
        .as_ref()
        .as_any()
        .downcast_ref::<InlineLLVMMakeUnionBody>()?;
    Some((body.variant_index(), body.payload_name().clone()))
}

/// What `expr` builds, where it builds a struct out of names or a union variant.
fn built_by(expr: &Arc<ExprNode>) -> Option<Construction> {
    if let Some((variant, payload)) = union_built_by(expr) {
        return Some(Construction::Union(variant, payload));
    }
    let (tycon, fields) = expr.destructure_make_struct()?;
    let fields = fields
        .iter()
        .map(|(name, _, value)| {
            value
                .is_var()
                .then(|| (name.clone(), value.get_var().name.clone()))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Construction::Struct(tycon, fields))
}

/// The expression a chain of `let`s ends in.
fn tail(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Let(_, _, value) => tail(value),
        _ => expr.clone(),
    }
}

/// `expr` with the expression its chain of `let`s ends in replaced by `tail`, every `let` along the
/// chain retyped to what it now evaluates to.
fn set_tail(expr: &Arc<ExprNode>, tail: Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Let(_, _, value) => expr.set_let_value_typed(set_tail(value, tail)),
        _ => tail,
    }
}

/// The bodies `expr` chooses between, where it is a case: a `match` gives its arms in order, an
/// `if` gives its two branches.
fn case_bodies(expr: &Arc<ExprNode>) -> Option<Vec<Arc<ExprNode>>> {
    match &*expr.expr {
        Expr::Match(_, arms) => Some(arms.iter().map(|(_, body)| body.clone()).collect()),
        Expr::If(_, then_expr, else_expr) => Some(vec![then_expr.clone(), else_expr.clone()]),
        _ => None,
    }
}

/// `expr`, a case, with the bodies it chooses between replaced and its type set to theirs.
fn set_case_bodies(expr: &Arc<ExprNode>, bodies: Vec<Arc<ExprNode>>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Match(_, arms) => expr.set_match_pat_vals_typed(
            arms.iter()
                .map(|(pat, _)| pat.clone())
                .zip(bodies)
                .collect::<Vec<_>>(),
        ),
        Expr::If(_, _, _) => expr.set_if_then_else_typed(bodies[0].clone(), bodies[1].clone()),
        _ => unreachable!("a case is a `match` or an `if`"),
    }
}

impl<'a> ExprVisitor for Collapser<'a> {
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let pat = expr.get_let_pat();
        let bound = expr.get_let_bound();

        // A construction nothing reads goes. Reading it where it is taken apart is what takes its
        // last reader away, and what it holds keeps a reference from it until it does — which is a
        // reference the stages below count, and a value counted twice is a value they will not
        // write in place.
        if pat.is_var() && built_by(&bound).is_some() {
            let value = expr.get_let_value();
            if !value.free_vars().contains(&pat.get_var().name) {
                return StartVisitResult::ReplaceAndRevisit(value);
            }
        }

        // A name bound to a construction, or to a name already holding one, carries what it holds.
        if pat.is_var() {
            if let Some(construction) = self.construction_of(&bound) {
                self.constructions
                    .insert(pat.get_var().name.clone(), construction);
            }
            return StartVisitResult::VisitChildren;
        }

        let Pattern::Struct(pat_tycon, field_to_pat) = &pat.pattern else {
            return StartVisitResult::VisitChildren;
        };
        let Some(Construction::Struct(tycon, fields)) = self.construction_of(&bound) else {
            return StartVisitResult::VisitChildren;
        };
        if &tycon != pat_tycon {
            return StartVisitResult::VisitChildren;
        }

        // Each field the pattern reads is bound to the name the construction put there.
        let mut collapsed = expr.get_let_value();
        for (field, _, field_pat) in field_to_pat.iter().rev() {
            let Some((_, held)) = fields.iter().find(|(name, _)| name == field) else {
                return StartVisitResult::VisitChildren;
            };
            let ty = field_pat.info.type_.as_ref().unwrap().clone();
            collapsed = expr_let_typed(
                field_pat.clone(),
                expr_var(held.clone(), None).set_type(ty),
                collapsed,
            );
        }
        StartVisitResult::ReplaceAndRevisit(collapsed)
    }

    fn start_visit_match(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let cond = expr.get_match_cond();
        let arms = expr.get_match_pat_vals();

        // A match on a variant the walk has seen built runs the arm that variant selects.
        if let Some(Construction::Union(variant, payload)) = self.construction_of(&cond) {
            let Some(arm) = self.arm_for_variant(&arms, variant) else {
                return StartVisitResult::VisitChildren;
            };
            let (pat, body) = &arms[arm];
            return StartVisitResult::ReplaceAndRevisit(Self::bound_arm(
                pat, &cond, &payload, body,
            ));
        }

        // A match on a case whose every arm builds a variant moves into those arms, where each meets
        // a construction it reads.
        let Some(inner_bodies) = case_bodies(&cond) else {
            return StartVisitResult::VisitChildren;
        };
        let Some(selected) = self.payload_and_arm_for_each_body(&arms, &inner_bodies) else {
            return StartVisitResult::VisitChildren;
        };

        let moved = inner_bodies
            .iter()
            .zip(selected.iter())
            .map(|(body, (payload, arm))| {
                let (pat, arm_body) = &arms[*arm];
                set_tail(body, Self::bound_arm(pat, &tail(body), payload, arm_body))
            })
            .collect::<Vec<_>>();
        StartVisitResult::ReplaceAndRevisit(set_case_bodies(&cond, moved))
    }

    fn start_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let fields = expr.get_make_struct_fields();
        if fields.iter().all(|(_, _, e)| e.is_var())
            || !self.is_unboxed_datatype(expr.type_.as_ref().unwrap())
        {
            return StartVisitResult::VisitChildren;
        }

        // Each field holding an expression is bound to a name first, in the order the construction
        // evaluates the fields, so that a reader of this struct is given names throughout.
        let mut bindings = vec![];
        let mut named = expr.clone();
        for (field, _, value) in &fields {
            if value.is_var() {
                continue;
            }
            let name = self.fresh_field_name();
            let ty = value.type_.as_ref().unwrap().clone();
            bindings.push((
                PatternNode::make_var(var_var(name.clone()), None).set_type(ty.clone()),
                value.clone(),
            ));
            named = named.set_make_struct_field(field, expr_var(name, None).set_type(ty));
        }
        let under_bindings = bindings
            .into_iter()
            .rev()
            .fold(named, |value, (pat, expr)| expr_let_typed(pat, expr, value));
        StartVisitResult::ReplaceAndRevisit(under_bindings)
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
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
    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_tyanno(
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
