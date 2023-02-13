use super::*;

#[derive(Clone)]
pub struct Scope<T> {
    var: HashMap<String, ScopeValue<T>>,
}

#[derive(Clone)]
struct ScopeValue<T> {
    global: HashMap<NameSpace, T>,
    local: Vec<T>,
}

impl<T> Default for ScopeValue<T> {
    fn default() -> Self {
        Self {
            global: Default::default(),
            local: Default::default(),
        }
    }
}

impl<T> Default for Scope<T> {
    fn default() -> Self {
        Self {
            var: Default::default(),
        }
    }
}

impl<T> Scope<T>
where
    T: Clone,
{
    // TODO: throw TypeError when unwrap fails.
    pub fn push(self: &mut Self, name: &str, ty: &T) {
        if !self.var.contains_key(name) {
            self.var.insert(String::from(name), Default::default());
        }
        self.var.get_mut(name).unwrap().local.push(ty.clone());
    }
    pub fn pop(self: &mut Self, name: &str) {
        self.var.get_mut(name).unwrap().local.pop();
    }
    pub fn local_names(&self) -> HashSet<Name> {
        let mut res: HashSet<Name> = Default::default();
        for (name, sv) in &self.var {
            if !sv.local.is_empty() {
                res.insert(name.clone());
            }
        }
        res
    }
    fn get_mut(self: &mut Self, name: &str) -> Option<&mut ScopeValue<T>> {
        self.var.get_mut(name)
    }
    pub fn add_global(&mut self, name: Name, namespace: &NameSpace, value: &T) {
        if !self.var.contains_key(&name) {
            self.var.insert(name.clone(), Default::default());
        }
        if self.var[&name].global.contains_key(namespace) {
            error_exit(&format!(
                "duplicate definition for `{}.{}`",
                namespace.to_string(),
                name
            ))
        }
        self.get_mut(&name)
            .unwrap()
            .global
            .insert(namespace.clone(), value.clone());
    }

    // Get candidates list for overload resolution.
    // If `namespace` is unspecified (None) and a local variable `name` is found, then that local variable is returned.
    // If `namespace` is unspecified and no local variable `name` is found, then all global variables are returned.
    // If `namespace` is specified and non-empty, then returns all global variables whose namespaces have `namespace` as suffix.
    // If `namespace` is specified and empty, then returns local variable `name`.
    fn overloaded_candidates(&self, name: &FullName) -> Vec<(NameSpace, T)> {
        if !self.var.contains_key(&name.name) {
            return vec![];
        }
        let sv = &self.var[&name.name];
        if name.is_local() && sv.local.len() > 0 {
            vec![(NameSpace::local(), sv.local.last().unwrap().clone())]
        } else {
            sv.global
                .iter()
                .filter(|(ns, _)| name.namespace.is_suffix(ns))
                .map(|(ns, v)| (ns.clone(), v.clone()))
                .collect()
        }
    }
}

// Type substitution. Name of type variable -> type.
// Managed so that the value (a type) of this HashMap doesn't contain a type variable that appears in keys. i.e.,
// when we want to COMPLETELY substitute type variables in a type by `substitution`, we only apply this mapy only ONCE.
#[derive(Clone)]
pub struct Substitution {
    pub data: HashMap<Name, Arc<TypeNode>>,
}

impl Default for Substitution {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl Substitution {
    // Make single substitution.
    pub fn single(var: &str, ty: Arc<TypeNode>) -> Self {
        let mut data = HashMap::<String, Arc<TypeNode>>::default();
        data.insert(var.to_string(), ty);
        Self { data }
    }

    // Add (=compose) substitution.
    pub fn add_substitution(&mut self, other: &Self) {
        for (_var, ty) in self.data.iter_mut() {
            let new_ty = other.substitute_type(&ty);
            *ty = new_ty;
        }
        for (var, ty) in &other.data {
            self.data.insert(var.to_string(), ty.clone());
        }
    }

    // Merge substitution.
    // Returns true when merge succeeds.
    fn merge_substitution(&mut self, other: &Self) -> bool {
        for (var, ty) in &other.data {
            if self.data.contains_key(var) {
                if self.data[var] != *ty {
                    return false;
                }
            } else {
                self.data.insert(var.to_string(), ty.clone());
            }
        }
        return true;
    }

    // Apply substitution to predicate.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        p.ty = self.substitute_type(&p.ty);
    }

    // Apply substitution to type
    pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        match &ty.ty {
            Type::TyVar(tyvar) => self
                .data
                .get(&tyvar.name)
                .map_or(ty.clone(), |sub| sub.clone()),
            Type::TyCon(_) => ty.clone(),
            Type::TyApp(fun, arg) => {
                let fun = self.substitute_type(fun);
                let arg = self.substitute_type(arg);
                type_tyapp(fun, arg)
            }
            Type::FunTy(param, body) => {
                type_fun(self.substitute_type(&param), self.substitute_type(&body))
            }
        }
    }

    // Apply substitution to qualified type.
    pub fn substitute_qualtype(&self, qual_type: &mut QualType) {
        for pred in &mut qual_type.preds {
            self.substitute_predicate(pred);
        }
        qual_type.ty = self.substitute_type(&qual_type.ty);
    }

    // Calculate minimum substitution to unify two types.
    pub fn unify(type_env: &TypeEnv, ty1: &Arc<TypeNode>, ty2: &Arc<TypeNode>) -> Option<Self> {
        match &ty1.ty {
            Type::TyVar(var1) => {
                return Self::unify_tyvar(type_env, &var1, ty2);
            }
            _ => {}
        }
        match &ty2.ty {
            Type::TyVar(var2) => {
                return Self::unify_tyvar(type_env, &var2, ty1);
            }
            _ => {}
        }
        match &ty1.ty {
            Type::TyVar(_) => unreachable!(),
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1 == tc2 {
                        return Some(Self::default());
                    } else {
                        return None;
                    }
                }
                _ => {
                    return None;
                }
            },
            Type::TyApp(fun1, arg1) => match &ty2.ty {
                Type::TyApp(fun2, arg2) => {
                    let mut ret = Self::default();
                    match Self::unify(type_env, &fun1, &fun2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    let arg1 = ret.substitute_type(arg1);
                    let arg2 = ret.substitute_type(arg2);
                    match Self::unify(type_env, &arg1, &arg2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    return Some(ret);
                }
                _ => {
                    return None;
                }
            },
            Type::FunTy(arg_ty1, ret_ty1) => match &ty2.ty {
                Type::FunTy(arg_ty2, ret_ty2) => {
                    let mut ret = Self::default();
                    match Self::unify(type_env, &arg_ty1, &arg_ty2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    let ret_ty1 = ret.substitute_type(ret_ty1);
                    let ret_ty2 = ret.substitute_type(ret_ty2);
                    match Self::unify(type_env, &ret_ty1, &ret_ty2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    return Some(ret);
                }
                _ => {
                    return None;
                }
            },
        }
    }

    // Subroutine of unify().
    fn unify_tyvar(type_env: &TypeEnv, tyvar1: &Arc<TyVar>, ty2: &Arc<TypeNode>) -> Option<Self> {
        match &ty2.ty {
            Type::TyVar(tyvar2) => {
                if tyvar1.name == tyvar2.name {
                    // Avoid adding circular subsitution.
                    return Some(Self::default());
                }
            }
            _ => {}
        };
        if ty2.free_vars().contains_key(&tyvar1.name) {
            // For example, this error occurs when
            // the user is making `f c` in the implementation of
            // `map: [f: Functor] (a -> b) -> f a -> f b; map = \f -> \c -> (...)`;
            error_exit(&format!(
                "cannot identify type `{}` and `{}`.",
                tyvar1.name,
                ty2.to_string_normalize()
            ));
        }
        if tyvar1.kind != ty2.kind(type_env) {
            error_exit("Kinds do not match.");
        }
        Some(Self::single(&tyvar1.name, ty2.clone()))
    }

    // Calculate minimum substitution s such that `s(ty1) = ty2`.
    pub fn matching(type_env: &TypeEnv, ty1: &Arc<TypeNode>, ty2: &Arc<TypeNode>) -> Option<Self> {
        match &ty1.ty {
            Type::TyVar(v1) => Self::unify_tyvar(type_env, v1, ty2),
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1 == tc2 {
                        Some(Self::default())
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Type::TyApp(fun1, arg1) => match &ty2.ty {
                Type::TyApp(fun2, arg2) => {
                    let mut ret = Self::default();
                    match Self::matching(type_env, fun1, fun2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    match Self::matching(type_env, arg1, arg2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    Some(ret)
                }
                _ => None,
            },
            Type::FunTy(src1, dst1) => match &ty2.ty {
                Type::FunTy(src2, dst2) => {
                    let mut ret = Self::default();
                    match Self::matching(type_env, src1, src2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    match Self::matching(type_env, dst1, dst2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    Some(ret)
                }
                _ => None,
            },
        }
    }
}

// Context under type-checking.
// Reference: https://uhideyuki.sakura.ne.jp/studs/index.cgi/ja/HindleyMilnerInHaskell#fn6
#[derive(Clone, Default)]
pub struct TypeCheckContext {
    // The identifier of type variables.
    tyvar_id: u32,
    // Scoped map of variable name -> scheme. (Assamptions of type inference.)
    pub scope: Scope<Arc<Scheme>>,
    // Substitution.
    substitution: Substitution,
    // Predicates
    pub predicates: Vec<Predicate>,
    // Trait environment.
    trait_env: TraitEnv,
    // List of type constructors.
    pub type_env: TypeEnv,
}

impl TypeCheckContext {
    // Creaate instance.
    pub fn new(trait_env: TraitEnv, type_env: TypeEnv) -> Self {
        Self {
            tyvar_id: Default::default(),
            scope: Default::default(),
            substitution: Default::default(),
            predicates: Default::default(),
            type_env,
            trait_env,
        }
    }

    // Generate new type variable.
    pub fn new_tyvar(&mut self) -> String {
        let id = self.tyvar_id;
        self.tyvar_id += 1;
        "%a".to_string() + &id.to_string() // To avlid confliction with user-defined type variable, we add prefix #.
    }

    // Apply substitution to type.
    pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        self.substitution.substitute_type(ty)
    }

    // Apply substitution to a predicate.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        self.substitution.substitute_predicate(p)
    }

    // Instantiate a scheme.
    // Returns predicates if append_predicates = false or append them to self if append_predicates = true.
    pub fn instantiate_scheme(
        &mut self,
        scheme: &Arc<Scheme>,
        append_predicates: bool,
    ) -> (Vec<Predicate>, Arc<TypeNode>) {
        let mut sub = Substitution::default();
        for (var, kind) in &scheme.vars {
            let new_var_name = self.new_tyvar();
            sub.add_substitution(&Substitution::single(&var, type_tyvar(&new_var_name, kind)));
        }
        let mut preds = scheme.preds.clone();
        for p in &mut preds {
            sub.substitute_predicate(p);
        }
        if append_predicates {
            self.predicates.append(&mut preds);
        }
        (preds, sub.substitute_type(&scheme.ty))
    }

    // // Make a scheme from a type by generalizing type variable that does not appear in fixed_vars.
    // fn generalize_to_scheme(
    //     &mut self,
    //     ty: &Arc<TypeNode>,
    //     fixed_vars: &HashSet<Name>,
    // ) -> Arc<Scheme> {
    //     // Get generalized type and predicates.
    //     let ty = self.substitute_type(ty);
    //     let mut preds = std::mem::replace(&mut self.predicates, vec![]);

    //     // Reduce predicates.
    //     for p in &mut preds {
    //         self.substitute_predicate(p);
    //     }
    //     let preds = match self.trait_env.reduce(&preds, &self.type_env) {
    //         Some(ps) => ps,
    //         None => self.error_exit_on_predicates(),
    //     };

    //     // Collect variables that appear in scope.
    //     // let mut vars_in_scope: HashSet<String> = Default::default();
    //     // for (_var, scp) in &self.scope.var {
    //     //     for scm in &scp.local {
    //     //         for (var_in_scope, _) in self.substitute_scheme(&scm).free_vars() {
    //     //             vars_in_scope.insert(var_in_scope);
    //     //         }
    //     //     }
    //     // }

    //     // Calculate genealized variables.
    //     let mut gen_vars = ty.free_vars();
    //     for v in fixed_vars {
    //         gen_vars.remove(v);
    //     }

    //     // Split predicates to generalized and deferred.
    //     let mut gen_preds: Vec<Predicate> = Default::default(); // Generalized predicates.
    //     let mut def_preds: Vec<Predicate> = Default::default(); // Deferred predicates.
    //     for p in preds {
    //         if p.ty.free_vars().iter().all(|(v, _)| fixed_vars.contains(v)) {
    //             // All free variables of p appears in fixed_vars.
    //             def_preds.push(p);
    //         } else if p
    //             .ty
    //             .free_vars()
    //             .iter()
    //             .any(|(v, _)| !fixed_vars.contains(v) && gen_vars.contains_key(v))
    //         {
    //             // A free variable of p appears neither in fixed_vars and generalized variables.
    //             error_exit(&format!("ambiguous type variable in `{}`", p.to_string()))
    //         } else {
    //             // A free variable of p appears in generalized variables.
    //             gen_preds.push(p);
    //         }
    //     }

    //     self.predicates = def_preds;
    //     Scheme::generalize(gen_vars, gen_preds, ty)
    // }

    // Update substitution to unify two types.
    // When substitution fails, it has no side effect to self.
    pub fn unify(&mut self, ty1: &Arc<TypeNode>, ty2: &Arc<TypeNode>) -> bool {
        let ty1 = &self.substitute_type(ty1);
        let ty2 = &self.substitute_type(ty2);
        match Substitution::unify(&self.type_env, ty1, ty2) {
            Some(sub) => {
                self.substitution.add_substitution(&sub);
                return true;
            }
            None => {
                return false;
            }
        }
    }

    // Reduce predicates.
    // If predicates are unsatisfiable, do nothing and return false.
    pub fn reduce_predicates(&mut self) -> bool {
        let mut preds = std::mem::replace(&mut self.predicates, vec![]);
        for p in &mut preds {
            self.substitute_predicate(p);
        }
        self.predicates.append(&mut preds);
        match self.trait_env.reduce(&self.predicates, &self.type_env) {
            Some(ps) => {
                self.predicates = ps;
                return true;
            }
            None => {
                return false;
            }
        }
    }

    // Perform typechecking.
    // Update type substitution so that `ei` has type `ty`.
    // Returns given AST augmented with inferred information.
    pub fn unify_type_of_expr(&mut self, ei: &Arc<ExprNode>, ty: Arc<TypeNode>) -> Arc<ExprNode> {
        let ei = ei.set_inferred_type(ty.clone());
        match &*ei.expr {
            Expr::Var(var) => {
                let candidates = self.scope.overloaded_candidates(&var.name);
                let candidates: Vec<(TypeCheckContext, NameSpace)> = candidates
                    .iter()
                    .filter_map(|(ns, scm)| {
                        let mut tc = self.clone();
                        let (_, var_ty) = tc.instantiate_scheme(&scm, true);
                        // if var_ty is unifiable to the required type and predicates are satisfiable, then thie candidate is ok.
                        if tc.unify(&var_ty, &ty) {
                            if tc.reduce_predicates() {
                                Some((tc, ns.clone()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                if candidates.is_empty() {
                    error_exit_with_src(
                        &format!(
                            "No name `{}` of type `{}` is found.",
                            var.name.to_string(),
                            &self.substitute_type(&ty).to_string_normalize()
                        ),
                        &var.source,
                    );
                } else if candidates.len() >= 2 {
                    let candidates_str = candidates
                        .iter()
                        .map(|(_, ns)| {
                            let nsn = FullName::new(ns, &var.name.name);
                            "`".to_string() + &nsn.to_string() + "`"
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    error_exit_with_src(
                        &format!(
                            "Name `{}` is ambiguous: there are {}.",
                            var.name.to_string(),
                            candidates_str
                        ),
                        &var.source,
                    );
                } else {
                    // candidates.len() == 1
                    let (tc, ns) = candidates[0].clone();
                    *self = tc;
                    ei.set_var_namespace(ns)
                }
            }
            Expr::Lit(lit) => {
                if !self.unify(&lit.ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &lit.ty.to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                ei.clone()
            }
            Expr::App(fun, args) => {
                assert_eq!(args.len(), 1); // lambda of multiple arguments generated in optimization.
                let arg = args[0].clone();
                let arg_ty = type_tyvar_star(&self.new_tyvar());
                if ei.app_order == AppSourceCodeOrderType::ArgumentIsFormer {
                    let arg = self.unify_type_of_expr(&arg, arg_ty.clone());
                    let fun = self.unify_type_of_expr(fun, type_fun(arg_ty.clone(), ty));
                    ei.set_app_args(vec![arg]).set_app_func(fun)
                } else {
                    let fun = self.unify_type_of_expr(fun, type_fun(arg_ty.clone(), ty));
                    let arg = self.unify_type_of_expr(&arg, arg_ty.clone());
                    ei.set_app_args(vec![arg]).set_app_func(fun)
                }
            }
            Expr::Lam(args, body) => {
                assert_eq!(args.len(), 1); // lambda of multiple arguments generated in optimization.
                let arg = args[0].clone();
                let arg_ty = type_tyvar_star(&self.new_tyvar());
                let body_ty = type_tyvar_star(&self.new_tyvar());
                let fun_ty = type_fun(arg_ty.clone(), body_ty.clone());
                if !self.unify(&fun_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&fun_ty).to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                assert!(arg.name.is_local());
                self.scope.push(&arg.name.name, &Scheme::from_type(arg_ty));
                let body = self.unify_type_of_expr(body, body_ty);
                self.scope.pop(&arg.name.name);
                ei.set_lam_body(body)
            }
            Expr::Let(pat, val, body) => {
                let (pat_ty, var_ty) = pat.get_type(self);
                let val = self.unify_type_of_expr(val, pat_ty.clone());
                let var_scm = var_ty.iter().map(|(name, ty)| {
                    (
                        name.clone(),
                        Scheme::generalize(HashMap::default(), vec![], ty.clone()),
                    )
                });
                for (name, scm) in var_scm.clone() {
                    assert!(name.is_local());
                    self.scope.push(&name.name, &scm);
                }
                let body = self.unify_type_of_expr(body, ty);
                for (name, _) in var_scm {
                    self.scope.pop(&name.name);
                }
                ei.set_let_bound(val).set_let_value(body)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.unify_type_of_expr(cond, bool_lit_ty());
                let then_expr = self.unify_type_of_expr(then_expr, ty.clone());
                let else_expr = self.unify_type_of_expr(else_expr, ty);
                ei.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::TyAnno(e, anno_ty) => {
                if !anno_ty.free_vars().is_empty() {
                    error_exit(&format!(
                        "unknown type variable `{}`",
                        ty.free_vars().iter().next().unwrap().0
                    ))
                }
                if !self.unify(&ty, anno_ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&anno_ty).to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                let e = self.unify_type_of_expr(e, ty.clone());
                ei.set_tyanno_expr(e)
            }
            Expr::MakeStruct(tc, fields) => {
                // Get list of field names.
                let ti = self.type_env.tycons.get(tc);
                if ti.is_none() {
                    error_exit(&format!("unknown type constructor `{}`", tc.to_string()));
                }
                let ti = ti.unwrap();
                let field_names = ti.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();

                // Validate fields.
                let field_names_in_struct_defn: HashSet<Name> =
                    HashSet::from_iter(field_names.iter().cloned());
                let field_names_in_expression: HashSet<Name> =
                    HashSet::from_iter(fields.iter().map(|(name, _)| name.clone()));
                for f in &field_names_in_struct_defn {
                    if !field_names_in_expression.contains(f) {
                        error_exit(&format!(
                            "missing field `{}` of struct `{}`",
                            f,
                            tc.to_string()
                        ))
                    }
                }
                for f in &field_names_in_expression {
                    if !field_names_in_struct_defn.contains(f) {
                        error_exit(&format!(
                            "unknown field `{}` for struct `{}`",
                            f,
                            tc.to_string()
                        ))
                    }
                }

                // Get field types.
                let struct_ty = tc.get_struct_union_value_type(self);
                if !self.unify(&struct_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&struct_ty).to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                let field_tys = struct_ty.field_types(&self.type_env);
                assert_eq!(field_tys.len(), fields.len());

                // Reorder fields as ordering of fields in struct definition.
                let fields: HashMap<Name, Arc<ExprNode>> =
                    HashMap::from_iter(fields.iter().cloned());
                let mut fields = field_names
                    .iter()
                    .map(|name| (name.clone(), fields[name].clone()))
                    .collect::<Vec<_>>();

                for (field_ty, (_, field_expr)) in field_tys.iter().zip(fields.iter_mut()) {
                    *field_expr = self.unify_type_of_expr(field_expr, field_ty.clone());
                }
                ei.set_make_struct_fields(fields)
            }
            Expr::ArrayLit(elems) => {
                // Prepare type of element.
                let elem_ty = type_tyvar_star(&self.new_tyvar());
                let array_ty = type_tyapp(array_lit_ty(), elem_ty.clone());
                if !self.unify(&array_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found an array.",
                            &self.substitute_type(&ty).to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                let mut ei = ei.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.unify_type_of_expr(e, elem_ty.clone());
                    ei = ei.set_array_lit_elem(e, i);
                }
                ei
            }
        }
    }

    // Check if expr has type scm.
    // Returns given AST augmented with inferred information.
    pub fn check_type(&mut self, expr: Arc<ExprNode>, expect_scm: Arc<Scheme>) -> Arc<ExprNode> {
        assert!(self.predicates.is_empty()); // This function is available only when predicates are empty.
        let (given_preds, specified_ty) = self.instantiate_scheme(&expect_scm, false);
        let expr = self.unify_type_of_expr(&expr, specified_ty.clone());
        let deduced_ty = self.substitute_type(&specified_ty);
        self.reduce_predicates();
        let required_preds = std::mem::replace(&mut self.predicates, Default::default());

        let s = Substitution::matching(&self.type_env, &deduced_ty, &specified_ty);
        if s.is_none() {
            error_exit(&format!(
                "Type mismatch. Expected `{}`, found `{}`",
                specified_ty.to_string_normalize(),
                deduced_ty.to_string_normalize()
            ));
        }
        let s = s.unwrap();
        for p in required_preds {
            let mut p = p.clone();
            s.substitute_predicate(&mut p);
            if !self.trait_env.entail(&given_preds, &p, &self.type_env) {
                error_exit(&format!(
                    "Condition `{}` is necessary for this expression but not assumed in the specified type.",
                    p.to_string()
                ));
            }
        }

        expr
    }
}
