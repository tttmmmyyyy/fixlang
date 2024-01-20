use serde::{Deserialize, Serialize};

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
                "Duplicate definition for `{}.{}`",
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
    fn overloaded_candidates(
        &self,
        name: &FullName,
        imported_modules: &HashSet<Name>,
    ) -> Vec<(NameSpace, T)> {
        if !self.var.contains_key(&name.name) {
            return vec![];
        }
        let sv = &self.var[&name.name];
        if name.is_local() && sv.local.len() > 0 {
            vec![(NameSpace::local(), sv.local.last().unwrap().clone())]
        } else {
            sv.global
                .iter()
                .filter(|(ns, _)| imported_modules.contains(&ns.module()))
                .filter(|(ns, _)| name.namespace.is_suffix(ns))
                .map(|(ns, v)| (ns.clone(), v.clone()))
                .collect()
        }
    }
}

// Type substitution. Name of type variable -> type.
// Managed so that the value (a type) of this HashMap doesn't contain a type variable that appears in keys. i.e.,
// when we want to COMPLETELY substitute type variables in a type by `substitution`, we only apply this mapy only ONCE.
#[derive(Clone, Serialize, Deserialize)]
pub struct Substitution {
    pub data: HashMap<Name, Rc<TypeNode>>,
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
    pub fn single(var: &str, ty: Rc<TypeNode>) -> Self {
        let mut data = HashMap::<String, Rc<TypeNode>>::default();
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
    pub fn substitute_type(&self, ty: &Rc<TypeNode>) -> Rc<TypeNode> {
        match &ty.ty {
            Type::TyVar(tyvar) => self.data.get(&tyvar.name).map_or(ty.clone(), |sub| {
                sub.set_source_if_none(ty.get_source().clone())
            }),
            Type::TyCon(_) => ty.clone(),
            Type::TyApp(fun, arg) => ty
                .set_tyapp_fun(self.substitute_type(fun))
                .set_tyapp_arg(self.substitute_type(arg)),
            Type::FunTy(src, dst) => ty
                .set_funty_src(self.substitute_type(&src))
                .set_funty_dst(self.substitute_type(&dst)),
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
    pub fn unify(
        kind_map: &HashMap<TyCon, Rc<Kind>>,
        ty1: &Rc<TypeNode>,
        ty2: &Rc<TypeNode>,
    ) -> Option<Self> {
        match &ty1.ty {
            Type::TyVar(var1) => {
                return Self::unify_tyvar(kind_map, &var1, ty2);
            }
            _ => {}
        }
        match &ty2.ty {
            Type::TyVar(var2) => {
                return Self::unify_tyvar(kind_map, &var2, ty1);
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
                    match Self::unify(kind_map, &fun1, &fun2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    let arg1 = ret.substitute_type(arg1);
                    let arg2 = ret.substitute_type(arg2);
                    match Self::unify(kind_map, &arg1, &arg2) {
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
                    match Self::unify(kind_map, &arg_ty1, &arg_ty2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    let ret_ty1 = ret.substitute_type(ret_ty1);
                    let ret_ty2 = ret.substitute_type(ret_ty2);
                    match Self::unify(kind_map, &ret_ty1, &ret_ty2) {
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
    fn unify_tyvar(
        kind_map: &HashMap<TyCon, Rc<Kind>>,
        tyvar1: &Rc<TyVar>,
        ty2: &Rc<TypeNode>,
    ) -> Option<Self> {
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
            // `map: [f: Functor] (a -> b) -> f a -> f b; map = |f, c| (...)`;
            return None;
        }
        if tyvar1.kind != ty2.kind(kind_map) {
            return None;
        }
        Some(Self::single(&tyvar1.name, ty2.clone()))
    }

    // Calculate minimum substitution s such that `s(ty1) = ty2`.
    pub fn matching(
        kind_map: &HashMap<TyCon, Rc<Kind>>,
        ty1: &Rc<TypeNode>,
        ty2: &Rc<TypeNode>,
    ) -> Option<Self> {
        match &ty1.ty {
            Type::TyVar(v1) => Self::unify_tyvar(kind_map, v1, ty2),
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
                    match Self::matching(kind_map, fun1, fun2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    match Self::matching(kind_map, arg1, arg2) {
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
                    match Self::matching(kind_map, src1, src2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    match Self::matching(kind_map, dst1, dst2) {
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
    pub scope: Scope<Rc<Scheme>>,
    // Type resolver.
    pub resolver: TypeResolver,
    // Collected predicates.
    pub predicates: Vec<Predicate>,
    // Trait environment.
    trait_env: TraitEnv,
    // List of type constructors.
    pub type_env: TypeEnv,
    // A map to represent modules imported by each submodule.
    // To decrease clone-cost, use Rc.
    pub imported_mod_map: Rc<HashMap<Name, HashSet<Name>>>,
    // In which module is the current expression defined?
    // This is used as a state variable for typechecking.
    pub current_module: Option<Name>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TypeResolver {
    // Substitution.
    pub substitution: Substitution,
    // Type to kind mapping.
    #[serde(skip)]
    pub kind_map: HashMap<TyCon, Rc<Kind>>,
}

impl TypeResolver {
    // Set type environment.
    pub fn set_type_env(&mut self, type_env: TypeEnv) {
        self.kind_map = type_env.kinds();
    }

    // Update substitution to unify two types.
    // When substitution fails, it has no side effect to self.
    pub fn unify(&mut self, ty1: &Rc<TypeNode>, ty2: &Rc<TypeNode>) -> bool {
        let ty1 = &self.substitute_type(ty1);
        let ty2 = &self.substitute_type(ty2);
        match Substitution::unify(&self.kind_map, ty1, ty2) {
            Some(sub) => {
                self.substitution.add_substitution(&sub);
                return true;
            }
            None => {
                return false;
            }
        }
    }

    // Apply substitution to type.
    pub fn substitute_type(&self, ty: &Rc<TypeNode>) -> Rc<TypeNode> {
        self.substitution.substitute_type(ty)
    }

    // Apply substitution to a predicate.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        self.substitution.substitute_predicate(p)
    }
}

impl TypeCheckContext {
    // Creaate instance.
    pub fn new(
        trait_env: TraitEnv,
        type_env: TypeEnv,
        imported_mod_map: HashMap<Name, HashSet<Name>>,
    ) -> Self {
        let mut resolver = TypeResolver::default();
        resolver.set_type_env(type_env.clone());
        Self {
            tyvar_id: Default::default(),
            scope: Default::default(),
            resolver,
            predicates: Default::default(),
            type_env,
            trait_env,
            imported_mod_map: Rc::new(imported_mod_map),
            current_module: None,
        }
    }

    // Get modules imported by current module.
    pub fn imported_modules(&self) -> &HashSet<Name> {
        self.imported_mod_map
            .get(self.current_module.as_ref().unwrap())
            .unwrap()
    }

    // Generate new type variable.
    pub fn new_tyvar(&mut self) -> String {
        let id = self.tyvar_id;
        self.tyvar_id += 1;
        "%a".to_string() + &id.to_string() // To avlid confliction with user-defined type variable, we add prefix #.
    }

    // Apply substitution to type.
    pub fn substitute_type(&self, ty: &Rc<TypeNode>) -> Rc<TypeNode> {
        self.resolver.substitute_type(ty)
    }

    // Apply substitution to a predicate.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        self.resolver.substitute_predicate(p)
    }

    // Instantiate a scheme.
    // Returns predicates if append_predicates = false or append them to self if append_predicates = true.
    pub fn instantiate_scheme(
        &mut self,
        scheme: &Rc<Scheme>,
        append_predicates: bool,
    ) -> (Vec<Predicate>, Rc<TypeNode>) {
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

    // Update substitution to unify two types.
    // When substitution fails, it has no side effect to self.
    pub fn unify(&mut self, ty1: &Rc<TypeNode>, ty2: &Rc<TypeNode>) -> bool {
        self.resolver.unify(ty1, ty2)
    }

    // Reduce predicates.
    // Returns Err(p) if predicates are unsatisfiable due to predicate p.
    pub fn reduce_predicates(&mut self) -> Result<(), Predicate> {
        let mut preds = std::mem::replace(&mut self.predicates, vec![]);
        for p in &mut preds {
            self.substitute_predicate(p);
        }
        self.predicates.append(&mut preds);
        self.predicates = self
            .trait_env
            .reduce(&self.predicates, &self.type_env.kinds())?;
        Ok(())
    }

    // Perform typechecking.
    // Update type substitution so that `ei` has type `ty`.
    // Returns given AST augmented with inferred information.
    pub fn unify_type_of_expr(&mut self, ei: &Rc<ExprNode>, ty: Rc<TypeNode>) -> Rc<ExprNode> {
        let ei = ei.set_inferred_type(ty.clone());
        match &*ei.expr {
            Expr::Var(var) => {
                let candidates = self
                    .scope
                    .overloaded_candidates(&var.name, self.imported_modules());
                if candidates.is_empty() {
                    error_exit_with_src(
                        &format!("No value `{}` is found.", var.name.to_string()),
                        &ei.source,
                    );
                }
                let candidates: Vec<_> = candidates
                    .iter()
                    .map(|(ns, scm)| {
                        let fullname = FullName::new(ns, &var.name.name);
                        let mut tc = self.clone();
                        let (_, var_ty) = tc.instantiate_scheme(&scm, true);
                        // if var_ty is unifiable to the expected type and predicates are satisfiable, then this candidate is ok.
                        if !tc.unify(&var_ty, &ty) {
                            let msg = format!(
                                "- `{}` of type `{}` does not match the expected type.",
                                fullname.to_string(),
                                scm.substitute(&self.resolver.substitution).to_string(),
                            );
                            Err(msg)
                        } else {
                            let reduce_result = tc.reduce_predicates();
                            if reduce_result.is_ok() {
                                Ok((tc, ns.clone()))
                            } else {
                                let mut fail_predicate = reduce_result.err().unwrap();
                                self.substitute_predicate(&mut fail_predicate);
                                let msg = format!(
                                    "- `{}` of type `{}` does not match since the constraint `{}` is not satisifed.",
                                    fullname.to_string(),
                                    scm.substitute(&self.resolver.substitution).to_string(),
                                    fail_predicate.to_string_normalize()
                                );
                                Err(msg)
                            }
                        }
                    })
                    .collect();
                let ok_count = candidates.iter().filter(|cand| cand.is_ok()).count();
                if ok_count == 0 {
                    error_exit_with_src(
                        &format!(
                            "No value named `{}` matches the expected type `{}`.\n{}",
                            var.name.to_string(),
                            &self.substitute_type(&ty).to_string_normalize(),
                            candidates
                                .iter()
                                .map(|cand| cand.as_ref().err().unwrap().clone())
                                .collect::<Vec<_>>()
                                .join("\n")
                        ),
                        &ei.source,
                    );
                } else if ok_count >= 2 {
                    let candidates_str = candidates
                        .iter()
                        .filter_map(|cand| cand.as_ref().ok())
                        .map(|(_, ns)| {
                            let fullname = FullName::new(&ns, &var.name.name);
                            "`".to_string() + &fullname.to_string() + "`"
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    error_exit_with_src(
                        &format!(
                            "Name `{}` is ambiguous: there are {}. Maybe you need to write (suffix of) its namespace to help overloading resolution.",
                            var.name.to_string(),
                            candidates_str
                        ),
                        &ei.source,
                    );
                } else {
                    // candidates.len() == 1
                    let (tc, ns) = candidates
                        .iter()
                        .find_map(|cand| cand.as_ref().ok())
                        .unwrap();
                    *self = tc.clone();
                    ei.set_var_namespace(ns.clone())
                }
            }
            Expr::LLVM(lit) => {
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
                pat.error_if_invalid(&self.type_env);
                let (pat_ty, var_ty) = pat.pattern.get_type(self);
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
                let cond = self.unify_type_of_expr(cond, make_bool_ty());
                let then_expr = self.unify_type_of_expr(then_expr, ty.clone());
                let else_expr = self.unify_type_of_expr(else_expr, ty);
                ei.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::TyAnno(e, anno_ty) => {
                if !anno_ty.free_vars().is_empty() {
                    error_exit_with_src(
                        &format!("Currently, cannot use type variable in type annotation.",),
                        anno_ty.get_source(),
                    )
                }
                if !self.unify(&ty, anno_ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
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
                    error_exit_with_src(
                        &format!("Unknown type name `{}`.", tc.to_string()),
                        &ei.source,
                    );
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
                        error_exit_with_src(
                            &format!("Missing field `{}` of struct `{}`.", f, tc.to_string()),
                            &ei.source,
                        )
                    }
                }
                for f in &field_names_in_expression {
                    if !field_names_in_struct_defn.contains(f) {
                        error_exit_with_src(
                            &format!("Unknown field `{}` for struct `{}`.", f, tc.to_string()),
                            &ei.source,
                        )
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
                let fields: HashMap<Name, Rc<ExprNode>> =
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
                let array_ty = type_tyapp(make_array_ty(), elem_ty.clone());
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
            Expr::CallC(_, ret_ty, param_tys, is_va_args, args) => {
                let ret_ty = type_tycon(ret_ty);
                if !self.unify(&ty, &ret_ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            ty.to_string(),
                            ret_ty.to_string()
                        ),
                        &ei.source,
                    );
                }
                let param_tys = param_tys
                    .iter()
                    .map(|tc| type_tycon(tc))
                    .collect::<Vec<_>>();
                let mut ei = ei.clone();
                for (i, e) in args.iter().enumerate() {
                    let expect_ty = if i < param_tys.len() {
                        param_tys[i].clone()
                    } else {
                        assert!(is_va_args);
                        type_tyvar_star(&self.new_tyvar())
                    };
                    let e = self.unify_type_of_expr(e, expect_ty);
                    ei = ei.set_call_c_arg(e, i);
                }
                ei
            }
        }
    }

    // Check if expr has type scm.
    // Returns given AST augmented with inferred information.
    pub fn check_type(&mut self, expr: Rc<ExprNode>, expect_scm: Rc<Scheme>) -> Rc<ExprNode> {
        assert!(self.predicates.is_empty()); // This function is available only when predicates are empty.
        let (given_preds, specified_ty) = self.instantiate_scheme(&expect_scm, false);
        let expr = self.unify_type_of_expr(&expr, specified_ty.clone());
        let deduced_ty = self.substitute_type(&specified_ty);
        let red_res = self.reduce_predicates();
        assert!(red_res.is_ok());
        let required_preds = std::mem::replace(&mut self.predicates, Default::default());

        let s = Substitution::matching(&self.type_env.kinds(), &deduced_ty, &specified_ty);
        if s.is_none() {
            error_exit_with_src(
                &format!(
                    "Type mismatch. Expected `{}`, found `{}`.",
                    specified_ty.to_string_normalize(),
                    deduced_ty.to_string_normalize()
                ),
                &expr.source,
            );
        }
        let s = s.unwrap();
        for p in required_preds {
            let mut p = p.clone();
            s.substitute_predicate(&mut p);
            if !self
                .trait_env
                .entail(&given_preds, &p, &self.type_env.kinds())
            {
                error_exit_with_src(
                    &format!(
                        "Constraint `{}` is required for this expression but is not assumed in its type.",
                        p.to_string_normalize()
                    ),
                    &expr.source,
                );
            }
        }

        expr
    }
}
