//! Unwrap the newtype pattern, i.e., type A = unbox struct { data : B } to B.
//!
//! The newtype pattern is an unboxed struct of exactly one field.
//!
//! A newtype whose field types lead back to itself stays as it is, since replacing it with its
//! field would not terminate. `NewtypeUnwrapping` decides that by walking the type constructors
//! reachable through field types.
//!
//! This optimization should be run after the remove-hk-tyvar transform.
//! The unwrap-newtype optimization cannot be applied to programs with generic type definitions such as `type [f : * -> *] Foo f = box struct { data : f () };`.
//! This is because if there is an expression with a type like `Foo IO`, `IO` is a partially applied type and cannot be unwrapped.

use crate::{
    ast::{
        export_statement::IOType,
        expr::{expr_let_typed, expr_make_struct, expr_match_typed, expr_var, Expr, ExprNode},
        pattern::{Pattern, PatternInfo, PatternNode},
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::{tycon, TyCon, TyConInfo, TyConVariant, Type, TypeNode},
    },
    fixstd::builtin::{
        make_tuple_name_abs, make_unit_ty, InlineLLVMStructGetBody, InlineLLVMStructPlugInBody,
        InlineLLVMStructPunchBody, InlineLLVMStructSetBody,
    },
    misc::{Map, Set},
};
use std::sync::Arc;

/// Replaces every unwrappable newtype of `prg` with the type of its one field, in the types
/// recorded throughout the program and in the type environment.
pub fn run(prg: &mut Program) {
    let unwrapping = NewtypeUnwrapping::new(prg.type_env.tycons.as_ref().clone());

    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym, &unwrapping);
    }
    run_on_exported_statements(prg, &unwrapping);
    run_on_entry_io_value(prg, &unwrapping);

    prg.type_env.tycons = Arc::new(unwrapping.unwrapped_tycons());
}

/// The type constructors this pass replaces with the type of their one field, together with the
/// type environment they were chosen against.
///
/// The choice is made once and every rewrite asks this same value, so a type constructor is
/// replaced wherever it heads a saturated type and dropped from the type environment only if it is
/// replaced: no saturated occurrence is left naming a type the environment no longer declares.
struct NewtypeUnwrapping {
    /// The type environment as this pass received it. Field types are read from here, so a
    /// declaration reads as it did when the choices were made.
    tycons: Map<TyCon, TyConInfo>,
    /// The type constructors to replace.
    unwrappable_tycons: Set<TyCon>,
}

impl NewtypeUnwrapping {
    /// Chooses the type constructors to replace: the newtypes of `tycons` whose field types do not
    /// lead back to them.
    fn new(tycons: Map<TyCon, TyConInfo>) -> Self {
        let mut unwrappable_tycons = Set::default();
        for tc in tycons.keys() {
            // The form of a struct with one field punched out is unwrapped exactly when the struct
            // it punches is, so it is the struct that `is_acyclic_newtype` is asked about.
            // Asking about the punched form itself answers a different question: its name appears
            // in no field type, so the walk finds nothing and says yes even where the struct names
            // itself and stays.
            let deciding_tc = match tc.unpunched_tycon() {
                Some(struct_tc) if tycons.contains_key(&struct_tc) => struct_tc,
                _ => tc.clone(),
            };
            if is_acyclic_newtype(&deciding_tc, &tycons) {
                unwrappable_tycons.insert(tc.clone());
            }
        }
        NewtypeUnwrapping {
            tycons,
            unwrappable_tycons,
        }
    }

    /// Whether `tc` is replaced by the type of its one field. The form of a struct with one field
    /// punched out answers as the struct it punches does.
    fn is_unwrappable(&self, tc: &TyCon) -> bool {
        self.unwrappable_tycons.contains(tc)
    }

    /// `ty` with each unwrappable type constructor in it replaced by the type of its one field.
    ///
    /// This is supposed to be called after type aliases are resolved.
    fn unwrap_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        // First, replace the top-level type constructor if it is a newtype. The field type is taken
        // at this instance, so `Foo Bool` of `type Foo a = unbox struct { data : () -> a }` becomes
        // `() -> Bool`.
        if let Some(top_tc) = ty.toplevel_tycon() {
            let top_ti = self.tycons.get(&top_tc).unwrap();
            let is_fully_applied = top_ti.tyvars.len() == ty.collect_type_argments().len();
            if is_fully_applied && self.is_unwrappable(&top_tc) {
                // A value of the form with the one field punched out holds nothing once that field
                // is the hole, so it becomes the unit type.
                if top_ti.fields[0].is_punched {
                    return make_unit_ty();
                }
                let field_ty = ty.field_types_via_tycons(&self.tycons)[0].clone();
                return self.unwrap_type(&field_ty);
            }
        }

        // If the top-level tycon is not a newtype, recursively process type arguments
        match &ty.ty {
            Type::TyVar(_) => ty.clone(),
            Type::TyCon(_) => ty.clone(),
            Type::TyApp(fun_ty, arg_ty) => ty
                .set_tyapp_fun(self.unwrap_type(fun_ty))
                .set_tyapp_arg(self.unwrap_type(arg_ty)),
            Type::AssocTy(_, _args) => {
                unimplemented!("AssocTy is not supported in unwrap_type")
            }
        }
    }

    /// `expr` with the type inferred for it unwrapped.
    fn unwrap_inferred_type(&self, expr: &Arc<ExprNode>) -> Arc<ExprNode> {
        let type_ = expr.type_.as_ref().unwrap();
        expr.set_type(self.unwrap_type(type_))
    }

    /// `pat` with each unwrappable struct pattern replaced by the pattern of its one field, and the
    /// type recorded in every pattern that stays unwrapped.
    ///
    /// This is supposed to be called after type aliases are resolved.
    fn unwrap_pattern(&self, pat: &Arc<PatternNode>) -> Arc<PatternNode> {
        match &pat.pattern {
            Pattern::Var(v, ty) => {
                // Ignore user-provided type annotation for variable patterns
                let mut info = pat.info.clone();
                self.unwrap_pattern_info(&mut info);
                Arc::new(PatternNode {
                    pattern: Pattern::Var(v.clone(), ty.clone()),
                    info,
                })
            }
            Pattern::Struct(tc, field_to_pat) => {
                if self.is_unwrappable(tc) {
                    assert_eq!(field_to_pat.len(), 1);
                    let (_, _, pat) = &field_to_pat[0];
                    self.unwrap_pattern(pat)
                } else {
                    let mut field_to_pat = field_to_pat.clone();
                    for (_, _, pat) in &mut field_to_pat {
                        *pat = self.unwrap_pattern(pat);
                    }
                    let mut info = pat.info.clone();
                    self.unwrap_pattern_info(&mut info);
                    Arc::new(PatternNode {
                        pattern: Pattern::Struct(tc.clone(), field_to_pat),
                        info,
                    })
                }
            }
            Pattern::Union(variant, variant_src, subpat) => {
                let mut info = pat.info.clone();
                self.unwrap_pattern_info(&mut info);
                Arc::new(PatternNode {
                    pattern: Pattern::Union(
                        variant.clone(),
                        variant_src.clone(),
                        self.unwrap_pattern(subpat),
                    ),
                    info,
                })
            }
        }
    }

    /// Unwraps the type recorded in `pat_info`, in place.
    fn unwrap_pattern_info(&self, pat_info: &mut PatternInfo) {
        if let Some(ty) = &mut pat_info.type_ {
            *ty = self.unwrap_type(ty);
        }
    }

    /// The type environment this pass leaves behind: the declarations of the type constructors that
    /// stay, with their field types unwrapped.
    ///
    /// A replaced type constructor loses its declaration, so a type this pass failed to rewrite
    /// fails at the first lookup of that declaration rather than laying its values out as the
    /// struct they were to stop being.
    fn unwrapped_tycons(&self) -> Map<TyCon, TyConInfo> {
        let mut env = Map::default();
        for (tc, ti) in &self.tycons {
            if self.is_unwrappable(tc) {
                continue;
            }
            let mut ti = ti.clone();
            for field in &mut ti.fields {
                field.ty = self.unwrap_type(&field.ty);
            }
            env.insert(tc.clone(), ti);
        }
        env
    }
}

/// Unwraps the types recorded in each export statement. An exported `IO` function is recorded as
/// state-passing, since unwrapping `IO` leaves the function that takes the state.
fn run_on_exported_statements(prg: &mut Program, unwrapping: &NewtypeUnwrapping) {
    for export in &mut prg.export_statements {
        if let Some(expr) = &export.value_expr {
            let expr = unwrapping.unwrap_inferred_type(expr);
            export.value_expr = Some(expr);
        }
        if let Some(ft) = &mut export.function_type {
            for dom in &mut ft.doms {
                *dom = unwrapping.unwrap_type(dom);
            }
            ft.codom = unwrapping.unwrap_type(&ft.codom);
            if matches!(ft.io_type, IOType::IO) {
                ft.io_type = IOType::IOState;
            }
        }
    }
}

/// Unwraps the type inferred for the `IO` value the program runs at entry.
fn run_on_entry_io_value(prg: &mut Program, unwrapping: &NewtypeUnwrapping) {
    if let Some(entry_io_value) = &mut prg.entry_io_value {
        let expr = unwrapping.unwrap_inferred_type(entry_io_value);
        prg.entry_io_value = Some(expr);
    }
}

/// Unwraps the type of one symbol and rewrites the expression that defines it.
fn run_on_symbol(sym: &mut Symbol, unwrapping: &NewtypeUnwrapping) {
    let mut unwrapper = ExprUnwrapper { unwrapping };
    sym.ty = unwrapping.unwrap_type(&sym.ty);
    sym.expr = Some(unwrapper.traverse(&sym.expr.as_ref().unwrap()).expr);
}

/// Rewrites the types recorded in one symbol's expression, and replaces the field operations of an
/// unwrapped newtype by the operations on the field itself.
struct ExprUnwrapper<'a> {
    /// The choices made for the whole program, applied here to one symbol.
    unwrapping: &'a NewtypeUnwrapping,
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
        let expr = self.unwrapping.unwrap_inferred_type(expr);
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
        let expr = self.unwrapping.unwrap_inferred_type(expr);
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
        let mut expr = self.unwrapping.unwrap_inferred_type(expr);
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
            assert!(struct_name.is_local());
            let struct_ty = state.scope.get_local(&struct_name.name).unwrap().unwrap();
            let struct_tc = struct_ty.toplevel_tycon().unwrap();
            if self.unwrapping.is_unwrappable(struct_tc.as_ref()) {
                expr = expr_var(struct_name, expr.source.clone()).set_type(field_ty);
            }
        } else if let Some(body) = gen.as_any().downcast_ref::<InlineLLVMStructSetBody>() {
            // set : F -> S -> S = |f, s| SetBody(f)
            // =>
            // set : F -> F -> F = |f, s| f
            let field_ty = new_ty;
            let struct_ty = old_ty;
            let struct_tc = struct_ty.toplevel_tycon().unwrap();
            if self.unwrapping.is_unwrappable(struct_tc.as_ref()) {
                let field_name = body.value_name.clone();
                expr = expr_var(field_name, expr.source.clone()).set_type(field_ty);
            }
        } else if let Some(body) = gen.as_any().downcast_ref::<InlineLLVMStructPunchBody>() {
            // punch : S -> (F, S*) = |s| Punch(s)
            // =>
            // punch : F -> (F, ()) = |s| (s, ())
            let field_unit_ty = new_ty;
            let struct_name = body.var_name.clone();
            assert!(struct_name.is_local());
            let struct_ty = state.scope.get_local(&struct_name.name).unwrap().unwrap();
            let struct_tc = struct_ty.toplevel_tycon().unwrap();
            if self.unwrapping.is_unwrappable(struct_tc.as_ref()) {
                let field_ty = field_unit_ty.collect_type_argments()[0].clone();
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
            if self.unwrapping.is_unwrappable(struct_tc.as_ref()) {
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
        let expr = self.unwrapping.unwrap_inferred_type(expr);
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
        let expr = self.unwrapping.unwrap_inferred_type(expr);
        EndVisitResult::changed(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = self.unwrapping.unwrap_inferred_type(expr);
        if let Expr::Let(pat, body, val) = expr.expr.as_ref() {
            let pat = self.unwrapping.unwrap_pattern(pat);
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
        let expr = self.unwrapping.unwrap_inferred_type(expr);
        EndVisitResult::changed(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = self.unwrapping.unwrap_inferred_type(expr);
        if let Expr::Match(scrut, arms) = expr.expr.as_ref() {
            let arms = arms
                .iter()
                .map(|(pat, arm_expr)| (self.unwrapping.unwrap_pattern(pat), arm_expr.clone()))
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
        let mut expr = self.unwrapping.unwrap_inferred_type(expr);
        if let Expr::MakeStruct(tycon, fields) = expr.expr.as_ref() {
            if self.unwrapping.is_unwrappable(tycon) {
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
        let expr = self.unwrapping.unwrap_inferred_type(expr);
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
        let expr = self.unwrapping.unwrap_inferred_type(expr);
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
        let expr = self.unwrapping.unwrap_inferred_type(expr);
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
/// end, so this answer is what makes `NewtypeUnwrapping::unwrap_type` a finite rewrite.
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
