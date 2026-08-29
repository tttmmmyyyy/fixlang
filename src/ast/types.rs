use crate::ast::equality::Equality;
use crate::ast::kind_scope::{KindEnv, KindScope};
use crate::ast::name::{FullName, Name, NameSpace};
use crate::ast::predicate::Predicate;
use crate::ast::program::{EndNode, TypeEnv};
use crate::ast::traits::{KindSignature, TraitEnv, TraitId};
use crate::ast::typedecl::Field;
use crate::constants::{
    TraverserWorkType, BOOL_NAME, F32_NAME, F64_NAME, I16_NAME, I32_NAME, I64_NAME, I8_NAME,
    PTR_NAME, PUNCHED_TYPE_SYMBOL, STD_NAME, STRING_NAME, TRAVERSER_WORK_MARK_GLOBAL,
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
    make_tuple_name_abs, make_unit_ty,
};
use crate::generator::Generator;
use crate::misc::{collect_results, number_to_varname, Map, Set};
use crate::object::{ty_to_object_ty, ObjectType};
use crate::parse::sourcefile::{SourcePos, Span};
use crate::rc_ir::ast::RcState;
use core::panic;
use inkwell::types::{BasicTypeEnum, StructType};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, OnceLock};

/// A type variable, identified by its name.
#[derive(Clone, Serialize, Deserialize)]
pub struct TyVar {
    /// The name the variable is written with, e.g. `a`.
    pub name: Name,
    /// The kind of the types this variable stands for. A variable built by the parser carries `*`
    /// until `Program::set_kinds` reads the kind signatures of the declaration it appears in.
    pub kind: Arc<Kind>,
}

impl PartialEq for TyVar {
    /// Compares the name alone, which is what decides which variable this is; see the note on
    /// `Hash`.
    ///
    /// Every place the compiler compares two type variables by hand reads the name alone, and the
    /// kind a variable carries is set later than the variable itself, so reading it here answers a
    /// question about kinds with whichever value happened to be stored.
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for TyVar {}

impl Hash for TyVar {
    /// Hashes the name alone, agreeing with the equality of `PartialEq`: the name is what decides
    /// which variable this is, and the kind is an attribute the variable carries.
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

    /// A copy of this type variable named `name`, leaving this one as it is.
    pub fn set_name(&self, name: Name) -> Arc<TyVar> {
        let mut ret = self.clone();
        ret.name = name;
        Arc::new(ret)
    }
}

/// An associated type as a type names it, e.g. `Item` in `Item iter`.
#[derive(Clone, Serialize, Deserialize)]
pub struct AssocType {
    /// The name the associated type is declared under, whose namespace is the trait declaring it.
    pub name: FullName,
    /// Where the name was written, e.g. the span of `Item` in `Item iter`. Left out of
    /// `PartialEq`, `Eq` and `Hash`.
    pub src: Option<Span>,
}

impl PartialEq for AssocType {
    /// Compares the name alone, which is what decides which associated type this is; where the
    /// name was written stays out of the comparison.
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for AssocType {}

impl Hash for AssocType {
    /// Hashes the name alone, agreeing with the equality of `PartialEq`.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl AssocType {
    /// Gives this associated type's name its full name, read in the context `ctx` carries.
    ///
    /// # Arguments
    /// * `span` — where the type this name stands in was written, which a report about a name
    ///   nothing is found for points at.
    pub fn resolve_namespace(
        &mut self,
        ctx: &mut NameResolutionContext,
        span: &Option<Span>,
    ) -> Result<(), Errors> {
        self.name = ctx.resolve(&self.name, &[NameResolutionType::AssocTy], span)?;
        Ok(())
    }

    /// The trait that declares this associated type, which is the namespace its name sits in.
    pub fn trait_id(&self) -> TraitId {
        let mut namespace = self.name.namespace.names.clone();
        let name = namespace.pop().unwrap();
        TraitId {
            name: FullName::new(&NameSpace::new(namespace), &name),
        }
    }

    /// This associated type with its name spelled as an absolute path, so that it names the same
    /// entity from any namespace.
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
    /// Whether this is `*`, the kind of a type that has values of its own.
    pub fn is_star(&self) -> bool {
        matches!(self, Kind::Star)
    }

    /// This kind written the way Fix source writes it: `*`, `*->*`, and `(*->*)->*`, where an arrow
    /// on the left of an arrow is parenthesized because `->` associates to the right.
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

/// What kind of declaration a type constructor comes from, which settles how its values are laid out
/// and what the fields recorded for it mean.
// PROOF: P1, P2 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Eq, PartialEq, Clone, Hash)]
pub enum TyConVariant {
    /// A built-in type laid out as a single machine scalar, such as `Std::I64` or `Std::Ptr`.
    /// `Std::IOState` is one too, and carries nothing.
    Primitive,
    /// The function type constructor `->`, whose values are closures.
    Arrow,
    /// `Std::Array`, whose one field is the type its elements share.
    Array,
    /// A struct, whose fields are laid out one after another in the order they are declared.
    Struct,
    /// A union, whose fields are its variants, sharing one payload buffer under a tag.
    Union,
    /// A dynamic object, which a closure holds its captured values in. Boxed and nullable, laid out
    /// as a control block, the traverser that reaches the captured values, and then those values.
    DynamicObject,
    /// The internal `#ArrayStorage` object: a control block and a raw element buffer, holding an
    /// array's elements. Boxed; its element lifetime is driven by the owning `Array` value, not by
    /// its own traverser.
    ArrayStorage,
    /// The type an opaque type variable `?it` is desugared into. It declares no field, and is
    /// resolved away before code generation.
    Opaque,
}

/// The names, in the `Std` namespace, of the types that cross to C as a single scalar value.
/// The names `CTypeSizes::get_c_types` builds for the C numeric type aliases must all appear here.
const C_SCALAR_NAMES: &[&str] = &[
    I8_NAME, U8_NAME, I16_NAME, U16_NAME, I32_NAME, U32_NAME, I64_NAME, U64_NAME, F32_NAME,
    F64_NAME, PTR_NAME,
];

/// A type constructor, such as `Std::I64` or `Std::Array`, before any type argument is applied to
/// it. A type constructor is determined by its name.
#[derive(Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct TyCon {
    /// The name the type is declared under.
    pub name: FullName,
}

impl TyCon {
    /// The type constructor named `fullname`.
    pub fn new(fullname: FullName) -> TyCon {
        TyCon { name: fullname }
    }

    /// This type constructor written the way Fix source writes it.
    ///
    /// # Examples
    /// `Std::Array` is written `Std::Array`, and the tuple of no element is written `()`.
    pub fn to_string(&self) -> String {
        if let Some(n) = get_tuple_n(&self.name) {
            if n == 0 {
                return "()".to_string();
            }
        }
        self.name.to_string()
    }

    /// Gives this type constructor's name its full name, read in the context `ctx` carries. A name
    /// that stands for an associated type resolves here too, and the caller reads it back out of
    /// `ctx`.
    ///
    /// # Arguments
    /// * `span` — where the type this name stands in was written, which a report about a name
    ///   nothing is found for points at.
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

    /// This type constructor with its name spelled as an absolute path, so that it names the same
    /// type from any namespace.
    pub fn global_to_absolute(&self) -> Arc<Self> {
        let mut ret = self.clone();
        ret.name.global_to_absolute();
        Arc::new(ret)
    }

    /// The type of a value of this struct or union: the type constructor applied to one type
    /// variable new to `typechecker` per parameter the declaration takes.
    ///
    /// # Examples
    /// A struct declared as `type Pair a b` gives `Pair` applied to two new type variables, and
    /// one declared as `type Point` gives `Point`.
    pub fn get_struct_union_value_type(
        self: &TyCon,
        typechecker: &mut TypeCheckContext,
    ) -> Arc<TypeNode> {
        let ti = typechecker.type_env.tycons().get(self).unwrap();
        assert!(ti.variant == TyConVariant::Struct || ti.variant == TyConVariant::Union);

        // Make type variables for type parameters.
        let mut new_tyvars: Vec<Arc<TypeNode>> = vec![];
        for tv in ti.tyvars.clone() {
            let tv = typechecker.new_tyvar_by(&tv);
            new_tyvars.push(type_from_tyvar(tv));
        }

        apply_type_args(&Arc::new(self.clone()), &new_tyvars)
    }

    /// Whether this is the unit type `()`, i.e. the tuple of no element.
    pub fn is_unit(self: &TyCon) -> bool {
        self.name == make_tuple_name_abs(0)
    }

    /// Whether a value of this type crosses to C as one scalar: an integer, a floating point
    /// number, or a pointer, which C and Fix lay down the same way. These are the types a C
    /// function signature can name, and the types an exported Fix function can exchange.
    pub fn is_c_scalar(self: &TyCon) -> bool {
        self.name.namespace == NameSpace::from_strs(&[STD_NAME])
            && C_SCALAR_NAMES.contains(&self.name.name.as_str())
    }

    /// Whether this is an integer type that carries a sign. Panics for a type that is not an
    /// integer type of `Std`.
    pub fn is_signed_integer(self: &TyCon) -> bool {
        if self.name.namespace != NameSpace::from_strs(&[STD_NAME]) {
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

    /// Whether this is the type `Bool` of `Std`.
    pub fn is_boolean(&self) -> bool {
        return self.name == FullName::from_strs(&[STD_NAME], BOOL_NAME);
    }

    /// Whether this is the type `String` of `Std`.
    pub fn is_string(&self) -> bool {
        return self.name == FullName::from_strs(&[STD_NAME], STRING_NAME);
    }

    /// Whether this is the type constructor `IO`.
    pub fn is_io(&self) -> bool {
        self == make_io_tycon().as_ref()
    }

    /// Whether this is the type `IOState`, the token that an `IO` action threads.
    #[allow(dead_code)]
    pub fn is_iostate(&self) -> bool {
        return self.name == make_iostate_name();
    }

    /// Renames this struct's type constructor to the one that stands for the struct with one field
    /// made a hole.
    ///
    /// # Arguments
    /// * `punched_at` — the position of the field made the hole, counted from 0 in the order the
    ///   fields are declared.
    pub fn into_punched_type_name(&mut self, punched_at: usize) {
        self.name.name += &format!("{}{}", PUNCHED_TYPE_SYMBOL, punched_at);
    }

    /// Whether this is the type constructor `->` that heads a function type.
    #[allow(dead_code)]
    pub fn is_arrow(&self) -> bool {
        self == &make_arrow_tycon()
    }

    /// Whether this is the type constructor `Std::Array`.
    #[allow(dead_code)]
    pub fn is_array(&self) -> bool {
        self == &make_array_tycon()
    }
}

/// The declaration a type constructor comes from: the kind of declaration it is, the parameters it
/// takes, and what its values hold. A type alias is declared by `TyAliasInfo`.
#[derive(Clone)]
pub struct TyConInfo {
    /// The kind of the type constructor, which follows from the parameters it takes.
    pub kind: Arc<Kind>,
    /// What kind of declaration this is, which settles what `fields` holds.
    pub variant: TyConVariant,
    /// Whether a value of this type is held in place, with its fields laid out where the value
    /// sits.
    pub is_unbox: bool,
    /// The parameters the declaration takes, in the order they are declared.
    pub tyvars: Vec<Arc<TyVar>>,
    /// The fields of a struct or the variants of a union, in the order they are declared. An array
    /// declares one field, the type its elements share.
    pub fields: Vec<Field>,
    /// Where the declaration was written.
    pub source: Option<Span>,
    /// The documentation of this type, for a declaration the compiler builds itself. A declaration
    /// read from a source file carries its documentation in the comment above it, which
    /// `get_document` answers with.
    pub document: Option<String>,
    /// The struct this declaration punches a field out of, for a declaration that has one.
    ///
    /// Such a declaration stands for the values of that struct with one field moved out, so a pass
    /// that rewrites the struct rewrites this declaration the same way. The struct's own
    /// declaration is in the same table as this one, so a pass that creates a punched declaration
    /// creates the struct's as well.
    pub punched_from: Option<TyCon>,
}

impl TyConInfo {
    /// Gives every type name standing in the declared field types its full name, read in the
    /// context `ctx` carries.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for field in &mut self.fields {
            errors.eat_err(field.resolve_namespace(ctx));
        }
        errors.to_result()
    }

    /// Expands every type alias standing in the declared field types.
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for field in &mut self.fields {
            errors.eat_err(field.resolve_type_aliases(type_env));
        }
        errors.to_result()
    }

    /// The documentation of this type: the comment written above the declaration in the source
    /// where the declaration was read from one, and the `document` field otherwise. Documentation
    /// with no text in it is answered as `None`.
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

/// A declaration of a type alias: the type it stands for, and the parameters it takes.
#[derive(Clone)]
pub struct TyAliasInfo {
    /// The kind of the type constructor the alias names.
    pub kind: Arc<Kind>,
    /// The type the alias stands for, written in terms of `tyvars`.
    pub value: Arc<TypeNode>,
    /// The parameters the alias takes, in the order they are declared.
    pub tyvars: Vec<Arc<TyVar>>,
    /// Where the declaration was written.
    pub source: Option<Span>,
}

impl TyAliasInfo {
    /// The documentation comment written above this declaration.
    pub fn get_document(&self) -> Option<String> {
        self.source.as_ref().and_then(|src| src.get_document().ok())
    }

    /// Resolves the namespaces of the type names in the type the alias stands for.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        self.value = self.value.resolve_namespace(ctx)?;
        Ok(())
    }
}

/// How deeply a single type may nest before the compiler calls the program endless.
///
/// This bounds one type: a chain of a thousand types that each hold the next is a thousand types of
/// depth one, and a project keeps compiling however many such types it gains. A type reached from
/// itself at a larger type argument, on the other hand, gains a level at every step and passes any
/// bound.
///
/// Over the benchmark corpus and the examples the deepest type reached is 10; a type written with
/// 25 nested tuples reaches 27. The bound also caps how deep the walks over a type go — hashing it,
/// substituting into it, printing it — so raising it costs stack on the programs it exists to
/// reject.
pub const MAX_TYPE_DEPTH: usize = 500;

/// A node of a type expression, together with the information the compiler carries alongside it.
#[derive(Serialize, Deserialize)]
pub struct TypeNode {
    /// The type expression, which is what equality and hashing of a node read.
    pub ty: Type,
    /// Where the type was written, for a type read from a source file.
    ///
    /// Left out of the serialized form, so that a node serializes to its type expression — the
    /// same thing `PartialEq` compares and `Hash` hashes. A reader that must not follow an edit
    /// shifting a position would otherwise have to take the type out of whatever holds it, and an
    /// inline-LLVM op holds its types behind a trait object no reader can reach into
    /// (`divide_program::generated_code_hash` reads one).
    #[serde(skip)]
    pub info: TypeInfo,
    /// The hash of `ty`, kept once computed.
    ///
    /// A type is a directed acyclic graph: substituting an argument that a declaration mentions
    /// twice makes both occurrences the same node. Hashing such a type by walking it costs as much
    /// as the tree it unfolds to, which doubles at every level of a type like `P (a, a)`. Keeping
    /// the hash on the node makes the walk cost one visit per node.
    ///
    /// `Clone` leaves this empty: the clone-then-replace idiom the setters use would otherwise
    /// carry the hash of the type the node held before.
    #[serde(skip)]
    hash_cache: OnceLock<u64>,
    /// Whether no type variable occurs in this type, kept once computed. Answered by walking the
    /// type, so it is kept for the same reason the hash is.
    #[serde(skip)]
    ground_cache: OnceLock<bool>,
    /// How deeply this type nests, kept once computed, for the same reason.
    #[serde(skip)]
    depth_cache: OnceLock<usize>,
}

impl PartialEq for TypeNode {
    /// Compares the type expressions; the source information a node carries stays out of the
    /// comparison.
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}

impl Eq for TypeNode {}

impl TypeNode {
    /// The hash of the type expression, which is what `PartialEq` compares; the source information
    /// the node carries stays out of both. The answer is kept on the node (`hash_cache`), so hashing
    /// a type that shares a subterm many times costs one visit per node rather than one per
    /// occurrence.
    pub fn type_hash(&self) -> u64 {
        *self.hash_cache.get_or_init(|| {
            let mut hasher = DefaultHasher::new();
            self.ty.hash(&mut hasher);
            hasher.finish()
        })
    }
}

impl Hash for TypeNode {
    /// Writes `type_hash`, so that two nodes `PartialEq` calls equal hash alike.
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.type_hash());
    }
}

impl Debug for TypeNode {
    /// Writes the type in source syntax, with its free type variables renamed `a`, `b`, ... in
    /// order of appearance, so that two types differing only in variable names print alike.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Arc::new(self.clone()).to_string_normalize())
    }
}

impl TypeNode {
    /// The smallest node of this type covering `pos`, for a type read from a source file.
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

    /// The type the `_` type wildcard at `pos` was inferred to.
    ///
    /// `self` is the annotation as it was written, where each `_` stands as a type variable named
    /// with `TYPE_WILDCARD_VAR_PREFIX` carrying the `_`'s source span.
    ///
    /// # Arguments
    /// * `resolved` — the same annotation after type inference, so that the two have the same
    ///   shape with each wildcard replaced by the type it was inferred to. The walk descends both
    ///   in lockstep and answers with the node of `resolved` under the wildcard. Where the two
    ///   shapes differ, which reducing an associated type can leave them, there is nothing to
    ///   answer with.
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

    /// Collects into `out_set` the module declaring each type constructor standing in this type.
    /// Panics for a type carrying an associated type application, which is resolved away before
    /// this is asked.
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

    /// Where this type was written; a type the compiler builds itself carries none.
    pub fn get_source(&self) -> &Option<Span> {
        &self.info.source
    }

    /// A copy of this type written at `src`, leaving this node as it is.
    pub fn set_source(&self, src: Option<Span>) -> Arc<Self> {
        let mut ret = self.clone();
        ret.info.source = src;
        Arc::new(ret)
    }

    /// A copy of this type written at `src` where it carries no source of its own, and this type
    /// itself where it does.
    pub fn set_source_if_none(self: &Arc<TypeNode>, src: Option<Span>) -> Arc<TypeNode> {
        if self.info.source.is_none() {
            self.set_source(src)
        } else {
            self.clone()
        }
    }

    /// This type with each type variable carrying the kind `scope` gives its name.
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

    /// Whether this type is built from type constructors and type variables alone, with no
    /// associated type application standing anywhere in it.
    pub fn is_assoc_ty_free(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => true,
            Type::TyCon(_) => true,
            Type::TyApp(head, arg) => head.is_assoc_ty_free() && arg.is_assoc_ty_free(),
            Type::AssocTy(_, _) => false,
        }
    }

    /// Whether the head of this type is a type constructor, as `Array` heads `Array a`.
    fn is_head_tycon(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => false,
            Type::TyCon(_) => true,
            Type::TyApp(head, _) => head.is_head_tycon(),
            Type::AssocTy(_, _) => false,
        }
    }

    /// Checks that a trait implementation may be written for this type: a type constructor heads
    /// it, and no associated type application stands in it.
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

    /// Whether this type is a type variable standing on its own.
    pub fn is_tyvar(&self) -> bool {
        match &self.ty {
            Type::TyVar(_) => true,
            _ => false,
        }
    }

    /// Whether this type is a type constructor standing on its own, with no argument applied.
    pub fn is_tycon(&self) -> bool {
        match &self.ty {
            Type::TyCon(_) => true,
            _ => false,
        }
    }

    /// The type constructor this type is. Panics for a type that is not a type constructor
    /// standing on its own.
    pub fn as_tycon(&self) -> &TyCon {
        match &self.ty {
            Type::TyCon(tc) => tc,
            _ => panic!(
                "`as_tycon` called for a type that is not a type constructor: {:?}",
                self
            ),
        }
    }

    /// The head of this type written out: the type constructor or the type variable being applied,
    /// or the name of the associated type.
    ///
    /// # Examples
    /// `Array I64` gives `Std::Array`, `f a` gives `f`, and `Item c` gives `Item`.
    pub fn get_head_string(self: &Arc<TypeNode>) -> String {
        match &self.ty {
            Type::TyVar(_) => self.to_string(),
            Type::TyCon(_) => self.to_string(),
            Type::TyApp(head, _) => head.get_head_string(),
            Type::AssocTy(assoc_ty, _) => assoc_ty.name.to_string(),
        }
    }

    /// A copy of this type variable carrying `kind`, leaving this node as it is. Panics for a type
    /// that is not a type variable.
    #[allow(dead_code)]
    pub fn set_tyvar_kind(&self, kind: Arc<Kind>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyVar(tv) => {
                ret.ty = Type::TyVar(tv.set_kind(kind));
            }
            _ => panic!(
                "`set_tyvar_kind` called for a type that is not a type variable: {:?}",
                self
            ),
        }
        Arc::new(ret)
    }

    /// A copy of this type with `tv` as its type variable, leaving this node as it is. Panics for
    /// a type that is not a type variable.
    pub fn set_tyvar(&self, tv: Arc<TyVar>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyVar(_) => ret.ty = Type::TyVar(tv),
            _ => panic!(
                "`set_tyvar` called for a type that is not a type variable: {:?}",
                self
            ),
        }
        Arc::new(ret)
    }

    /// A copy of this application with `fun` as the type being applied, keeping the argument.
    /// Panics for a type that is not a type application.
    pub fn set_tyapp_fun(&self, fun: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyApp(_, arg) => ret.ty = Type::TyApp(fun, arg.clone()),
            _ => panic!(
                "`set_tyapp_fun` called for a type that is not a type application: {:?}",
                self
            ),
        }
        Arc::new(ret)
    }

    /// A copy of this application with `arg` as the argument, keeping the type being applied.
    /// Panics for a type that is not a type application.
    pub fn set_tyapp_arg(&self, arg: Arc<TypeNode>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyApp(fun, _) => ret.ty = Type::TyApp(fun.clone(), arg),
            _ => panic!(
                "`set_tyapp_arg` called for a type that is not a type application: {:?}",
                self
            ),
        }
        Arc::new(ret)
    }

    /// A copy of this associated type application named `name`, keeping the arguments. Panics for
    /// a type that is not an associated type application.
    pub fn set_assocty_name(&self, name: AssocType) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::AssocTy(_, args) => ret.ty = Type::AssocTy(name, args.clone()),
            _ => panic!(
                "`set_assocty_name` called for a type that is not an associated type: {:?}",
                self
            ),
        }
        Arc::new(ret)
    }

    /// A copy of this associated type application applied to `args`, keeping the name. Panics for
    /// a type that is not an associated type application.
    pub fn set_assocty_args(&self, args: Vec<Arc<TypeNode>>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::AssocTy(assoc_ty, _) => ret.ty = Type::AssocTy(assoc_ty.clone(), args),
            _ => panic!(
                "`set_assocty_args` called for a type that is not an associated type: {:?}",
                self
            ),
        }
        Arc::new(ret)
    }

    /// The argument types of a closure type or a function pointer type. Panics for any other type.
    ///
    /// # Examples
    /// `a -> b` gives `[a]`, and `#FunPtr2 a b c` gives `[a, b]`.
    pub fn get_lambda_srcs(self: &Arc<TypeNode>) -> Vec<Arc<TypeNode>> {
        if self.is_funptr() || self.is_closure() {
            let mut type_args = self.collect_type_arguments();
            type_args.pop(); // Discard the destination type.
            return type_args;
        }
        panic!(
            "`get_lambda_srcs` called for non-lambda type: {}",
            self.to_string()
        );
    }

    /// The result type of a closure type or a function pointer type. Panics for any other type.
    ///
    /// # Examples
    /// `a -> b` gives `b`, and `#FunPtr2 a b c` gives `c`.
    pub fn get_lambda_dst(&self) -> Arc<TypeNode> {
        if self.is_funptr() || self.is_closure() {
            let mut type_args = self.collect_type_arguments();
            type_args.pop().unwrap()
        } else {
            panic!("`get_lambda_dst` called for non-lambda type: {:?}", self)
        }
    }

    /// A copy of this type with `tc` as its type constructor, leaving this node as it is. Panics
    /// for a type that is not a type constructor standing on its own.
    pub fn set_tycon_tc(&self, tc: Arc<TyCon>) -> Arc<TypeNode> {
        let mut ret = self.clone();
        match &self.ty {
            Type::TyCon(_) => ret.ty = Type::TyCon(tc),
            _ => panic!(
                "`set_tycon_tc` called for a type that is not a type constructor: {:?}",
                self
            ),
        }
        Arc::new(ret)
    }

    /// This type with every name standing in it — type constructors, type aliases and associated
    /// types — given its full name, read in the context `ctx` carries.
    ///
    /// A name that stands for an associated type becomes a `Type::AssocTy` node holding as many of
    /// the arguments it is applied to as the associated type's arity, and an occurrence given
    /// fewer arguments than that is reported as an error.
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

    /// This struct type punched at field `punched_at`: a type of the same memory layout, whose
    /// declaration marks that field as a hole the value has moved out of.
    pub fn to_punched_struct(self: &Arc<TypeNode>, punched_at: usize) -> Arc<TypeNode> {
        let mut tycon = self.toplevel_tycon().unwrap().as_ref().clone();
        tycon.into_punched_type_name(punched_at);
        self.set_toplevel_tycon(Arc::new(tycon))
    }

    /// The type of every field slot this type declares: one per field for a struct, one per variant
    /// for a union, and the element type alone for `Array`, whose elements all share it. This type's
    /// arguments are substituted in, so the results are the field types at this instance.
    ///
    /// A punched field's slot is among them, at the type it was declared with, so a reader that can
    /// meet a punched type wants this one only to lay the fields out or to address one by its index;
    /// `unpunched_field_types` answers which of the slots hold a value.
    // PROOF: P1, P2 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn field_types(&self, type_env: &TypeEnv) -> Vec<Arc<TypeNode>> {
        self.instance_field_types(self.toplevel_tycon_info(type_env), type_env)
    }

    /// `self` with every unwrapped newtype it applies saturated replaced by the type of that
    /// newtype's one field at the instance. The form of a newtype with its one field punched out
    /// holds nothing, so it becomes the unit type.
    ///
    /// With `IO` unwrapped, `IO ()` becomes `IOState -> (IOState, ())` and `Array (IO ())` becomes
    /// `Array (IOState -> (IOState, ()))`. `Foo IO` stays as it is: the `IO` there takes no
    /// arguments, and an unsaturated occurrence is not a type any value has — a type of kind `*`
    /// headed by a type constructor is saturated.
    pub fn unwrap_newtypes(self: &Arc<TypeNode>, type_env: &TypeEnv) -> Arc<TypeNode> {
        self.unwrap_newtypes_memoized(type_env, &mut Map::default())
    }

    /// `unwrap_newtypes`, answering from `unwrapped` a node the walk has already reached.
    ///
    /// A type is a directed acyclic graph: substituting an argument that a declaration mentions
    /// twice makes both occurrences the same node. Walking such a type as a tree costs as much as
    /// the tree it unfolds to, which doubles at every level of a type like `P (a, a)`. A node the
    /// walk leaves alone is answered with itself, so the graph the answer is stands as shared as the
    /// one that was walked.
    fn unwrap_newtypes_memoized(
        self: &Arc<TypeNode>,
        type_env: &TypeEnv,
        unwrapped: &mut Map<Arc<TypeNode>, Arc<TypeNode>>,
    ) -> Arc<TypeNode> {
        if let Some(ty) = unwrapped.get(self) {
            return ty.clone();
        }
        let ty = self.unwrap_newtypes_node(type_env, unwrapped);
        unwrapped.insert(self.clone(), ty.clone());
        ty
    }

    /// One node of the `unwrap_newtypes` walk, with the type this node stands for on the way out.
    fn unwrap_newtypes_node(
        self: &Arc<TypeNode>,
        type_env: &TypeEnv,
        unwrapped: &mut Map<Arc<TypeNode>, Arc<TypeNode>>,
    ) -> Arc<TypeNode> {
        if let Some(tycon) = self.toplevel_tycon() {
            if let Some(tycon_info) = type_env.unwrapped_newtype_info(&tycon) {
                if tycon_info.tyvars.len() == self.collect_type_arguments().len() {
                    if tycon_info.fields[0].is_punched {
                        return make_unit_ty();
                    }
                    let field_ty = self.declared_field_types(tycon_info)[0].clone();
                    return field_ty.unwrap_newtypes_memoized(type_env, unwrapped);
                }
            }
        }
        match &self.ty {
            Type::TyVar(_) => self.clone(),
            Type::TyCon(_) => self.clone(),
            Type::TyApp(fun_ty, arg_ty) => {
                let new_fun_ty = fun_ty.unwrap_newtypes_memoized(type_env, unwrapped);
                let new_arg_ty = arg_ty.unwrap_newtypes_memoized(type_env, unwrapped);
                if Arc::ptr_eq(&new_fun_ty, fun_ty) && Arc::ptr_eq(&new_arg_ty, arg_ty) {
                    return self.clone();
                }
                self.set_tyapp_fun(new_fun_ty).set_tyapp_arg(new_arg_ty)
            }
            Type::AssocTy(_, _) => {
                unimplemented!("AssocTy is not supported in unwrap_newtypes")
            }
        }
    }

    /// The type each field of `tycon_info` holds at this instance: the type the declaration writes,
    /// with `self`'s type arguments substituted for the declaration's type variables, and with the
    /// unwrapped newtypes the substitution saturates replaced by what they unwrap to.
    ///
    /// Substituting can saturate one: the field `data : f ()` of `Foo` becomes `IO ()` at `Foo IO`,
    /// and a value holds that field at the closure `IO ()` unwraps to. Only a declaration taking a
    /// parameter of a higher kind can saturate anything, since a parameter of kind `*` is never
    /// applied to arguments and substituting for one leaves every application spine as it stands;
    /// the field types a declaration is stored with are unwrapped once, by the pass that unwraps
    /// newtypes.
    fn instance_field_types(
        &self,
        tycon_info: &TyConInfo,
        type_env: &TypeEnv,
    ) -> Vec<Arc<TypeNode>> {
        let mut field_types = self.declared_field_types(tycon_info);
        let takes_higher_kinded_parameter = tycon_info.tyvars.iter().any(|tv| !tv.kind.is_star());
        if takes_higher_kinded_parameter {
            let mut unwrapped = Map::default();
            for field_ty in &mut field_types {
                *field_ty = field_ty.unwrap_newtypes_memoized(type_env, &mut unwrapped);
            }
        }
        field_types
    }

    /// The type each field of `tycon_info` is declared with, with `self`'s type arguments
    /// substituted for the declaration's type variables. The types are as the declaration writes
    /// them, so one can name a newtype the program has unwrapped; `instance_field_types` answers
    /// with the types values are built at.
    fn declared_field_types(&self, tycon_info: &TyConInfo) -> Vec<Arc<TypeNode>> {
        let args = self.collect_type_arguments();
        assert_eq!(args.len(), tycon_info.tyvars.len()); // Assumes fully applied
        let mut subst = Substitution::default();
        for (i, tv) in tycon_info.tyvars.iter().enumerate() {
            let merge_ok = subst.merge(&Substitution::single(&tv.name, args[i].clone()));
            assert!(merge_ok);
        }
        tycon_info
            .fields
            .iter()
            .map(|field| subst.substitute_type(&field.ty))
            .collect()
    }

    /// The types of the fields that are not punched, each with the index it sits at. A punched field
    /// is a hole: the value it held has moved out, so it holds nothing, while its slot stays in the
    /// layout and the other fields keep the indices they are addressed by.
    ///
    /// This is what a walk over the values a type holds descends: reference counting reaches a hole's
    /// slot through no path, and reading one would read a value that has moved on.
    // PROOF: P1, P2, P3, P4, P7, P18 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn unpunched_field_types(&self, type_env: &TypeEnv) -> Vec<(usize, Arc<TypeNode>)> {
        let tycon_info = self.toplevel_tycon_info(type_env);
        self.instance_field_types(tycon_info, type_env)
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !tycon_info.fields[*i].is_punched)
            .collect()
    }

    /// The index of the struct field or the union variant named `field_name`, which is the index it
    /// sits at in the layout.
    pub fn field_index(&self, type_env: &TypeEnv, field_name: &str) -> Option<usize> {
        self.toplevel_tycon_info(type_env)
            .fields
            .iter()
            .position(|f| f.name == field_name)
    }

    /// This type split into the head being applied and the arguments applied to it: `f a b` gives
    /// `vec![f, a, b]`.
    pub fn flatten_type_application(&self) -> Vec<Arc<TypeNode>> {
        /// Appends to `tys` the head of `ty`, then the arguments applied to it in order.
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

    /// The arguments applied to this type's head, in the order they are applied: `f a b c` gives
    /// `vec![a, b, c]`.
    pub fn collect_type_arguments(&self) -> Vec<Arc<TypeNode>> {
        let mut ret: Vec<Arc<TypeNode>> = vec![];
        match &self.ty {
            Type::TyApp(fun, arg) => {
                ret.append(&mut fun.collect_type_arguments());
                ret.push(arg.clone());
            }
            Type::TyCon(_) => {}
            _ => unreachable!(),
        }
        ret
    }

    /// This function type split into its argument types and the result type they lead to.
    ///
    /// # Arguments
    /// * `vars_limit` — how many argument types to take at most. The split stops before an arrow
    ///   that would carry the count past it, leaving the remaining arrows in the result type.
    ///
    /// # Examples
    /// `A -> B -> C` with a `vars_limit` of 2 gives `([A, B], C)`, and with a `vars_limit` of 1
    /// gives `([A], B -> C)`. A type that is no function gives `([], B)`.
    pub fn collect_app_src(
        self: &Arc<TypeNode>,
        vars_limit: usize,
    ) -> (Vec<Arc<TypeNode>>, Arc<TypeNode>) {
        /// Appends to `vars` the argument types of `ty` while `vars_limit` has room for them, and
        /// answers with the type reached once it has not.
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
        let dst_ty = collect_app_src_inner(self, &mut vars, vars_limit);
        (vars, dst_ty)
    }

    /// This type with every type alias standing in it expanded to the type it stands for. An alias
    /// that leads back to itself, and one applied to fewer arguments than it takes, are reported
    /// as errors.
    pub fn resolve_type_aliases(
        self: &Arc<TypeNode>,
        env: &TypeEnv,
    ) -> Result<Arc<TypeNode>, Errors> {
        let self_src = self.get_source().clone();
        let ty = self.resolve_type_aliases_internal(env, vec![], &self_src)?;
        Ok(ty)
    }

    /// One step of the `resolve_type_aliases` walk.
    ///
    /// # Arguments
    /// * `type_name_path` — the types the walk has expanded on the way to this one, written out
    ///   with normalized variable names; a type met twice is an alias leading back to itself.
    /// * `entry_type_src` — where the type the walk started from was written, which a report about
    ///   such an alias points at.
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

    /// The type constructor at the head of this type, as `Array` heads `Array I64`. A type
    /// variable and an associated type application have none.
    pub fn toplevel_tycon(&self) -> Option<Arc<TyCon>> {
        match &self.ty {
            Type::TyVar(_) => None,
            Type::TyCon(tc) => Some(tc.clone()),
            Type::TyApp(fun, _) => fun.toplevel_tycon(),
            Type::AssocTy(_, _) => None,
        }
    }

    /// This type with `tycon` at its head, keeping the arguments applied to it: `Array I64` with
    /// `Option` given here becomes `Option I64`. Panics for a type headed by a type variable or by
    /// an associated type application.
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

    /// Whether the top-level type constructor of this type satisfies `pred`. A type variable and an
    /// associated type application have no such constructor, and satisfy nothing.
    fn toplevel_tycon_satisfies(&self, pred: impl FnOnce(&TyCon) -> bool) -> bool {
        match self.toplevel_tycon() {
            Some(tc) => pred(tc.as_ref()),
            None => false,
        }
    }

    /// Whether this type is a function type `a -> b`, a value of which pairs the code to run with
    /// the values it captured.
    // PROOF: P1, P2 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn is_closure(&self) -> bool {
        self.toplevel_tycon_satisfies(|tc| tc.name == make_arrow_name_abs())
    }

    /// Whether this type is one of the `Std::#FunPtr{n}` constructors, a pointer to code of `n`
    /// arguments that carries no captured value.
    pub fn is_funptr(&self) -> bool {
        self.toplevel_tycon_satisfies(|tc| is_funptr_tycon(tc).is_some())
    }

    /// Whether this type is `Std::Array`.
    // PROOF: P1, P2 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn is_array(&self) -> bool {
        self.toplevel_tycon_satisfies(is_array_tycon)
    }

    /// Whether this is the internal `#ArrayStorage` type.
    pub fn is_array_storage(&self) -> bool {
        self.toplevel_tycon_satisfies(is_array_storage_tycon)
    }

    /// Whether this type is `Std::PunchedArray`, an array with one element moved out of it.
    pub fn is_punched_array(&self) -> bool {
        self.toplevel_tycon_satisfies(is_punched_array_tycon)
    }

    /// Whether this is the unit type `()`, i.e. the tuple of no element.
    pub fn is_unit(&self) -> bool {
        self.toplevel_tycon_satisfies(TyCon::is_unit)
    }

    /// Whether this is the type `Bool`.
    pub fn is_boolean(&self) -> bool {
        self.toplevel_tycon_satisfies(TyCon::is_boolean)
    }

    /// Whether this is the type `String`.
    pub fn is_string(&self) -> bool {
        self.toplevel_tycon_satisfies(TyCon::is_string)
    }

    /// Whether the top-level type constructor of this type is `IO`, i.e. whether this is `IO` or
    /// `IO a`.
    pub fn is_io(&self) -> bool {
        self.toplevel_tycon_satisfies(TyCon::is_io)
    }

    /// Whether the top-level type constructor of this type is a struct.
    /// Panics for a closure type, a type variable, or a type constructor absent from `type_env`.
    pub fn is_struct(&self, type_env: &TypeEnv) -> bool {
        let ti = self.toplevel_tycon_info(type_env);
        match ti.variant {
            TyConVariant::Struct => true,
            _ => false,
        }
    }

    /// Whether the top-level type constructor of this type is a union, so that a value of it
    /// carries one of the declared fields and a tag saying which.
    /// Panics for a closure type, a type variable, or a type constructor absent from `type_env`.
    // PROOF: P7 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn is_union(&self, type_env: &TypeEnv) -> bool {
        let ti = self.toplevel_tycon_info(type_env);
        match ti.variant {
            TyConVariant::Union => true,
            _ => false,
        }
    }

    /// Whether this type is `#DynamicObject`, the boxed object a closure keeps its captured values
    /// in. Its fields vary with the closure, so its layout follows from the capture types passed to
    /// `ty_to_object_ty` together with the type.
    pub fn is_dynamic(&self) -> bool {
        self.toplevel_tycon_satisfies(is_dynamic_object_tycon)
    }

    /// Whether this type is `Std::FFI::Destructor`, which runs the destructor function it holds
    /// over its value as it is destroyed.
    // PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn is_destructor_object(&self) -> bool {
        self.toplevel_tycon_satisfies(is_destructor_object_tycon)
    }

    /// The declaration of this type's outermost type constructor: its variant, boxedness, type
    /// parameters and fields. Panics for a closure type, a type variable, or a type constructor
    /// absent from `type_env`.
    // PROOF: P1, P2 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn toplevel_tycon_info<'a>(&self, type_env: &'a TypeEnv) -> &'a TyConInfo {
        assert!(!self.is_closure());
        let tycon = self.toplevel_tycon().unwrap();
        type_env.tycons().get(&tycon).unwrap()
    }

    /// Whether a value of this type is held in place, with its fields laid out where the value
    /// sits. A closure is unboxed: it is a function pointer beside the object its captures live in.
    // PROOF: P1, P2 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn is_unbox(&self, type_env: &TypeEnv) -> bool {
        self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox
    }

    /// Whether a value of this type is a pointer to a heap block that holds its fields, so that the
    /// value costs one pointer wherever it is stored and its lifetime is reference-counted.
    // PROOF: P1, P2 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn is_box(&self, type_env: &TypeEnv) -> bool {
        !self.is_unbox(type_env)
    }

    /// Whether a value of this type holds no boxed value, so that reference counting has nothing to
    /// do to it.
    ///
    /// Deciding this walks the fields of unboxed types, and that walk would not end on a type
    /// reaching itself that way; `Program::validate_layouts` rejects such a type before any of this
    /// runs.
    // PROOF: P1, P2, P7, P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
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
        self.unpunched_field_types(type_env)
            .iter()
            .all(|(_, field_ty)| field_ty.is_fully_unboxed(type_env))
    }

    /// A node holding `ty`, written nowhere.
    fn new(ty: Type) -> Self {
        Self {
            ty,
            info: TypeInfo::default(),
            hash_cache: OnceLock::new(),
            ground_cache: OnceLock::new(),
            depth_cache: OnceLock::new(),
        }
    }

    /// A shared node holding `ty`, written nowhere.
    fn new_arc(ty: Type) -> Arc<Self> {
        Arc::new(Self::new(ty))
    }

    /// A copy of this type carrying `info`, leaving this node as it is.
    #[allow(dead_code)]
    pub fn set_info(self: Arc<Self>, info: TypeInfo) -> Arc<Self> {
        let mut ret = (*self).clone();
        ret.info = info;
        Arc::new(ret)
    }

    /// A copy of this node holding the type expression `ty`, keeping the source it was written at.
    #[allow(dead_code)]
    pub fn set_ty(self: &Arc<Self>, ty: Type) -> Arc<Self> {
        let mut ret = (**self).clone();
        ret.ty = ty;
        Arc::new(ret)
    }

    /// The kind of this type, read from the kinds `kind_env` gives the type constructors and the
    /// associated types, and from the kinds the type variables carry. A type applied to an
    /// argument whose kind its own kind does not take is reported as an error.
    pub fn kind(self: &Arc<TypeNode>, kind_env: &KindEnv) -> Result<Arc<Kind>, Errors> {
        /// The error reported where `application` applies `fun` of kind `fun_kind` to `arg` of
        /// kind `arg_kind`, which `fun_kind` does not accept.
        fn kind_mismatch_error(
            application: &Arc<TypeNode>,
            fun: &Arc<TypeNode>,
            fun_kind: &Arc<Kind>,
            arg: &Arc<TypeNode>,
            arg_kind: &Arc<Kind>,
        ) -> Errors {
            let type_strs = TypeNode::to_string_normalize_many(&[
                application.clone(),
                fun.clone(),
                arg.clone(),
            ]);
            let application_str = &type_strs[0];
            let fun_str = &type_strs[1];
            let arg_str = &type_strs[2];
            Errors::from_msg_srcs(
                format!(
                    "Kind mismatch in `{}`. Type `{}` of kind `{}` cannot be applied to type `{}` of kind `{}`.",
                    application_str,
                    fun_str,
                    fun_kind.to_string(),
                    arg_str,
                    arg_kind.to_string()
                ),
                &[application.get_source()],
            )
        }

        match &self.ty {
            Type::TyVar(tv) => Ok(tv.kind.clone()),
            Type::TyCon(tc) => Ok(kind_env.tycons.get(&tc).unwrap().clone()),
            Type::TyApp(fun, arg) => {
                let fun_kind = fun.kind(kind_env)?;
                let arg_kind = arg.kind(kind_env)?;
                match &*fun_kind {
                    Kind::Arrow(arg2, res) => {
                        if arg_kind != *arg2 {
                            return Err(kind_mismatch_error(self, fun, &fun_kind, arg, &arg_kind));
                        }
                        Ok(res.clone())
                    }
                    Kind::Star => Err(kind_mismatch_error(self, fun, &fun_kind, arg, &arg_kind)),
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

    /// The layout of the object a value of this type lives in: its fields, their types, and
    /// whether it is boxed.
    ///
    /// # Arguments
    /// * `capture` — the types a dynamic object holds, which are its fields. Empty for every other
    ///   type.
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

    /// Checks that this type is the head of an associated type definition or implementation, which
    /// is written `{AssocTypeName} {impl_type} {tv1} ... {tvN}`: the name is a local one, the
    /// first argument is the implemented type, and the arguments after it are type variables that
    /// are distinct from one another and free from the implemented type.
    ///
    /// # Arguments
    /// * `impl_type` — the type the trait is implemented for, which the first argument names.
    /// * `src_for_err` — where to draw a report about the head.
    /// * `is_impl` — whether the head belongs to an implementation (`type Item Foo = ...;`) rather
    ///   than to a declaration in a trait (`type Item Foo;`). It decides the wording of the
    ///   reports, and whether the first argument is compared against `impl_type` here; a name
    ///   written with its namespace matches only once name resolution has run, so
    ///   `validate_trait_impl` compares an implementation's.
    ///
    /// # Returns
    /// The name, the parameters and the implemented type as written, read out of the head.
    pub fn validate_as_associated_type_impl_defn(
        &self,
        impl_type: &Arc<TypeNode>,
        src_for_err: &Option<Span>,
        is_impl: bool,
    ) -> Result<AssocTypeDefnHead, Errors> {
        /// The report that the head is not of the form a definition or an implementation is
        /// written in.
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
    /// Copies the type expression and where it was written, leaving the values kept on the node —
    /// its hash, whether it is ground, how deeply it nests — to be computed again.
    fn clone(&self) -> Self {
        TypeNode {
            ty: self.ty.clone(),
            info: self.info.clone(),
            hash_cache: OnceLock::new(),
            ground_cache: OnceLock::new(),
            depth_cache: OnceLock::new(),
        }
    }
}

/// A type expression, which is a type variable, a type constructor, or one of these applied to
/// arguments.
#[derive(Eq, Hash, Serialize, Deserialize, Clone)]
pub enum Type {
    /// A type variable, e.g. `a`.
    TyVar(Arc<TyVar>),
    /// A type constructor with no argument applied, e.g. `Std::I64`.
    TyCon(Arc<TyCon>),
    /// A type applied to one argument, so that `Array I64` is `Array` applied to `I64`.
    TyApp(Arc<TypeNode>, Arc<TypeNode>),
    /// An associated type applied to as many arguments as its arity, e.g. `Item c`. The first
    /// argument is the type the trait is implemented for.
    AssocTy(AssocType, Vec<Arc<TypeNode>>),
}

/// Whether two nodes hold the same type expression. Two occurrences of one node are the same type
/// without looking inside, which is what keeps comparing a type that shares a subterm cheap.
fn type_node_eq(lhs: &Arc<TypeNode>, rhs: &Arc<TypeNode>) -> bool {
    Arc::ptr_eq(lhs, rhs) || lhs.ty == rhs.ty
}

impl PartialEq for Type {
    /// Compares the parts of the type expression, taking two occurrences of one node as equal on
    /// sight (`type_node_eq`). The derived `Hash` agrees with this, reading the expression a node
    /// holds.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::TyVar(lhs), Type::TyVar(rhs)) => lhs == rhs,
            (Type::TyCon(lhs), Type::TyCon(rhs)) => lhs == rhs,
            (Type::TyApp(lhs_fun, lhs_arg), Type::TyApp(rhs_fun, rhs_arg)) => {
                type_node_eq(lhs_fun, rhs_fun) && type_node_eq(lhs_arg, rhs_arg)
            }
            (Type::AssocTy(lhs_assoc_ty, lhs_args), Type::AssocTy(rhs_assoc_ty, rhs_args)) => {
                lhs_assoc_ty == rhs_assoc_ty
                    && lhs_args.len() == rhs_args.len()
                    && lhs_args
                        .iter()
                        .zip(rhs_args.iter())
                        .all(|(lhs, rhs)| type_node_eq(lhs, rhs))
            }
            _ => false,
        }
    }
}

impl TypeNode {
    /// This type written in source syntax, with its type variables renamed `a`, `b`, ... in order
    /// of appearance, so that two types differing only in the names of their variables are written
    /// alike.
    pub fn to_string_normalize(self: &Arc<TypeNode>) -> String {
        TypeNode::to_string_normalize_many(&[self.clone()])
            .pop()
            .unwrap()
    }

    /// The types written in source syntax under one renaming of their type variables to `a`, `b`,
    /// ... in order of appearance across the whole list, so that a variable two of them share is
    /// written with one name in both.
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
            let merge_ok = s.merge(&Substitution::single(
                &fv.name,
                type_tyvar(&new_name, &fv.kind),
            ));
            assert!(merge_ok, "`{}` is renamed twice.", fv.name);
            next_tyvar_no += 1;
        }

        // Substitute and stringify all types.
        tys.iter()
            .map(|ty| s.substitute_type(ty).to_string())
            .collect()
    }

    /// This type written in source syntax, under the names its own type variables carry.
    ///
    /// # Examples
    /// A saturated tuple is written `(a, b)`, a saturated arrow `a -> b`, and an argument that is
    /// itself an application stands in parentheses, as in `Array (Option a)`.
    pub fn to_string(self: &Arc<TypeNode>) -> String {
        /// Whether `arg`, standing as the argument of an application, is written in parentheses.
        fn should_braced_as_arg(arg: &Arc<TypeNode>) -> bool {
            match &arg.ty {
                Type::TyVar(_) => false,
                Type::TyCon(_) => false,
                Type::TyApp(fun, _) => {
                    let tycon = fun.toplevel_tycon();
                    if let Some(tycon) = tycon {
                        if let Some(tuple_n) = get_tuple_n(&tycon.name) {
                            return tuple_n as usize != arg.collect_type_arguments().len();
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
                        let args = self.collect_type_arguments();
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
                        let args = self.collect_type_arguments();
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
        let mut key = "".to_string();
        key += &self.to_string_normalize();
        if capture.len() > 0 {
            key += "_capturing[";
        }
        for ty in capture {
            key += ", ";
            key += &ty.to_string_normalize();
        }
        if capture.len() > 0 {
            key += "]";
        }
        format!("{:x}", md5::compute(key))
    }

    /// A digest of this type, short enough to embed in a symbol name. Two types with the same
    /// normalized form hash alike.
    pub fn hash(self: &Arc<TypeNode>) -> String {
        let type_string = self.to_string_normalize();
        format!("{:x}", md5::compute(type_string))
    }

    /// The trait constraints this type has to meet to be well-formed: each associated type
    /// application standing in it asks that its first argument implement the trait declaring it.
    ///
    /// # Examples
    /// `Elem c`, where `Collects` declares `Elem`, gives `c : Collects`.
    #[allow(dead_code)]
    pub fn predicates_from_associated_types(&self) -> Vec<Predicate> {
        /// Appends to `buf` the constraint of each associated type application standing in `ty`.
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

/// The kind `*`, of a type that has values of its own.
pub fn kind_star() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

/// The kind of a type constructor taking a type of kind `src` to a type of kind `dst`.
pub fn kind_arrow(src: Arc<Kind>, dst: Arc<Kind>) -> Arc<Kind> {
    Arc::new(Kind::Arrow(src, dst))
}

/// A type variable named `var_name`, standing for types of kind `kind`.
pub fn make_tyvar(var_name: &str, kind: &Arc<Kind>) -> Arc<TyVar> {
    Arc::new(TyVar {
        name: String::from(var_name),
        kind: kind.clone(),
    })
}

/// The type that is the type variable named `var_name`, standing for types of kind `kind`.
pub fn type_tyvar(var_name: &str, kind: &Arc<Kind>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(make_tyvar(var_name, kind)))
}

/// The type that is the type variable named `var_name`, standing for types of kind `*`.
pub fn type_tyvar_star(var_name: &str) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyVar(make_tyvar(var_name, &kind_star())))
}

/// The type that is the type variable `tyvar`.
pub fn type_from_tyvar(tyvar: Arc<TyVar>) -> Arc<TypeNode> {
    let ty = TypeNode::new(Type::TyVar(tyvar.clone()));
    Arc::new(ty)
}

/// The function type `src -> dst`.
pub fn type_fun(src: Arc<TypeNode>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    type_fun_with_arrow_src(src, dst, None)
}

/// The function type `src -> dst`, with the `->` itself written at `arrow_src`.
///
/// # Arguments
/// * `arrow_src` — where the `->` itself was written, which a report drawn on the arrow points
///   at.
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

/// The function pointer type taking `srcs` to `dst`, i.e. `Std::#FunPtr{n}` applied to them,
/// where `n` is the number of arguments.
pub fn type_funptr(srcs: Vec<Arc<TypeNode>>, dst: Arc<TypeNode>) -> Arc<TypeNode> {
    let mut ty = TypeNode::new_arc(Type::TyCon(Arc::new(make_funptr_tycon(srcs.len() as u32))));
    for src in srcs {
        ty = type_tyapp(ty, src);
    }
    ty = type_tyapp(ty, dst);
    ty
}

/// The type `tyfun` applied to `param`.
pub fn type_tyapp(tyfun: Arc<TypeNode>, param: Arc<TypeNode>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyApp(tyfun, param))
}

/// The associated type `assoc_ty` applied to `args`, the first of which is the type the trait is
/// implemented for.
pub fn type_assocty(assoc_ty: AssocType, args: Vec<Arc<TypeNode>>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::AssocTy(assoc_ty, args))
}

/// The type that is the type constructor `tycon`, with no argument applied.
pub fn type_tycon(tycon: &Arc<TyCon>) -> Arc<TypeNode> {
    TypeNode::new_arc(Type::TyCon(tycon.clone()))
}

/// The type constructor named `name`.
pub fn tycon(name: FullName) -> Arc<TyCon> {
    Arc::new(TyCon { name })
}

/// The TyCon applied to the given type arguments, in order.
///
/// # Examples
/// `apply_type_args(Array, [I64])` is `Array I64`, and `apply_type_args(Array, [])` is `Array`.
pub fn apply_type_args(tycon: &Arc<TyCon>, args: &[Arc<TypeNode>]) -> Arc<TypeNode> {
    let mut applied = type_tycon(tycon);
    for arg in args {
        applied = type_tyapp(applied, arg.clone());
    }
    applied
}

/// What a type node carries beside the type itself.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    /// The span of the source text the type was written at. A type the compiler builds itself has
    /// none.
    source: Option<Span>,
}

impl TypeNode {
    /// Whether no type variable occurs in this type.
    ///
    /// `free_vars` answers the same question by collecting the variables, which walks a type that
    /// shares a subterm once per occurrence rather than once per node. Every type reaching code
    /// generation is asked this, so it is answered here and kept on the node.
    pub fn is_ground(&self) -> bool {
        *self.ground_cache.get_or_init(|| match &self.ty {
            Type::TyVar(_) => false,
            Type::TyCon(_) => true,
            Type::TyApp(fun, arg) => fun.is_ground() && arg.is_ground(),
            Type::AssocTy(_, args) => args.iter().all(|arg| arg.is_ground()),
        })
    }

    /// How deeply this type nests: a name is one, and an application or an associated type is one
    /// more than the deepest part it is made of.
    ///
    /// This measures the type expression the program wrote or the compiler built: a chain of a
    /// thousand types that each hold the next is a thousand types of depth one. What grows this is
    /// a type reached from itself at a larger type argument.
    pub fn depth(&self) -> usize {
        *self.depth_cache.get_or_init(|| match &self.ty {
            Type::TyVar(_) | Type::TyCon(_) => 1,
            Type::TyApp(fun, arg) => 1 + fun.depth().max(arg.depth()),
            Type::AssocTy(_, args) => {
                1 + args
                    .iter()
                    .map(|arg| arg.depth())
                    .max()
                    .expect("an associated type is applied to at least its implementing type")
            }
        })
    }

    /// The type variables standing in this type, under their names.
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

    /// Appends to `buf` each type variable standing in this type that `buf` lacks, so that every
    /// variable is held once, under the first occurrence met.
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

    /// Appends to `buf` each type variable standing in this type that `buf` lacks, together with
    /// the source it stands at, so that a report about a variable can point at it.
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

    /// The type variables standing in this type, each one once.
    pub fn free_vars_vec(self: &Arc<TypeNode>) -> Vec<Arc<TyVar>> {
        let mut buf = vec![];
        self.free_vars_to_vec(&mut buf);
        buf
    }

    /// Collect into `out` the type variables that are "fixed" in this type, in the sense of `Fixv`
    /// from the section "Well-formed programs" of "Associated Type Synonyms" (Chakravarty, Keller,
    /// Peyton Jones, ICFP '05).
    ///
    /// A type variable is fixed if unifying the type with a ground type would determine it.
    /// Associated type applications are not injective, so their arguments are not fixed; this
    /// function stops recursing into them.
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

    /// Collect into `tycons` all type constructors that appear in this type.
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

    /// Collect into `tyvar_names` the names of all type variables that appear in this type,
    /// arguments of an associated type application included.
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

    /// This type with every name standing in it spelled as an absolute path, so that it names the
    /// same entity from any namespace.
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

/// A type with the constraints it carries, generalized over a set of type variables: the type a
/// global value, a trait member or a trait method implementation is checked against.
#[derive(Clone, Serialize, Deserialize)]
pub struct Scheme {
    /// The type variables the scheme is generalized over; a use site chooses a type for each.
    pub gen_vars: Vec<Arc<TyVar>>,
    /// Kind annotations on type variables, as the signature writes them, e.g. `f : *->*`.
    #[serde(default)]
    pub kind_signs: Vec<KindSignature>,
    /// Trait constraints, e.g. `a : Show`.
    pub predicates: Vec<Predicate>,
    /// Equality constraints on associated types, e.g. `Item c = e`.
    pub equalities: Vec<Equality>,
    /// The type the constraints qualify.
    pub ty: Arc<TypeNode>,
}

impl Scheme {
    /// Reject a scheme whose constraints are in a form the type checker cannot work with, or whose
    /// generalized variables a use site would be left unable to determine.
    ///
    /// The first thing found decides the report.
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
                // An opaque type variable stands there too; an equality on one is checked where
                // `on_opaque_tyvar` holds.
                if !eq.args[0].is_tyvar() {
                    return Err(Errors::from_msg_srcs(
                        "The first argument of the left side of an equality constraint should be a type variable or an opaque type.".to_string(),
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
        // An opaque type variable stands for a type this signature hides from the use site. So an
        // equality naming one has to have an opaque type variable as `args[0]`: such an equality
        // states what a hidden type is like and is given to the use site. An equality on another
        // type states a condition the use site has to meet, and a use site can meet only
        // conditions about types it sees.
        for eq in &self.equalities {
            if eq.on_opaque_tyvar() {
                continue;
            }
            let mut vars = vec![];
            eq.free_vars_to_vec(&mut vars);
            if vars.iter().any(|tv| is_opaque_tyvar(&tv.name)) {
                return Err(Errors::from_msg_srcs(
                    "The first argument of the left side of an equality constraint involving an \
                     opaque type should be an opaque type."
                        .to_string(),
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
        // be "fixed" in the sense of `Fixv` from the section "Well-formed
        // programs" of "Associated Type Synonyms". A variable is fixed iff it
        // appears outside of any associated type application, either in the
        // main type or on the right-hand side of an equality constraint. A
        // variable that only appears under an associated type application (or
        // only in a class predicate) would not be determined by unification at
        // a use site, which would make the scheme ambiguous.
        let fixed_vars = self.fixed_vars();
        // First occurrence wins, which gives a useful span pointing at the
        // offending position.
        let occurrences = self.all_tyvar_occurrences_with_span();
        for gen_var in &self.gen_vars {
            if fixed_vars.contains(&gen_var.name) {
                continue;
            }
            let Some((_, span)) = occurrences.iter().find(|(tv, _)| tv.name == gen_var.name) else {
                // A generalized variable can be absent from the body:
                // `gen_vars` is determined when the scheme is generalized, and
                // expanding a type alias that drops a parameter (`type Ignore
                // a = I64;` used as `f : Ignore a -> I64;`) afterwards removes
                // the variable from the body. Such a variable constrains
                // nothing at a use site, so there is no ambiguity to report.
                continue;
            };
            return Err(unfixed_type_variable_error(
                &gen_var.name,
                "outside of any associated type application",
                span,
            ));
        }

        Ok(())
    }

    /// The scheme written out with `s` applied to its constraints and its type, the constraints
    /// standing in brackets in front of the type.
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

    /// The scheme written out with its generalized variables renamed `a`, `b`, ... in the order
    /// `gen_vars` lists them, so that the names the source happened to use stay out of the text.
    pub fn to_string_normalize(&self) -> String {
        // Change names of generalized type variables to a, b, ...
        let mut s = Substitution::default();
        let mut tyvar_num = -1;
        for tyvar in &self.gen_vars {
            tyvar_num += 1;
            let new_name = number_to_varname(tyvar_num as usize);
            let merge_ok = s.merge(&Substitution::single(
                &tyvar.name,
                type_tyvar(&new_name, &tyvar.kind.clone()),
            ));
            assert!(merge_ok, "`{}` is generalized twice.", tyvar.name);
        }
        self.to_string_substituted(&s)
    }

    /// The scheme written out under the names its own variables carry.
    pub fn to_string(&self) -> String {
        let s = Substitution::default();
        self.to_string_substituted(&s)
    }

    /// Appends to `buf` the type variables standing in the constraints and in the type that the
    /// scheme leaves free, that is, all but the generalized ones.
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

    /// The type variables this scheme's body fixes, in the sense of `Fixv` from the section
    /// "Well-formed programs" of "Associated Type Synonyms".
    ///
    /// The main type `self.ty` contributes, and so does the right-hand side of each equality
    /// constraint; class predicates contribute nothing, per `Fixv (D τ ⇒ ρ) = Fixv ρ`.
    pub fn fixed_vars(&self) -> Set<Name> {
        let mut out = Set::default();
        self.ty.fixed_vars_to_set(&mut out);
        for eq in &self.equalities {
            eq.fixed_vars_to_set(&mut out);
        }
        out
    }

    /// Each type variable standing in the scheme body — in the predicates, in the equalities and in
    /// the main type — with the source of its first occurrence, generalized variables included.
    ///
    /// A report about one variable of a signature reads the place to point at from here.
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

    /// The same scheme with every type variable carrying its kind, taken from the scheme's own kind
    /// signatures and from the kinds the traits its predicates and equalities name demand.
    pub fn set_kinds(&self, kind_env: &KindEnv) -> Result<Arc<Scheme>, Errors> {
        let mut ret = self.clone();
        let mut kind_scope = KindScope::new();
        // Insert user-specified kind annotations from kind_signs.
        for ks in &self.kind_signs {
            kind_scope
                .insert(ks.tyvar.clone(), ks.kind.clone())
                .map_err(|msg| Errors::from_msg_srcs(msg, &[&ret.ty.get_source()]))?;
        }
        let extend_result = kind_scope.extend(&ret.predicates, &ret.equalities, &vec![], kind_env);
        if let Err(msg) = extend_result {
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

    /// Check that the kinds the constraints and the type demand of each type variable agree.
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

    /// The scheme generalized over exactly `vars`.
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

    /// The scheme generalized over the type variables standing in the constraints and in the type.
    ///
    /// An opaque type variable stays free, and so does a variable serving as a formal parameter of
    /// an equality on one: the definition determines those, and a use site chooses nothing for
    /// them.
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

    /// The scheme of `ty` alone, carrying no constraint and generalized over nothing.
    pub fn from_type(ty: Arc<TypeNode>) -> Arc<Scheme> {
        Scheme::new_arc(vec![], vec![], vec![], vec![], ty)
    }

    /// The scheme with every trait, type and associated type named in its constraints and in its
    /// type given its full name, read in the context the signature is written in.
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

    /// The scheme with every type alias standing in its constraints and in its type expanded.
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

    /// Find the minimum node which includes the specified source code position.
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

    /// The scheme with every global name in it written as an absolute path.
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

/// The report that a type signature leaves `tyvar_name` undetermined, so that a use site cannot
/// tell which types the signature is instantiated at.
///
/// # Arguments
/// * `where_it_must_appear` — the places the variable may stand in for that signature to determine
///   it, as a phrase completing "`x` must appear ...".
/// * `src` — where to draw the report; the variable's own occurrence where the signature has one.
pub fn unfixed_type_variable_error(
    tyvar_name: &Name,
    where_it_must_appear: &str,
    src: &Option<Span>,
) -> Errors {
    Errors::from_msg_srcs(
        format!(
            "Type variable `{}` is not fixed by this type signature, which makes it ambiguous. \
             NOTE: `{}` must appear {}.",
            tyvar_name, tyvar_name, where_it_must_appear,
        ),
        &[src],
    )
}

/// Whether a type variable name represents an opaque type variable, which a source line writes
/// with a leading `?`.
pub fn is_opaque_tyvar(name: &str) -> bool {
    name.starts_with('?')
}

/// Whether a type variable name represents a `_` type wildcard, which carries
/// `TYPE_WILDCARD_VAR_PREFIX` in front of it.
pub fn is_type_wildcard_tyvar(name: &str) -> bool {
    name.starts_with(TYPE_WILDCARD_VAR_PREFIX)
}

/// The type variables standing in `preds`, `eqs` and `ty`, each one once.
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

/// Mapping from an opaque TyCon application to the concrete type inferred by type-checking.
///
/// Example: for `repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it`,
/// after desugaring and type-checking:
///   lhs = `?it a`
///   rhs = `MapIterator (RangeIterator I64) a`
///
/// For a trait impl like `impl Array a : ToIter`:
///   lhs = `?it (Array a)`
///   rhs = `ArrayIterator a`
#[derive(Clone, Serialize, Deserialize)]
pub struct OpaqueTyConResolution {
    /// Opaque TyCon applied to type arguments.
    /// E.g., `?it a` for a simple value, `?it (Array a)` for a trait impl.
    pub lhs: Arc<TypeNode>,
    /// The concrete type. E.g., `MapIterator (RangeIterator I64) a`.
    /// None until type-checking resolves it.
    pub rhs: Option<Arc<TypeNode>>,
    /// The source of the definition whose type-checking fills in `rhs`: the declaration of a
    /// simple value, and the member's name in the implementation for a trait member.
    pub src: Option<Span>,
}

#[cfg(test)]
mod tests {
    use super::{kind_arrow, kind_star, make_tyvar, type_tyapp, TypeNode};
    use crate::fixstd::builtin::{make_array_ty, make_i64_ty};
    use crate::misc::Set;
    use crate::parse::sourcefile::{SourceFile, Span};
    use std::path::PathBuf;
    use std::sync::Arc;

    /// A span over a source file held in memory, at `start`.
    fn span_at(start: usize) -> Option<Span> {
        let source = SourceFile::from_file_path_and_content(
            PathBuf::from("written_here.fix"),
            "type T = I64;\n".to_string(),
        );
        Some(Span {
            input: source,
            start,
            end: start + 3,
        })
    }

    /// `Array I64` with the element node written at `start`.
    fn array_of_i64_written_at(start: usize) -> Arc<TypeNode> {
        type_tyapp(make_array_ty(), make_i64_ty().set_source(span_at(start)))
    }

    /// A type serializes to its type expression, which is what `PartialEq` compares and `Hash`
    /// hashes: two nodes for one type written at different places serialize alike.
    ///
    /// The digest naming a compilation unit's object file serializes the RC IR
    /// (`divide_program::generated_code_hash`), and a type reaches it inside values that give no
    /// way to take the type out — an inline-LLVM op holds the types of a closure's captures behind
    /// a trait object. So a position in the serialized form of a type makes an edit that shifts it
    /// regenerate a unit whose code that edit leaves as it was.
    #[test]
    fn test_a_type_serializes_to_its_expression() {
        let bytes = |ty: &Arc<TypeNode>| postcard::to_allocvec(ty).unwrap();

        let nowhere = type_tyapp(make_array_ty(), make_i64_ty());
        let here = array_of_i64_written_at(0);
        let further_down = array_of_i64_written_at(7);

        assert_eq!(
            bytes(&here),
            bytes(&further_down),
            "a type written further down its file serialized differently"
        );
        assert_eq!(
            bytes(&here),
            bytes(&nowhere),
            "a type written in a source file serialized differently from one the compiler built"
        );
    }

    /// Two type variables of one name are one variable whatever kinds they carry, and hashing agrees
    /// with that.
    ///
    /// The kind a variable carries is set later than the variable itself, so a container keyed by a
    /// type would otherwise hold one variable under two keys, one of them stale.
    #[test]
    fn a_type_variable_is_identified_by_its_name_alone() {
        let star = make_tyvar("a", &kind_star());
        let higher = make_tyvar("a", &kind_arrow(kind_star(), kind_star()));
        let other_name = make_tyvar("b", &kind_star());

        assert!(star == higher, "`a : *` and `a : *->*` are one variable.");
        assert!(star != other_name, "`a` and `b` are two variables.");

        let mut set = Set::default();
        set.insert(star.clone());
        set.insert(higher.clone());
        set.insert(other_name.clone());
        assert_eq!(set.len(), 2);
    }
}
