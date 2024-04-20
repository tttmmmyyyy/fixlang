use core::panic;
use std::sync::Arc;

use inkwell::types::BasicType;
use serde::{de, Deserialize, Serialize};

use super::*;

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Eq, PartialEq, Serialize, Deserialize)]
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

impl Serialize for TyCon {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name.to_string())
    }
}

impl<'de> Deserialize<'de> for TyCon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(TyConVisitor)
    }
}

struct TyConVisitor;
impl<'de> serde::de::Visitor<'de> for TyConVisitor {
    type Value = TyCon;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match FullName::parse(v) {
            Some(name) => Ok(TyCon::new(name)),
            None => Err(de::Error::custom(format!(
                "Failed to parse `{}` as FullName.",
                v
            ))),
        }
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("String for TyCon")
    }
}

impl TyCon {
    pub fn new(fullname: FullName) -> TyCon {
        TyCon { name: fullname }
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
        self: &Arc<TyCon>,
        typechcker: &mut TypeCheckContext,
    ) -> Arc<TypeNode> {
        let ti = typechcker.type_env.tycons.get(self).unwrap();
        assert!(ti.variant == TyConVariant::Struct || ti.variant == TyConVariant::Union);

        // Make type variables for type parameters.
        let new_tyvars_kind = ti
            .tyvars
            .iter()
            .map(|tv| tv.kind.clone())
            .collect::<Vec<_>>();
        let mut new_tyvars: Vec<Arc<TypeNode>> = vec![];
        for new_tyvar_kind in new_tyvars_kind {
            new_tyvars.push(type_tyvar(&typechcker.new_tyvar(), &new_tyvar_kind));
        }

        // Make type.
        let mut ty = type_tycon(self);
        for tv in new_tyvars {
            ty = type_tyapp(ty, tv);
        }
        ty
    }

    // Convert "()", "I8", "Ptr", etc to corresponding c_type.
    // Returns none if it is VoidType.
    pub fn get_c_type<'c>(self: &TyCon, ctx: &'c Context) -> Option<BasicTypeEnum<'c>> {
        if self.name.namespace != NameSpace::new_str(&[STD_NAME]) {
            panic!("call get_c_type for {}", self.to_string())
        }
        if self.name == make_tuple_name(0) {
            return None;
        }
        if self.name.name == I8_NAME {
            return Some(ctx.i8_type().as_basic_type_enum());
        }
        if self.name.name == U8_NAME {
            return Some(ctx.i8_type().as_basic_type_enum());
        }
        if self.name.name == I16_NAME {
            return Some(ctx.i16_type().as_basic_type_enum());
        }
        if self.name.name == U16_NAME {
            return Some(ctx.i16_type().as_basic_type_enum());
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
            U16_NAME => false,
            U32_NAME => false,
            U64_NAME => false,
            I8_NAME => true,
            I16_NAME => true,
            I32_NAME => true,
            I64_NAME => true,
            _ => unreachable!(),
        }
    }

    pub fn is_boolean(self: &TyCon) -> bool {
        return self.name == FullName::from_strs(&[STD_NAME], BOOL_NAME);
    }
}

// Information of type constructor.
// For type alias, this struct is not used; use TyAliasInfo instead.
#[derive(Clone)]
pub struct TyConInfo {
    pub kind: Arc<Kind>,
    pub variant: TyConVariant,
    pub is_unbox: bool,
    pub tyvars: Vec<Arc<TyVar>>,
    pub fields: Vec<Field>, // For array, element type.
    pub source: Option<Span>,
}

impl TyConInfo {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for field in &mut self.fields {
            field.resolve_namespace(ctx);
        }
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        for field in &mut self.fields {
            field.resolve_type_aliases(type_env);
        }
    }
}

#[derive(Clone)]
pub struct TyAliasInfo {
    pub kind: Arc<Kind>,
    pub value: Arc<TypeNode>,
    pub tyvars: Vec<Arc<TyVar>>,
    pub source: Option<Span>,
}

impl TyAliasInfo {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        self.value = self.value.resolve_namespace(ctx);
    }
}

// Node of type ast tree with user defined additional information
#[derive(Serialize, Deserialize)]
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
        write!(f, "{}", Arc::new(self.clone()).to_string_normalize())
    }
}

impl TypeNode {
    // The set of defining modules of type constructors that appear in this type.
    pub fn define_modules_of_tycons(&self, out_set: &mut HashSet<Name>) {
        match &self.ty {
            Type::TyVar(_) => {}
            Type::TyCon(tc) => {
                out_set.insert(tc.name.module());
            }
            Type::TyApp(fun, arg) => {
                fun.define_modules_of_tycons(out_set);
                arg.define_modules_of_tycons(out_set);
            }
            Type::FunTy(src, dst) => {
                src.define_modules_of_tycons(out_set);
                dst.define_modules_of_tycons(out_set);
            }
        }
    }

    // Check if the given type variable appears in `self`.
    pub fn contains_tyvar(&self, tv: &TyVar) -> bool {
        match &self.ty {
            Type::TyVar(tv2) => tv.name == tv2.name, // Ignore kind.
            Type::TyCon(_) => false,
            Type::TyApp(fun, arg) => fun.contains_tyvar(tv) || arg.contains_tyvar(tv),
            Type::FunTy(src, dst) => src.contains_tyvar(tv) || dst.contains_tyvar(tv),
        }
    }

    // Get source.
    pub fn get_source(&self) -> &Option<Span> {
        &self.info.source
    }

    // Set source.
    pub fn set_source(&self, src: Option<Span>) -> Arc<Self> {
        let mut ret = self.clone();
        ret.info.source = src;
        Arc::new(ret)
    }

    // Set source if only when self does not have source info.
    pub fn set_source_if_none(self: &Arc<TypeNode>, src: Option<Span>) -> Arc<TypeNode> {
        if self.info.source.is_none() {
            self.set_source(src)
        } else {
            self.clone()
        }
    }

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

    // Is this type head normal form? i.e., begins with type variable.
    pub fn is_hnf(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => true,
            Type::TyCon(_) => false,
            Type::TyApp(head, _) => head.is_hnf(),
            Type::FunTy(_, _) => false,
        }
    }

    pub fn get_head_string(&self) -> String {
        match &self.ty {
            Type::TyVar(_) => self.to_string(),
            Type::TyCon(_) => self.to_string(),
            Type::TyApp(head, _) => head.get_head_string(),
            Type::FunTy(_, _) => "->".to_string(),
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

    pub fn get_lambda_srcs(&self) -> Vec<Arc<TypeNode>> {
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

    pub fn get_lambda_dst(&self) -> Arc<TypeNode> {
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

    pub fn set_tycon_tc(&self, tc: Arc<TyCon>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyCon(_) => ret.ty = Type::TyCon(tc),
            _ => panic!(),
        }
        Arc::new(ret)
    }

    pub fn resolve_namespace(self: &Arc<TypeNode>, ctx: &NameResolutionContext) -> Arc<TypeNode> {
        match &self.ty {
            Type::TyVar(_tv) => self.clone(),
            Type::TyCon(tc) => {
                let mut tc = tc.as_ref().clone();
                let resolve_result = tc.resolve_namespace(ctx);
                if resolve_result.is_err() {
                    error_exit_with_src(&resolve_result.unwrap_err(), &self.info.source)
                }
                self.set_tycon_tc(Arc::new(tc))
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
    // For Array, return the element type.
    pub fn field_types(&self, type_env: &TypeEnv) -> Vec<Arc<TypeNode>> {
        let args = self.collect_type_argments();
        let ti = self.toplevel_tycon_info(type_env);
        assert_eq!(args.len(), ti.tyvars.len());
        let mut s = Substitution::default();
        for (i, tv) in ti.tyvars.iter().enumerate() {
            s.add_substitution(&Substitution::single(&tv.name, args[i].clone()));
        }
        ti.fields.iter().map(|f| s.substitute_type(&f.ty)).collect()
    }

    // Flatten type application.
    // ex. If given `f a b`, returns `vec![f, a, b]`.
    pub fn flatten_type_application(&self) -> Vec<Arc<TypeNode>> {
        fn flatten_type_application_inner(ty: &TypeNode, args: &mut Vec<Arc<TypeNode>>) {
            match &ty.ty {
                Type::TyApp(fun, arg) => {
                    flatten_type_application_inner(fun, args);
                    args.push(arg.clone());
                }
                _ => {
                    assert!(args.is_empty());
                    args.push(Arc::new(ty.clone()));
                }
            }
        }

        let mut args: Vec<Arc<TypeNode>> = vec![];
        flatten_type_application_inner(self, &mut args);
        args
    }

    fn collect_type_argments(&self) -> Vec<Arc<TypeNode>> {
        let mut ret: Vec<Arc<TypeNode>> = vec![];
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

    // Remove type aliases in a type.
    pub fn resolve_type_aliases(self: &Arc<TypeNode>, env: &TypeEnv) -> Arc<TypeNode> {
        self.resolve_type_aliases_inner(env, vec![], &self.to_string_normalize())
    }

    // Remove type aliases in a type.
    // * `path`, `entry_typename` - Arguments to detect circular aliasing.
    fn resolve_type_aliases_inner(
        self: &Arc<TypeNode>,
        env: &TypeEnv,
        mut path: Vec<String>,
        entry_typename: &str,
    ) -> Arc<TypeNode> {
        // First, treat the case where top-level type constructor is a type alias.
        // As an example, consider type alias `type Lazy a = () -> a`. Then `Lazy Bool` should be resolved to `() -> Bool`.
        let app_seq = self.flatten_type_application();
        let toplevel_ty = &app_seq[0];
        match &toplevel_ty.ty {
            Type::TyCon(tc) => {
                if let Some(ta) = env.aliases.get(&tc) {
                    // Check recursive aliasing.
                    if path.contains(&tc.name.to_string()) {
                        error_exit(&format!(
                            "Cannot resolve type aliasing in `{}`. Circular aliasing is found.",
                            entry_typename
                        ))
                    }
                    path.push(tc.name.to_string());

                    // When the type alias is not fully applied, raise error.
                    if app_seq.len() - 1 < ta.tyvars.len() {
                        error_exit_with_src(
                            &format!(
                                "Cannot resolve type alias `{}` in `{}` because it is not fully applied.",
                                tc.to_string(),
                                Arc::new(self.clone()).to_string_normalize()
                            ),
                            toplevel_ty.get_source(),
                        )
                    }

                    // Resolve alias and calculate type application.
                    let mut s = Substitution::default();
                    let mut src: Option<Span> = toplevel_ty.get_source().clone();
                    for i in 0..ta.tyvars.len() {
                        let param = &ta.tyvars[i].name;
                        let arg = app_seq[i + 1].clone();
                        src = Span::unite_opt(&src, arg.get_source());
                        s.add_substitution(&Substitution::single(&param, arg));
                    }
                    let resolved = s.substitute_type(&ta.value);

                    // Set source for `resolved`.
                    let mut resolved = resolved.set_source(src);
                    for i in (ta.tyvars.len() + 1)..app_seq.len() {
                        let arg = app_seq[i].clone();
                        let src = Span::unite_opt(resolved.get_source(), arg.get_source());
                        resolved = type_tyapp(resolved, arg).set_source(src);
                    }
                    return resolved.resolve_type_aliases_inner(env, path, entry_typename);
                }
            }
            _ => {}
        }
        // Treat other cases.
        match &self.ty {
            Type::TyVar(_) => self.clone(),
            Type::FunTy(dom_ty, codom_ty) => self
                .set_funty_src(dom_ty.resolve_type_aliases_inner(env, path.clone(), entry_typename))
                .set_funty_dst(codom_ty.resolve_type_aliases_inner(env, path, entry_typename)),
            Type::TyCon(_) => self.clone(),
            Type::TyApp(fun_ty, arg_ty) => self
                .set_tyapp_fun(fun_ty.resolve_type_aliases_inner(env, path.clone(), entry_typename))
                .set_tyapp_arg(arg_ty.resolve_type_aliases_inner(env, path, entry_typename)),
        }
    }

    // Is this type head normal form? i.e., begins with type variable.
    pub fn is_funty(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => false,
            Type::TyCon(_) => false,
            Type::TyApp(_, _) => false,
            Type::FunTy(_, _) => true,
        }
    }

    // Get top-level type constructor of a type.
    pub fn toplevel_tycon(&self) -> Option<Arc<TyCon>> {
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

    // Check if `self` is fully unboxed.
    // Here, a type is fully unboxed if and only if it does not contain any boxed type.
    pub fn is_fully_unboxed(&self, type_env: &TypeEnv) -> bool {
        if self.is_box(type_env) {
            return false;
        }
        if self.is_closure() {
            return false;
        }
        if self.is_funptr() {
            return true;
        }
        let field_types = self.field_types(type_env);
        field_types
            .iter()
            .all(|field_ty| field_ty.is_fully_unboxed(type_env))
    }

    // Create new type node with default info.
    fn new(ty: Type) -> Self {
        Self {
            ty,
            info: TypeInfo::default(),
        }
    }

    // Create shared new type node with default info.
    fn new_arc(ty: Type) -> Arc<Self> {
        Arc::new(Self::new(ty))
    }

    // Set new info for shared instance.
    #[allow(dead_code)]
    pub fn set_info(self: Arc<Self>, info: TypeInfo) -> Arc<Self> {
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
    pub fn kind(self: &Arc<TypeNode>, kind_map: &HashMap<TyCon, Arc<Kind>>) -> Arc<Kind> {
        match &self.ty {
            Type::TyVar(tv) => tv.kind.clone(),
            Type::TyCon(tc) => kind_map.get(&tc).unwrap().clone(),
            Type::TyApp(fun, arg) => {
                let fun_kind = fun.kind(kind_map);
                let arg_kind = arg.kind(kind_map);
                match &*fun_kind {
                    Kind::Arrow(arg2, res) => {
                        if arg_kind != *arg2 {
                            error_exit_with_src(
                                &format!("Kind mismatch in `{}`. Type `{}` of kind `{}` cannot be applied to type `{}` of kind `{}`.", self.to_string_normalize(), fun.to_string_normalize(), fun_kind.to_string(), arg.to_string_normalize(), arg_kind.to_string()),
                                &self.get_source(),
                            );
                        }
                        res.clone()
                    }
                    Kind::Star => error_exit_with_src(
                        &format!("Kind mismatch in `{}`. Type `{}` of kind `{}` cannot be applied to type `{}` of kind `{}`.", self.to_string_normalize(), fun.to_string_normalize(), fun_kind.to_string(), arg.to_string_normalize(), arg_kind.to_string()),
                        &self.get_source(),
                    ),
                }
            }
            Type::FunTy(dom, codom) => {
                let arg_kind = dom.kind(kind_map);
                if arg_kind != kind_star() {
                    error_exit_with_src(
                        &format!(
                            "Cannot form function type `{}` since its domain type `{}` has kind `{}`.",
                            self.to_string_normalize(),
                            dom.to_string_normalize(),
                            arg_kind.to_string()
                        ),
                        self.get_source(),
                    )
                }
                let ret_kind = codom.kind(kind_map);
                if ret_kind != kind_star() {
                    error_exit_with_src(
                        &format!("Cannot form function type `{}` since its codomain type `{}` has kind `{}`.", self.to_string_normalize(), codom.to_string_normalize(), ret_kind.to_string()),
                        self.get_source(),
                    )
                }
                kind_star()
            }
        }
    }

    pub fn get_object_type(
        self: &Arc<TypeNode>,
        capture: &Vec<Arc<TypeNode>>,
        type_env: &TypeEnv,
    ) -> ObjectType {
        ty_to_object_ty(self, capture, type_env)
    }

    pub fn get_struct_type<'c, 'm>(
        self: &Arc<TypeNode>,
        gc: &mut GenerationContext<'c, 'm>,
        capture: &Vec<Arc<TypeNode>>,
    ) -> StructType<'c> {
        self.get_object_type(capture, gc.type_env())
            .to_struct_type(gc, vec![])
    }

    pub fn get_embedded_type<'c, 'm>(
        self: &Arc<TypeNode>,
        gc: &mut GenerationContext<'c, 'm>,
        capture: &Vec<Arc<TypeNode>>,
    ) -> BasicTypeEnum<'c> {
        self.get_object_type(capture, gc.type_env())
            .to_embedded_type(gc, vec![])
    }

    // Check if the type takes the form of the definition of associated type.
    // Definition of an associated type has to be of the form `type {AssocTypeName} self {tv1} ... {tvN}`,
    // - where `{AssocTypeName}` is a local name and
    // - `self` is the special type variable.
    // If ok, return the name and arity of the associated type.
    pub fn is_associated_type_defn(&self) -> Option<(Name, Vec<Arc<TyVar>>)> {
        // Validate the type application sequence.
        let app_seq = self.flatten_type_application();
        if app_seq.len() < 2 {
            return None;
        }
        let assoc_type_name: Name;
        match &app_seq[0].ty {
            Type::TyCon(tc) => {
                if !tc.name.is_local() {
                    return None;
                }
                assoc_type_name = tc.name.to_string();
            }
            _ => return None,
        }
        match &app_seq[1].ty {
            Type::TyVar(tv) => {
                if tv.name != TRAIT_IMPL_SELF {
                    return None;
                }
            }
            _ => return None,
        }
        let mut tyvars = vec![];
        let mut tyvars_set: HashSet<Name> = HashSet::new();
        for i in 2..app_seq.len() {
            match &app_seq[i].ty {
                Type::TyVar(tv) => {
                    // Since `tv` should be free, it cannot be `self`.
                    if tv.name == TRAIT_IMPL_SELF {
                        return None;
                    }
                    if tyvars_set.contains(&tv.name) {
                        return None;
                    }
                    tyvars.push(tv.clone());
                    tyvars_set.insert(tv.name.clone());
                }
                _ => return None,
            }
        }
        Some((assoc_type_name, tyvars))
    }

    // Check if the type takes the form of the usage of associated type.
    // Usage of an associated type has to be of the form `AssocTypeName {t1} ... {tvN}`.
    pub fn is_associated_type_usage(
        &self,
        is_assoc_type: impl Fn(&FullName) -> bool,
    ) -> Option<(FullName, Vec<Arc<TypeNode>>)> {
        // Validate the type application sequence.
        let app_seq = self.flatten_type_application();
        if app_seq.len() < 2 {
            return None;
        }
        let assoc_type_name: FullName;
        match &app_seq[0].ty {
            Type::TyCon(tc) => {
                assoc_type_name = tc.name.clone();
            }
            _ => return None,
        }
        if !is_assoc_type(&assoc_type_name) {
            return None;
        }
        let mut type_args = vec![];
        for i in 1..app_seq.len() {
            type_args.push(app_seq[i].clone());
        }
        Some((assoc_type_name, type_args))
    }

    // // Check if the type takes the form of the implementation of associated type.
    // // Implementation of an associated type has to be of the form `type AssocTypeName self tv1 ... tvN`,
    // // - where `AssocTypeName` is a local name,
    // // - `self` is the special type variable, and
    // // - `tv1 ... tvN` are type variables.
    // // If ok, return the pair of the name of the associated type and the list of type variables.
    // pub fn is_associated_type_impl(&self) -> Option<(Name, Vec<Arc<TyVar>>)> {
    //     // `self` has to be a type application.
    //     match &self.ty {
    //         Type::TyApp(_, _) => {}
    //         _ => {
    //             return None;
    //         }
    //     }
    //     let assoc_ty_name;
    //     match &self.ty {
    //         Type::TyApp(fun, _arg) => {
    //             let assoc_ty = match &fun.ty {
    //                 Type::TyCon(tc) => tc,
    //                 _ => return None,
    //             };
    //             // Name of `assoc_ty` should be local.
    //             if !assoc_ty.name.is_local() {
    //                 return None;
    //             }
    //             assoc_ty_name = assoc_ty.name.to_string();
    //         }
    //         _ => return None,
    //     };
    //     let applications = self.flatten_type_application();
    //     match &applications[1].ty {
    //         Type::TyVar(tv) => {
    //             if tv.name != TRAIT_IMPL_SELF {
    //                 return None;
    //             }
    //         }
    //         _ => return None,
    //     }
    //     let mut tyvars = vec![];
    //     for i in 2..applications.len() {
    //         match &applications[i].ty {
    //             Type::TyVar(tv) => {
    //                 // Since `tv` should be free, it cannot be `self`.
    //                 if tv.name == TRAIT_IMPL_SELF {
    //                     return None;
    //                 }
    //                 tyvars.push(tv.clone());
    //             }
    //             _ => return None,
    //         }
    //     }
    //     Some((assoc_ty_name, tyvars))
    // }
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
#[derive(PartialEq, Eq, Serialize, Deserialize)]
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
    // Stringify. Name of type variables are normalized to names such as "t0", "t1", etc.
    pub fn to_string_normalize(self: &Arc<TypeNode>) -> String {
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
                            let mut arg_strs =
                                args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
                            assert!(args.len() <= n as usize);
                            // If args.len() < n, then `self` is a partial application to a tuple.
                            // In this case, we show missing arguments by `*` (e.g., `(Std::I64, *)`).
                            for _ in args.len()..n as usize {
                                arg_strs.push("*".to_string());
                            }
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

    // Get traverser name.
    pub fn traverser_name(self: &Arc<TypeNode>, capture: &Vec<Arc<TypeNode>>) -> String {
        let mut str = "".to_string();
        str += &self.to_string_normalize();
        if capture.len() > 0 {
            str += "_capturing[";
        }
        for ty in capture {
            str += ", ";
            str += &ty.to_string_normalize();
        }
        if capture.len() > 0 {
            str += "]";
        }
        "trav_".to_string() + &format!("{:x}", md5::compute(str))
    }

    // Get hash value.
    pub fn hash(self: &Arc<TypeNode>) -> String {
        let type_string = self.to_string_normalize();
        format!("{:x}", md5::compute(type_string))
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

pub fn type_from_tyvar(tyvar: Arc<TyVar>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(tyvar))
}

pub fn type_fun(src: Arc<TypeNode>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::FunTy(src, dst))
}

pub fn type_funptr(srcs: Vec<Arc<TypeNode>>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    let mut ty = TypeNode::new_arc(Type::TyCon(Arc::new(make_funptr_tycon(srcs.len() as u32))));
    for src in srcs {
        ty = type_tyapp(ty, src);
    }
    ty = type_tyapp(ty, dst);
    ty
}

pub fn type_tyapp(tyfun: Arc<TypeNode>, param: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyApp(tyfun, param))
}

pub fn type_tycon(tycon: &Arc<TyCon>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyCon(tycon.clone()))
}

pub fn tycon(name: FullName) -> Arc<TyCon> {
    Arc::new(TyCon { name })
}

// Additional information of types.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    source: Option<Span>,
}

impl TypeNode {
    // Calculate free type variables.
    pub fn free_vars(self: &Arc<TypeNode>) -> HashMap<Name, Arc<Kind>> {
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

    // Append free type variables to a buffer of type Vec. Elements of the resulting buf may be duplicated.
    pub fn free_vars_vec(self: &Arc<TypeNode>, buf: &mut Vec<Name>) {
        match &self.ty {
            Type::TyVar(tv) => buf.push(tv.name.clone()),
            Type::TyApp(tyfun, arg) => {
                tyfun.free_vars_vec(buf);
                arg.free_vars_vec(buf);
            }
            Type::FunTy(input, output) => {
                input.free_vars_vec(buf);
                output.free_vars_vec(buf);
            }
            Type::TyCon(_) => {}
        };
    }
}

// Type scheme.
#[derive(Clone, Serialize, Deserialize)]
pub struct Scheme {
    // Generalized variables.
    pub vars: HashMap<Name, Arc<Kind>>,
    // Constraints
    pub context: Vec<Predicate>,
    // Generalized type.
    pub ty: Arc<TypeNode>,
}

impl Scheme {
    pub fn to_string(&self) -> String {
        // First, fix ordering of generalized variables (self.vars) following the ordering they appear.
        let mut vars0 = vec![];
        for p in &self.context {
            p.ty.free_vars_vec(&mut vars0);
        }
        self.ty.free_vars_vec(&mut vars0);
        let mut added: HashSet<Name> = Default::default();
        let mut vars = vec![];
        for v in vars0 {
            if added.contains(&v) {
                continue;
            }
            if !self.vars.contains_key(&v) {
                continue;
            }
            vars.push((v.clone(), self.vars.get(&v).unwrap().clone()));
            added.insert(v.clone());
        }

        // Change name of type variables to t0, t1, ...
        let free_vars = self.free_vars();
        let mut s = Substitution::default();
        let mut tyvar_num = -1;
        for (tyvar, kind) in &vars {
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

        // Substitute type variables in predicates and type to chosen names.
        let preds = self
            .context
            .clone()
            .iter()
            .map(|p| {
                let mut p = p.clone();
                s.substitute_predicate(&mut p);
                p
            })
            .collect::<Vec<_>>();
        let ty = s.substitute_type(&self.ty);

        // Stringify.
        let preds_str = if preds.is_empty() {
            "".to_string()
        } else {
            format!(
                "[{}] ",
                preds
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        preds_str + &ty.to_string()
    }

    #[allow(dead_code)]
    pub fn set_ty(&self, ty: Arc<TypeNode>) -> Arc<Scheme> {
        let mut ret = self.clone();
        ret.ty = ty;
        Arc::new(ret)
    }

    pub fn set_kinds(&self, trait_kind_map: &HashMap<TraitId, Arc<Kind>>) -> Arc<Scheme> {
        let mut ret = self.clone();
        let mut scope: HashMap<Name, Arc<Kind>> = Default::default();
        // If a kind in `self.vars` is not `*`, then the kind is explicitly specified by user, so we insert it into `scope`.
        for (v, k) in &self.vars {
            if *k != kind_star() {
                scope.insert(v.clone(), k.clone());
            }
        }
        let res =
            QualPredicate::extend_kind_scope(&mut scope, &ret.context, &vec![], trait_kind_map);
        if let Err(msg) = res {
            let mut span = ret.context[0].source.clone();
            for i in 1..ret.context.len() {
                span = Span::unite_opt(&span, &ret.context[i].source);
            }
            error_exit_with_src(&msg, &span);
        }
        for p in &mut ret.context {
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

    pub fn check_kinds(
        &self,
        kind_map: &HashMap<TyCon, Arc<Kind>>,
        trait_kind_map: &HashMap<TraitId, Arc<Kind>>,
    ) {
        for p in &self.context {
            p.check_kinds(kind_map, trait_kind_map);
        }
        self.ty.kind(kind_map);
    }

    // Create new instance.
    fn new_arc(
        vars: HashMap<String, Arc<Kind>>,
        preds: Vec<Predicate>,
        ty: Arc<TypeNode>,
    ) -> Arc<Scheme> {
        Arc::new(Scheme {
            vars,
            context: preds,
            ty,
        })
    }

    pub fn substitute(&self, s: &Substitution) -> Arc<Scheme> {
        // Generalized variables cannot be replaced.
        for (v, _) in &self.vars {
            assert!(!s.data.contains_key(v));
        }
        let mut preds = self.context.clone();
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

    pub fn resolve_namespace(&self, ctx: &NameResolutionContext) -> Arc<Scheme> {
        let mut res = self.clone();
        for p in &mut res.context {
            p.resolve_namespace(ctx);
        }
        res.ty = res.ty.resolve_namespace(ctx);
        Arc::new(res)
    }

    pub fn resolve_type_aliases(&self, type_env: &TypeEnv) -> Arc<Scheme> {
        let mut res = self.clone();
        for p in &mut res.context {
            p.resolve_type_aliases(type_env);
        }
        res.ty = res.ty.resolve_type_aliases(type_env);
        Arc::new(res)
    }
}
