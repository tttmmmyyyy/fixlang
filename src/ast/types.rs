use core::panic;

use super::*;

#[derive(Eq, PartialEq, Clone)]
pub struct TyVar {
    pub name: String,
    pub kind: Arc<Kind>,
}

impl TyVar {
    pub fn set_kind(&self, kind: Arc<Kind>) -> Arc<TyVar> {
        let mut ret = self.clone();
        ret.kind = kind;
        Arc::new(ret)
    }
}

#[derive(Eq, PartialEq)]
pub enum Kind {
    Star,
    Arrow(Arc<Kind>, Arc<Kind>),
}

impl Kind {
    pub fn to_string(&self) -> String {
        match self {
            Kind::Star => "*".to_string(),
            Kind::Arrow(src, dst) => {
                let src_braced = match **src {
                    Kind::Star => false,
                    Kind::Arrow(_, _) => true,
                };
                if src_braced {
                    format!("({})->{}", src.to_string(), dst.to_string())
                } else {
                    format!("{}->{}", src.to_string(), dst.to_string())
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct TyCon {
    pub name: NameSpacedName,
}

impl TyCon {
    pub fn new(nsn: NameSpacedName) -> TyCon {
        TyCon { name: nsn }
    }

    pub fn to_string(&self) -> String {
        self.name.to_string()
    }
}

// Node of type ast tree with user defined additional information
pub struct TypeNode {
    pub ty: Type,
    pub info: Arc<TypeInfo>,
}

impl PartialEq for TypeNode {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}

impl Eq for TypeNode {}

impl TypeNode {
    // Set kinds to type variables.
    pub fn set_kinds(self: &Arc<TypeNode>, kinds: &HashMap<Name, Arc<Kind>>) -> Arc<TypeNode> {
        match &self.ty {
            Type::TyVar(tv) => {
                if kinds.contains_key(&tv.name) {
                    self.set_tyvar_kind(kinds[&tv.name].clone())
                } else {
                    self.clone()
                }
            }
            Type::TyCon(_tc) => self.clone(),
            Type::TyApp(fun, arg) => self
                .set_tyapp_fun(fun.set_kinds(kinds))
                .set_tyapp_arg(arg.set_kinds(kinds)),
            Type::FunTy(src, dst) => self
                .set_funty_src(src.set_kinds(kinds))
                .set_funty_dst(dst.set_kinds(kinds)),
        }
    }

    // Set kinds to type variables.
    pub fn set_kinds_vec(self: &Arc<TypeNode>, kinds: &Vec<KindPredicate>) -> Arc<TypeNode> {
        let mut kinds_map: HashMap<Name, Arc<Kind>> = Default::default();
        for kp in kinds {
            kinds_map.insert(kp.name.clone(), kp.kind.clone());
        }
        self.set_kinds(&kinds_map)
    }

    // Is this type head normal form? i.e., begins with type variable.
    pub fn is_hnf(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => true,
            Type::TyCon(_) => false,
            Type::TyApp(head, _) => head.is_hnf(),
            Type::FunTy(head, _) => head.is_hnf(),
        }
    }

    pub fn set_tyvar_kind(&self, kind: Arc<Kind>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyVar(tv) => {
                ret.ty = Type::TyVar(tv.set_kind(kind));
            }
            _ => panic!(),
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_tyapp_fun(&self, fun: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyApp(_, arg) => ret.ty = Type::TyApp(fun, arg.clone()),
            _ => panic!(),
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_tyapp_arg(&self, arg: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyApp(fun, _) => ret.ty = Type::TyApp(fun.clone(), arg),
            _ => panic!(),
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_funty_src(&self, src: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::FunTy(_, dst) => ret.ty = Type::FunTy(src, dst.clone()),
            _ => panic!(),
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_funty_dst(&self, dst: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::FunTy(src, _) => ret.ty = Type::FunTy(src.clone(), dst),
            _ => panic!(),
        }
        Arc::new(ret)
    }
}

impl Clone for TypeNode {
    fn clone(&self) -> Self {
        TypeNode {
            ty: self.ty.clone(),
            info: self.info.clone(),
        }
    }
}

impl TypeNode {
    // Create new type node with default info.
    fn new(ty: Type) -> Self {
        Self {
            ty: ty,
            info: Arc::new(TypeInfo::default()),
        }
    }

    // Create shared new type node with default info.
    fn new_arc(ty: Type) -> Arc<Self> {
        Arc::new(Self::new(ty))
    }

    // Set new info for shared instance.
    #[allow(dead_code)]
    pub fn set_info(self: Arc<Self>, info: Arc<TypeInfo>) -> Arc<Self> {
        let mut ret = (*self).clone();
        ret.info = info;
        Arc::new(ret)
    }

    // Set new type for shared instance.
    #[allow(dead_code)]
    pub fn set_ty(self: &Arc<Self>, ty: Type) -> Arc<Self> {
        let mut ret = (**self).clone();
        ret.ty = ty;
        Arc::new(ret)
    }

    // Calculate kind.
    pub fn kind(&self, type_env: &TypeEnv) -> Arc<Kind> {
        match &self.ty {
            Type::TyVar(tv) => tv.kind.clone(),
            Type::TyCon(tc) => type_env.kind(&tc),
            Type::TyApp(fun, arg) => {
                let arg_kind = arg.kind(type_env);
                let fun_kind = fun.kind(type_env);
                match &*fun_kind {
                    Kind::Arrow(arg2, res) => {
                        if arg_kind != *arg2 {
                            error_exit("Kind mismatch.");
                        }
                        res.clone()
                    }
                    Kind::Star => error_exit("Kind mismatch."),
                }
            }
            Type::FunTy(arg, ret) => {
                if arg.kind(type_env) != kind_star() {
                    error_exit("Kind mismatch.")
                }
                if ret.kind(type_env) != kind_star() {
                    error_exit("Kind mismatch.")
                }
                kind_star()
            }
        }
    }
}

// Variant of type
#[derive(PartialEq, Eq)]
pub enum Type {
    TyVar(Arc<TyVar>),
    TyCon(Arc<TyCon>),
    TyApp(Arc<TypeNode>, Arc<TypeNode>),
    FunTy(Arc<TypeNode>, Arc<TypeNode>),
}

impl Clone for Type {
    fn clone(&self) -> Self {
        match self {
            Type::TyVar(x) => Type::TyVar(x.clone()),
            Type::TyApp(x, y) => Type::TyApp(x.clone(), y.clone()),
            Type::FunTy(x, y) => Type::FunTy(x.clone(), y.clone()),
            Type::TyCon(tc) => Type::TyCon(tc.clone()),
        }
    }
}

impl TypeNode {
    // Stringify.
    pub fn to_string(&self) -> String {
        self.to_string_inner(&mut None)
    }

    // Stringify. Name of type variables are normalized to names such as "a0", "a1", etc.
    pub fn to_string_normalize(&self) -> String {
        let mut id: u32 = 0;
        self.to_string_inner(&mut Some(&mut id))
    }

    // Stringify.
    // If "tyvar_id" is specified, then names of type variables are normalized to names such as "t0", "t1", etc.
    fn to_string_inner(&self, tyvar_id: &mut Option<&mut u32>) -> String {
        match &self.ty {
            Type::TyVar(v) => match tyvar_id {
                Some(id) => {
                    let ret = format!("t{}", *id);
                    **id += 1;
                    ret
                }
                None => v.name.clone(),
            },
            Type::TyApp(tyfun, arg) => {
                let arg_brace_needed = match arg.ty {
                    Type::TyVar(_) => false,
                    Type::TyCon(_) => false,
                    Type::TyApp(_, _) => true,
                    Type::FunTy(_, _) => true,
                };
                let tyfun = tyfun.to_string_inner(tyvar_id);
                let arg = arg.to_string_inner(tyvar_id);
                if arg_brace_needed {
                    format!("{} ({})", tyfun, arg)
                } else {
                    format!("{} {}", tyfun, arg)
                }
            }
            Type::FunTy(src, dst) => {
                let src_brace_needed = match src.ty {
                    Type::FunTy(_, _) => true,
                    _ => false,
                };
                let src = src.clone().to_string_inner(tyvar_id);
                let dst = dst.clone().to_string_inner(tyvar_id);
                let mut res: String = Default::default();
                if src_brace_needed {
                    res += "(";
                    res += &src;
                    res += ")";
                } else {
                    res += &src;
                }
                res += " -> ";
                res += &dst;
                res
            }
            Type::TyCon(tc) => tc.to_string(),
        }
    }
}

pub fn kind_star() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

pub fn kind_arrow(src: Arc<Kind>, dst: Arc<Kind>) -> Arc<Kind> {
    Arc::new(Kind::Arrow(src, dst))
}

pub fn tyvar_from_name(var_name: &str, kind: &Arc<Kind>) -> Arc<TyVar> {
    Arc::new(TyVar {
        name: String::from(var_name),
        kind: kind.clone(),
    })
}

pub fn type_tyvar(var_name: &str, kind: &Arc<Kind>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar_from_name(var_name, kind)))
}

pub fn type_tyvar_star(var_name: &str) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar_from_name(var_name, &kind_star())))
}

pub fn type_var_from_tyvar(tyvar: Arc<TyVar>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar))
}

pub fn type_fun(src: Arc<TypeNode>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::FunTy(src, dst))
}

pub fn type_tyapp(tyfun: Arc<TypeNode>, param: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyApp(tyfun, param))
}

pub fn type_tycon(tycon: &Arc<TyCon>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyCon(tycon.clone()))
}

pub fn tycon(name: NameSpacedName) -> Arc<TyCon> {
    Arc::new(TyCon { name })
}

// Additional information of types.
#[derive(Default, Clone)]
pub struct TypeInfo {}

impl TypeNode {
    // Calculate free type variables.
    pub fn free_vars(self: &Arc<Self>) -> HashMap<Name, Arc<Kind>> {
        let mut free_vars: HashMap<String, Arc<Kind>> = HashMap::default();
        match &self.ty {
            Type::TyVar(tv) => {
                free_vars.insert(tv.name.clone(), tv.kind.clone());
            }
            Type::TyApp(tyfun, arg) => {
                free_vars.extend(tyfun.free_vars());
                free_vars.extend(arg.free_vars());
            }
            Type::FunTy(input, output) => {
                free_vars.extend(input.free_vars());
                free_vars.extend(output.free_vars());
            }
            Type::TyCon(_) => {}
        };
        free_vars
    }

    // Get set of free type variables.
    pub fn free_vars_set(self: &Arc<Self>) -> HashSet<Name> {
        self.free_vars().iter().map(|(k, _)| k.clone()).collect()
    }
}

// Type scheme.
#[derive(Clone)]
pub struct Scheme {
    pub vars: HashMap<Name, Arc<Kind>>,
    pub preds: Vec<Predicate>,
    pub ty: Arc<TypeNode>,
}

impl Scheme {
    #[allow(dead_code)]
    pub fn set_ty(&self, ty: Arc<TypeNode>) -> Arc<Scheme> {
        let mut ret = self.clone();
        ret.ty = ty;
        Arc::new(ret)
    }

    pub fn set_kinds(&self, trait_kind_map: &HashMap<TraitId, Arc<Kind>>) -> Arc<Scheme> {
        let mut ret = self.clone();
        let mut scope: HashMap<Name, Arc<Kind>> = Default::default();
        QualPredicate::extend_kind_scope(&mut scope, &ret.preds, &vec![], trait_kind_map);
        for p in &mut ret.preds {
            p.set_kinds(&scope);
        }
        ret.ty = ret.ty.set_kinds(&scope);
        for (v, k) in &mut ret.vars {
            if scope.contains_key(v) {
                *k = scope[v].clone();
            }
        }
        Arc::new(ret)
    }

    pub fn check_kinds(&self, type_env: &TypeEnv, trait_kind_map: &HashMap<TraitId, Arc<Kind>>) {
        for p in &self.preds {
            p.check_kinds(type_env, trait_kind_map);
        }
        self.ty.kind(type_env);
    }

    // Create new instance.
    fn new_arc(
        vars: HashMap<String, Arc<Kind>>,
        preds: Vec<Predicate>,
        ty: Arc<TypeNode>,
    ) -> Arc<Scheme> {
        Arc::new(Scheme { vars, preds, ty })
    }

    // Create new instance.
    // fn new_arc_from_str(
    //     vars: &[(&str, Arc<Kind>)],
    //     preds: Vec<Predicate>,
    //     ty: Arc<TypeNode>,
    // ) -> Arc<Scheme> {
    //     Self::new_arc(
    //         HashMap::from_iter(
    //             vars.iter()
    //                 .map(|(name, kind)| (name.to_string(), kind.clone())),
    //         ),
    //         preds,
    //         ty,
    //     )
    // }

    pub fn substitute(&self, s: &Substitution) -> Arc<Scheme> {
        // Generalized variables cannot be replaced.
        for (v, _) in &self.vars {
            assert!(!s.data.contains_key(v));
        }
        let mut preds = self.preds.clone();
        for p in &mut preds {
            s.substitute_predicate(p)
        }
        Scheme::new_arc(self.vars.clone(), preds, s.substitute_type(&self.ty))
    }

    // Create instance by generalizaing type.
    pub fn generalize(
        vars: HashMap<String, Arc<Kind>>,
        mut preds: Vec<Predicate>,
        ty: Arc<TypeNode>,
    ) -> Arc<Scheme> {
        // All predicates should be head normal form.
        assert!(preds.iter().all(|p| p.ty.is_hnf()));

        let mut s = Substitution::default();
        let mut gen_vars: HashMap<String, Arc<Kind>> = Default::default();
        for (i, (v, k)) in vars.iter().enumerate() {
            let gen_name = format!("%g{}", i); // To avoid confliction with user-defined type varible, add prefix %.
            s.add_substitution(&Substitution::single(v, type_tyvar(&gen_name, k)));
            gen_vars.insert(gen_name, k.clone());
        }
        for p in &mut preds {
            s.substitute_predicate(p);
        }
        let ty = s.substitute_type(&ty);
        Scheme::new_arc(gen_vars, preds, ty)
    }

    pub fn from_type(ty: Arc<TypeNode>) -> Arc<Scheme> {
        Scheme::generalize(HashMap::default(), vec![], ty)
    }

    // Get free type variables.
    pub fn free_vars(&self) -> HashMap<Name, Arc<Kind>> {
        let mut ret = self.ty.free_vars();
        for var in &self.vars {
            ret.remove(var.0);
        }
        ret
    }
}
