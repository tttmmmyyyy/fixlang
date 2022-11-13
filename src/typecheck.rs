use core::panic;

use super::*;

// #[derive(Debug)]
// pub struct TypeError {}

fn error_exit_with_src(msg: &str, src: &Option<Span>) -> ! {
    let mut str = String::default();
    str += "error: ";
    str += msg;
    str += "\n";
    match src {
        None => todo!(),
        Some(v) => {
            str += &v.to_string();
        }
    };
    error_exit(&str)
}

#[derive(Clone)]
struct LocalTypeVar {
    ty: Arc<TypeNode>,
    /* field for type class */
}
struct Scope<T> {
    var: HashMap<String, Vec<T>>,
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
    fn push(self: &mut Self, name: &str, ty: &T) {
        if !self.var.contains_key(name) {
            self.var.insert(String::from(name), Default::default());
        }
        self.var.get_mut(name).unwrap().push(ty.clone());
    }
    fn pop(self: &mut Self, name: &str) {
        self.var.get_mut(name).unwrap().pop();
        if self.var.get(name).unwrap().is_empty() {
            self.var.remove(name);
        }
    }
    fn get(self: &Self, name: &str) -> Option<&T> {
        self.var.get(name).map(|v| v.last().unwrap())
    }
    fn get_mut(self: &mut Self, name: &str) -> Option<&mut T> {
        self.var.get_mut(name).map(|v| v.last_mut().unwrap())
    }
}

pub fn check_type(ei: Arc<ExprNode>, ty: Arc<TypeNode>) {
    let mut ctx = TypeCheckContext::default();
    ctx.deduce_expr(&ei, ty);
}

// Type substitution. Name of type variable -> type.
// Managed so that the value (a type) of this HashMap doesn't contain a type variable that appears in keys. i.e.,
// when we want to COMPLETELY substitute type variables in a type by `substitution`, we only apply this mapy only ONCE.
struct Substitution {
    data: HashMap<String, Arc<TypeNode>>,
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
    fn single(var: &str, ty: Arc<TypeNode>) -> Self {
        let mut data = HashMap::<String, Arc<TypeNode>>::default();
        data.insert(var.to_string(), ty);
        Self { data }
    }

    // Add substitution.
    fn add_substitution(&mut self, other: &Self) {
        for (_var, ty) in self.data.iter_mut() {
            let new_ty = other.substitute_type(&ty);
            *ty = new_ty;
        }
        for (var, ty) in &other.data {
            self.data.insert(var.to_string(), ty.clone());
        }
    }

    // Apply substitution.
    fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        match &ty.ty {
            Type::TyVar(tyvar) => self
                .data
                .get(&tyvar.name)
                .map_or(ty.clone(), |sub| sub.clone()),
            Type::TyCon(tc) => ty.clone(),
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

    // Calculate minimum substitution to unify two types.
    fn unify(
        tycons: &HashMap<String, Arc<Kind>>,
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
    ) -> Option<Self> {
        match &ty1.ty {
            Type::TyVar(var1) => {
                return Self::unify_tyvar(tycons, &var1, ty2);
            }
            _ => {}
        }
        match &ty2.ty {
            Type::TyVar(var2) => {
                return Self::unify_tyvar(tycons, &var2, ty1);
            }
            _ => {}
        }
        match &ty1.ty {
            Type::TyVar(_) => unreachable!(),
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1.name == tc2.name {
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
                    match Self::unify(tycons, &fun1, &fun2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    let arg1 = ret.substitute_type(arg1);
                    let arg2 = ret.substitute_type(arg2);
                    match Self::unify(tycons, &arg1, &arg2) {
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
                    match Self::unify(tycons, &arg_ty1, &arg_ty2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    let ret_ty1 = ret.substitute_type(ret_ty1);
                    let ret_ty2 = ret.substitute_type(ret_ty2);
                    match Self::unify(tycons, &ret_ty1, &ret_ty2) {
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
        tycons: &HashMap<String, Arc<Kind>>,
        tyvar1: &Arc<TyVar>,
        ty2: &Arc<TypeNode>,
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
            panic!("unify_tyvar is making circular substitution.")
        }
        if tyvar1.kind != ty2.kind(tycons) {
            error_exit_with_src("Kinds do not match.", &None);
        }
        Some(Self::single(&tyvar1.name, ty2.clone()))
    }
}

// Context under type-checking.
// Reference: https://uhideyuki.sakura.ne.jp/studs/index.cgi/ja/HindleyMilnerInHaskell#fn6
pub struct TypeCheckContext {
    // The identifier of type variables.
    tyvar_id: u32,
    // Scoped map of variable name -> scheme. (Assamptions of type inference.)
    scope: Scope<Arc<Scheme>>,
    // Substitution.
    substitution: Substitution,
    // Set of TyCons associated with kinds
    tycons: HashMap<String, Arc<Kind>>,
}

impl Default for TypeCheckContext {
    fn default() -> Self {
        Self {
            tyvar_id: Default::default(),
            scope: Default::default(),
            substitution: Default::default(),
            tycons: bulitin_type_to_kind_map(),
        }
    }
}

impl TypeCheckContext {
    // Generate new type variable.
    fn new_tyvar(&mut self) -> String {
        let id = self.tyvar_id;
        self.tyvar_id += 1;
        "a".to_string() + &id.to_string()
    }

    // Apply substitution to type.
    fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        self.substitution.substitute_type(ty)
    }

    // Apply substitution to scheme.
    fn substitute_scheme(&self, scm: &Arc<Scheme>) -> Arc<Scheme> {
        Scheme::new_arc(scm.vars.clone(), self.substitute_type(&scm.ty))
    }

    // Instantiate a scheme.
    fn instantiate_scheme(&mut self, scheme: &Arc<Scheme>) -> Arc<TypeNode> {
        let mut sub = Substitution::default();
        for (var, kind) in &scheme.vars {
            let new_var_name = self.new_tyvar();
            sub.add_substitution(&Substitution::single(&var, type_tyvar(&new_var_name, kind)));
        }
        sub.substitute_type(&scheme.ty)
    }

    // Make a scheme from a type by abstracting type variable that does not appear in scope.
    fn abstract_to_scheme(&self, ty: &Arc<TypeNode>) -> Arc<Scheme> {
        let ty = self.substitute_type(ty);
        let mut vars = ty.free_vars();
        for (_var, scms) in &self.scope.var {
            for scm in scms {
                for (var_in_scope, _) in self.substitute_scheme(&scm).free_vars() {
                    vars.remove(&var_in_scope);
                }
            }
        }
        Scheme::new_arc(vars, ty)
    }

    // Update substitution to unify two types.
    fn unify(&mut self, ty1: &Arc<TypeNode>, ty2: &Arc<TypeNode>) -> bool {
        let ty1 = &self.substitute_type(ty1);
        let ty2 = &self.substitute_type(ty2);
        match Substitution::unify(&self.tycons, ty1, ty2) {
            Some(sub) => {
                self.substitution.add_substitution(&sub);
                return true;
            }
            None => {
                return false;
            }
        }
    }

    // Update type substitution so that `ei` has type `ty`.
    fn deduce_expr(&mut self, ei: &Arc<ExprNode>, ty: Arc<TypeNode>) {
        match &*ei.expr {
            Expr::Var(var) => {
                let scm = self
                    .scope
                    .get(&var.name)
                    .unwrap_or_else(|| {
                        error_exit_with_src(
                            &format!("unknown variable `{}`", var.name),
                            &var.source,
                        );
                    })
                    .clone();
                let var_ty = self.instantiate_scheme(&scm);
                let var_ty = self.substitute_type(&var_ty);
                if !self.unify(&var_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "type mismatch. Expected `{}`, Found `{}`",
                            &self.substitute_type(&ty).to_string(),
                            &self.substitute_type(&var_ty).to_string(),
                        ),
                        &ei.source,
                    );
                }
            }
            Expr::Lit(lit) => {
                if !self.unify(&lit.ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "type mismatch. Expected `{}`, Found `{}`",
                            &self.substitute_type(&ty).to_string(),
                            &lit.ty.to_string(),
                        ),
                        &ei.source,
                    );
                }
            }
            Expr::App(fun, arg) => {
                let arg_ty = type_tyvar_star(&self.new_tyvar());
                self.deduce_expr(arg, arg_ty.clone());
                // NOTE: to help name-resolution of fields, we should deduce_expr of arg before that of fun.
                self.deduce_expr(fun, type_fun(arg_ty, ty));
            }
            Expr::Lam(arg, body) => {
                let arg_ty = type_tyvar_star(&self.new_tyvar());
                let body_ty = type_tyvar_star(&self.new_tyvar());
                let fun_ty = type_fun(arg_ty.clone(), body_ty.clone());
                if !self.unify(&fun_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "type mismatch. Expected `{}`, Found `{}`",
                            &self.substitute_type(&ty).to_string(),
                            &self.substitute_type(&fun_ty).to_string(),
                        ),
                        &ei.source,
                    );
                }
                self.scope
                    .push(&arg.name, &Scheme::new_arc(Default::default(), arg_ty));
                self.deduce_expr(body, body_ty);
                self.scope.pop(&arg.name);
            }
            Expr::Let(var, val, body) => {
                let var_ty = match &var.type_annotation {
                    Some(scm) => {
                        let free_vars = scm.free_vars();
                        if !free_vars.is_empty() {
                            error_exit_with_src(
                                &format!(
                                    "unknown type variable `{}`",
                                    free_vars.iter().next().unwrap().0
                                ),
                                &var.source,
                            )
                        }
                        self.instantiate_scheme(&scm)
                    }
                    None => type_tyvar_star(&self.new_tyvar()),
                };
                self.deduce_expr(val, var_ty.clone());
                let var_scm = self.abstract_to_scheme(&var_ty);
                self.scope.push(&var.name, &var_scm);
                self.deduce_expr(body, ty);
                self.scope.pop(&var.name);
            }
            Expr::If(cond, then_expr, else_expr) => {
                self.deduce_expr(cond, bool_lit_ty());
                self.deduce_expr(then_expr, ty.clone());
                self.deduce_expr(else_expr, ty);
            }
        }
    }

    // Read type declarations to verity it and extend type-to-kind mapping.
    pub fn add_tycons(&mut self, type_decls: &Vec<TypeDecl>) {
        for type_decl in type_decls {
            self.add_tycon(type_decl);
        }
    }

    // Read type declaration to verity it and extend type-to-kind mapping.
    fn add_tycon(&mut self, decl: &TypeDecl) {
        if self.tycons.contains_key(&decl.name) {
            error_exit_with_src(&format!("Type `{}` is already defined.", decl.name), &None);
        }
        self.tycons.insert(decl.name.clone(), kind_star());
    }
}
