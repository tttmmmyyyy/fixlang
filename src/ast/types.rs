use core::panic;
use std::sync::Arc;

use crate::error::error_exit_with_src;
use crate::error::Errors;
use inkwell::types::BasicType;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TyVar {
    pub name: Name,
    pub kind: Arc<Kind>,
}

impl TyVar {
    pub fn set_kind(&self, kind: Arc<Kind>) -> Arc<TyVar> {
        let mut ret = self.clone();
        ret.kind = kind;
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_name(&self, name: Name) -> Arc<TyVar> {
        let mut ret = self.clone();
        ret.name = name;
        Arc::new(ret)
    }
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Hash)]
pub struct TyAssoc {
    pub name: FullName,
}

impl TyAssoc {
    pub fn resolve_namespace(
        &mut self,
        ctx: &NameResolutionContext,
        span: &Option<Span>,
    ) -> Result<(), Errors> {
        self.name = ctx.resolve(&self.name, &[NameResolutionType::AssocTy], span)?;
        Ok(())
    }

    pub fn trait_id(&self) -> Trait {
        let mut namespace = self.name.namespace.names.clone();
        let name = namespace.pop().unwrap();
        Trait {
            name: FullName::new(&NameSpace::new(namespace), &name),
        }
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

#[derive(Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct TyCon {
    pub name: FullName,
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

    pub fn resolve_namespace(
        &mut self,
        ctx: &NameResolutionContext,
        span: &Option<Span>,
    ) -> Result<(), Errors> {
        self.name = ctx.resolve(
            &self.name,
            &[NameResolutionType::TyCon, NameResolutionType::AssocTy],
            span,
        )?;
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

    pub fn into_punched_type_name(&mut self, punched_at: usize) {
        self.name.name += &format!("{}{}", PUNCHED_TYPE_SYMBOL, punched_at);
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
    pub fields: Vec<Field>, // For an array type, this is `vec![{element_type}]`.
    pub source: Option<Span>,
    // The document of this type.
    // If `def_src` is available, we can also get document from the source code.
    // We use this field only when document is not available in the source code.
    pub document: Option<String>,
}

impl TyConInfo {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for field in &mut self.fields {
            errors.eat_err(field.resolve_namespace(ctx));
        }
        errors.to_result()
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for field in &mut self.fields {
            errors.eat_err(field.resolve_type_aliases(type_env));
        }
        errors.to_result()
    }

    // Get the document of this type.
    pub fn get_document(&self) -> Option<String> {
        // Try to get document from the source code.
        let docs = self.source.as_ref().and_then(|src| src.get_document().ok());

        // If the documentation is empty, treat it as None.
        let docs = match docs {
            Some(docs) if docs.is_empty() => None,
            _ => docs,
        };

        // If the document is not available in the source code, use the document field.
        let docs = match docs {
            Some(_) => docs,
            None => self.document.clone(),
        };

        // Again, if the documentation is empty, treat it as None.
        match docs {
            Some(docs) if docs.is_empty() => None,
            _ => docs,
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
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.value = self.value.resolve_namespace(ctx)?;
        Ok(())
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
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        if self.info.source.is_none() {
            return None;
        }
        let src: &Span = self.info.source.as_ref().unwrap();
        if !src.includes_pos(pos) {
            return None;
        }
        match &self.ty {
            Type::TyVar(_arc) => None,
            Type::TyCon(arc) => Some(EndNode::Type(arc.as_ref().clone())),
            Type::TyApp(arc, arc1) => {
                let node = arc.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                arc1.find_node_at(pos)
            }
            Type::FunTy(arc, arc1) => {
                let node = arc.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                arc1.find_node_at(pos)
            }
            Type::AssocTy(_ty_assoc, vec) => {
                for ty in vec {
                    let node = ty.find_node_at(pos);
                    if node.is_some() {
                        return node;
                    }
                }
                None
            }
        }
    }

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
            Type::AssocTy(_, _) => panic!(
                "Upto this function is called, all associated types should have been resolved."
            ),
        }
    }

    // Check if the given type variable appears in `self`.
    pub fn contains_tyvar(&self, tv: &TyVar) -> bool {
        match &self.ty {
            Type::TyVar(tv2) => tv.name == tv2.name, // Ignore kind.
            Type::TyCon(_) => false,
            Type::TyApp(fun, arg) => fun.contains_tyvar(tv) || arg.contains_tyvar(tv),
            Type::FunTy(src, dst) => src.contains_tyvar(tv) || dst.contains_tyvar(tv),
            Type::AssocTy(_, args) => {
                // NOTE: The special `self` type variable should be resolved in parser.
                for arg in args {
                    if arg.contains_tyvar(tv) {
                        return true;
                    }
                }
                return false;
            }
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
    pub fn set_kinds(self: &Arc<TypeNode>, tv_to_kind: &HashMap<Name, Arc<Kind>>) -> Arc<TypeNode> {
        match &self.ty {
            Type::TyVar(tv) => {
                if tv_to_kind.contains_key(&tv.name) {
                    self.set_tyvar_kind(tv_to_kind[&tv.name].clone())
                } else {
                    self.clone()
                }
            }
            Type::TyCon(_tc) => self.clone(),
            Type::TyApp(fun, arg) => self
                .set_tyapp_fun(fun.set_kinds(tv_to_kind))
                .set_tyapp_arg(arg.set_kinds(tv_to_kind)),
            Type::FunTy(src, dst) => self
                .set_funty_src(src.set_kinds(tv_to_kind))
                .set_funty_dst(dst.set_kinds(tv_to_kind)),
            Type::AssocTy(_, args) => {
                let args = args
                    .iter()
                    .map(|arg| arg.set_kinds(tv_to_kind))
                    .collect::<Vec<_>>();
                self.set_assocty_args(args)
            }
        }
    }

    // Is this type constructed from type constructor, not from associated types?
    pub fn is_assoc_ty_free(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => true,
            Type::TyCon(_) => true,
            Type::TyApp(head, _) => head.is_assoc_ty_free(),
            Type::FunTy(src, dst) => src.is_assoc_ty_free() && dst.is_assoc_ty_free(),
            Type::AssocTy(_, _) => false,
        }
    }

    // Is head of this type type constructor?
    fn is_head_tycon(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => false,
            Type::TyCon(_) => true,
            Type::TyApp(head, _) => head.is_head_tycon(),
            Type::FunTy(_, _) => true,
            Type::AssocTy(_, _) => false,
        }
    }

    // Is this type can be instance head of trait?
    pub fn is_implementable(&self) -> bool {
        self.is_head_tycon() && self.is_assoc_ty_free()
    }

    pub fn is_tyvar(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => true,
            _ => false,
        }
    }

    pub fn is_tycon(&self) -> bool {
        match &self.ty {
            Type::TyCon(_) => true,
            _ => false,
        }
    }

    pub fn as_tycon(&self) -> &TyCon {
        match &self.ty {
            Type::TyCon(tc) => tc,
            _ => panic!(),
        }
    }

    pub fn get_head_string(&self) -> String {
        match &self.ty {
            Type::TyVar(_) => self.to_string(),
            Type::TyCon(_) => self.to_string(),
            Type::TyApp(head, _) => head.get_head_string(),
            Type::FunTy(_, _) => "->".to_string(),
            Type::AssocTy(assoc_ty, _) => assoc_ty.name.to_string(),
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

    pub fn set_tyapp_fun(&self, fun: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyApp(_, arg) => ret.ty = Type::TyApp(fun, arg.clone()),
            _ => panic!(),
        }
        Arc::new(ret)
    }

    pub fn set_tyapp_arg(&self, arg: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyApp(fun, _) => ret.ty = Type::TyApp(fun.clone(), arg),
            _ => panic!(),
        }
        Arc::new(ret)
    }

    pub fn set_funty_src(&self, src: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::FunTy(_, dst) => ret.ty = Type::FunTy(src, dst.clone()),
            _ => panic!(),
        }
        Arc::new(ret)
    }

    pub fn set_assocty_name(&self, name: TyAssoc) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::AssocTy(_, args) => ret.ty = Type::AssocTy(name, args.clone()),
            _ => panic!(),
        }
        Arc::new(ret)
    }

    pub fn set_assocty_args(&self, args: Vec<Arc<TypeNode>>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::AssocTy(assoc_ty, _) => ret.ty = Type::AssocTy(assoc_ty.clone(), args),
            _ => panic!(),
        }
        Arc::new(ret)
    }

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

    // Resolve namespaces of tycons / type aliases / trait / trait aliases / associated types that appear in a type.
    // Also, replaces TyCon node to an AssocTy node if necessary.
    pub fn resolve_namespace(
        self: &Arc<TypeNode>,
        ctx: &NameResolutionContext,
    ) -> Result<Arc<TypeNode>, Errors> {
        match &self.ty {
            Type::TyVar(_tv) => Ok(self.clone()),
            Type::TyCon(tc) => {
                let mut tc = tc.as_ref().clone();
                tc.resolve_namespace(ctx, &self.info.source)?;
                Ok(self.set_tycon_tc(Arc::new(tc)))
            }
            Type::TyApp(fun, arg) => {
                let app_seq = self.flatten_type_application();
                match &app_seq[0].ty {
                    Type::TyCon(tc) => {
                        // In this case, replase self to associated type application if necessary.
                        let mut tc = tc.as_ref().clone();
                        tc.resolve_namespace(ctx, &app_seq[0].info.source)?;
                        if ctx.candidates[&tc.name] == NameResolutionType::AssocTy {
                            let assoc_ty_name = tc.name;
                            let arity: usize = ctx.assoc_ty_to_arity[&assoc_ty_name];
                            let (_, args) = app_seq.split_at(1);
                            if args.len() < arity {
                                return Err(Errors::from_msg_srcs(format!(
                                    "Associated type `{}` has arity {}, but supplied {} types. All appearance of associated type has to be saturated.",
                                    assoc_ty_name.to_string(),
                                    arity,
                                    args.len()
                                ), &[&app_seq[0].info.source]));
                            }
                            let (assoc_ty_args, following_args) = args.split_at(arity);
                            let assoc_ty_span = args[0].get_source().clone();
                            let mut assoc_ty = type_assocty(
                                TyAssoc {
                                    name: assoc_ty_name,
                                },
                                assoc_ty_args.iter().cloned().collect(),
                            )
                            .set_source(assoc_ty_span);
                            for arg in following_args {
                                let fun_src = assoc_ty.get_source();
                                let arg_src = arg.get_source();
                                let span = Span::unite_opt(fun_src, arg_src);
                                assoc_ty = type_tyapp(assoc_ty, arg.clone()).set_source(span);
                            }
                            return assoc_ty.resolve_namespace(ctx);
                        }
                    }
                    _ => {}
                }
                Ok(self
                    .set_tyapp_fun(fun.resolve_namespace(ctx)?)
                    .set_tyapp_arg(arg.resolve_namespace(ctx)?))
            }
            Type::FunTy(src, dst) => Ok(self
                .set_funty_src(src.resolve_namespace(ctx)?)
                .set_funty_dst(dst.resolve_namespace(ctx)?)),
            Type::AssocTy(assoc_ty, args) => {
                let mut assoc_ty = assoc_ty.clone();
                assoc_ty.resolve_namespace(ctx, &self.info.source)?;
                let mut res_args: Vec<Arc<TypeNode>> = vec![];
                for arg in args {
                    res_args.push(arg.resolve_namespace(ctx)?);
                }
                Ok(self.set_assocty_name(assoc_ty).set_assocty_args(res_args))
            }
        }
    }

    // Take a struct type, and convert it to a punched version.
    pub fn to_punched_struct(self: &Arc<TypeNode>, punched_at: usize) -> Arc<TypeNode> {
        let mut tycon = self.toplevel_tycon().unwrap().as_ref().clone();
        tycon.into_punched_type_name(punched_at);
        self.set_toplevel_tycon(Arc::new(tycon))
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

    // For structs and unions, return the fields.
    // For Array, return the element type.
    pub fn fields(&self, type_env: &TypeEnv) -> Vec<Field> {
        let args = self.collect_type_argments();
        let ti = self.toplevel_tycon_info(type_env);
        assert_eq!(args.len(), ti.tyvars.len());
        ti.fields
    }

    // Flatten type application.
    // ex. If given `f a b`, returns `vec![f, a, b]`.
    pub fn flatten_type_application(&self) -> Vec<Arc<TypeNode>> {
        fn flatten_type_application_inner(ty: &TypeNode, tys: &mut Vec<Arc<TypeNode>>) {
            match &ty.ty {
                Type::TyApp(fun, arg) => {
                    flatten_type_application_inner(fun, tys);
                    tys.push(arg.clone());
                }
                _ => {
                    assert!(tys.is_empty());
                    tys.push(Arc::new(ty.clone()));
                }
            }
        }

        let mut tys: Vec<Arc<TypeNode>> = vec![];
        flatten_type_application_inner(self, &mut tys);
        tys
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

    // Convert a type `A1 -> A2 -> ... -> An -> B` into `([A1, A2, ..., An], C)`.
    // - `vars_limit`: the maximum number of variables to be collected.
    pub fn collect_app_src(
        self: &Arc<TypeNode>,
        vars_limit: usize,
    ) -> (Vec<Arc<TypeNode>>, Arc<TypeNode>) {
        fn collect_app_src_inner(
            ty: &Arc<TypeNode>,
            vars: &mut Vec<Arc<TypeNode>>,
            vars_limit: usize,
        ) -> Arc<TypeNode> {
            match &ty.ty {
                Type::FunTy(var, val) => {
                    vars.push(var.clone());
                    if vars.len() >= vars_limit {
                        return val.clone();
                    }
                    return collect_app_src_inner(&val, vars, vars_limit);
                }
                _ => {
                    if ty.is_funptr() {
                        let mut vs = ty.get_lambda_srcs();
                        if vars.len() + vs.len() > vars_limit {
                            return ty.clone();
                        }
                        vars.append(&mut vs);
                        return collect_app_src_inner(&ty.get_lambda_dst(), vars, vars_limit);
                    } else {
                        ty.clone()
                    }
                }
            }
        }

        let mut vars: Vec<Arc<TypeNode>> = vec![];
        let val = collect_app_src_inner(self, &mut vars, vars_limit);
        (vars, val)
    }

    // Remove type aliases in a type.
    pub fn resolve_type_aliases(
        self: &Arc<TypeNode>,
        env: &TypeEnv,
    ) -> Result<Arc<TypeNode>, Errors> {
        let self_src = self.get_source().clone();
        self.resolve_type_aliases_inner(env, vec![], &self_src)
    }

    // Remove type aliases in a type.
    // * `type_name_path` - argument to detect circular aliasing.
    // * `entry_type` - argument to show good error message.
    fn resolve_type_aliases_inner(
        self: &Arc<TypeNode>,
        env: &TypeEnv,
        mut type_name_path: Vec<String>,
        entry_type_src: &Option<Span>,
    ) -> Result<Arc<TypeNode>, Errors> {
        // Check circular aliasing.
        let type_name = self.to_string_normalize();
        if type_name_path.contains(&type_name) {
            return Err(Errors::from_msg_srcs(
                format!("Circular type aliasing is found in `{}`.", type_name),
                &[entry_type_src],
            ));
        }
        type_name_path.push(type_name);

        // First, treat the case where top-level type constructor is a type alias.
        // As an example, consider type alias `type Lazy a = () -> a`. Then `Lazy Bool` should be resolved to `() -> Bool`.
        let app_seq = self.flatten_type_application();
        let toplevel_ty = &app_seq[0];
        match &toplevel_ty.ty {
            Type::TyCon(tc) => {
                if let Some(ta) = env.aliases.get(&tc) {
                    // When the type alias is not fully applied, raise error.
                    if app_seq.len() - 1 < ta.tyvars.len() {
                        return Err(Errors::from_msg_srcs(
                            format!(
                                "Cannot resolve type alias `{}` in `{}` because it is not fully applied.",
                                tc.to_string(),
                                self.to_string_normalize()
                            ),
                            &[toplevel_ty.get_source()],
                        ));
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
                    return resolved.resolve_type_aliases_inner(
                        env,
                        type_name_path,
                        entry_type_src,
                    );
                }
            }
            _ => {}
        }
        // Treat other cases.
        match &self.ty {
            Type::TyVar(_) => Ok(self.clone()),
            Type::FunTy(dom_ty, codom_ty) => Ok(self
                .set_funty_src(dom_ty.resolve_type_aliases_inner(
                    env,
                    type_name_path.clone(),
                    entry_type_src,
                )?)
                .set_funty_dst(codom_ty.resolve_type_aliases_inner(
                    env,
                    type_name_path,
                    entry_type_src,
                )?)),
            Type::TyCon(_) => Ok(self.clone()),
            Type::TyApp(fun_ty, arg_ty) => Ok(self
                .set_tyapp_fun(fun_ty.resolve_type_aliases_inner(
                    env,
                    type_name_path.clone(),
                    entry_type_src,
                )?)
                .set_tyapp_arg(arg_ty.resolve_type_aliases_inner(
                    env,
                    type_name_path,
                    entry_type_src,
                )?)),
            Type::AssocTy(_, args) => {
                let args = collect_results(args.iter().map(|arg| {
                    arg.resolve_type_aliases_inner(env, type_name_path.clone(), entry_type_src)
                }))?;
                Ok(self.set_assocty_args(args))
            }
        }
    }

    // Is this type head normal form? i.e., begins with type variable.
    pub fn is_funty(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => false,
            Type::TyCon(_) => false,
            Type::TyApp(_, _) => false,
            Type::FunTy(_, _) => true,
            Type::AssocTy(_, _) => false,
        }
    }

    // Get top-level type constructor of a type.
    pub fn toplevel_tycon(&self) -> Option<Arc<TyCon>> {
        match &self.ty {
            Type::TyVar(_) => None,
            Type::TyCon(tc) => Some(tc.clone()),
            Type::TyApp(fun, _) => fun.toplevel_tycon(),
            Type::FunTy(_, _) => None,
            Type::AssocTy(_, _) => None,
        }
    }

    // Update top-level type constructor of a type.
    pub fn set_toplevel_tycon(&self, tycon: Arc<TyCon>) -> Arc<TypeNode> {
        match &self.ty {
            Type::TyVar(_) => {
                panic!("`set_toplevel_tycon` reached to a type variable.")
            }
            Type::TyCon(_) => type_tycon(&tycon),
            Type::TyApp(fun, arg) => type_tyapp(fun.set_toplevel_tycon(tycon), arg.clone()),
            Type::FunTy(_, _) => {
                panic!("`set_toplevel_tycon` reached to a function type.")
            }
            Type::AssocTy(_, _) => {
                panic!("`set_toplevel_tycon` reached to an associated type application.")
            }
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
    pub fn kind(self: &Arc<TypeNode>, kind_env: &KindEnv) -> Result<Arc<Kind>, Errors> {
        match &self.ty {
            Type::TyVar(tv) => Ok(tv.kind.clone()),
            Type::TyCon(tc) => Ok(kind_env.tycons.get(&tc).unwrap().clone()),
            Type::TyApp(fun, arg) => {
                let fun_kind = fun.kind(kind_env)?;
                let arg_kind = arg.kind(kind_env)?;
                match &*fun_kind {
                    Kind::Arrow(arg2, res) => {
                        if arg_kind != *arg2 {
                            let type_strs = TypeNode::to_string_normalize_many(&[
                                self.clone(),
                                fun.clone(),
                                arg.clone(),
                            ]);
                            let self_str = &type_strs[0];
                            let fun_str = &type_strs[1];
                            let arg_str = &type_strs[2];
                            return Err(Errors::from_msg_srcs(
                                format!(
                                    "Kind mismatch in `{}`. Type `{}` of kind `{}` cannot be applied to type `{}` of kind `{}`.",
                                    self_str,
                                    fun_str,
                                    fun_kind.to_string(),
                                    arg_str,
                                    arg_kind.to_string()
                                ),
                                &[self.get_source()],
                            ));
                        }
                        Ok(res.clone())
                    }
                    Kind::Star => {
                        let type_strs = TypeNode::to_string_normalize_many(&[
                            self.clone(),
                            fun.clone(),
                            arg.clone(),
                        ]);
                        let self_str = &type_strs[0];
                        let fun_str = &type_strs[1];
                        let arg_str = &type_strs[2];
                        return Err(Errors::from_msg_srcs(
                            format!(
                                "Kind mismatch in `{}`. Type `{}` of kind `{}` cannot be applied to type `{}` of kind `{}`.",
                                self_str,
                                fun_str,
                                fun_kind.to_string(),
                                arg_str,
                                arg_kind.to_string()
                            ),
                            &[self.get_source()],
                        ));
                    }
                }
            }
            Type::FunTy(dom, codom) => {
                let arg_kind = dom.kind(kind_env)?;
                if arg_kind != kind_star() {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Cannot form function type `{}` since its domain type `{}` has kind `{}`.",
                            self.to_string_normalize(),
                            dom.to_string_normalize(),
                            arg_kind.to_string()
                        ),
                        &[self.get_source()],
                    ));
                }
                let ret_kind = codom.kind(kind_env)?;
                if ret_kind != kind_star() {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Cannot form function type `{}` since its codomain type `{}` has kind `{}`.",
                            self.to_string_normalize(),
                            codom.to_string_normalize(),
                            ret_kind.to_string()
                        ),
                        &[self.get_source()],
                    ));
                }
                Ok(kind_star())
            }
            Type::AssocTy(assoc_ty, args) => {
                let kind_info = kind_env.assoc_tys.get(&assoc_ty).unwrap().clone();
                assert_eq!(kind_info.param_kinds.len(), args.len());
                for i in 0..args.len() {
                    let expected = &kind_info.param_kinds[i];
                    let found = args[i].kind(kind_env)?;
                    if *expected != found {
                        return Err(Errors::from_msg_srcs(
                            format!(
                                "Kind mismatch in `{}`. Expected `{}`, found `{}`.",
                                self.to_string_normalize(),
                                expected.to_string(),
                                found.to_string()
                            ),
                            &[args[i].get_source()],
                        ));
                    }
                }
                Ok(kind_info.value_kind.clone())
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
    // Definition of an associated type has to be of the form `type AssocTypeName ty1 tv2 ... tvN`,
    // - where `{AssocTypeName}` is a local name,
    // - `ty1` is equal to the implemented type.
    // - type variables appears in the arguments are distinct.
    // If ok, return the name and the array `[tv1, tv2, ..., tvN]`, where tv1 is a special type variable `%impl_type`.
    pub fn validate_as_associated_type_defn(
        &self,
        impl_type: &Arc<TypeNode>,
        src_for_err: &Option<Span>,
        err_msg_for_impl: bool,
    ) -> (Name, Vec<Arc<TyVar>>) {
        fn general_err(
            for_implememtation: bool,
            imple_type: &Arc<TypeNode>,
            src_for_err: &Option<Span>,
        ) -> ! {
            if for_implememtation {
                error_exit_with_src(
                    &format!("The implementation of an associated type should be in the form `type {{AssocTyName}} {{impl_type}} {{type_var1}} ... {{type_varN}} = {{value_type}};`, where {{impl_type}} is `{}` here.", imple_type.to_string()),
                    src_for_err,
                );
            } else {
                // for definition
                error_exit_with_src(
                    &format!("The definition of an associated type should be in the form `type {{AssocTyName}} {{impl_type}} {{type_var1}} ... {{type_varN}};`, where {{impl_type}} is `{}` here.", imple_type.to_string()),
                    src_for_err,
                );
            }
        }
        // Validate the type application sequence.
        let app_seq = self.flatten_type_application();
        if app_seq.len() < 2 {
            general_err(err_msg_for_impl, impl_type, src_for_err);
        }
        let assoc_type_name: Name;
        match &app_seq[0].ty {
            Type::TyCon(tc) => {
                if !tc.name.is_local() {
                    error_exit_with_src(
                        "Do not specify namespace of the associated type here; the namespace of an associated type is determined by the trait name.",
                        src_for_err,
                    );
                }
                assoc_type_name = tc.name.to_string();
            }
            _ => {
                general_err(err_msg_for_impl, impl_type, src_for_err);
            }
        }
        if app_seq[1].to_string() != impl_type.to_string() {
            general_err(err_msg_for_impl, impl_type, src_for_err);
        }
        let mut tyvars = vec![tyvar_from_name("%impl_type", &kind_star())];
        let impl_ty_tyvar_set: HashSet<Name> = impl_type
            .free_vars_vec()
            .iter()
            .map(|tv| tv.name.clone())
            .collect();
        let mut tyvars_set: HashSet<Name> = HashSet::default();
        for i in 2..app_seq.len() {
            match &app_seq[i].ty {
                Type::TyVar(tv) => {
                    if impl_ty_tyvar_set.contains(&tv.name) {
                        if err_msg_for_impl {
                            error_exit_with_src(&format!("In associated type implementation, each type variable should be free from the implemented type (`{}` here).", impl_type.to_string()), src_for_err);
                        } else {
                            error_exit_with_src(&format!("In associated type definition, each type variable should be free from the implemented type (`{}` here).", impl_type.to_string()), src_for_err);
                        }
                    }
                    if tyvars_set.contains(&tv.name) {
                        if err_msg_for_impl {
                            error_exit_with_src("In associated type implementation, each type variable should be different.", src_for_err);
                        } else {
                            error_exit_with_src("In associated type implementation, each type variable should be different.", src_for_err);
                        }
                    }
                    tyvars.push(tv.clone());
                    tyvars_set.insert(tv.name.clone());
                }
                _ => {
                    general_err(err_msg_for_impl, impl_type, src_for_err);
                }
            }
        }
        (assoc_type_name, tyvars)
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
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum Type {
    TyVar(Arc<TyVar>),
    TyCon(Arc<TyCon>),
    TyApp(Arc<TypeNode>, Arc<TypeNode>),
    FunTy(Arc<TypeNode>, Arc<TypeNode>),
    AssocTy(TyAssoc, Vec<Arc<TypeNode>>),
}

impl TypeNode {
    // Stringify a type.
    // Name of type variables are normalized to names such as "t0", "t1", etc.
    pub fn to_string_normalize(self: &Arc<TypeNode>) -> String {
        TypeNode::to_string_normalize_many(&[self.clone()])
            .pop()
            .unwrap()
    }

    // Stringify many types in a consistent way.
    // Name of type variables are normalized to names such as "t0", "t1", etc.
    pub fn to_string_normalize_many(tys: &[Arc<TypeNode>]) -> Vec<String> {
        // Collect free variables keeping the order of appearance.
        let mut free_vars = vec![];
        for ty in tys {
            ty.free_vars_to_vec(&mut free_vars);
        }

        // Create substitution that normalizes the names of type variables.
        let mut s = Substitution::default();
        let mut next_tyvar_no = 0;
        let mut appeared: HashSet<Name> = HashSet::default();
        for fv in free_vars {
            if appeared.contains(&fv.name) {
                continue;
            }
            appeared.insert(fv.name.clone());
            let new_name = number_to_varname(next_tyvar_no);
            s.merge_substitution(&Substitution::single(
                &fv.name,
                type_tyvar(&new_name, &fv.kind),
            ));
            next_tyvar_no += 1;
        }

        // Substitute and stringify all types.
        tys.iter()
            .map(|ty| s.substitute_type(ty).to_string())
            .collect()
    }

    // Stringify.
    pub fn to_string(&self) -> String {
        fn brace_needed_if_used_as_arg(arg: &Arc<TypeNode>) -> bool {
            match &arg.ty {
                Type::TyVar(_) => false,
                Type::TyCon(_) => false,
                Type::TyApp(fun, _) => {
                    let tycon = fun.toplevel_tycon();
                    let is_tuple = tycon.is_some() && get_tuple_n(&tycon.unwrap().name).is_some();
                    !is_tuple
                }
                Type::FunTy(_, _) => true,
                Type::AssocTy(_, _) => true,
            }
        }

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

                            // We cannot assume here even `args.len() <= n`
                            // because this function is used for generating error messages where the user apply too many arguments to a type constructor!
                            // assert!(args.len() <= n as usize);

                            // If args.len() < n, then `self` is a partial application to a tuple.
                            // In this case, we show missing arguments by `*` (e.g., `(Std::I64, *)`).
                            for _ in args.len()..n as usize {
                                arg_strs.push(kind_star().to_string());
                            }
                            if n == 1 {
                                return format!("({},)", arg_strs[0]);
                            } else {
                                return format!("({})", arg_strs.join(", "));
                            }
                        }
                        None => {}
                    }
                }
                let tyfun = fun.to_string();
                let arg_str = arg.to_string();
                if brace_needed_if_used_as_arg(arg) {
                    format!("{} ({})", tyfun, arg_str)
                } else {
                    format!("{} {}", tyfun, arg_str)
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
            Type::AssocTy(assoc_ty, args) => {
                format!(
                    "{} {}",
                    assoc_ty.name.to_string(),
                    args.iter()
                        .map(|arg| {
                            let arg_str = arg.to_string();
                            if brace_needed_if_used_as_arg(arg) {
                                format!("({})", arg_str)
                            } else {
                                arg_str
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
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

    // See all associated type usages (e.g., `Elem c`) in this type and return the list of predicates required (e.g., `c : Collects`).
    pub fn predicates_from_associated_types(&self) -> Vec<Predicate> {
        fn predicates_from_associated_types_inner(ty: &TypeNode, buf: &mut Vec<Predicate>) {
            match &ty.ty {
                Type::TyVar(_) => {}
                Type::TyCon(_) => {}
                Type::TyApp(fun, arg) => {
                    predicates_from_associated_types_inner(fun, buf);
                    predicates_from_associated_types_inner(arg, buf);
                }
                Type::FunTy(src, dst) => {
                    predicates_from_associated_types_inner(src, buf);
                    predicates_from_associated_types_inner(dst, buf);
                }
                Type::AssocTy(assoc_ty, args) => {
                    let pred = Predicate {
                        trait_id: assoc_ty.trait_id(),
                        ty: args[0].clone(),
                        source: ty.get_source().clone(),
                    };
                    buf.push(pred);
                    for arg in args {
                        predicates_from_associated_types_inner(arg, buf);
                    }
                }
            }
        }
        let mut buf = vec![];
        predicates_from_associated_types_inner(self, &mut buf);
        buf
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

pub fn type_assocty(assoc_ty: TyAssoc, args: Vec<Arc<TypeNode>>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::AssocTy(assoc_ty, args))
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
    pub fn free_vars(self: &Arc<TypeNode>) -> HashMap<Name, Arc<TyVar>> {
        let mut free_vars: HashMap<String, Arc<TyVar>> = HashMap::default();
        match &self.ty {
            Type::TyVar(tv) => {
                free_vars.insert(tv.name.clone(), tv.clone());
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
            Type::AssocTy(_, args) => {
                for arg in args {
                    free_vars.extend(arg.free_vars());
                }
            }
        };
        free_vars
    }

    // Append free type variables to a buffer of type Vec.
    pub fn free_vars_to_vec(self: &Arc<TypeNode>, buf: &mut Vec<Arc<TyVar>>) {
        match &self.ty {
            Type::TyVar(tv) => {
                if buf.iter().any(|tv0| tv0.name == tv.name) {
                    return;
                }
                buf.push(tv.clone())
            }
            Type::TyApp(tyfun, arg) => {
                tyfun.free_vars_to_vec(buf);
                arg.free_vars_to_vec(buf);
            }
            Type::FunTy(input, output) => {
                input.free_vars_to_vec(buf);
                output.free_vars_to_vec(buf);
            }
            Type::TyCon(_) => {}
            Type::AssocTy(_, args) => {
                for arg in args {
                    arg.free_vars_to_vec(buf);
                }
            }
        };
    }

    pub fn free_vars_vec(self: &Arc<TypeNode>) -> Vec<Arc<TyVar>> {
        let mut buf = vec![];
        self.free_vars_to_vec(&mut buf);
        buf
    }
}

// Type scheme.
#[derive(Clone, Serialize, Deserialize)]
pub struct Scheme {
    // Generalized variables.
    pub gen_vars: Vec<Arc<TyVar>>,
    // Predicates
    pub predicates: Vec<Predicate>,
    // Equalities
    pub equalities: Vec<Equality>,
    // Generalized type.
    pub ty: Arc<TypeNode>,
}

impl Scheme {
    pub fn validate_constraints(&self, trait_env: &TraitEnv) -> Result<(), Errors> {
        // Validate constraints.
        // NOTE:
        // This validation is too restrictive.
        // We should allow more general constraints in a future by converting user-specified constraints to a form where the following restrictions are satisfied.
        for pred in &self.predicates {
            // Each predicate constraint should be in the form of `type_var : Trait`.
            // This ensures that the predicate constraint is on the "terminal" type, i.e., a type which cannot be reduced further.
            // For example, if a user is writing `Elem c = e, Elem c : ToString`, then the typechecker may fail to prove `Elem c : ToString`.
            // Writing `Elem c = e, e : ToString` instead is ok.
            if !pred.ty.is_tyvar() {
                return Err(Errors::from_msg_srcs(
                    "Trait constraint should be in the form of `{type_var} : {Trait}`. \
                     NOTE: If you want to put a constraint on an associated type application, e.g., `Elem c : ToString`, you should write `Elem c = e, e : ToString` instead. \
                     We will support more general constraints by implementing such conversion in a future.".to_string(),
                    &[&pred.source],
                ));
            }
        }
        let mut preds = vec![];
        for pred in &self.predicates {
            let mut pred = pred.resolve_trait_aliases(trait_env)?;
            preds.append(&mut pred);
        }
        for eq in &self.equalities {
            // Right hand side of an equality should be free from associated type.
            // This ensures that the reduction of a type terminates in a finite number of steps.
            if !eq.value.is_assoc_ty_free() {
                return Err(Errors::from_msg_srcs(
                    "Right side of an equality constraint cannot contain an associated type. \
                     NOTE: Instead of using associated type in the right side, e.g., `Elem c1 = Elem c2`, you can write `Elem c1 = e, Elem c2 = e`. \
                     We will support more general constraints by implementing such conversion in a future.".to_string(),
                    &[&eq.source],
                ));
            }
            // The first argument of the left side of an equality constraint should be a type variable.
            // If this condition is not satisified, then a type can be reduce in two ways, by this equality and by an instance of the associated type,
            // which implies that there is no "normal form" of the type.
            if !eq.args[0].is_tyvar() {
                return Err(Errors::from_msg_srcs(
                    "The first argument of the left side of an equality constraint should be a type variable.".to_string(),
                    &[&eq.source],
                ));
            }
            // The left side of an equality constraint should be free from associated type.
            // This ensures that this equality constraint can be applied without reducing the left side of the equality.
            for arg in &eq.args[1..] {
                if !arg.is_assoc_ty_free() {
                    return Err(Errors::from_msg_srcs(
                        "In left side of an equality constraint, arguments of an associated type cannot contain an associated type. \
                         NOTE: Instead of using associated type in the argument, e.g., `Elem (Elem c) = I64`, you can write `Elem c = e, Elem e = I64`. \
                         We will support more general constraints by implementing such conversion in a future.".to_string(),
                        &[&eq.source],
                    ));
                }
            }
            // For each associated type usage, e.g., `Elem c = I64`, we check that `c : Collects` is in the constraint.
            let mut ok = false;
            for pred in &preds {
                if pred.trait_id != eq.assoc_type.trait_id() {
                    continue;
                }
                if pred.ty.to_string() != eq.args[0].to_string() {
                    continue;
                }
                ok = true;
                break;
            }
            if !ok {
                let pred = Predicate {
                    trait_id: eq.assoc_type.trait_id(),
                    ty: eq.args[0].clone(),
                    source: None,
                };
                return Err(Errors::from_msg_srcs(
                    format!(
                        "The equality constraint `{}` is invalid here because `{}` is not assumed.",
                        eq.to_string(),
                        pred.to_string()
                    ),
                    &[&eq.source],
                ));
            }
        }
        // We do not allow there are two equality constraints with the same left side.
        for i in 0..self.equalities.len() {
            for j in i + 1..self.equalities.len() {
                if self.equalities[i].lhs().to_string() == self.equalities[j].lhs().to_string() {
                    return Err(Errors::from_msg_srcs(
                        "Multiple equality constraints with the same left side are not allowed."
                            .to_string(),
                        &[&self.equalities[i].source, &self.equalities[j].source],
                    ));
                }
            }
        }
        Ok(())
    }

    fn to_string_substituted(&self, s: &Substitution) -> String {
        // Substitute type variables in predicates, equalities and the type to chosen names.
        let preds = self
            .predicates
            .iter()
            .map(|p| {
                let mut p = p.clone();
                s.substitute_predicate(&mut p);
                p
            })
            .collect::<Vec<_>>();
        let eqs = self
            .equalities
            .iter()
            .map(|eq| {
                let mut eq = eq.clone();
                s.substitute_equality(&mut eq);
                eq
            })
            .collect::<Vec<_>>();
        let ty = s.substitute_type(&self.ty);

        // Stringify.
        let constraints_str = if preds.is_empty() && eqs.is_empty() {
            "".to_string()
        } else {
            let mut constraint_strs = vec![];
            for p in &preds {
                constraint_strs.push(p.to_string());
            }
            for eq in &eqs {
                constraint_strs.push(eq.to_string());
            }
            format!("[{}] ", constraint_strs.join(", "))
        };
        constraints_str + &ty.to_string()
    }

    pub fn to_string_normalize(&self) -> String {
        // Change names of generalized type variables to a, b, ...
        let mut s = Substitution::default();
        let mut tyvar_num = -1;
        for tyvar in &self.gen_vars {
            tyvar_num += 1;
            let new_name = number_to_varname(tyvar_num as usize);
            s.merge_substitution(&Substitution::single(
                &tyvar.name,
                type_tyvar(&new_name, &tyvar.kind.clone()),
            ));
        }
        self.to_string_substituted(&s)
    }

    pub fn to_string(&self) -> String {
        let s = Substitution::default();
        self.to_string_substituted(&s)
    }

    pub fn set_kinds(&self, kind_env: &KindEnv) -> Result<Arc<Scheme>, Errors> {
        let mut ret = self.clone();
        let mut scope: HashMap<Name, Arc<Kind>> = Default::default();
        // If a kind in `self.vars` is not `*`, then the kind is explicitly specified by user, so we insert it into `scope`.
        for tv in &self.gen_vars {
            if tv.kind != kind_star() {
                scope.insert(tv.name.clone(), tv.kind.clone());
            }
        }
        let res = QualPredicate::extend_kind_scope(
            &mut scope,
            &ret.predicates,
            &ret.equalities,
            &vec![],
            kind_env,
        );
        if let Err(msg) = res {
            let mut span = ret.predicates[0].source.clone();
            for i in 1..ret.predicates.len() {
                span = Span::unite_opt(&span, &ret.predicates[i].source);
            }
            return Err(Errors::from_msg_srcs(msg, &[&span]));
        }
        for p in &mut ret.predicates {
            p.set_kinds(&scope);
        }
        for eq in &mut ret.equalities {
            eq.set_kinds(&scope);
        }
        ret.ty = ret.ty.set_kinds(&scope);
        for tv in &mut ret.gen_vars {
            if scope.contains_key(&tv.name) {
                *tv = tv.set_kind(scope[&tv.name].clone());
            }
        }
        Ok(Arc::new(ret))
    }

    pub fn check_kinds(&self, kind_env: &KindEnv) -> Result<(), Errors> {
        for p in &self.predicates {
            p.check_kinds(kind_env)?;
        }
        for eq in &self.equalities {
            eq.check_kinds(kind_env)?;
        }
        self.ty.kind(kind_env)?;
        Ok(())
    }

    // Create new instance.
    pub fn new_arc(
        vars: Vec<Arc<TyVar>>,
        preds: Vec<Predicate>,
        eqs: Vec<Equality>,
        ty: Arc<TypeNode>,
    ) -> Arc<Scheme> {
        Arc::new(Scheme {
            gen_vars: vars,
            predicates: preds,
            equalities: eqs,
            ty,
        })
    }

    // Create instance by generalizaing type.
    pub fn generalize(
        kind_signs: &[KindSignature],
        preds: Vec<Predicate>,
        eqs: Vec<Equality>,
        ty: Arc<TypeNode>,
    ) -> Arc<Scheme> {
        let mut vars = vec![];
        for pred in &preds {
            pred.free_vars_to_vec(&mut vars);
        }
        for eq in &eqs {
            eq.free_vars_to_vec(&mut vars);
        }
        ty.free_vars_to_vec(&mut vars);
        for tv in &mut vars {
            for kind_sign in kind_signs {
                if tv.name == kind_sign.tyvar {
                    *tv = tv.set_kind(kind_sign.kind.clone());
                }
            }
        }
        Scheme::new_arc(vars, preds, eqs, ty)
    }

    pub fn from_type(ty: Arc<TypeNode>) -> Arc<Scheme> {
        Scheme::new_arc(vec![], vec![], vec![], ty)
    }

    pub fn resolve_namespace(&self, ctx: &NameResolutionContext) -> Result<Arc<Scheme>, Errors> {
        let mut res = self.clone();
        for p in &mut res.predicates {
            p.resolve_namespace(ctx)?;
        }
        for eq in &mut res.equalities {
            eq.resolve_namespace(ctx)?;
        }
        res.ty = res.ty.resolve_namespace(ctx)?;
        Ok(Arc::new(res))
    }

    pub fn resolve_type_aliases(&self, type_env: &TypeEnv) -> Result<Arc<Scheme>, Errors> {
        let mut res = self.clone();
        for p in &mut res.predicates {
            p.resolve_type_aliases(type_env)?;
        }
        for eq in &mut res.equalities {
            eq.resolve_type_aliases(type_env)?;
        }
        res.ty = res.ty.resolve_type_aliases(type_env)?;
        Ok(Arc::new(res))
    }

    // Find the minimum expression node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        for p in &self.predicates {
            let node = p.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        for eq in &self.equalities {
            let node = eq.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        self.ty.find_node_at(pos)
    }
}

#[derive(Default, Clone)]
pub struct KindEnv {
    pub tycons: HashMap<TyCon, Arc<Kind>>,
    pub assoc_tys: HashMap<TyAssoc, AssocTypeKindInfo>,
    pub traits_and_aliases: HashMap<Trait, Arc<Kind>>,
}
