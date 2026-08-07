use crate::ast::equality::Equality;
use crate::ast::kind_scope::{KindEnv, KindScope};
use crate::ast::name::FullName;
use crate::ast::name::Name;
use crate::ast::name::NameSpace;
use crate::ast::predicate::Predicate;
use crate::ast::program::{EndNode, TypeEnv};
use crate::ast::traits::{KindSignature, TraitEnv, TraitId};
use crate::ast::typedecl::Field;
use crate::constants::{
    TraverserWorkType, BOOL_NAME, F32_NAME, F64_NAME, I16_NAME, I32_NAME, I64_NAME, I8_NAME,
    PTR_NAME, PUNCHED_TYPE_SYMBOL, STD_NAME, TRAVERSER_WORK_MARK_GLOBAL,
    TRAVERSER_WORK_MARK_THREADED, TRAVERSER_WORK_RELEASE, TYPE_WILDCARD_VAR_PREFIX, U16_NAME,
    U32_NAME, U64_NAME, U8_NAME,
};
use crate::elaboration::name_resolution::{NameResolutionContext, NameResolutionType};
use crate::elaboration::typecheck::{Substitution, TypeCheckContext};
use crate::error::Errors;
use crate::fixstd::builtin::{
    get_tuple_n, is_array_storage_tycon, is_array_tycon, is_destructor_object_tycon,
    is_dynamic_object_tycon, is_funptr_tycon, is_punched_array_tycon, make_array_tycon,
    make_arrow_name_abs, make_arrow_tycon, make_funptr_tycon, make_io_tycon, make_iostate_name,
    make_tuple_name_abs,
};
use crate::generator::Generator;
use crate::misc::collect_results;
use crate::misc::number_to_varname;
use crate::misc::Map;
use crate::misc::Set;
use crate::object::{ty_to_object_ty, ObjectType};
use crate::parse::sourcefile::{SourcePos, Span};
use crate::rc_ir::ast::RcState;
use core::panic;
use inkwell::context::Context;
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::AddressSpace;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct TyVar {
    pub name: Name,
    pub kind: Arc<Kind>,
}

impl PartialEq for TyVar {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.kind == other.kind
    }
}

impl Eq for TyVar {}

impl Hash for TyVar {
    /// Hashes the name alone. The kind is an attribute of a variable rather than part of which
    /// variable it is, so two variables of one name are one variable whatever kinds they carry --
    /// a shape a well-formed program does not produce, and one a hash should not distinguish.
    /// Leaving the kind out also keeps this consistent with an equality that stopped reading it.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl TyVar {
    /// A copy of this type variable carrying `kind`, leaving this one as it is.
    pub fn set_kind(&self, kind: Arc<Kind>) -> Arc<TyVar> {
        let mut ret = self.clone();
        ret.kind = kind;
        Arc::new(ret)
    }

    pub fn set_name(&self, name: Name) -> Arc<TyVar> {
        let mut ret = self.clone();
        ret.name = name;
        Arc::new(ret)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AssocType {
    pub name: FullName,
    // Source span of the associated type name (e.g., `Item` in `Item iter`).
    // Ignored in PartialEq, Eq, and Hash.
    pub src: Option<Span>,
}

impl PartialEq for AssocType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for AssocType {}

impl Hash for AssocType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl AssocType {
    pub fn resolve_namespace(
        &mut self,
        ctx: &mut NameResolutionContext,
        span: &Option<Span>,
    ) -> Result<(), Errors> {
        self.name = ctx.resolve(&self.name, &[NameResolutionType::AssocTy], span)?;
        Ok(())
    }

    pub fn trait_id(&self) -> TraitId {
        let mut namespace = self.name.namespace.names.clone();
        let name = namespace.pop().unwrap();
        TraitId {
            name: FullName::new(&NameSpace::new(namespace), &name),
        }
    }

    // Convert global FullName to absolute path.
    pub fn global_to_absolute(&self) -> AssocType {
        let mut name = self.name.clone();
        name.global_to_absolute();
        AssocType {
            name,
            src: self.src.clone(),
        }
    }
}

/// The kind of a type, which classifies types the way a type classifies values.
#[derive(Eq, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    /// `*`, the kind of a type that has values of its own.
    Star,
    /// `k -> l`, the kind of a type constructor that yields a type of kind `l` when applied to a
    /// type of kind `k`.
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
    Arrow,
    Array,
    Struct,
    Union,
    // Dynamic object is nullble and has the destructor as the first field.
    DynamicObject,
    // The internal `#ArrayStorage` object: a control block and a raw element buffer, holding an
    // array's elements. Boxed; its element lifetime is driven by the owning `Array` value, not by
    // its own traverser.
    ArrayStorage,
    // Opaque type generated from opaque type variable `?it`.
    Opaque,
}

// The names, in the `Std` namespace, of the types that cross to C as a single scalar value.
// The names `CTypeSizes::get_c_types` builds for the C numeric type aliases must all appear here.
const C_SCALAR_NAMES: &[&str] = &[
    I8_NAME, U8_NAME, I16_NAME, U16_NAME, I32_NAME, U32_NAME, I64_NAME, U64_NAME, F32_NAME,
    F64_NAME, PTR_NAME,
];

// A type constructor, such as `Std::I64` or `Std::Array`, before any type argument is applied to
// it. A type constructor is determined by its name.
#[derive(Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct TyCon {
    pub name: FullName,
}

impl TyCon {
    pub fn new(fullname: FullName) -> TyCon {
        TyCon { name: fullname }
    }

    pub fn to_string(&self) -> String {
        if let Some(n) = get_tuple_n(&self.name) {
            if n == 0 {
                return "()".to_string();
            }
        }
        self.name.to_string()
    }

    pub fn resolve_namespace(
        &mut self,
        ctx: &mut NameResolutionContext,
        span: &Option<Span>,
    ) -> Result<(), Errors> {
        self.name = ctx.resolve(
            &self.name,
            &[NameResolutionType::TyCon, NameResolutionType::AssocTy],
            span,
        )?;
        Ok(())
    }

    // Convert all global FullNames to absolute paths.
    pub fn global_to_absolute(&self) -> Arc<Self> {
        let mut ret = self.clone();
        ret.name.global_to_absolute();
        Arc::new(ret)
    }

    // Get the type of struct / union value.
    // If struct / union have type parameter, introduces new type arguments.
    pub fn get_struct_union_value_type(
        self: &TyCon,
        typechecker: &mut TypeCheckContext,
    ) -> Arc<TypeNode> {
        let ti = typechecker.type_env.tycons.get(self).unwrap();
        assert!(ti.variant == TyConVariant::Struct || ti.variant == TyConVariant::Union);

        // Make type variables for type parameters.
        let mut new_tyvars: Vec<Arc<TypeNode>> = vec![];
        for tv in ti.tyvars.clone() {
            let tv = typechecker.new_tyvar_by(&tv);
            new_tyvars.push(type_from_tyvar(tv));
        }

        // Make type.
        let mut ty = type_tycon(&Arc::new(self.clone()));
        for tv in new_tyvars {
            ty = type_tyapp(ty, tv);
        }
        ty
    }

    // Whether this is the unit type `()`, i.e. the tuple of no element.
    pub fn is_unit(self: &TyCon) -> bool {
        self.name == make_tuple_name_abs(0)
    }

    // Whether a value of this type crosses to C as one scalar: an integer, a floating point
    // number, or a pointer, which C and Fix lay down the same way. These are the types a C
    // function signature can name, and the types an exported Fix function can exchange.
    pub fn is_c_scalar(self: &TyCon) -> bool {
        self.name.namespace == NameSpace::new_str(&[STD_NAME])
            && C_SCALAR_NAMES.contains(&self.name.name.as_str())
    }

    // Convert `()`, `I8`, `Ptr`, etc. to the corresponding C type.
    // `()` is C's `void`, which carries no value, so it maps to `None`.
    pub fn get_c_type<'c>(self: &TyCon, ctx: &'c Context) -> Option<BasicTypeEnum<'c>> {
        if self.is_unit() {
            return None;
        }
        assert!(
            self.is_c_scalar(),
            "call get_c_type for {}",
            self.to_string()
        );
        Some(match self.name.name.as_str() {
            I8_NAME | U8_NAME => ctx.i8_type().as_basic_type_enum(),
            I16_NAME | U16_NAME => ctx.i16_type().as_basic_type_enum(),
            I32_NAME | U32_NAME => ctx.i32_type().as_basic_type_enum(),
            I64_NAME | U64_NAME => ctx.i64_type().as_basic_type_enum(),
            F32_NAME => ctx.f32_type().as_basic_type_enum(),
            F64_NAME => ctx.f64_type().as_basic_type_enum(),
            PTR_NAME => ctx.ptr_type(AddressSpace::from(0)).as_basic_type_enum(),
            // `C_SCALAR_NAMES` gained a name that this mapping does not cover.
            name => unreachable!("no C type for `{}`", name),
        })
    }

    // Whether this is an integer type that carries a sign. Panics for a type that is not an
    // integer type of `Std`.
    // Whether a value of this type occupies fewer bits than the 32-bit unit a C signature extends
    // narrow integers to. Such a value travels in the low bits of a register, and the ABI decides
    // which side of the call extends it; a wider type fills the register and needs no extension.
    //
    // The 32-bit threshold holds for the targets Fix builds for. An ABI that extends a 32-bit
    // integer to the width of a register — RISC-V 64 does — widens this set.
    pub fn is_narrow_c_integer(self: &TyCon) -> bool {
        self.name.namespace == NameSpace::new_str(&[STD_NAME])
            && matches!(
                self.name.name.as_str(),
                I8_NAME | U8_NAME | I16_NAME | U16_NAME
            )
    }

    pub fn is_signed_integer(self: &TyCon) -> bool {
        if self.name.namespace != NameSpace::new_str(&[STD_NAME]) {
            panic!("call is_signed_integer for {}", self.to_string())
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

    // Whether this is the type `Bool` of `Std`.
    pub fn is_boolean(&self) -> bool {
        return self.name == FullName::from_strs(&[STD_NAME], BOOL_NAME);
    }

    // Whether this is the type constructor `IO`.
    pub fn is_io(&self) -> bool {
        self == make_io_tycon().as_ref()
    }

    // Whether this is the type `IOState`, the token that an `IO` action threads.
    #[allow(dead_code)]
    pub fn is_iostate(&self) -> bool {
        return self.name == make_iostate_name();
    }

    pub fn into_punched_type_name(&mut self, punched_at: usize) {
        self.name.name += &format!("{}{}", PUNCHED_TYPE_SYMBOL, punched_at);
    }

    #[allow(dead_code)]
    pub fn is_arrow(&self) -> bool {
        self == &make_arrow_tycon()
    }

    #[allow(dead_code)]
    pub fn is_array(&self) -> bool {
        self == &make_array_tycon()
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
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
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
    // Get the document of this type alias.
    pub fn get_document(&self) -> Option<String> {
        self.source.as_ref().and_then(|src| src.get_document().ok())
    }

    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
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

impl Hash for TypeNode {
    /// Hashes the type expression, which is what `PartialEq` compares; the source information the
    /// node carries stays out of both.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ty.hash(state);
    }
}

impl Debug for TypeNode {
    /// Writes the type in source syntax, with its free type variables renamed to `t0`, `t1`, ... in
    /// order of appearance, so that two types differing only in variable names print alike.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
        if !src.includes_pos_lsp(pos) {
            return None;
        }
        match &self.ty {
            Type::TyVar(_arc) => None,
            Type::TyCon(arc) => Some(EndNode::Type(arc.as_ref().clone())),
            Type::TyApp(func, arg) => {
                // Check the argument first, then the function.
                // This prioritizes inner/more specific nodes over outer/wider nodes,
                // which matters for synthetic TyCon nodes (like Tuple2) whose span
                // covers the entire expression including all arguments.
                let node = arg.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                func.find_node_at(pos)
            }
            Type::AssocTy(ty_assoc, vec) => {
                for ty in vec {
                    let node = ty.find_node_at(pos);
                    if node.is_some() {
                        return node;
                    }
                }
                // If cursor is on the associated type name itself, return AssocType.
                if let Some(src) = &ty_assoc.src {
                    if src.includes_pos_lsp(pos) {
                        return Some(EndNode::AssocType(ty_assoc.clone()));
                    }
                }
                None
            }
        }
    }

    // Locate a `_` type wildcard at `pos` and return the type it was inferred to.
    //
    // `self` is the syntactic annotation (wildcards still present as
    // `TYPE_WILDCARD_VAR_PREFIX` type variables, carrying the `_`'s source span);
    // `resolved` is the same annotation after type inference, so the two trees
    // have the same shape with each wildcard replaced by its inferred type. The
    // walk descends both in lockstep and, on reaching the hovered wildcard, returns
    // the matching node from `resolved`. A structural mismatch (possible when
    // associated-type reduction reshaped `resolved`) yields `None`.
    pub fn find_wildcard_inferred_type(
        self: &Arc<TypeNode>,
        resolved: &Arc<TypeNode>,
        pos: &SourcePos,
    ) -> Option<EndNode> {
        let src = self.info.source.as_ref()?;
        if !src.includes_pos_lsp(pos) {
            return None;
        }
        if let Type::TyVar(tv) = &self.ty {
            if is_type_wildcard_tyvar(&tv.name) {
                return Some(EndNode::InferredType(resolved.clone()));
            }
            return None;
        }
        match (&self.ty, &resolved.ty) {
            (Type::TyApp(sfun, sarg), Type::TyApp(rfun, rarg)) => sarg
                .find_wildcard_inferred_type(rarg, pos)
                .or_else(|| sfun.find_wildcard_inferred_type(rfun, pos)),
            (Type::AssocTy(_, sargs), Type::AssocTy(_, rargs)) if sargs.len() == rargs.len() => {
                sargs
                    .iter()
                    .zip(rargs)
                    .find_map(|(s, r)| s.find_wildcard_inferred_type(r, pos))
            }
            _ => None,
        }
    }

    // The set of defining modules of type constructors that appear in this type.
    pub fn define_modules_of_tycons(&self, out_set: &mut Set<Name>) {
        match &self.ty {
            Type::TyVar(_) => {}
            Type::TyCon(tc) => {
                out_set.insert(tc.name.module());
            }
            Type::TyApp(fun, arg) => {
                fun.define_modules_of_tycons(out_set);
                arg.define_modules_of_tycons(out_set);
            }
            Type::AssocTy(_, _) => panic!(
                "Upto this function is called, all associated types should have been resolved."
            ),
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
    pub fn set_kinds(self: &Arc<TypeNode>, scope: &KindScope) -> Arc<TypeNode> {
        match &self.ty {
            Type::TyVar(tv) => self.set_tyvar(scope.set_tv(tv)),
            Type::TyCon(_tc) => self.clone(),
            Type::TyApp(fun, arg) => self
                .set_tyapp_fun(fun.set_kinds(scope))
                .set_tyapp_arg(arg.set_kinds(scope)),
            Type::AssocTy(_, args) => {
                let args = args
                    .iter()
                    .map(|arg| arg.set_kinds(scope))
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
            Type::TyApp(head, arg) => head.is_assoc_ty_free() && arg.is_assoc_ty_free(),
            Type::AssocTy(_, _) => false,
        }
    }

    // Is the head a type constructor?
    fn is_head_tycon(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => false,
            Type::TyCon(_) => true,
            Type::TyApp(head, _) => head.is_head_tycon(),
            Type::AssocTy(_, _) => false,
        }
    }

    // Is this type can be instance head of trait?
    pub fn is_implementable(self: &Arc<TypeNode>) -> Result<(), Errors> {
        if !self.is_head_tycon() {
            return Err(Errors::from_msg_srcs(
                        format!(
                            "Implementing trait for type `{}` is not allowed. \
                            The head (in this case, `{}`) of the type should be a type constructor.",
                            self.to_string(),
                            self.get_head_string(),
                        ),
                        &[&self.get_source()],
                    ));
        }
        if !self.is_assoc_ty_free() {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Implementing trait for type `{}` is not allowed. \
                    Associated types cannot appear in the type.",
                    self.to_string(),
                ),
                &[&self.get_source()],
            ));
        }
        return Ok(());
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

    pub fn get_head_string(self: &Arc<TypeNode>) -> String {
        match &self.ty {
            Type::TyVar(_) => self.to_string(),
            Type::TyCon(_) => self.to_string(),
            Type::TyApp(head, _) => head.get_head_string(),
            Type::AssocTy(assoc_ty, _) => assoc_ty.name.to_string(),
        }
    }

    #[allow(dead_code)]
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

    pub fn set_tyvar(&self, tv: Arc<TyVar>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyVar(_) => ret.ty = Type::TyVar(tv),
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

    pub fn set_assocty_name(&self, name: AssocType) -> Arc<TypeNode> {
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

    // For a lambda type (i.e., a closure or a function pointer), return the source types.
    // Returns an single element vector for a closure type.
    pub fn get_lambda_srcs(self: &Arc<TypeNode>) -> Vec<Arc<TypeNode>> {
        if self.is_funptr() || self.is_closure() {
            let mut type_args = self.collect_type_argments();
            type_args.pop(); // Discard the destination type.
            return type_args;
        }
        panic!(
            "`get_lambda_srcs` called for non-lambda type: {}",
            self.to_string()
        );
    }

    // For a lambda type (i.e., a closure or a function pointer), return the destination type.
    pub fn get_lambda_dst(&self) -> Arc<TypeNode> {
        if self.is_funptr() || self.is_closure() {
            let mut type_args = self.collect_type_argments();
            type_args.pop().unwrap()
        } else {
            panic!()
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
        ctx: &mut NameResolutionContext,
    ) -> Result<Arc<TypeNode>, Errors> {
        match &self.ty {
            Type::TyVar(_tv) => Ok(self.clone()),
            Type::TyCon(tc) => {
                let mut tc = tc.as_ref().clone();
                tc.resolve_namespace(ctx, &self.info.source)?;
                if ctx.env.candidates[&tc.name] == NameResolutionType::AssocTy {
                    let arity: usize = ctx.env.assoc_ty_to_arity[&tc.name];
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Associated type `{}` has arity {}, but supplied 0 types. All appearance of associated type has to be saturated.",
                            tc.name.to_string(),
                            arity,
                        ),
                        &[self.get_source()],
                    ));
                }
                Ok(self.set_tycon_tc(Arc::new(tc)))
            }
            Type::TyApp(fun, arg) => {
                let app_seq = self.flatten_type_application();
                match &app_seq[0].ty {
                    Type::TyCon(tc) => {
                        // In this case, replace self to associated type application if necessary.
                        let mut tc = tc.as_ref().clone();
                        tc.resolve_namespace(ctx, &app_seq[0].info.source)?;
                        if ctx.env.candidates[&tc.name] == NameResolutionType::AssocTy {
                            let assoc_ty_name = tc.name;
                            let arity: usize = ctx.env.assoc_ty_to_arity[&assoc_ty_name];
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
                            let assoc_ty_name_src = app_seq[0].get_source().clone();
                            let last_assoc_arg_src =
                                assoc_ty_args.last().unwrap().get_source().clone();
                            let assoc_ty_span =
                                Span::unite_opt(&assoc_ty_name_src, &last_assoc_arg_src);
                            let mut assoc_ty = type_assocty(
                                AssocType {
                                    name: assoc_ty_name,
                                    src: assoc_ty_name_src,
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
        self.field_types_via_tycons(&type_env.tycons)
    }

    pub fn field_types_via_tycons(&self, tycons: &Map<TyCon, TyConInfo>) -> Vec<Arc<TypeNode>> {
        let args = self.collect_type_argments();
        let ti = self.toplevel_tycon_info_via_tycons(tycons);
        assert_eq!(args.len(), ti.tyvars.len()); // Assumes fully applied
        let mut s = Substitution::default();
        for (i, tv) in ti.tyvars.iter().enumerate() {
            let merge_ok = s.merge(&Substitution::single(&tv.name, args[i].clone()));
            assert!(merge_ok);
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

    // The index of the struct/union field named `field_name`.
    pub fn field_index(&self, type_env: &TypeEnv, field_name: &str) -> Option<usize> {
        self.toplevel_tycon_info(type_env)
            .fields
            .iter()
            .position(|f| f.name == field_name)
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

    // For type `f a b c` where `f` is a type constructor returns `vec![a, b, c]`.
    pub fn collect_type_argments(&self) -> Vec<Arc<TypeNode>> {
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

    // Given a type `A1 -> A2 -> ... -> An -> B`, returns `([A1, A2, ..., An], B)`.
    // n = 0 is allowed. In this case, returns `([], B)`.
    // - `vars_limit`: limits the number of type variables to be collected.
    pub fn collect_app_src(
        self: &Arc<TypeNode>,
        vars_limit: usize,
    ) -> (Vec<Arc<TypeNode>>, Arc<TypeNode>) {
        fn collect_app_src_inner(
            ty: &Arc<TypeNode>,
            vars: &mut Vec<Arc<TypeNode>>,
            vars_limit: usize,
        ) -> Arc<TypeNode> {
            if ty.is_closure() || ty.is_funptr() {
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
        let ty = self.resolve_type_aliases_internal(env, vec![], &self_src)?;
        Ok(ty)
    }

    // Remove type aliases in a type.
    // * `type_name_path` - argument to detect circular aliasing.
    // * `entry_type` - argument to show good error message.
    fn resolve_type_aliases_internal(
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
        if let Type::TyCon(tc) = &toplevel_ty.ty {
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
                    let merge_ok = s.merge(&Substitution::single(&param, arg));
                    assert!(merge_ok);
                }
                let resolved = s.substitute_type(&ta.value);

                // Set source for `resolved`.
                let mut resolved = resolved.set_source(src);
                for i in (ta.tyvars.len() + 1)..app_seq.len() {
                    let arg = app_seq[i].clone();
                    let src = Span::unite_opt(resolved.get_source(), arg.get_source());
                    resolved = type_tyapp(resolved, arg).set_source(src);
                }
                return resolved.resolve_type_aliases_internal(env, type_name_path, entry_type_src);
            }
        }
        // Treat other cases.
        match &self.ty {
            Type::TyVar(_) => Ok(self.clone()),
            Type::TyCon(_) => Ok(self.clone()),
            Type::TyApp(fun_ty, arg_ty) => Ok(self
                .set_tyapp_fun(fun_ty.resolve_type_aliases_internal(
                    env,
                    type_name_path.clone(),
                    entry_type_src,
                )?)
                .set_tyapp_arg(arg_ty.resolve_type_aliases_internal(
                    env,
                    type_name_path,
                    entry_type_src,
                )?)),
            Type::AssocTy(_, args) => {
                let args = collect_results(args.iter().map(|arg| {
                    arg.resolve_type_aliases_internal(env, type_name_path.clone(), entry_type_src)
                }))?;
                Ok(self.set_assocty_args(args))
            }
        }
    }

    // Get top-level type constructor of a type.
    pub fn toplevel_tycon(&self) -> Option<Arc<TyCon>> {
        match &self.ty {
            Type::TyVar(_) => None,
            Type::TyCon(tc) => Some(tc.clone()),
            Type::TyApp(fun, _) => fun.toplevel_tycon(),
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
            Type::AssocTy(_, _) => {
                panic!("`set_toplevel_tycon` reached to an associated type application.")
            }
        }
    }

    pub fn is_closure(&self) -> bool {
        let tc = self.toplevel_tycon();
        if tc.is_none() {
            return false;
        }
        let tc = tc.unwrap();
        tc.name == make_arrow_name_abs()
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

    // Whether this is the internal `#ArrayStorage` type.
    pub fn is_array_storage(&self) -> bool {
        match self.toplevel_tycon() {
            Some(tc) => is_array_storage_tycon(tc.as_ref()),
            None => false,
        }
    }

    pub fn is_punched_array(&self) -> bool {
        let tc = self.toplevel_tycon();
        if tc.is_none() {
            return false;
        }
        let tc = tc.unwrap();
        return is_punched_array_tycon(tc.as_ref());
    }

    // Whether this is the unit type `()`, i.e. the tuple of no element.
    pub fn is_unit(&self) -> bool {
        match self.toplevel_tycon() {
            Some(tc) => tc.is_unit(),
            None => false,
        }
    }

    // Whether this is the type `Bool`.
    pub fn is_boolean(&self) -> bool {
        match self.toplevel_tycon() {
            Some(tc) => tc.is_boolean(),
            None => false,
        }
    }

    // Whether the top-level type constructor of this type is `IO`, i.e. whether this is `IO` or
    // `IO a`.
    pub fn is_io(&self) -> bool {
        match self.toplevel_tycon() {
            Some(tc) => tc.is_io(),
            None => false,
        }
    }

    // Whether the top-level type constructor of this type is a struct.
    // Panics for a closure type, a type variable, or a type constructor absent from `type_env`.
    pub fn is_struct(&self, type_env: &TypeEnv) -> bool {
        let ti = self.toplevel_tycon_info(type_env);
        match ti.variant {
            TyConVariant::Struct => true,
            _ => false,
        }
    }

    pub fn is_union(&self, type_env: &TypeEnv) -> bool {
        let ti = self.toplevel_tycon_info(type_env);
        match ti.variant {
            TyConVariant::Union => true,
            _ => false,
        }
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
        self.toplevel_tycon_info_via_tycons(&type_env.tycons)
    }

    pub fn toplevel_tycon_info_via_tycons(&self, tycons: &Map<TyCon, TyConInfo>) -> TyConInfo {
        assert!(!self.is_closure());
        let tycon = self.toplevel_tycon().unwrap();
        tycons.get(&tycon).unwrap().clone()
    }

    pub fn is_unbox(&self, type_env: &TypeEnv) -> bool {
        self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox
    }

    pub fn is_box(&self, type_env: &TypeEnv) -> bool {
        !self.is_unbox(type_env)
    }

    // Check if `self` is fully unboxed.
    // Here, a type is fully unboxed if and only if it does not contain any boxed type.
    //
    // A type reaching itself through unboxed fields has no layout, and this walk would not end on
    // one; `Program::validate_layouts` rejects such a type before any of this runs.
    pub fn is_fully_unboxed(&self, type_env: &TypeEnv) -> bool {
        if self.is_box(type_env) {
            return false;
        }
        if self.is_closure() {
            return false;
        }
        // `Array` is unboxed but holds its elements in a boxed storage, so it is never fully
        // unboxed. `field_types` of an array returns its element type, not the storage, so this
        // must be checked here rather than by recursing.
        if self.is_array() {
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

    /// Why a value of `self` has no size, given the types its layout came through, and `None` where
    /// it has one. `object::no_layout_reason` walks a type's layout and asks this at each step.
    ///
    /// # Arguments
    /// * `in_place` - the types `self` sits inside with no pointer in between, outermost first.
    ///   Reaching one of them again is a value that contains itself.
    /// * `across_pointers` - every type the layout came through, the ones behind a pointer included.
    ///   Reaching a larger type of the same type constructor there has no end either: the same
    ///   fields lead from that one to a larger one again.
    pub(crate) fn no_layout_message(
        self: &Arc<TypeNode>,
        in_place: &[Arc<TypeNode>],
        across_pointers: &[Arc<TypeNode>],
    ) -> Option<String> {
        if let Some(i) = in_place.iter().position(|ancestor| ancestor == self) {
            let cause = format!("its unboxed fields reach `{}` itself", self.to_string());
            return Some(self.no_size_report(&in_place[i..], cause));
        }
        // A function value is a pair of pointers whatever it takes and returns, so its size is
        // settled. Every function type shares the `->` constructor, so the growth of one function's
        // argument would otherwise be read off another's.
        if self.is_closure() || self.is_funptr() {
            return None;
        }
        // The same type constructor with arguments that have grown: the fields that led from that
        // one here lead on to a larger one again. A type merely appearing inside another (`Tree`
        // inside `(Tree, Tree)`) is how an ordinary recursive type is written, and the walk ends
        // there by meeting `Tree` a second time.
        let mine = self.flatten_type_application();
        let grows_from = |ancestor: &Arc<TypeNode>| {
            if ancestor == self {
                return false;
            }
            let theirs = ancestor.flatten_type_application();
            theirs.len() == mine.len()
                && theirs[0] == mine[0]
                && theirs[1..]
                    .iter()
                    .zip(mine[1..].iter())
                    .all(|(theirs, mine)| theirs.embeds_in(mine))
        };
        if let Some(i) = across_pointers.iter().position(|a| grows_from(a)) {
            let cause = "its fields reach ever larger types".to_string();
            return Some(self.no_size_report(&across_pointers[i..], cause));
        }
        None
    }

    /// Whether this type is embedded in `other`: it appears there with its own shape intact, with
    /// more type around it or inside its arguments. An argument grown this way is what tells a type
    /// reached again at a larger argument from one reached at a smaller or unrelated one, which a
    /// count of symbols cannot tell apart.
    fn embeds_in(self: &Arc<TypeNode>, other: &Arc<TypeNode>) -> bool {
        // Inside one of `other`'s parts.
        let inside = match &other.ty {
            Type::TyApp(fun, arg) => self.embeds_in(fun) || self.embeds_in(arg),
            Type::AssocTy(_, args) => args.iter().any(|arg| self.embeds_in(arg)),
            Type::TyVar(_) | Type::TyCon(_) => false,
        };
        if inside {
            return true;
        }
        // The same shape at the top, each part embedded in the part facing it.
        match (&self.ty, &other.ty) {
            (Type::TyVar(mine), Type::TyVar(theirs)) => mine.name == theirs.name,
            (Type::TyCon(mine), Type::TyCon(theirs)) => mine == theirs,
            (Type::TyApp(my_fun, my_arg), Type::TyApp(their_fun, their_arg)) => {
                my_fun.embeds_in(their_fun) && my_arg.embeds_in(their_arg)
            }
            (Type::AssocTy(mine, my_args), Type::AssocTy(theirs, their_args)) => {
                mine == theirs
                    && my_args.len() == their_args.len()
                    && my_args
                        .iter()
                        .zip(their_args.iter())
                        .all(|(mine, theirs)| mine.embeds_in(theirs))
            }
            _ => false,
        }
    }

    /// The report for a type with no size: what its fields do, the way down to it from the type that
    /// shows it, and which types the fix is among.
    fn no_size_report(
        self: &Arc<TypeNode>,
        from_ancestor: &[Arc<TypeNode>],
        cause: String,
    ) -> String {
        let descent = from_ancestor
            .iter()
            .chain([self])
            .map(|ty| ty.to_string())
            .collect::<Vec<_>>();
        // A type holding itself directly is the whole story already, so the way down is spelled
        // out only where it passes through another type.
        let holds_itself = from_ancestor.iter().all(|ty| ty == self);
        let (way_down, remedy) = if holds_itself {
            (String::new(), format!("Make `{}` boxed.", descent[0]))
        } else {
            (
                format!(
                    " ({})",
                    descent
                        .iter()
                        .map(|ty| format!("`{}`", ty))
                        .collect::<Vec<_>>()
                        .join(" -> ")
                ),
                "Make one of these types boxed.".to_string(),
            )
        };
        format!(
            "`{}` has no size: {}{}. {}",
            descent[0], cause, way_down, remedy,
        )
    }

    /// Whether a value of this type is one indivisible reference-counting unit — counted as a whole by
    /// a custom traverser rather than by descending into its fields. This holds for a boxed value, an
    /// unboxed union (only its active variant is live, so a refcount operation must dispatch on the tag
    /// rather than name a variant's leaf), and a punched array (its traversal skips the moved-out hole
    /// at a run-time index). A type whose reference counting is a whole-value operation belongs here.
    ///
    /// The caller must have already handled a closure, whose capture is the unit: this asserts the type
    /// is not a closure (via `is_union`).
    pub fn is_rc_unit_root(&self, type_env: &TypeEnv) -> bool {
        // `Array` is unboxed but is one indivisible unit: its own custom traverser drives element
        // lifetime through the storage, so the reference-count machinery must not descend into it.
        self.is_box(type_env)
            || self.is_union(type_env)
            || self.is_punched_array()
            || self.is_array()
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

    /// The LLVM struct a value of this type is laid out as.
    pub fn get_struct_type<'c, 'm>(
        self: &Arc<TypeNode>,
        gc: &mut Generator<'c, 'm>,
    ) -> StructType<'c> {
        gc.struct_type_of(self)
    }

    /// The LLVM type a value of this type takes where it is embedded in another value: the struct
    /// it is laid out as when it is unboxed, a pointer when it is boxed.
    pub fn get_embedded_type<'c, 'm>(
        self: &Arc<TypeNode>,
        gc: &mut Generator<'c, 'm>,
    ) -> BasicTypeEnum<'c> {
        gc.embedded_type_of(self)
    }

    // Check if the type takes the form of the definition of associated type.
    // Definition of an associated type has to be of the form `type AssocTypeName ty1 tv2 ... tvN`,
    // - where `{AssocTypeName}` is a local name,
    // - `ty1` is equal to the implemented type.
    // - type variables appears in the arguments are distinct.
    // If ok, return an `AssocTypeDefnHead` with the parsed information.
    pub fn validate_as_associated_type_impl_defn(
        &self,
        impl_type: &Arc<TypeNode>,
        src_for_err: &Option<Span>,
        is_impl: bool,
    ) -> Result<AssocTypeDefnHead, Errors> {
        fn general_err(
            is_impl: bool,
            imple_type: &Arc<TypeNode>,
            src_for_err: &Option<Span>,
        ) -> Errors {
            if is_impl {
                Errors::from_msg_srcs(
                    format!("The implementation of an associated type should be in the form `type {{AssocTyName}} {{impl_type}} {{type_var1}} ... {{type_varN}} = {{value_type}};`, where {{impl_type}} is `{}` here.", imple_type.to_string()),
                    &[src_for_err],
                )
            } else {
                Errors::from_msg_srcs(
                    format!("The definition of an associated type should be in the form `type {{AssocTyName}} {{impl_type}} {{type_var1}} ... {{type_varN}};`, where {{impl_type}} is `{}` here.", imple_type.to_string()),
                    &[src_for_err],
                )
            }
        }
        // Validate the type application sequence.
        let app_seq = self.flatten_type_application();
        if app_seq.len() < 2 {
            return Err(general_err(is_impl, impl_type, src_for_err));
        }
        let assoc_type_name: Name;
        match &app_seq[0].ty {
            Type::TyCon(tc) => {
                if !tc.name.is_local() {
                    return Err(Errors::from_msg_srcs(
                        "Do not specify namespace of the associated type here; the namespace of an associated type is determined by the trait name.".to_string(),
                        &[src_for_err],
                    ));
                }
                assoc_type_name = tc.name.to_string();
            }
            _ => {
                return Err(general_err(is_impl, impl_type, src_for_err));
            }
        }
        // For trait definitions (`is_impl=false`), verify the impl_type token matches
        // the expected impl_type at parse time.
        // For trait implementations (`is_impl=true`), skip this check here: the user
        // may write a namespace-qualified type (e.g., `Main::MyType`) which would not
        // match the unresolved local name in `impl_type` at this stage.  The equivalent
        // check is performed after name resolution in `validate_trait_impl`.
        if !is_impl && app_seq[1].to_string() != impl_type.to_string() {
            return Err(general_err(is_impl, impl_type, src_for_err));
        }
        let impl_type_as_written = app_seq[1].clone();
        let mut tyvars = vec![make_tyvar("#impl_type", &kind_star())];
        let impl_ty_tyvar_set: Set<Name> = impl_type
            .free_vars_vec()
            .iter()
            .map(|tv| tv.name.clone())
            .collect();
        let mut tyvars_set: Set<Name> = Set::default();
        for i in 2..app_seq.len() {
            match &app_seq[i].ty {
                Type::TyVar(tv) => {
                    if impl_ty_tyvar_set.contains(&tv.name) {
                        if is_impl {
                            return Err(Errors::from_msg_srcs(
                                format!(
                                    "In associated type implementation, each type variable should be free from the implemented type (`{}` here).",
                                    impl_type.to_string()
                                ),
                                &[src_for_err],
                            ));
                        } else {
                            return Err(Errors::from_msg_srcs(
                                format!(
                                    "In associated type definition, each type variable should be free from the implemented type (`{}` here).",
                                    impl_type.to_string()
                                ),
                                &[src_for_err],
                            ));
                        }
                    }
                    if tyvars_set.contains(&tv.name) {
                        if is_impl {
                            return Err(Errors::from_msg_srcs(
                                "In associated type implementation, each type variable should be different.".to_string(),
                                &[src_for_err],
                            ));
                        } else {
                            return Err(Errors::from_msg_srcs(
                                "In associated type definition, each type variable should be different.".to_string(),
                                &[src_for_err],
                            ));
                        }
                    }
                    tyvars.push(tv.clone());
                    tyvars_set.insert(tv.name.clone());
                }
                _ => {
                    return Err(general_err(is_impl, impl_type, src_for_err));
                }
            }
        }
        let assoc_type_src = app_seq[0].get_source().clone();
        Ok(AssocTypeDefnHead {
            name: assoc_type_name,
            name_src: assoc_type_src,
            params: tyvars,
            impl_type_as_written,
        })
    }
}

/// Parsed and validated result of the head of an associated type definition or implementation.
/// Represents the `AssocTypeName ImplType tv1 tv2 ... tvN` part of
/// `type AssocTypeName ImplType tv1 tv2 ... tvN [= ValueType]`.
pub struct AssocTypeDefnHead {
    /// Local name of the associated type (e.g., `MyElem`).
    pub name: Name,
    /// Source span of the associated type name.
    pub name_src: Option<Span>,
    /// Type parameters of the associated type equation.
    /// The first element is always the special `#impl_type` type variable;
    /// the remaining elements are the user-supplied extra type variables.
    pub params: Vec<Arc<TyVar>>,
    /// The impl_type token exactly as written in the source
    /// (e.g., `Main::MyType` in `type Item Main::MyType = ...`).
    /// Retained so that it can be compared against the resolved impl_type
    /// after name resolution, catching mismatched impl_types in trait impls.
    pub impl_type_as_written: Arc<TypeNode>,
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
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum Type {
    TyVar(Arc<TyVar>),
    TyCon(Arc<TyCon>),
    TyApp(Arc<TypeNode>, Arc<TypeNode>),
    AssocTy(AssocType, Vec<Arc<TypeNode>>),
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
        let mut appeared: Set<Name> = Set::default();
        for fv in free_vars {
            if appeared.contains(&fv.name) {
                continue;
            }
            appeared.insert(fv.name.clone());
            let new_name = number_to_varname(next_tyvar_no);
            s.merge(&Substitution::single(
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
    pub fn to_string(self: &Arc<TypeNode>) -> String {
        fn should_braced_as_arg(arg: &Arc<TypeNode>) -> bool {
            match &arg.ty {
                Type::TyVar(_) => false,
                Type::TyCon(_) => false,
                Type::TyApp(fun, _) => {
                    let tycon = fun.toplevel_tycon();
                    if let Some(tycon) = tycon {
                        if let Some(tuple_n) = get_tuple_n(&tycon.name) {
                            return tuple_n as usize != arg.collect_type_argments().len();
                        }
                    }
                    return true;
                }
                Type::AssocTy(_, _) => true,
            }
        }

        match &self.ty {
            Type::TyVar(v) => v.name.clone(),
            Type::TyApp(fun, arg) => {
                let tycon = self.toplevel_tycon();
                if let Some(tycon) = tycon {
                    if let Some(n) = get_tuple_n(&tycon.name) {
                        // Tuple case.
                        let args = self.collect_type_argments();
                        let arg_strs = args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();

                        // In this case, we use special notation when n = 1 or n = args.len().
                        if n == 1 {
                            return format!("({},)", arg_strs[0]);
                        }
                        if n as usize == args.len() {
                            return format!("({})", arg_strs.join(", "));
                        }
                    }
                    if tycon.name == make_arrow_name_abs() {
                        // `->` case.
                        // In this case we use special notation when the `Arrow` type is fully applied.
                        let args = self.collect_type_argments();
                        if args.len() == 2 {
                            if args[0].is_closure() {
                                return format!(
                                    "({}) -> {}",
                                    args[0].to_string(),
                                    args[1].to_string()
                                );
                            } else {
                                return format!(
                                    "{} -> {}",
                                    args[0].to_string(),
                                    args[1].to_string()
                                );
                            }
                        }
                    }
                }
                let tyfun = fun.to_string();
                let arg_str = arg.to_string();
                if should_braced_as_arg(arg) {
                    format!("{} ({})", tyfun, arg_str)
                } else {
                    format!("{} {}", tyfun, arg_str)
                }
            }
            Type::TyCon(tc) => tc.to_string(),
            Type::AssocTy(assoc_ty, args) => {
                format!(
                    "{} {}",
                    assoc_ty.name.to_string(),
                    args.iter()
                        .map(|arg| {
                            let arg_str = arg.to_string();
                            if should_braced_as_arg(arg) {
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

    /// The symbol name of the traverser function for this type. It keys the memoization of the
    /// generated traversers, so one name is minted per distinct `(type, capture, work, state)`.
    ///
    /// # Arguments
    /// * `capture` — the types a dynamic object's destructor traverses, empty for every other type.
    pub fn traverser_name(
        self: &Arc<TypeNode>,
        capture: &Vec<Arc<TypeNode>>,
        work: Option<TraverserWorkType>,
        state: RcState,
    ) -> String {
        let work_name = match work {
            None => "trav_dyn",
            Some(work) => match work.0 {
                TRAVERSER_WORK_RELEASE => "trav_release",
                TRAVERSER_WORK_MARK_GLOBAL => "trav_mark_global",
                TRAVERSER_WORK_MARK_THREADED => "trav_mark_threaded",
                _ => unreachable!(),
            },
        };
        format!(
            "{}{}_{}",
            work_name,
            state.name_suffix(),
            self.hash_with_capture(capture)
        )
    }

    /// A digest of this type together with `capture`, short enough to embed in a symbol name. Two
    /// types with the same normalized form and the same captures hash alike.
    ///
    /// # Arguments
    /// * `capture` — the captured types of a dynamic object, which distinguish two dynamic objects
    ///   of the same type. Empty for every other type.
    pub fn hash_with_capture(self: &Arc<TypeNode>, capture: &Vec<Arc<TypeNode>>) -> String {
        // If the type is not dynamic, then the capturing types should be empty.
        assert!(self.is_dynamic() || capture.len() == 0);
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
        format!("{:x}", md5::compute(str))
    }

    // Get hash value.
    pub fn hash(self: &Arc<TypeNode>) -> String {
        let type_string = self.to_string_normalize();
        format!("{:x}", md5::compute(type_string))
    }

    // Returns the list of predicates for this type to be well-formed.
    // See all associated type usages (for example, `Elem c`) in this type and returns a preducate `c : Collects`.
    #[allow(dead_code)]
    pub fn predicates_from_associated_types(&self) -> Vec<Predicate> {
        fn predicates_from_associated_types_internal(ty: &TypeNode, buf: &mut Vec<Predicate>) {
            match &ty.ty {
                Type::TyVar(_) => {}
                Type::TyCon(_) => {}
                Type::TyApp(fun, arg) => {
                    predicates_from_associated_types_internal(fun, buf);
                    predicates_from_associated_types_internal(arg, buf);
                }
                Type::AssocTy(assoc_ty, args) => {
                    let pred = Predicate {
                        trait_id: assoc_ty.trait_id(),
                        ty: args[0].clone(),
                        src: ty.get_source().clone(),
                        trait_src: None,
                    };
                    buf.push(pred);
                    for arg in args {
                        predicates_from_associated_types_internal(arg, buf);
                    }
                }
            }
        }
        let mut buf = vec![];
        predicates_from_associated_types_internal(self, &mut buf);
        buf
    }
}

pub fn kind_star() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

pub fn kind_arrow(src: Arc<Kind>, dst: Arc<Kind>) -> Arc<Kind> {
    Arc::new(Kind::Arrow(src, dst))
}

pub fn make_tyvar(var_name: &str, kind: &Arc<Kind>) -> Arc<TyVar> {
    Arc::new(TyVar {
        name: String::from(var_name),
        kind: kind.clone(),
    })
}

pub fn type_tyvar(var_name: &str, kind: &Arc<Kind>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(make_tyvar(var_name, kind)))
}

pub fn type_tyvar_star(var_name: &str) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(make_tyvar(var_name, &kind_star())))
}

pub fn type_from_tyvar(tyvar: Arc<TyVar>) -> Arc<TypeNode> {
    let ty = TypeNode::new(Type::TyVar(tyvar.clone()));
    Arc::new(ty)
}

pub fn type_fun(src: Arc<TypeNode>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    type_fun_with_arrow_src(src, dst, None)
}

pub fn type_fun_with_arrow_src(
    src: Arc<TypeNode>,
    dst: Arc<TypeNode>,
    arrow_src: Option<Span>,
) -> Arc<TypeNode> {
    let src_span = src.get_source().clone();
    let partial = type_tyapp(
        type_tycon(&tycon(make_arrow_name_abs())).set_source(arrow_src),
        src,
    )
    .set_source_if_none(src_span);
    type_tyapp(partial, dst)
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

pub fn type_assocty(assoc_ty: AssocType, args: Vec<Arc<TypeNode>>) -> Arc<TypeNode> {
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
    pub fn free_vars(self: &Arc<TypeNode>) -> Map<Name, Arc<TyVar>> {
        let mut free_vars: Map<String, Arc<TyVar>> = Map::default();
        match &self.ty {
            Type::TyVar(tv) => {
                free_vars.insert(tv.name.clone(), tv.clone());
            }
            Type::TyApp(tyfun, arg) => {
                free_vars.extend(tyfun.free_vars());
                free_vars.extend(arg.free_vars());
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
            Type::TyCon(_) => {}
            Type::AssocTy(_, args) => {
                for arg in args {
                    arg.free_vars_to_vec(buf);
                }
            }
        };
    }

    pub fn free_vars_to_vec_with_span(
        self: &Arc<TypeNode>,
        buf: &mut Vec<(Arc<TyVar>, Option<Span>)>,
    ) {
        match &self.ty {
            Type::TyVar(tv) => {
                if buf.iter().any(|(tv0, _)| tv0.name == tv.name) {
                    return;
                }
                buf.push((tv.clone(), self.get_source().clone()))
            }
            Type::TyApp(tyfun, arg) => {
                tyfun.free_vars_to_vec_with_span(buf);
                arg.free_vars_to_vec_with_span(buf);
            }
            Type::TyCon(_) => {}
            Type::AssocTy(_, args) => {
                for arg in args {
                    arg.free_vars_to_vec_with_span(buf);
                }
            }
        };
    }

    pub fn free_vars_vec(self: &Arc<TypeNode>) -> Vec<Arc<TyVar>> {
        let mut buf = vec![];
        self.free_vars_to_vec(&mut buf);
        buf
    }

    // Collect type variables that are "fixed" in this type, in the sense of
    // `Fixv` from section 5.1 of "Associated Type Synonyms"
    // (Chakravarty, Keller, Peyton Jones, ICFP '05).
    //
    // A type variable is fixed if unifying the type with a ground type would
    // determine it. Associated type applications are not injective, so their
    // arguments are not fixed; this function stops recursing into them.
    pub fn fixed_vars_to_set(self: &Arc<TypeNode>, out: &mut Set<Name>) {
        match &self.ty {
            Type::TyVar(tv) => {
                out.insert(tv.name.clone());
            }
            Type::TyApp(tyfun, arg) => {
                tyfun.fixed_vars_to_set(out);
                arg.fixed_vars_to_set(out);
            }
            Type::TyCon(_) => {}
            Type::AssocTy(_, _) => {}
        };
    }

    // Collect all TyCons that appear in this type.
    pub fn collect_tycons(&self, tycons: &mut Set<TyCon>) {
        match &self.ty {
            Type::TyVar(_) => {
                // Type variables don't contain TyCons
            }
            Type::TyCon(tycon) => {
                tycons.insert(tycon.as_ref().clone());
            }
            Type::TyApp(tyfun, arg) => {
                tyfun.collect_tycons(tycons);
                arg.collect_tycons(tycons);
            }
            Type::AssocTy(_, args) => {
                for arg in args {
                    arg.collect_tycons(tycons);
                }
            }
        }
    }

    pub fn collect_tyvar_names(&self, tyvar_names: &mut Set<Name>) {
        match &self.ty {
            Type::TyVar(tv) => {
                tyvar_names.insert(tv.name.clone());
            }
            Type::TyCon(_) => {
                // Type constructors don't contain type variables
            }
            Type::TyApp(tyfun, arg) => {
                tyfun.collect_tyvar_names(tyvar_names);
                arg.collect_tyvar_names(tyvar_names);
            }
            Type::AssocTy(_, args) => {
                for arg in args {
                    arg.collect_tyvar_names(tyvar_names);
                }
            }
        }
    }

    // Convert all global FullNames to absolute paths.
    pub fn global_to_absolute(&self) -> Arc<TypeNode> {
        match &self.ty {
            Type::TyVar(_) => Arc::new(self.clone()),
            Type::TyCon(tycon) => {
                let new_tycon = tycon.global_to_absolute();
                self.set_tycon_tc(new_tycon)
            }
            Type::TyApp(tyfun, arg) => {
                let new_fun = tyfun.global_to_absolute();
                let new_arg = arg.global_to_absolute();
                self.set_tyapp_fun(new_fun).set_tyapp_arg(new_arg)
            }
            Type::AssocTy(assoc_ty, args) => {
                let mut new_assoc_ty = assoc_ty.clone();
                new_assoc_ty.name.global_to_absolute();
                let new_args = args.iter().map(|arg| arg.global_to_absolute()).collect();
                self.set_assocty_name(new_assoc_ty)
                    .set_assocty_args(new_args)
            }
        }
    }
}

// Type scheme.
#[derive(Clone, Serialize, Deserialize)]
pub struct Scheme {
    // Generalized variables.
    pub gen_vars: Vec<Arc<TyVar>>,
    // Kind signatures (user-specified kind annotations).
    #[serde(default)]
    pub kind_signs: Vec<KindSignature>,
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
                    &[&pred.src],
                ));
            }
        }
        let mut preds = vec![];
        for pred in &self.predicates {
            let mut pred = pred.resolve_trait_aliases(&trait_env.aliases)?;
            preds.append(&mut pred);
        }
        for eq in &self.equalities {
            if !eq.on_opaque_tyvar() {
                // Right hand side of an equality should be free from associated type.
                // This ensures that the reduction of a type terminates in a finite number of steps.
                if !eq.value.is_assoc_ty_free() {
                    return Err(Errors::from_msg_srcs(
                        "Right side of an equality constraint cannot contain an associated type. \
                         NOTE: Instead of using associated type in the right side, e.g., `Elem c1 = Elem c2`, you can write `Elem c1 = e, Elem c2 = e`. \
                         We will support more general constraints by implementing such conversion in a future.".to_string(),
                        &[&eq.src],
                    ));
                }
                // The first argument of the left side of an equality constraint should be a type variable.
                // If this condition is not satisified, then a type can be reduced in two ways, by this equality and by an instance of the associated type,
                // which implies that there is no "normal form" of the type.
                if !eq.args[0].is_tyvar() {
                    return Err(Errors::from_msg_srcs(
                        "The first argument of the left side of an equality constraint should be a type variable.".to_string(),
                        &[&eq.src],
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
                            &[&eq.src],
                        ));
                    }
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
                    src: None,
                    trait_src: None,
                };
                return Err(Errors::from_msg_srcs(
                    format!(
                        "The equality constraint `{}` is invalid here because `{}` is not assumed.",
                        eq.to_string(),
                        pred.to_string()
                    ),
                    &[&eq.src],
                ));
            }
        }
        // If the right side of an equality contains an opaque type variable,
        // then the equality must be on an opaque type variable (i.e., args[0] is an opaque tyvar).
        for eq in &self.equalities {
            let rhs_has_opaque = eq
                .value
                .free_vars_vec()
                .iter()
                .any(|tv| is_opaque_tyvar(&tv.name));
            if rhs_has_opaque && !eq.on_opaque_tyvar() {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "The left side of an equality constraint involving an opaque type must be \
                         an associated type applied to an opaque type variable.",
                    ),
                    &[&eq.src],
                ));
            }
        }
        // For an equality on an opaque type variable, the extra arguments (args[1..]) must be
        // mutually distinct type variables, and they must not appear elsewhere in the scheme
        // (other equalities, predicates, or the main type).
        // First, collect free variables appearing in everything except opaque equality.
        let mut outside_vars = Set::<Name>::default();
        for v in self.ty.free_vars_vec() {
            outside_vars.insert(v.name.clone());
        }
        for p in &self.predicates {
            let mut vars = vec![];
            p.free_vars_to_vec(&mut vars);
            for v in vars {
                outside_vars.insert(v.name.clone());
            }
        }
        for eq in &self.equalities {
            if eq.on_opaque_tyvar() {
                continue;
            }
            let mut vars = vec![];
            eq.free_vars_to_vec(&mut vars);
            for v in vars {
                outside_vars.insert(v.name.clone());
            }
        }
        // Validate args[1..] of each opaque equality.
        for eq in &self.equalities {
            if !eq.on_opaque_tyvar() {
                continue;
            }
            let mut param_set = Set::<Name>::default();
            for arg in &eq.args[1..] {
                let is_non_opaque_tyvar = match &arg.ty {
                    Type::TyVar(tv) => !is_opaque_tyvar(&tv.name),
                    _ => false,
                };
                if !is_non_opaque_tyvar {
                    return Err(Errors::from_msg_srcs(
                        "Extra arguments on the left side of an equality constraint involving an opaque type must be type variables.".to_string(),
                        &[&eq.src],
                    ));
                }
                let Type::TyVar(tv) = &arg.ty else {
                    unreachable!()
                };
                param_set.insert(tv.name.clone());
            }
            if param_set.len() != eq.args[1..].len() {
                return Err(Errors::from_msg_srcs(
                    "Extra arguments on the left side of an equality constraint involving an opaque type must be mutually distinct type variables.".to_string(),
                    &[&eq.src],
                ));
            }
            for name in &param_set {
                if outside_vars.contains(name) {
                    return Err(Errors::from_msg_srcs(
                        "Extra arguments on the left side of an equality constraint involving an opaque type must not appear elsewhere in the type signature.".to_string(),
                        &[&eq.src],
                    ));
                }
            }
            outside_vars.extend(param_set);
        }
        // We do not allow there are two equality constraints with the same left side.
        //
        // We should check if two left sides are not unifiable, but this syntactic check is sufficient for now.
        // Since type variables in the type scheme of a global value are fixed during type checking,
        // being unifiable is equivalent to being syntactically equal.
        //
        // This restriction is necessary to ensure that type reduction by equalities is deterministic.
        for i in 0..self.equalities.len() {
            for j in i + 1..self.equalities.len() {
                if self.equalities[i].lhs().to_string() == self.equalities[j].lhs().to_string() {
                    return Err(Errors::from_msg_srcs(
                        "Multiple equality constraints with the same left side are not allowed."
                            .to_string(),
                        &[&self.equalities[i].src, &self.equalities[j].src],
                    ));
                }
            }
        }

        // Each generalized type variable that appears in the scheme body must
        // be "fixed" in the sense of `Fixv` from section 5.1 of "Associated
        // Type Synonyms". A variable is fixed iff it appears outside of any
        // associated type application, either in the main type or on the
        // right-hand side of an equality constraint. A variable that only
        // appears under an associated type application (or only in a class
        // predicate) would not be determined by unification at a use site,
        // which would make the scheme ambiguous.
        let fixed = self.fixed_vars();
        // First occurrence wins, which gives a useful span pointing at the
        // offending position.
        let occurrences = self.all_tyvar_occurrences_with_span();
        for gv in &self.gen_vars {
            if fixed.contains(&gv.name) {
                continue;
            }
            let Some((_, span)) = occurrences.iter().find(|(tv, _)| tv.name == gv.name) else {
                // Variable does not appear anywhere in the body; it cannot be
                // used and would be ambiguous, but this situation should not
                // arise because `Scheme::generalize` only collects free vars
                // into `gen_vars`. Skip defensively.
                continue;
            };
            return Err(Errors::from_msg_srcs(
                format!(
                    "Type variable `{}` is not fixed by this type signature, which makes it ambiguous. \
                     NOTE: `{}` must appear outside of any associated type application.",
                    gv.name, gv.name,
                ),
                &[span],
            ));
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

            let mut pred_strs = preds.iter().map(|p| p.to_string()).collect::<Vec<_>>();
            pred_strs.sort();
            pred_strs.dedup();
            constraint_strs.extend(pred_strs);

            let mut eq_strs = eqs.iter().map(|eq| eq.to_string()).collect::<Vec<_>>();
            eq_strs.sort();
            eq_strs.dedup();
            constraint_strs.extend(eq_strs);

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
            s.merge(&Substitution::single(
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

    // Append free type variables to a buffer of type Vec.
    pub fn free_vars_to_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        let mut free_vars = vec![];
        for p in &self.predicates {
            p.free_vars_to_vec(&mut free_vars);
        }
        for eq in &self.equalities {
            eq.free_vars_to_vec(&mut free_vars);
        }
        self.ty.free_vars_to_vec(&mut free_vars);

        // Add non-generalized type variables to `buf`.
        for tv in &free_vars {
            if !self.gen_vars.iter().any(|tv0| tv0.name == tv.name) {
                buf.push(tv.clone());
            }
        }
    }

    // Collect type variables that are "fixed" by this scheme's body, in the
    // sense of `Fixv` from section 5.1 of "Associated Type Synonyms".
    //
    // Contributions:
    // - the main type `self.ty`
    // - the right-hand side of each equality constraint
    // Class predicates do not contribute (per `Fixv (D τ ⇒ ρ) = Fixv ρ`).
    pub fn fixed_vars(&self) -> Set<Name> {
        let mut out = Set::default();
        self.ty.fixed_vars_to_set(&mut out);
        for eq in &self.equalities {
            eq.fixed_vars_to_set(&mut out);
        }
        out
    }

    // Collect every type variable occurrence in the scheme body together
    // with a source span, walking predicates, equalities, and the main
    // type. Unlike `free_vars_to_vec`, this does NOT exclude generalized
    // variables - it is intended for diagnostics that need to locate a
    // specific variable (including gen_vars) in the source.
    pub fn all_tyvar_occurrences_with_span(&self) -> Vec<(Arc<TyVar>, Option<Span>)> {
        let mut out = vec![];
        for pred in &self.predicates {
            pred.ty.free_vars_to_vec_with_span(&mut out);
        }
        for eq in &self.equalities {
            for arg in &eq.args {
                arg.free_vars_to_vec_with_span(&mut out);
            }
            eq.value.free_vars_to_vec_with_span(&mut out);
        }
        self.ty.free_vars_to_vec_with_span(&mut out);
        out
    }

    pub fn set_kinds(&self, kind_env: &KindEnv) -> Result<Arc<Scheme>, Errors> {
        let mut ret = self.clone();
        let mut kind_scope = KindScope::new();
        // Insert user-specified kind annotations from kind_signs.
        for ks in &self.kind_signs {
            kind_scope
                .insert(ks.tyvar.clone(), ks.kind.clone())
                .map_err(|msg| Errors::from_msg_srcs(msg, &[&ret.ty.get_source()]))?;
        }
        let res = kind_scope.extend(&ret.predicates, &ret.equalities, &vec![], kind_env);
        if let Err(msg) = res {
            let mut span = ret.predicates[0].src.clone();
            for i in 1..ret.predicates.len() {
                span = Span::unite_opt(&span, &ret.predicates[i].src);
            }
            return Err(Errors::from_msg_srcs(msg, &[&span]));
        }
        for p in &mut ret.predicates {
            p.set_kinds(&kind_scope);
        }
        for eq in &mut ret.equalities {
            eq.set_kinds(&kind_scope);
        }
        ret.ty = ret.ty.set_kinds(&kind_scope);
        for tv in &mut ret.gen_vars {
            *tv = kind_scope.set_tv(tv);
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
        kind_signs: Vec<KindSignature>,
        preds: Vec<Predicate>,
        eqs: Vec<Equality>,
        ty: Arc<TypeNode>,
    ) -> Arc<Scheme> {
        Arc::new(Scheme {
            gen_vars: vars,
            kind_signs,
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
        let mut vars = collect_free_vars(&preds, &eqs, &ty);
        // Exclude opaque type variables and equality formal parameters from gen_vars.
        // Collect names of type variables that appear as formal parameters (args[1..]) of
        // opaque-related equality constraints.
        let mut opaque_eq_params = Set::<Name>::default();
        for eq in &eqs {
            // Check if this equality involves an opaque type variable.
            if eq.on_opaque_tyvar() {
                // Collect free type variables from args[1..].
                for arg in &eq.args[1..] {
                    for tv in arg.free_vars_vec() {
                        opaque_eq_params.insert(tv.name.clone());
                    }
                }
            }
        }
        vars.retain(|tv| !is_opaque_tyvar(&tv.name) && !opaque_eq_params.contains(&tv.name));
        Scheme::new_arc(vars, kind_signs.to_vec(), preds, eqs, ty)
    }

    // Create the type scheme from a type with no generalization.
    pub fn from_type(ty: Arc<TypeNode>) -> Arc<Scheme> {
        Scheme::new_arc(vec![], vec![], vec![], vec![], ty)
    }

    pub fn resolve_namespace(
        &self,
        ctx: &mut NameResolutionContext,
    ) -> Result<Arc<Scheme>, Errors> {
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

    // Convert all global FullNames to absolute paths.
    pub fn global_to_absolute(&self) -> Arc<Scheme> {
        Arc::new(Scheme {
            gen_vars: self.gen_vars.clone(),
            kind_signs: self.kind_signs.clone(),
            predicates: self
                .predicates
                .iter()
                .map(|p| p.global_to_absolute())
                .collect(),
            equalities: self
                .equalities
                .iter()
                .map(|eq| eq.global_to_absolute())
                .collect(),
            ty: self.ty.global_to_absolute(),
        })
    }
}

// Check if a type variable name represents an opaque type variable (starts with '?').
pub fn is_opaque_tyvar(name: &str) -> bool {
    name.starts_with('?')
}

// Check if a type variable name represents a `_` type wildcard (starts with
// `TYPE_WILDCARD_VAR_PREFIX`).
pub fn is_type_wildcard_tyvar(name: &str) -> bool {
    name.starts_with(TYPE_WILDCARD_VAR_PREFIX)
}

// Collect all free type variables from predicates, equalities, and a type into a Vec.
pub fn collect_free_vars(
    preds: &[Predicate],
    eqs: &[Equality],
    ty: &Arc<TypeNode>,
) -> Vec<Arc<TyVar>> {
    let mut vars = vec![];
    for pred in preds {
        pred.free_vars_to_vec(&mut vars);
    }
    for eq in eqs {
        eq.free_vars_to_vec(&mut vars);
    }
    ty.free_vars_to_vec(&mut vars);
    vars
}

// Mapping from an opaque TyCon application to the concrete type inferred by type-checking.
//
// Example: for `repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it`,
// after desugaring and type-checking:
//   lhs = `?it a`
//   rhs = `MapIterator (RangeIterator I64) a`
//
// For a trait impl like `impl Array a : ToIter`:
//   lhs = `?it (Array a)`
//   rhs = `ArrayIterator a`
#[derive(Clone, Serialize, Deserialize)]
pub struct OpaqueTyConResolution {
    // Opaque TyCon applied to type arguments.
    // E.g., `?it a` for a simple value, `?it (Array a)` for a trait impl.
    pub lhs: Arc<TypeNode>,
    // The concrete type. E.g., `MapIterator (RangeIterator I64) a`.
    // None until type-checking resolves it.
    pub rhs: Option<Arc<TypeNode>>,
}
