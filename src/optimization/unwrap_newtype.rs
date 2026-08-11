//! Unwrap the newtype pattern, i.e., type A = unbox struct { data : B } to B.
//!
//! The newtype pattern is an unboxed struct of exactly one field.
//!
//! A newtype whose field types lead back to itself stays as it is, since replacing it with its
//! field would not terminate. `is_acyclic_newtype` decides that by walking the type constructors
//! reachable through field types.
//!
//! A newtype that takes type parameters is replaced at each instance, with the field type read at
//! that instance, so `Foo Bool` of `type Foo a = unbox struct { data : () -> a };` becomes
//! `() -> Bool`. The declarations stay where they are, and a field-type query unwraps what its
//! substitution saturates: `TypeNode::unwrap_newtypes` is that rewrite, and this pass is what tells
//! the type environment which newtypes to apply it to.
//!
//! The declarations stay because a newtype can occur without its arguments. Take
//! `type [f : *->*] Foo f = box struct { data : f () };` and a program holding a `Foo IO`. `Foo` is
//! boxed, so it stays, and the bare `IO` stays inside it, still naming its declaration. No value has
//! that occurrence for its type: a type of kind `*` headed by a type constructor is saturated, and
//! every saturated occurrence is unwrapped. `field_types(Foo IO)` substitutes `f := IO` into `f ()`
//! and answers with what `IO ()` unwraps to, so a value of `Foo IO` holds a closure.

use crate::{
    ast::{
        export_statement::IOType,
        expr::{expr_let_typed, expr_make_struct, expr_match_typed, expr_var, Expr, ExprNode},
        name::FullName,
        pattern::{Pattern, PatternInfo, PatternNode},
        program::{Program, Symbol, TypeEnv},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::{tycon, TyCon, TyConInfo, TyConVariant},
    },
    fixstd::builtin::{
        make_tuple_name_abs, make_unit_ty, InlineLLVMStructGetBody, InlineLLVMStructPlugInBody,
        InlineLLVMStructPunchBody, InlineLLVMStructSetBody,
    },
    misc::{Map, Set},
};
use std::sync::Arc;

/// Replaces every unwrappable newtype of `prg` with the type of its one field, in the types
/// recorded throughout the program and in the field types the type environment answers with.
pub fn run(prg: &mut Program) {
    let unwrappable_tycons = unwrappable_tycons(&prg.type_env.tycons);
    prg.type_env.unwrap_newtypes(unwrappable_tycons);

    let type_env = prg.type_env.clone();
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym, &type_env);
    }
    run_on_exported_statements(prg, &type_env);
    run_on_entry_io_value(prg, &type_env);
}

/// The type constructors to replace with the type of their one field: the newtypes of `tycons`
/// whose field types do not lead back to them.
fn unwrappable_tycons(tycons: &Map<TyCon, TyConInfo>) -> Set<TyCon> {
    let mut unwrappable_tycons = Set::default();
    for (tc, ti) in tycons {
        // The form of a struct with one field punched out is unwrapped exactly when the struct it
        // punches is, so it is the struct that `is_acyclic_newtype` is asked about. Asking about the
        // punched form itself answers a different question: it names no field type, so the walk
        // finds nothing and says yes even where the struct names itself and stays.
        assert_eq!(
            ti.punched_from.is_some(),
            ti.fields.iter().any(|field| field.is_punched),
            "The declaration of `{}` names a struct it punches iff it has a hole.",
            tc.to_string()
        );
        let deciding_tc = ti.punched_from.as_ref().unwrap_or(tc);
        if is_acyclic_newtype(deciding_tc, tycons) {
            unwrappable_tycons.insert(tc.clone());
        }
    }
    unwrappable_tycons
}

/// `expr` with the type inferred for it unwrapped.
fn unwrap_inferred_type(expr: &Arc<ExprNode>, type_env: &TypeEnv) -> Arc<ExprNode> {
    let type_ = expr.type_.as_ref().unwrap();
    expr.set_type(type_.unwrap_newtypes(type_env))
}

/// `pat` with the pattern of an unwrapped newtype's one field in place of the struct pattern that
/// matched it, and with the type recorded in every pattern unwrapped.
///
/// This is supposed to be called after type aliases are resolved.
fn unwrap_pattern(pat: &Arc<PatternNode>, type_env: &TypeEnv) -> Arc<PatternNode> {
    match &pat.pattern {
        Pattern::Var(v, ty) => {
            // Ignore user-provided type annotation for variable patterns
            let mut info = pat.info.clone();
            unwrap_pattern_info(&mut info, type_env);
            Arc::new(PatternNode {
                pattern: Pattern::Var(v.clone(), ty.clone()),
                info,
            })
        }
        Pattern::Struct(tc, field_to_pat) => {
            if type_env.is_unwrapped_newtype(tc) {
                assert_eq!(field_to_pat.len(), 1);
                let (_, _, pat) = &field_to_pat[0];
                unwrap_pattern(pat, type_env)
            } else {
                let mut field_to_pat = field_to_pat.clone();
                for (_, _, pat) in &mut field_to_pat {
                    *pat = unwrap_pattern(pat, type_env);
                }
                let mut info = pat.info.clone();
                unwrap_pattern_info(&mut info, type_env);
                Arc::new(PatternNode {
                    pattern: Pattern::Struct(tc.clone(), field_to_pat),
                    info,
                })
            }
        }
        Pattern::Union(variant, variant_src, subpat) => {
            let mut info = pat.info.clone();
            unwrap_pattern_info(&mut info, type_env);
            Arc::new(PatternNode {
                pattern: Pattern::Union(
                    variant.clone(),
                    variant_src.clone(),
                    unwrap_pattern(subpat, type_env),
                ),
                info,
            })
        }
    }
}

/// Unwraps the type recorded in `pat_info`, in place.
fn unwrap_pattern_info(pat_info: &mut PatternInfo, type_env: &TypeEnv) {
    if let Some(ty) = &mut pat_info.type_ {
        *ty = ty.unwrap_newtypes(type_env);
    }
}

/// Unwraps the types recorded in each export statement. An exported `IO` function is recorded as
/// state-passing, since unwrapping `IO` leaves the function that takes the state.
fn run_on_exported_statements(prg: &mut Program, type_env: &TypeEnv) {
    for export in &mut prg.export_statements {
        if let Some(expr) = &export.value_expr {
            let expr = unwrap_inferred_type(expr, type_env);
            export.value_expr = Some(expr);
        }
        if let Some(ft) = &mut export.function_type {
            for dom in &mut ft.doms {
                *dom = dom.unwrap_newtypes(type_env);
            }
            ft.codom = ft.codom.unwrap_newtypes(type_env);
            if matches!(ft.io_type, IOType::IO) {
                ft.io_type = IOType::IOState;
            }
        }
    }
}

/// Unwraps the type inferred for the `IO` value the program runs at entry.
fn run_on_entry_io_value(prg: &mut Program, type_env: &TypeEnv) {
    if let Some(entry_io_value) = &mut prg.entry_io_value {
        let expr = unwrap_inferred_type(entry_io_value, type_env);
        prg.entry_io_value = Some(expr);
    }
}

/// Unwraps the type of one symbol and rewrites the expression that defines it.
fn run_on_symbol(sym: &mut Symbol, type_env: &TypeEnv) {
    let mut unwrapper = ExprUnwrapper { type_env };
    sym.ty = sym.ty.unwrap_newtypes(type_env);
    sym.expr = Some(unwrapper.traverse(&sym.expr.as_ref().unwrap()).expr);
}

/// Rewrites the types recorded in one symbol's expression, and replaces the field operations of an
/// unwrapped newtype by the operations on the field itself.
struct ExprUnwrapper<'a> {
    /// The type environment holding which newtypes are unwrapped.
    type_env: &'a TypeEnv,
}

impl<'a> ExprUnwrapper<'a> {
    /// Whether the local name `var_name` is bound to a value of an unwrapped newtype, read from the
    /// type the scope records for that binding.
    fn is_local_of_unwrapped_newtype(&self, var_name: &FullName, state: &VisitState) -> bool {
        assert!(var_name.is_local());
        let ty = state.scope.get_local(&var_name.name).unwrap().unwrap();
        self.type_env
            .is_unwrapped_newtype(ty.toplevel_tycon().unwrap().as_ref())
    }
}

impl<'a> ExprVisitor for ExprUnwrapper<'a> {
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
        let expr = unwrap_inferred_type(expr, self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = unwrap_inferred_type(expr, self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Unwraps the type recorded for an inline LLVM expression, and replaces the read, the write,
    /// the punch and the plug-in of an unwrapped newtype's field by the field value itself, which
    /// is what a value of that type has become.
    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        let old_ty = expr.type_.as_ref().unwrap().clone();
        let mut expr = unwrap_inferred_type(expr, self.type_env);
        let new_ty = expr.type_.as_ref().unwrap().clone();

        let llvm = if let Expr::LLVM(llvm) = expr.expr.as_ref() {
            llvm.as_ref().clone()
        } else {
            unreachable!()
        };

        // `llvm.generic_ty` stays as it is: type checking is the last pass that reads it.

        // Replace StructGetBody, StructSetBody, StructPunchBody, and StructPlugInBody for structures defined by the newtype pattern.
        let gen = llvm.generator.as_ref();
        if let Some(body) = gen.as_any().downcast_ref::<InlineLLVMStructGetBody>() {
            // @ : S -> F = |s| GetBody(s)
            // =>
            // @ : F -> F = |s| s
            let field_ty = new_ty;
            let struct_name = body.var_name.clone();
            if self.is_local_of_unwrapped_newtype(&struct_name, state) {
                expr = expr_var(struct_name, expr.source.clone()).set_type(field_ty);
            }
        } else if let Some(body) = gen.as_any().downcast_ref::<InlineLLVMStructSetBody>() {
            // set : F -> S -> S = |f, s| SetBody(f)
            // =>
            // set : F -> F -> F = |f, s| f
            let field_ty = new_ty;
            let struct_ty = old_ty;
            let struct_tc = struct_ty.toplevel_tycon().unwrap();
            if self.type_env.is_unwrapped_newtype(struct_tc.as_ref()) {
                let field_name = body.value_name.clone();
                expr = expr_var(field_name, expr.source.clone()).set_type(field_ty);
            }
        } else if let Some(body) = gen.as_any().downcast_ref::<InlineLLVMStructPunchBody>() {
            // punch : S -> (F, S*) = |s| Punch(s)
            // =>
            // punch : F -> (F, ()) = |s| (s, ())
            let field_unit_ty = new_ty;
            let struct_name = body.var_name.clone();
            if self.is_local_of_unwrapped_newtype(&struct_name, state) {
                let field_ty = field_unit_ty.collect_type_arguments()[0].clone();
                let unit_ty = make_unit_ty();
                let struct_expr = expr_var(struct_name, expr.source.clone()).set_type(field_ty);
                let unit_expr =
                    expr_make_struct(tycon(make_tuple_name_abs(0)), vec![]).set_type(unit_ty);
                expr = expr_make_struct(
                    tycon(make_tuple_name_abs(2)),
                    vec![("0".to_string(), struct_expr), ("1".to_string(), unit_expr)],
                )
                .set_type(field_unit_ty);
            }
        } else if let Some(body) = gen.as_any().downcast_ref::<InlineLLVMStructPlugInBody>() {
            // plug_in : S* -> F -> S = |s, f| PlugIn(s, f)
            // =>
            // plug_in : () -> F -> F = |_, f| f
            let struct_ty = old_ty;
            let struct_tc = struct_ty.toplevel_tycon().unwrap();
            if self.type_env.is_unwrapped_newtype(struct_tc.as_ref()) {
                let field_ty = new_ty;
                let field_name = body.field_name.clone();
                assert!(field_name.is_local());
                expr = expr_var(field_name, expr.source.clone()).set_type(field_ty);
            }
        }

        EndVisitResult::changed(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = unwrap_inferred_type(expr, self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = unwrap_inferred_type(expr, self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Unwraps the type recorded for a `let`, and rewrites the pattern it binds with, so that a
    /// pattern matching an unwrapped newtype's struct becomes the pattern of its one field.
    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = unwrap_inferred_type(expr, self.type_env);
        if let Expr::Let(pat, body, val) = expr.expr.as_ref() {
            let pat = unwrap_pattern(pat, self.type_env);
            expr = expr_let_typed(pat, body.clone(), val.clone());
        } else {
            unreachable!()
        }
        EndVisitResult::changed(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = unwrap_inferred_type(expr, self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Unwraps the type recorded for a `match`, and rewrites the pattern of each arm, so that a
    /// pattern matching an unwrapped newtype's struct becomes the pattern of its one field.
    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = unwrap_inferred_type(expr, self.type_env);
        if let Expr::Match(scrut, arms) = expr.expr.as_ref() {
            let arms = arms
                .iter()
                .map(|(pat, arm_expr)| (unwrap_pattern(pat, self.type_env), arm_expr.clone()))
                .collect();
            expr = expr_match_typed(scrut.clone(), arms);
        } else {
            unreachable!()
        }
        EndVisitResult::changed(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Unwraps the type recorded for a struct literal, and replaces a literal of an unwrapped
    /// newtype by the expression its one field is built from.
    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let mut expr = unwrap_inferred_type(expr, self.type_env);
        if let Expr::MakeStruct(tycon, fields) = expr.expr.as_ref() {
            if self.type_env.is_unwrapped_newtype(tycon) {
                expr = fields[0].2.clone();
            }
        } else {
            unreachable!()
        }
        EndVisitResult::changed(expr)
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
        let expr = unwrap_inferred_type(expr, self.type_env);
        EndVisitResult::changed(expr)
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
        let expr = unwrap_inferred_type(expr, self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = unwrap_inferred_type(expr, self.type_env);
        EndVisitResult::changed(expr)
    }
}

/// Is this type constructor a "newtype", i.e., is it an unbox struct type with only one field?
fn is_newtype(tycon: &TyCon, env: &Map<TyCon, TyConInfo>) -> bool {
    let ti = env.get(tycon).unwrap();
    ti.is_unbox && ti.variant == TyConVariant::Struct && ti.fields.len() == 1
}

/// Is this type constructor a newtype whose field types do not lead back to it?
///
/// Replacing a newtype with the type of its field terminates exactly when this walk reaches the
/// end, so this answer is what makes `TypeNode::unwrap_newtypes` a finite rewrite.
fn is_acyclic_newtype(tc: &TyCon, env: &Map<TyCon, TyConInfo>) -> bool {
    // If this TyCon is not a newtype, return false.
    if !is_newtype(tc, env) {
        return false;
    }

    let mut visited = Set::default();
    let mut pending_tcs = vec![tc.clone()];
    while let Some(visiting_tc) = pending_tcs.pop() {
        visited.insert(visiting_tc.clone());
        if !is_newtype(&visiting_tc, env) {
            continue;
        }
        let ti = env.get(&visiting_tc).unwrap();
        let field_ty = &ti.fields[0].ty;
        let mut field_tcs = Set::default();
        field_ty.collect_tycons(&mut field_tcs);
        for field_tc in field_tcs {
            if field_tc == *tc {
                return false;
            }
            if visited.contains(&field_tc) {
                continue;
            }
            pending_tcs.push(field_tc);
        }
    }

    return true;
}
