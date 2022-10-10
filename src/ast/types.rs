use super::*;

#[derive(Eq, PartialEq)]
pub struct TyVar {
    pub name: String,
}

// impl Var {
//     pub fn name(self: &Self) -> &String {
//         match self {
//             Var::TermVar {
//                 name,
//                 type_annotation: _,
//             } => name,
//             Var::TyVar { name } => name,
//         }
//     }
// }

#[derive(Eq, PartialEq)]
pub enum Kind {
    Star,
    Arrow(Arc<Kind>, Arc<Kind>),
}

#[derive(Eq, PartialEq)]
pub struct TyLit {
    pub id: u32,
    pub name: String,
}

// Node of type ast tree with user defined additional information
pub struct TypeNode {
    pub ty: Type,
    pub info: Arc<TypeInfo>,
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
    pub fn set_info(self: Arc<Self>, info: Arc<TypeInfo>) -> Arc<Self> {
        let mut ret = (*self).clone();
        ret.info = info;
        Arc::new(ret)
    }

    // Set new type for shared instance.
    pub fn set_ty(self: &Arc<Self>, ty: Type) -> Arc<Self> {
        let mut ret = (**self).clone();
        ret.ty = ty;
        Arc::new(ret)
    }
}

// Variant of type
pub enum Type {
    TyVar(Arc<TyVar>),
    LitTy(Arc<TyLit>),
    AppTy(Arc<TypeNode>, Arc<TypeNode>),
    TyConApp(Arc<TyCon>, Vec<Arc<TypeNode>>),
    FunTy(Arc<TypeNode>, Arc<TypeNode>),
    ForAllTy(Arc<TyVar>, Arc<TypeNode>),
}

impl Clone for Type {
    fn clone(&self) -> Self {
        match self {
            Type::TyVar(x) => Type::TyVar(x.clone()),
            Type::LitTy(x) => Type::LitTy(x.clone()),
            Type::AppTy(x, y) => Type::AppTy(x.clone(), y.clone()),
            Type::TyConApp(x, y) => Type::TyConApp(x.clone(), y.clone()),
            Type::FunTy(x, y) => Type::FunTy(x.clone(), y.clone()),
            Type::ForAllTy(x, y) => Type::ForAllTy(x.clone(), y.clone()),
        }
    }
}

impl TypeNode {
    pub fn to_string(self: Arc<Self>) -> String {
        match &self.ty {
            Type::TyVar(v) => v.name.clone(),
            Type::LitTy(l) => l.name.clone(),
            Type::AppTy(_, _) => {
                let (ty, args) = self.decompose_appty();
                let ty = ty.to_string();
                let args: Vec<String> = args.iter().map(|a| a.clone().to_string()).collect();
                let mut res: String = Default::default();
                res += &ty;
                res += "<";
                res += &args.join(", ");
                res += ">";
                res
            }
            Type::TyConApp(tycon, args) => {
                let tycon = tycon.name.clone();
                let args: Vec<String> = args.iter().map(|a| a.clone().to_string()).collect();
                let mut res: String = Default::default();
                res += &tycon;
                res += "<";
                res += &args.join(", ");
                res += ">";
                res
            }
            Type::FunTy(src, dst) => {
                let src_brace_needed = match src.ty {
                    Type::FunTy(_, _) => true,
                    Type::ForAllTy(_, _) => true,
                    _ => false,
                };
                let src = src.clone().to_string();
                let dst = dst.clone().to_string();
                let mut res: String = Default::default();
                if src_brace_needed {
                    res += "(";
                    res += &src;
                    res += ")";
                } else {
                    res += &src;
                }
                res += " => ";
                res += &dst;
                res
            }
            Type::ForAllTy(_, _) => {
                let (vars, ty) = self.decompose_forall();
                let vars: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
                let mut res: String = Default::default();
                res += "for<";
                res += &vars.join(", ");
                res += "> ";
                res += &ty.to_string();
                res
            }
        }
    }

    // Decompose AppTy as many as possible.
    // Example: a<b, c> --> (a, vec![b, c])
    fn decompose_appty(self: Arc<Self>) -> (Arc<Self>, Vec<Arc<Self>>) {
        match &self.ty {
            Type::AppTy(ty, arg) => {
                let (ty, mut args) = ty.clone().decompose_appty();
                args.push(arg.clone());
                (ty, args)
            }
            _ => (self.clone(), vec![]),
        }
    }

    // Decompose ForAllTy as many as possible.
    // Example: for<b, c> a --> (vec![b, c], a)
    pub fn decompose_forall(self: Arc<Self>) -> (Vec<Arc<TyVar>>, Arc<Self>) {
        let (mut vars, ty) = self.decompose_forall_reversed();
        vars.reverse();
        (vars, ty)
    }

    // Decompose ForAllTy as many as possible. (returned vars are reversed)
    pub fn decompose_forall_reversed(self: Arc<Self>) -> (Vec<Arc<TyVar>>, Arc<Self>) {
        match &self.ty {
            Type::ForAllTy(var, ty) => {
                let (mut vars, ty) = ty.clone().decompose_forall();
                vars.push(var.clone());
                (vars, ty)
            }
            _ => (vec![], self.clone()),
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct TyCon {
    pub name: String,
    pub arity: u32, // kind: Arc<Kind>,
}

pub fn tycon(name: &str, arity: u32) -> Arc<TyCon> {
    Arc::new(TyCon {
        name: String::from(name),
        arity,
    })
}

pub fn star_kind() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

pub fn arrow_kind(src: Arc<Kind>, dst: Arc<Kind>) -> Arc<Kind> {
    Arc::new(Kind::Arrow(src, dst))
}

pub fn type_func(src: Arc<TypeNode>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::FunTy(src, dst))
}

pub fn tyvar_from_name(var_name: &str) -> Arc<TyVar> {
    Arc::new(TyVar {
        name: String::from(var_name),
    })
}

pub fn type_tyvar(var_name: &str) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar_from_name(var_name)))
}

pub fn type_var_from_tyvar(tyvar: Arc<TyVar>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar))
}

pub fn type_lit(id: u32, name: &str) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::LitTy(Arc::new(TyLit {
        id,
        name: String::from(name),
    })))
}

pub fn type_app(head: Arc<TypeNode>, param: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::AppTy(head, param))
}

pub fn type_fun(src: Arc<TypeNode>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::FunTy(src, dst))
}

pub fn type_forall(var: Arc<TyVar>, ty: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::ForAllTy(var, ty))
}

pub fn tycon_app(tycon: Arc<TyCon>, params: Vec<Arc<TypeNode>>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyConApp(tycon, params))
}

// Additional information of types.
#[derive(Default, Clone)]
pub struct TypeInfo {
    pub free_vars: Option<HashSet<String>>,
}

impl TypeNode {
    // Calculate free type variables.
    pub fn calculate_free_vars(self: &Arc<Self>) -> Arc<Self> {
        if self.info.free_vars.is_some() {
            return self.clone();
        }
        let mut free_vars = HashSet::<String>::default();
        let ty = match &self.ty {
            Type::TyVar(tv) => {
                free_vars.insert(tv.name.clone());
                self.ty.clone()
            }
            Type::LitTy(_) => self.ty.clone(),
            Type::AppTy(forallty, argty) => {
                let forallty = forallty.calculate_free_vars();
                let argty = argty.calculate_free_vars();
                free_vars.extend(forallty.info.free_vars.clone().unwrap());
                Type::AppTy(forallty, argty)
            }
            Type::TyConApp(tycon, args) => {
                let tycon = tycon.clone();
                let args: Vec<Arc<Self>> = args.iter().map(|ty| ty.calculate_free_vars()).collect();
                for arg in args.iter() {
                    free_vars.extend(arg.info.free_vars.clone().unwrap());
                }
                Type::TyConApp(tycon, args)
            }
            Type::FunTy(input, output) => {
                let input = input.calculate_free_vars();
                let output = output.calculate_free_vars();
                free_vars.extend(input.info.free_vars.clone().unwrap());
                free_vars.extend(output.info.free_vars.clone().unwrap());
                Type::FunTy(input, output)
            }
            Type::ForAllTy(var, body) => {
                let body = body.calculate_free_vars();
                free_vars.extend(body.info.free_vars.clone().unwrap());
                free_vars.remove(&var.name);
                Type::ForAllTy(var.clone(), body)
            }
        };
        self.set_ty(ty).set_free_vars(free_vars)
    }

    // Set free variables.
    pub fn set_free_vars(self: &Arc<Self>, free_vars: HashSet<String>) -> Arc<Self> {
        let mut info = (*self.info).clone();
        info.free_vars = Some(free_vars);
        self.clone().set_info(Arc::new(info))
    }
}
