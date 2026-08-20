//! Give a function that takes a struct a twin taking one argument per field of it.
//!
//! A closure a function is handed as an argument is a value later stages can follow: `inline` can
//! see which function it is, and `closure_specialization` can copy the function it is passed to so
//! that the call is made by name. A closure sitting in a field of a struct argument is not, so the
//! iterator combinators — which put the function they are given in a field — leave an indirect call
//! at every element.
//!
//! This pass moves such a closure out to where it can be followed. For a global whose argument is an
//! unboxed struct, it writes a second global taking that struct's fields in place of it, and rewrites
//! the original to destructure its argument and call the second one:
//!
//! ```text
//! f#split = |a, b| (               f = |s| (
//!     ...body, reading a and b         let T { g : a, h : b } = s;
//! );                                   f#split(a, b)
//!                                  );
//! ```
//!
//! The original keeps its type and its meaning, so no call site changes. `inline` then puts its two
//! lines into the callers, where the struct a caller builds meets the pattern taking it apart and
//! the two cancel, leaving the fields handed over one by one.

use super::capture_struct::fresh_global_name;
use crate::{
    ast::{
        expr::{
            expr_abs_typed, expr_app_typed, expr_let_typed, expr_make_struct, expr_var, var_local,
            ExprNode,
        },
        name::{FullName, Name},
        pattern::{Pattern, PatternNode},
        program::{Program, Symbol, TypeEnv},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::{type_fun, TyCon, TyConVariant, TypeNode},
    },
    constants::SPLIT_ARG_SUFFIX,
    misc::Set,
};
use std::sync::Arc;

/// The most arguments a function may take once its struct arguments have been split.
///
/// Splitting a struct whose field is another struct grows the count again on the next round, so a
/// deeply nested value would otherwise turn into an argument list no calling convention passes in
/// registers.
const MAX_ARGS: usize = 24;

/// Split the struct arguments of every global of `prg`, repeating until the program stops changing.
///
/// One round splits one struct argument per global. A field that is itself a struct becomes an
/// argument of the twin, and the round after that splits it in turn, so a nested value flattens one
/// level per round.
pub fn run(prg: &mut Program) {
    // What a split leaves behind takes the struct apart and hands the fields on, which is the very
    // shape a split looks for, so a wrapper split again would produce another wrapper without end.
    // Whatever is left to split is in the twin.
    let mut wrappers = Set::default();
    while run_once(prg, &mut wrappers) {}
}

/// Split one struct argument of each global that has one, and report whether anything was split.
fn run_once(prg: &mut Program, wrappers: &mut Set<FullName>) -> bool {
    let symbols = std::mem::take(&mut prg.symbols);
    let mut global_names = symbols.keys().cloned().collect::<Set<_>>();
    let mut new_symbols = crate::misc::Map::default();
    let mut changed = false;

    for (name, sym) in symbols {
        if wrappers.contains(&name) {
            new_symbols.insert(name, sym);
            continue;
        }
        match split_one_argument(&sym, &prg.type_env, &mut global_names) {
            Some((twin, wrapper)) => {
                changed = true;
                global_names.insert(twin.name.clone());
                wrappers.insert(name.clone());
                new_symbols.insert(twin.name.clone(), twin);
                new_symbols.insert(name, wrapper);
            }
            None => {
                new_symbols.insert(name, sym);
            }
        }
    }

    prg.symbols = new_symbols;
    changed
}

/// The twin of `sym` taking the fields of one struct argument, and `sym` rewritten to call it.
///
/// The argument split is the first one the body takes apart, since a body that never takes its
/// argument apart makes no use of what is inside it.
fn split_one_argument(
    sym: &Symbol,
    type_env: &TypeEnv,
    global_names: &mut Set<FullName>,
) -> Option<(Symbol, Symbol)> {
    let expr = sym.expr.as_ref()?;
    let (param_lists, body) = expr.destructure_lam_sequence();
    let params = param_lists
        .iter()
        .map(|param_list| {
            assert_eq!(param_list.len(), 1);
            param_list[0].clone()
        })
        .collect::<Vec<_>>();
    let doms = sym.ty.collect_app_src(params.len()).0;
    let codom = sym.ty.collect_app_src(params.len()).1;

    let (arg, taken_apart) = params.iter().enumerate().find_map(|(arg, param)| {
        let tycon = splittable_struct(&doms[arg], type_env)?;
        let declared = declared_field_names(&tycon, type_env);
        let taken_apart = destructuring_of(&body, &param.name, &tycon, &declared)?;
        (params.len() + declared.len() - 1 <= MAX_ARGS).then_some((arg, taken_apart))
    })?;

    let field_tys = doms[arg].field_types(type_env);
    let mut counter = 0;
    let twin_name = fresh_global_name(&sym.name, SPLIT_ARG_SUFFIX, &mut counter, global_names);

    // The twin's arguments: the ones the original takes, with the struct replaced by its fields.
    let mut twin_params = params.clone();
    let mut twin_doms = doms.clone();
    let field_params = taken_apart
        .field_names
        .iter()
        .map(|name| var_local(&name.name))
        .collect::<Vec<_>>();
    twin_params.splice(arg..arg + 1, field_params.clone());
    twin_doms.splice(arg..arg + 1, field_tys.iter().cloned());

    let twin = Symbol {
        name: twin_name.clone(),
        generic_name: sym.generic_name.clone(),
        ty: fun_ty(&twin_doms, codom.clone()),
        expr: Some(lambda_over(
            &twin_params,
            &twin_doms,
            twin_body(&taken_apart, &params[arg], &doms[arg], &field_tys),
        )),
        inline_into_callers: false,
    };

    let wrapper = Symbol {
        name: sym.name.clone(),
        generic_name: sym.generic_name.clone(),
        ty: sym.ty.clone(),
        expr: Some(lambda_over(
            &params,
            &doms,
            wrapper_body(
                &taken_apart,
                &params,
                arg,
                &doms,
                &field_tys,
                &twin_name,
                &twin.ty,
            ),
        )),
        inline_into_callers: sym.inline_into_callers,
    };

    Some((twin, wrapper))
}

/// The twin's body: the original body with the destructuring gone, since the names it bound are the
/// arguments now.
///
/// The struct is rebuilt ahead of it where the body still names it — a body that reads its argument
/// somewhere other than the destructuring needs the value it was handed.
fn twin_body(
    taken_apart: &Destructuring,
    param: &Arc<crate::ast::expr::Var>,
    struct_ty: &Arc<TypeNode>,
    field_tys: &[Arc<TypeNode>],
) -> Arc<ExprNode> {
    let body = taken_apart.body.clone();
    if !body.free_vars().contains(&param.name) {
        return body;
    }
    let fields = taken_apart
        .field_order
        .iter()
        .zip(taken_apart.field_names.iter())
        .zip(field_tys.iter())
        .map(|((field, name), ty)| {
            (
                field.clone(),
                expr_var(name.clone(), None).set_type(ty.clone()),
            )
        })
        .collect();
    let rebuilt = expr_make_struct(taken_apart.tycon.clone(), fields).set_type(struct_ty.clone());
    expr_let_typed(
        PatternNode::make_var(param.clone(), None).set_type(struct_ty.clone()),
        rebuilt,
        body,
    )
}

/// The original's body once it is a wrapper: take the struct apart and hand the fields to the twin.
fn wrapper_body(
    taken_apart: &Destructuring,
    params: &[Arc<crate::ast::expr::Var>],
    arg: usize,
    doms: &[Arc<TypeNode>],
    field_tys: &[Arc<TypeNode>],
    twin_name: &FullName,
    twin_ty: &Arc<TypeNode>,
) -> Arc<ExprNode> {
    let mut args = params
        .iter()
        .zip(doms.iter())
        .map(|(param, ty)| expr_var(param.name.clone(), None).set_type(ty.clone()))
        .collect::<Vec<_>>();
    let field_args = taken_apart
        .field_names
        .iter()
        .zip(field_tys.iter())
        .map(|(name, ty)| expr_var(name.clone(), None).set_type(ty.clone()))
        .collect::<Vec<_>>();
    args.splice(arg..arg + 1, field_args);

    let mut call = expr_var(twin_name.clone(), None).set_type(twin_ty.clone());
    for arg in args {
        call = expr_app_typed(call, vec![arg]);
    }
    expr_let_typed(
        taken_apart.pattern.clone(),
        expr_var(params[arg].name.clone(), None).set_type(doms[arg].clone()),
        call,
    )
}

/// Where a body takes a struct argument apart, and what is left once it has.
struct Destructuring {
    /// The type constructor the pattern names.
    tycon: Arc<TyCon>,
    /// The pattern itself, which the wrapper takes the struct apart with.
    pattern: Arc<PatternNode>,
    /// The declared name of each field, in the order the declaration writes them.
    field_order: Vec<Name>,
    /// The name the pattern binds each of those fields to, in that same order. A pattern writes its
    /// fields in the order the source has them, which is not the order they are declared, so this is
    /// where the two are brought together.
    field_names: Vec<FullName>,
    /// The body with that `let` removed.
    body: Arc<ExprNode>,
}

/// Where `body` takes the struct named `param` apart, where it does so by naming every field.
///
/// A pattern that leaves a field out, or binds one to a pattern rather than a name, gives the twin
/// no argument to stand for it.
fn destructuring_of(
    body: &Arc<ExprNode>,
    param: &FullName,
    tycon: &Arc<TyCon>,
    declared: &[Name],
) -> Option<Destructuring> {
    let mut remover = DestructuringRemover {
        param,
        tycon,
        field_count: declared.len(),
        found: None,
    };
    let removed = remover.traverse(body);
    let (pattern, written, bound) = remover.found?;
    let field_names = declared
        .iter()
        .map(|field| {
            let at = written.iter().position(|name| name == field)?;
            Some(bound[at].clone())
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Destructuring {
        tycon: tycon.clone(),
        pattern,
        field_order: declared.to_vec(),
        field_names,
        body: removed.expr,
    })
}

/// The name of each field the struct `tycon` declares, in the order it declares them.
fn declared_field_names(tycon: &Arc<TyCon>, type_env: &TypeEnv) -> Vec<Name> {
    type_env.tycons()[tycon.as_ref()]
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect()
}

/// The walk that finds the first `let` taking `param` apart and drops it from the body.
struct DestructuringRemover<'a> {
    param: &'a FullName,
    tycon: &'a Arc<TyCon>,
    /// How many fields the struct declares, which a pattern has to name all of.
    field_count: usize,
    found: Option<(Arc<PatternNode>, Vec<Name>, Vec<FullName>)>,
}

impl ExprVisitor for DestructuringRemover<'_> {
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if self.found.is_some() {
            return StartVisitResult::VisitChildren;
        }
        let pat = expr.get_let_pat();
        let bound = expr.get_let_bound();
        let Pattern::Struct(pat_tycon, field_to_pat) = &pat.pattern else {
            return StartVisitResult::VisitChildren;
        };
        if pat_tycon != self.tycon
            || !bound.is_var()
            || &bound.get_var().name != self.param
            || field_to_pat.len() != self.field_count
            || !field_to_pat.iter().all(|(_, _, pat)| pat.is_var())
        {
            return StartVisitResult::VisitChildren;
        }
        self.found = Some((
            pat.clone(),
            field_to_pat.iter().map(|(name, _, _)| name.clone()).collect(),
            field_to_pat
                .iter()
                .map(|(_, _, pat)| pat.get_var().name.clone())
                .collect(),
        ));
        StartVisitResult::ReplaceAndRevisit(expr.get_let_value())
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

/// The type constructor of `ty`, where a value of it is laid out as fields an argument list can
/// stand in for.
///
/// A boxed struct is left alone: its fields sit behind a pointer, so handing them over one by one
/// would copy what the program means to share. So is a struct with a field punched out, whose hole
/// holds no value to hand over.
fn splittable_struct(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Option<Arc<TyCon>> {
    let tycon = ty.toplevel_tycon()?;
    let info = type_env.tycons().get(tycon.as_ref())?;
    if info.variant != TyConVariant::Struct || !info.is_unbox || info.fields.is_empty() {
        return None;
    }
    if info.punched_from.is_some() || info.fields.iter().any(|field| field.is_punched) {
        return None;
    }
    Some(tycon)
}

/// `body` taken as a function of `params`, each standing at the type beside it.
fn lambda_over(
    params: &[Arc<crate::ast::expr::Var>],
    doms: &[Arc<TypeNode>],
    body: Arc<ExprNode>,
) -> Arc<ExprNode> {
    let mut expr = body;
    for (param, dom) in params.iter().zip(doms.iter()).rev() {
        expr = expr_abs_typed(param.clone(), dom.clone(), expr);
    }
    expr
}

/// The function type taking `doms` in order and returning `codom`.
fn fun_ty(doms: &[Arc<TypeNode>], codom: Arc<TypeNode>) -> Arc<TypeNode> {
    let mut ty = codom;
    for dom in doms.iter().rev() {
        ty = type_fun(dom.clone(), ty);
    }
    ty
}
