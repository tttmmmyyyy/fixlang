use core::panic;

use inkwell::types::BasicType;

use super::*;

#[derive(Eq, PartialEq, Clone)]
pub struct TyVar {
    pub name: String,
    pub kind: Rc<Kind>,
}

impl TyVar {
    pub fn set_kind(&self, kind: Rc<Kind>) -> Rc<TyVar> {
        let mut ret = self.clone();
        ret.kind = kind;
        Rc::new(ret)
    }
}

#[derive(Eq, PartialEq)]
pub enum Kind {
    Star,
    Arrow(Rc<Kind>, Rc<Kind>),
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

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum TyConVariant {
    Primitive,
    Array,
    Struct,
    Union,
    // Dynamic object is nullble and has the destructor as the first field.
    DynamicObject,
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct TyCon {
    pub name: FullName,
}

impl TyCon {
    pub fn new(nsn: FullName) -> TyCon {
        TyCon { name: nsn }
    }

    pub fn to_string(&self) -> String {
        match get_tuple_n(&self.name) {
            Some(n) => {
                if n == 0 {
                    return "()".to_string();
                }
            }
            None => {}
        }
        self.name.to_string()
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), String> {
        self.name = ctx.resolve(&self.name, NameResolutionType::Type)?;
        Ok(())
    }

    // Get the type of struct / union value.
    // If struct / union have type parameter, introduces new type arguments.
    pub fn get_struct_union_value_type(
        self: &Rc<TyCon>,
        typechcker: &mut TypeCheckContext,
    ) -> Rc<TypeNode> {
        let ti = typechcker.type_env.tycons.get(self).unwrap();
        assert!(ti.variant == TyConVariant::Struct || ti.variant == TyConVariant::Union);
        // Make type variables for type parameters.
        // Currently type parameters for struct / union must have kind *.
        let mut tyvars: Vec<Rc<TypeNode>> = vec![];
        let tyvars_cnt = ti.tyvars.len();
        for _ in 0..tyvars_cnt {
            tyvars.push(type_tyvar_star(&typechcker.new_tyvar()));
        }

        // Make type.
        let mut ty = type_tycon(self);
        for tv in tyvars {
            ty = type_tyapp(ty, tv);
        }
        ty
    }

    // Convert "()", "I8", "Ptr", etc to corresponding c_type.
    // Returns none if it's VoidType.
    pub fn get_c_type<'c>(self: &TyCon, ctx: &'c Context) -> Option<BasicTypeEnum<'c>> {
        if self.name.namespace != NameSpace::new_str(&[STD_NAME]) {
            panic!("call get_c_type for {}", self.to_string())
        }
        if self.name == make_tuple_name(0) {
            return None;
        }
        if self.name.name == U8_NAME {
            return Some(ctx.i8_type().as_basic_type_enum());
        }
        if self.name.name == I32_NAME {
            return Some(ctx.i32_type().as_basic_type_enum());
        }
        if self.name.name == U32_NAME {
            return Some(ctx.i32_type().as_basic_type_enum());
        }
        if self.name.name == I64_NAME {
            return Some(ctx.i64_type().as_basic_type_enum());
        }
        if self.name.name == U64_NAME {
            return Some(ctx.i64_type().as_basic_type_enum());
        }
        if self.name.name == F32_NAME {
            return Some(ctx.f32_type().as_basic_type_enum());
        }
        if self.name.name == F64_NAME {
            return Some(ctx.f64_type().as_basic_type_enum());
        }
        if self.name.name == PTR_NAME {
            return Some(
                ctx.i8_type()
                    .ptr_type(AddressSpace::from(0))
                    .as_basic_type_enum(),
            );
        }
        panic!("call get_c_type for {}", self.to_string())
    }

    pub fn is_singned_intger(self: &TyCon) -> bool {
        if self.name.namespace != NameSpace::new_str(&[STD_NAME]) {
            panic!("call is_singned_intger for {}", self.to_string())
        }
        match self.name.name.as_str() {
            U8_NAME => false,
            U32_NAME => false,
            U64_NAME => false,
            I32_NAME => true,
            I64_NAME => true,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct TyConInfo {
    pub kind: Rc<Kind>,
    pub variant: TyConVariant,
    pub is_unbox: bool,
    pub tyvars: Vec<Name>,
    pub fields: Vec<Field>, // For array, element type.
}

impl TyConInfo {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for field in &mut self.fields {
            field.resolve_namespace(ctx);
        }
    }
}

// Node of type ast tree with user defined additional information
pub struct TypeNode {
    pub ty: Type,
    pub info: TypeInfo,
}

impl PartialEq for TypeNode {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}

impl Eq for TypeNode {}

impl std::fmt::Debug for TypeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Rc::new(self.clone()).to_string_normalize())
    }
}

impl TypeNode {
    // Set source.
    pub fn set_source(&self, src: Option<Span>) -> Rc<Self> {
        let mut ret = self.clone();
        ret.info.source = src;
        Rc::new(ret)
    }

    // Set kinds to type variables.
    pub fn set_kinds(self: &Rc<TypeNode>, kinds: &HashMap<Name, Rc<Kind>>) -> Rc<TypeNode> {
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

    // Is this type head normal form? i.e., begins with type variable.
    pub fn is_hnf(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => true,
            Type::TyCon(_) => false,
            Type::TyApp(head, _) => head.is_hnf(),
            Type::FunTy(_, _) => false,
        }
    }

    pub fn set_tyvar_kind(&self, kind: Rc<Kind>) -> Rc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyVar(tv) => {
                ret.ty = Type::TyVar(tv.set_kind(kind));
            }
            _ => panic!(),
        }
        Rc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_tyapp_fun(&self, fun: Rc<TypeNode>) -> Rc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyApp(_, arg) => ret.ty = Type::TyApp(fun, arg.clone()),
            _ => panic!(),
        }
        Rc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_tyapp_arg(&self, arg: Rc<TypeNode>) -> Rc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyApp(fun, _) => ret.ty = Type::TyApp(fun.clone(), arg),
            _ => panic!(),
        }
        Rc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_funty_src(&self, src: Rc<TypeNode>) -> Rc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::FunTy(_, dst) => ret.ty = Type::FunTy(src, dst.clone()),
            _ => panic!(),
        }
        Rc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_funty_dst(&self, dst: Rc<TypeNode>) -> Rc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::FunTy(src, _) => ret.ty = Type::FunTy(src.clone(), dst),
            _ => panic!(),
        }
        Rc::new(ret)
    }

    pub fn get_lambda_srcs(&self) -> Vec<Rc<TypeNode>> {
        match &self.ty {
            Type::FunTy(src, _dst) => vec![src.clone()],
            _ => {
                if self.is_funptr() {
                    let mut type_args = self.collect_type_argments();
                    type_args.pop();
                    type_args
                } else {
                    panic!();
                }
            }
        }
    }

    pub fn get_lambda_dst(&self) -> Rc<TypeNode> {
        match &self.ty {
            Type::FunTy(_src, dst) => dst.clone(),
            _ => {
                if self.is_funptr() {
                    let mut type_args = self.collect_type_argments();
                    type_args.pop().unwrap()
                } else {
                    panic!()
                }
            }
        }
    }

    pub fn set_tycon_tc(&self, tc: Rc<TyCon>) -> Rc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyCon(_) => ret.ty = Type::TyCon(tc),
            _ => panic!(),
        }
        Rc::new(ret)
    }

    pub fn resolve_namespace(self: &Rc<TypeNode>, ctx: &NameResolutionContext) -> Rc<TypeNode> {
        match &self.ty {
            Type::TyVar(_tv) => self.clone(),
            Type::TyCon(tc) => {
                let mut tc = tc.as_ref().clone();
                let resolve_result = tc.resolve_namespace(ctx);
                if resolve_result.is_err() {
                    error_exit_with_src(&resolve_result.unwrap_err(), &self.info.source)
                }
                self.set_tycon_tc(Rc::new(tc))
            }
            Type::TyApp(fun, arg) => self
                .set_tyapp_fun(fun.resolve_namespace(ctx))
                .set_tyapp_arg(arg.resolve_namespace(ctx)),
            Type::FunTy(src, dst) => self
                .set_funty_src(src.resolve_namespace(ctx))
                .set_funty_dst(dst.resolve_namespace(ctx)),
        }
    }

    // For structs and unions, return types of fields.
    // For Array, return element type.
    pub fn field_types(&self, type_env: &TypeEnv) -> Vec<Rc<TypeNode>> {
        let args = self.collect_type_argments();
        let ti = self.toplevel_tycon_info(type_env);
        assert_eq!(args.len(), ti.tyvars.len());
        let mut s = Substitution::default();
        for (i, tv) in ti.tyvars.iter().enumerate() {
            s.add_substitution(&Substitution::single(tv, args[i].clone()));
        }
        ti.fields.iter().map(|f| s.substitute_type(&f.ty)).collect()
    }

    fn collect_type_argments(&self) -> Vec<Rc<TypeNode>> {
        let mut ret: Vec<Rc<TypeNode>> = vec![];
        match &self.ty {
            Type::TyApp(fun, arg) => {
                ret.append(&mut fun.collect_type_argments());
                ret.push(arg.clone());
            }
            Type::TyCon(_) => {}
            _ => unreachable!(),
        }
        ret
    }

    // Get top-level type constructor.
    pub fn toplevel_tycon(&self) -> Option<Rc<TyCon>> {
        match &self.ty {
            Type::TyVar(_) => None,
            Type::TyCon(tc) => Some(tc.clone()),
            Type::TyApp(fun, _) => fun.toplevel_tycon(),
            Type::FunTy(_, _) => None,
        }
    }

    pub fn is_closure(&self) -> bool {
        match self.ty {
            Type::FunTy(_, _) => true,
            _ => false,
        }
    }

    pub fn is_funptr(&self) -> bool {
        let tc = self.toplevel_tycon();
        if tc.is_none() {
            return false;
        }
        let tc = tc.unwrap();
        if let Some(_) = is_funptr_tycon(tc.as_ref()) {
            return true;
        } else {
            return false;
        }
    }

    pub fn is_array(&self) -> bool {
        let tc = self.toplevel_tycon();
        if tc.is_none() {
            return false;
        }
        let tc = tc.unwrap();
        return is_array_tycon(tc.as_ref());
    }

    pub fn is_dynamic(&self) -> bool {
        let tc = self.toplevel_tycon();
        if tc.is_none() {
            return false;
        }
        let tc = tc.unwrap();
        is_dynamic_object_tycon(tc.as_ref())
    }

    pub fn is_destructor_object(&self) -> bool {
        let tc = self.toplevel_tycon();
        if tc.is_none() {
            return false;
        }
        let tc = tc.unwrap();
        is_destructor_object_tycon(tc.as_ref())
    }

    pub fn toplevel_tycon_info(&self, type_env: &TypeEnv) -> TyConInfo {
        assert!(!self.is_closure());
        type_env
            .tycons
            .get(&self.toplevel_tycon().unwrap())
            .unwrap()
            .clone()
    }

    pub fn is_unbox(&self, type_env: &TypeEnv) -> bool {
        self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox
    }

    pub fn is_box(&self, type_env: &TypeEnv) -> bool {
        !self.is_unbox(type_env)
    }

    // Create new type node with default info.
    fn new(ty: Type) -> Self {
        Self {
            ty,
            info: TypeInfo::default(),
        }
    }

    // Create shared new type node with default info.
    fn new_arc(ty: Type) -> Rc<Self> {
        Rc::new(Self::new(ty))
    }

    // Set new info for shared instance.
    #[allow(dead_code)]
    pub fn set_info(self: Rc<Self>, info: TypeInfo) -> Rc<Self> {
        let mut ret = (*self).clone();
        ret.info = info;
        Rc::new(ret)
    }

    // Set new type for shared instance.
    #[allow(dead_code)]
    pub fn set_ty(self: &Rc<Self>, ty: Type) -> Rc<Self> {
        let mut ret = (**self).clone();
        ret.ty = ty;
        Rc::new(ret)
    }

    // Calculate kind.
    pub fn kind(&self, type_env: &TypeEnv) -> Rc<Kind> {
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

    pub fn get_object_type(
        self: &Rc<TypeNode>,
        capture: &Vec<Rc<TypeNode>>,
        type_env: &TypeEnv,
    ) -> ObjectType {
        get_object_type(self, capture, type_env)
    }

    pub fn get_struct_type<'c, 'm>(
        self: &Rc<TypeNode>,
        gc: &mut GenerationContext<'c, 'm>,
        capture: &Vec<Rc<TypeNode>>,
    ) -> StructType<'c> {
        self.get_object_type(capture, gc.type_env())
            .to_struct_type(gc)
    }

    pub fn get_embedded_type<'c, 'm>(
        self: &Rc<TypeNode>,
        gc: &mut GenerationContext<'c, 'm>,
        capture: &Vec<Rc<TypeNode>>,
    ) -> BasicTypeEnum<'c> {
        self.get_object_type(capture, gc.type_env())
            .to_embedded_type(gc)
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

// Variant of type
#[derive(PartialEq, Eq)]
pub enum Type {
    TyVar(Rc<TyVar>),
    TyCon(Rc<TyCon>),
    TyApp(Rc<TypeNode>, Rc<TypeNode>),
    FunTy(Rc<TypeNode>, Rc<TypeNode>),
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
    // Stringify. Name of type variables are normalized to names such as "t0", "t1", etc.
    pub fn to_string_normalize(self: &Rc<TypeNode>) -> String {
        let mut tyvar_num = -1;
        let mut s = Substitution::default();
        for (tyvar, kind) in self.free_vars() {
            tyvar_num += 1;
            let new_name = format!("t{}", tyvar_num);
            s.add_substitution(&Substitution::single(&tyvar, type_tyvar(&new_name, &kind)))
        }
        s.substitute_type(self).to_string()
    }

    // Stringify.
    pub fn to_string(&self) -> String {
        match &self.ty {
            Type::TyVar(v) => v.name.clone(),
            Type::TyApp(fun, arg) => {
                let tycon = self.toplevel_tycon();
                if tycon.is_some() {
                    match get_tuple_n(&tycon.unwrap().name) {
                        Some(n) => {
                            let args = self.collect_type_argments();
                            assert_eq!(args.len(), n as usize);
                            let arg_strs =
                                args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
                            return format!("({})", arg_strs.join(", "));
                        }
                        None => {}
                    }
                }
                let arg_brace_needed = match &arg.ty {
                    Type::TyVar(_) => false,
                    Type::TyCon(_) => false,
                    Type::TyApp(fun, _) => {
                        let tycon = fun.toplevel_tycon();
                        let is_tuple =
                            tycon.is_some() && get_tuple_n(&tycon.unwrap().name).is_some();
                        !is_tuple
                    }
                    Type::FunTy(_, _) => true,
                };
                let tyfun = fun.to_string();
                let arg = arg.to_string();
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
                res += " -> ";
                res += &dst;
                res
            }
            Type::TyCon(tc) => tc.to_string(),
        }
    }

    // Get dtor name.
    pub fn dtor_name(self: &Rc<TypeNode>, capture: &Vec<Rc<TypeNode>>) -> String {
        let mut str = "".to_string();
        str += &self.to_string_normalize();
        for ty in capture {
            str += &ty.to_string_normalize();
        }
        "dtor_".to_string() + &format!("{:x}", md5::compute(str))
    }

    // Get hash value.
    pub fn hash(self: &Rc<TypeNode>) -> String {
        let type_string = self.to_string_normalize();
        format!("{:x}", md5::compute(type_string))
    }
}

pub fn kind_star() -> Rc<Kind> {
    Rc::new(Kind::Star)
}

pub fn kind_arrow(src: Rc<Kind>, dst: Rc<Kind>) -> Rc<Kind> {
    Rc::new(Kind::Arrow(src, dst))
}

pub fn tyvar_from_name(var_name: &str, kind: &Rc<Kind>) -> Rc<TyVar> {
    Rc::new(TyVar {
        name: String::from(var_name),
        kind: kind.clone(),
    })
}

pub fn type_tyvar(var_name: &str, kind: &Rc<Kind>) -> Rc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar_from_name(var_name, kind)))
}

pub fn type_tyvar_star(var_name: &str) -> Rc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar_from_name(var_name, &kind_star())))
}

pub fn type_var_from_tyvar(tyvar: Rc<TyVar>) -> Rc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar))
}

pub fn type_fun(src: Rc<TypeNode>, dst: Rc<TypeNode>) -> Rc<TypeNode> {
    TypeNode::new_arc(Type::FunTy(src, dst))
}

pub fn type_funptr(srcs: Vec<Rc<TypeNode>>, dst: Rc<TypeNode>) -> Rc<TypeNode> {
    let mut ty = TypeNode::new_arc(Type::TyCon(Rc::new(make_funptr_tycon(srcs.len() as u32))));
    for src in srcs {
        ty = type_tyapp(ty, src);
    }
    ty = type_tyapp(ty, dst);
    ty
}

pub fn type_tyapp(tyfun: Rc<TypeNode>, param: Rc<TypeNode>) -> Rc<TypeNode> {
    TypeNode::new_arc(Type::TyApp(tyfun, param))
}

pub fn type_tycon(tycon: &Rc<TyCon>) -> Rc<TypeNode> {
    TypeNode::new_arc(Type::TyCon(tycon.clone()))
}

pub fn tycon(name: FullName) -> Rc<TyCon> {
    Rc::new(TyCon { name })
}

// Additional information of types.
#[derive(Default, Clone)]
pub struct TypeInfo {
    #[allow(dead_code)]
    source: Option<Span>,
}

impl TypeNode {
    // Calculate free type variables.
    pub fn free_vars(self: &Rc<TypeNode>) -> HashMap<Name, Rc<Kind>> {
        let mut free_vars: HashMap<String, Rc<Kind>> = HashMap::default();
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
}

// Type scheme.
#[derive(Clone)]
pub struct Scheme {
    pub vars: HashMap<Name, Rc<Kind>>,
    pub preds: Vec<Predicate>,
    pub ty: Rc<TypeNode>,
}

impl Scheme {
    pub fn to_string(&self) -> String {
        // Change name of type variables to t0, t1, ...
        let free_vars = self.free_vars();
        let mut s = Substitution::default();
        let mut tyvar_num = -1;

        for (tyvar, kind) in &self.vars {
            let new_name = loop {
                tyvar_num += 1;
                let new_name = format!("t{}", tyvar_num);
                if free_vars.contains_key(&new_name) {
                    continue;
                }
                break new_name;
            };
            s.add_substitution(&Substitution::single(tyvar, type_tyvar(&new_name, kind)))
        }
        let preds = self
            .preds
            .clone()
            .iter()
            .map(|p| {
                let mut p = p.clone();
                s.substitute_predicate(&mut p);
                p
            })
            .collect::<Vec<_>>();
        let ty = s.substitute_type(&self.ty);

        let preds_str = if preds.is_empty() {
            "".to_string()
        } else {
            format!(
                "[{}] ",
                preds
                    .iter()
                    .map(|p| p.to_string_normalize())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        preds_str + &ty.to_string()
    }

    #[allow(dead_code)]
    pub fn set_ty(&self, ty: Rc<TypeNode>) -> Rc<Scheme> {
        let mut ret = self.clone();
        ret.ty = ty;
        Rc::new(ret)
    }

    pub fn set_kinds(&self, trait_kind_map: &HashMap<TraitId, Rc<Kind>>) -> Rc<Scheme> {
        let mut ret = self.clone();
        let mut scope: HashMap<Name, Rc<Kind>> = Default::default();
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
        Rc::new(ret)
    }

    pub fn check_kinds(&self, type_env: &TypeEnv, trait_kind_map: &HashMap<TraitId, Rc<Kind>>) {
        for p in &self.preds {
            p.check_kinds(type_env, trait_kind_map);
        }
        self.ty.kind(type_env);
    }

    // Create new instance.
    fn new_arc(
        vars: HashMap<String, Rc<Kind>>,
        preds: Vec<Predicate>,
        ty: Rc<TypeNode>,
    ) -> Rc<Scheme> {
        Rc::new(Scheme { vars, preds, ty })
    }

    #[allow(dead_code)]
    pub fn substitute(&self, s: &Substitution) -> Rc<Scheme> {
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
        vars: HashMap<String, Rc<Kind>>,
        mut preds: Vec<Predicate>,
        ty: Rc<TypeNode>,
    ) -> Rc<Scheme> {
        // All predicates should be head normal form.
        assert!(preds.iter().all(|p| p.ty.is_hnf()));

        let mut s = Substitution::default();
        let mut gen_vars: HashMap<String, Rc<Kind>> = Default::default();
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

    pub fn from_type(ty: Rc<TypeNode>) -> Rc<Scheme> {
        Scheme::generalize(HashMap::default(), vec![], ty)
    }

    // Get free type variables.
    #[allow(dead_code)]
    pub fn free_vars(&self) -> HashMap<Name, Rc<Kind>> {
        let mut ret = self.ty.free_vars();
        for var in &self.vars {
            ret.remove(var.0);
        }
        ret
    }

    pub fn resolve_namespace(&self, ctx: &NameResolutionContext) -> Rc<Scheme> {
        let mut res = self.clone();
        for p in &mut res.preds {
            p.resolve_namespace(ctx);
        }
        res.ty = res.ty.resolve_namespace(ctx);
        Rc::new(res)
    }
}
