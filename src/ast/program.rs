use crate::ast::deprecation::{DeprecationInfo, DeprecationStatement};
use crate::ast::equality::Equality;
use crate::ast::export_statement::{ExportStatement, ExportedFunctionType, IOType};
use crate::ast::expr::{expr_var, Expr, ExprNode, Var};
use crate::ast::import::{is_accessible, ImportItem, ImportStatement};
use crate::ast::kind_scope::KindEnv;
use crate::ast::name::{FullName, Name, NameSpace};
use crate::ast::pattern::PatternNode;
use crate::ast::traits::{TraitAlias, TraitDefn, TraitEnv, TraitId, TraitImpl};
use crate::ast::typedecl::{Field, TypeDeclValue, TypeDefn};
use crate::ast::types::{
    is_opaque_tyvar, AssocType, Kind, OpaqueTyConResolution, Scheme, TyAliasInfo, TyCon, TyConInfo,
    TyConVariant, TypeNode,
};
use crate::configuration::{Configuration, DeprecationMode, SubCommand};
use crate::constants::{
    DOT_FIXLANG, INSTANCIATED_NAME_SEPARATOR, MAIN_FUNCTION_NAME, MAIN_MODULE_NAME,
    MARK_THREADED_NAME, STD_NAME, STRUCT_ACT_SYMBOL, STRUCT_GETTER_SYMBOL, STRUCT_MODIFIER_SYMBOL,
    STRUCT_PLUG_IN_FORCE_UNIQUE_SYMBOL, STRUCT_PLUG_IN_SYMBOL, STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL,
    STRUCT_PUNCH_SYMBOL, STRUCT_SETTER_SYMBOL, TEST_FUNCTION_NAME, TEST_MODULE_NAME,
    TUPLE_SIZE_BASE, UNION_AS_SYMBOL, UNION_IS_SYMBOL, UNION_MOD_SYMBOL,
};
use crate::elaboration::desugar_opaque::{
    remove_opaque_wrapper_func, resolve_opaque_tycon_in_expr, resolve_opaque_type_in_type,
};
use crate::elaboration::name_resolution::{NameResolutionContext, NameResolutionEnv};
use crate::elaboration::typecheck::TypeCheckContext;
use crate::error::{panic_if_err, Error, Errors, WARN_DEPRECATED};
use crate::fixstd::builtin::{
    boxed_trait_instance, bulitin_tycons, make_io_unit_ty, make_unit_ty, struct_act,
    struct_act_const, struct_act_identity, struct_act_tuple2, struct_get, struct_mod,
    struct_plug_in, struct_punch, struct_set, tuple_defn, union_as, union_is, union_mod_function,
    union_new,
};
use crate::graph::Graph;
use crate::misc::{
    collect_results, insert_to_map_vec_many, join_compiler_threads, spawn_compiler_thread,
    to_absolute_path, HashSource, Map, Set,
};
use crate::parse::sourcefile::{SourceFile, SourcePos, Span};
use crate::printer::Text;
use crate::type_size::{no_size_reason, LayoutWalk};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::mem::replace;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::vec;

/// What a program declares about its types: the type constructors and the type aliases it can name,
/// and which of the newtypes among them a value has stopped being built at.
#[derive(Clone)]
pub struct TypeEnv {
    /// The declaration of every type constructor, built-in and user-defined, by its name.
    ///
    /// Private, because the field types held here answer what a value of a type is laid out as, and
    /// `unwrap_newtypes` puts them in a form the rest of the compiler relies on. A declaration
    /// enters through `add_tycons`, which puts it in that same form.
    tycons: Arc<Map<TyCon, TyConInfo>>,
    /// The declaration of every type alias, by its name.
    pub aliases: Arc<Map<TyCon, TyAliasInfo>>,
    /// The newtypes a value of which has become a value of its one field. Empty until the pass that
    /// unwraps newtypes runs.
    ///
    /// A field type this environment reports has none of these saturated in it, so a value is never
    /// built at the struct one of them was declared as. The declarations stay, because a newtype
    /// carried by a higher-kinded type variable occurs without its arguments, and such an occurrence
    /// still names one.
    unwrapped_newtypes: Arc<Set<TyCon>>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self {
            tycons: Arc::new(Default::default()),
            aliases: Arc::new(Default::default()),
            unwrapped_newtypes: Arc::new(Default::default()),
        }
    }
}

impl TypeEnv {
    /// An environment holding `tycons` and `aliases` as declared, with every newtype among them
    /// still a type values are built at.
    pub fn new(tycons: Map<TyCon, TyConInfo>, aliases: Map<TyCon, TyAliasInfo>) -> TypeEnv {
        TypeEnv {
            tycons: Arc::new(tycons),
            aliases: Arc::new(aliases),
            unwrapped_newtypes: Arc::new(Default::default()),
        }
    }

    /// Makes a value of each newtype in `newtypes` a value of its one field: records them, then
    /// rewrites the stored declarations so that no field type this environment reports has one of
    /// them saturated in it.
    ///
    /// The declarations are read as they stand while they are rewritten, so each one is unwrapped
    /// from the same starting point.
    ///
    /// Every newtype recorded is one this environment declares, which is what lets
    /// `unwrapped_newtype_info` answer with a declaration rather than with the possibility of one.
    pub fn unwrap_newtypes(&mut self, newtypes: Set<TyCon>) {
        for tycon in &newtypes {
            assert!(
                self.tycons.contains_key(tycon),
                "`{}` is unwrapped, though this environment holds no declaration of it.",
                tycon.to_string()
            );
        }
        self.unwrapped_newtypes = Arc::new(newtypes);
        let declared_type_env = self.clone();
        let mut rewritten = self.tycons.as_ref().clone();
        for (_tycon, tycon_info) in &mut rewritten {
            for field in &mut tycon_info.fields {
                field.ty = field.ty.unwrap_newtypes(&declared_type_env);
            }
        }
        self.tycons = Arc::new(rewritten);
    }

    /// The declaration of `tycon` if a value of it has become a value of its one field, and `None`
    /// otherwise. A recorded newtype is one this environment declares, which `unwrap_newtypes`
    /// states where it records them.
    pub fn unwrapped_newtype_info(&self, tycon: &TyCon) -> Option<&TyConInfo> {
        if !self.unwrapped_newtypes.contains(tycon) {
            return None;
        }
        Some(self.tycons.get(tycon).unwrap())
    }

    /// Whether a value of `tycon` has become a value of its one field.
    pub fn is_unwrapped_newtype(&self, tycon: &TyCon) -> bool {
        self.unwrapped_newtypes.contains(tycon)
    }

    /// Adds each declaration of `new_tycons` to this environment, replacing the one already held
    /// under the same name, each with its field types unwrapped, so that a declaration minted after
    /// the newtype-unwrapping pass answers as the ones that were there before it do.
    pub fn add_tycons(&mut self, new_tycons: Map<TyCon, TyConInfo>) {
        let declared_type_env = self.clone();
        let mut tycons = self.tycons.as_ref().clone();
        for (tycon, mut tycon_info) in new_tycons.into_iter() {
            for field in &mut tycon_info.fields {
                field.ty = field.ty.unwrap_newtypes(&declared_type_env);
            }
            tycons.insert(tycon, tycon_info);
        }
        self.tycons = Arc::new(tycons);
    }

    /// The declaration of every type constructor this environment holds, by its name.
    pub fn tycons(&self) -> &Map<TyCon, TyConInfo> {
        &self.tycons
    }

    /// The kind of every name this environment gives a meaning to, type constructors and type
    /// aliases together in one table.
    pub fn kinds(&self) -> Map<TyCon, Arc<Kind>> {
        let mut res = Map::default();
        for (tc, ti) in self.tycons.as_ref().iter() {
            res.insert(tc.clone(), ti.kind.clone());
        }
        for (tc, ta) in self.aliases.as_ref().iter() {
            res.insert(tc.clone(), ta.kind.clone());
        }
        res
    }

    /// The struct and the field that `name` is the `act_{field}` function of: `name` is a global
    /// name whose namespace is a struct this environment declares, and whose last component names
    /// one of that struct's fields.
    ///
    /// The answer comes from the name alone, so ask it while a value of the struct is still built as
    /// that struct. A newtype keeps its declaration after `unwrap_newtypes` records it, so this
    /// still names the struct of a newtype whose values have become values of its one field.
    pub fn is_struct_act(&self, name: &FullName) -> Option<(TyCon, Name)> {
        if name.is_local() {
            return None;
        }
        let str_name = name.namespace.clone().to_fullname();
        let str_name = TyCon { name: str_name };
        match self.tycons.get(&str_name) {
            Some(tycon_info) => {
                if tycon_info.variant != TyConVariant::Struct {
                    return None;
                }
                for field in &tycon_info.fields {
                    let act_func_name = format!("{}{}", STRUCT_ACT_SYMBOL, field.name);
                    if act_func_name == name.name {
                        return Some((str_name, field.name.clone()));
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Replace every type alias written in the definition of a type constructor of this environment
    /// by the type it stands for, so that a stage reading a field or variant type meets no alias.
    pub fn resolve_type_aliases_in_tycons(&mut self) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        let type_env = self.clone();
        let mut tycons = (*self.tycons).clone();
        for (_, ti) in &mut tycons {
            errors.eat_err(ti.resolve_type_aliases(&type_env));
        }
        errors.to_result()?; // Throw errors if any.
        self.tycons = Arc::new(tycons);
        Ok(())
    }
}

/// A Fix value at one concrete type, under a name of its own. A generic definition becomes one
/// symbol per type it is used at, and the program that reaches code generation is made of these.
#[derive(Clone)]
pub struct Symbol {
    /// The name this symbol is known by, unique across the program. Instantiation builds it from
    /// `generic_name` and a hash of `ty` (`determine_symbol_name`); a pass that mints a symbol
    /// appends a segment of its own.
    pub name: FullName,
    /// The name of the global value this symbol is an instantiation of, shared by the instantiations
    /// of it at every type.
    pub generic_name: FullName,
    /// The type this symbol stands at. It holds no type variable: a value whose type is still open
    /// after instantiation is an error.
    pub ty: Arc<TypeNode>,
    /// The expression computing the value, specialized to `ty`. `None` between the moment the
    /// instantiation is required and the moment `instantiate_symbol` fills it in.
    pub expr: Option<Arc<ExprNode>>,
    /// Whether the back end is asked to inline every call of this global. Everything that builds a
    /// symbol leaves it `false`, so no request reaches the back end; the field and the path that
    /// carries it to code generation are here for a pass that decides to make one.
    pub inline_into_callers: bool,
    // If you add new fields, be sure to update `hash()` method.
}

impl Symbol {
    /// The set of modules that this symbol depends on directly.
    /// If any of these modules, or any of their importee are changed, then they are required to be re-compiled.
    /// The full set of modules a change can reach is obtained by walking the importing graph from
    /// this set.
    pub fn dependent_modules(&self) -> Set<Name> {
        let mut dep_mods = Set::default();
        dep_mods.insert(self.name.module());
        self.ty.define_modules_of_tycons(&mut dep_mods);
        dep_mods
        // Even for implemented trait methods, it is enough to add the module where the trait is defined and the modules where the types of the symbol are defined.
        // This is because,
        // - By orphan rule, trait implementations are given in the module where the trait is defined, or the module where the type is defined.
        // - Moreover, we forbid unrelated trait implementation (see `test_unrelated_trait_method()`),
        // so the type the trait is implemented appears in the type of the symbol.
    }

    /// The MD5 hash of everything about this symbol that decides the code generated for it — its
    /// name, its type, its expression, and what it asks of the back end — in hexadecimal.
    pub fn hash(&self) -> String {
        let mut hash_source = String::new();
        hash_source.push_str("<name>");
        hash_source.push_str(&self.name.to_string());

        hash_source.push_str("<type>");
        hash_source.push_str(&self.ty.to_string());

        hash_source.push_str("<expr>");
        if let Some(expr) = &self.expr {
            hash_source.push_str(&expr.expr.stringify().to_string());
        }

        hash_source.push_str("<inline_into_callers>");
        hash_source.push_str(&self.inline_into_callers.to_string());

        format!("{:x}", md5::compute(hash_source))
    }
}

/// Declaration (name and its type) of global value.
/// e.g., `main : IO()`
pub struct GlobalValueDecl {
    /// The declared name.
    pub name: FullName,
    /// The declared type scheme.
    pub ty: Arc<Scheme>,
    /// The left hand side of the declaration of this value,
    /// e.g., `main` in `main : IO ()`.
    pub src: Option<Span>,
}

/// Definition (name and its value) of global value.
/// e.g., `main = println("Hello World")`
pub struct GlobalValueDefn {
    /// The defined name.
    pub name: FullName,
    /// The expression the name is bound to.
    pub expr: Arc<ExprNode>,
    /// The left hand side of the definition of this value,
    /// e.g., `main` in `main = println("Hello World")`.
    pub src: Option<Span>,
}

/// The global value, which is either a value or trait method.
pub struct GlobalValue {
    /// Type of this symbol.
    /// For example, in case `trait a : Show { show : a -> String; }`, the type of method `show` is `[a : Show] a -> String`.
    pub scm: Arc<Scheme>,
    /// Type of this symbol, with aliases retained.
    pub syn_scm: Option<Arc<Scheme>>,
    /// The expression or implementation of this value.
    pub expr: SymbolExpr,
    /// Source code where this value is declared.
    ///
    /// This is the left hand side of the declaration of this value,
    /// e.g., `main` in `main : IO ()`.
    ///
    /// For a trait method, this is the source code of the member declaration in the trait
    /// definition.
    pub decl_src: Option<Span>,
    /// The source code position of the left hand side of the definition of this value.
    /// For example, if there is a definition `main = println("Hello World")`, this is the position of `main`.
    /// If the definition is written together with the declaration, e.g., `main : IO () = println("Hello World")`,
    /// this is the same as `decl_src`.
    /// For trait members, this is also the same as `decl_src`.
    pub defn_src: Option<Span>,
    /// The document of this value.
    /// This field carries the document of a value whose `decl_src` is unavailable; otherwise the
    /// document is read from the source code.
    pub document: Option<String>,
    /// Is this value compiler-defined method?
    /// True for methods such as `@{field}`, `set_{field}`, etc.
    /// Such a value is omitted from the document generated by `fix docs`.
    pub compiler_defined_method: bool,
    /// Deprecation metadata, set during elaboration when a matching
    /// `DEPRECATED[...]` pragma exists.
    pub deprecation: Option<DeprecationInfo>,
}

impl GlobalValue {
    pub fn resolve_namespace_in_declaration(
        &mut self,
        ctx: &mut NameResolutionContext,
    ) -> Result<(), Errors> {
        // Currently, global values generated from member implementations do not come here.
        // This is because name resolution is performed on TraitEnv, and then global values are generated from trait member implementations.
        assert!(matches!(self.expr, SymbolExpr::Simple(_)));
        self.scm = self.scm.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.syn_scm = Some(self.scm.clone());
        self.scm = self.scm.resolve_type_aliases(type_env)?;
        self.expr.resolve_type_aliases(type_env)?;
        Ok(())
    }

    pub fn set_kinds(&mut self, kind_env: &KindEnv) -> Result<(), Errors> {
        self.scm = self.scm.set_kinds(kind_env)?;
        self.scm.check_kinds(kind_env)?;
        match &mut self.expr {
            SymbolExpr::Simple(_) => {}
            SymbolExpr::Method(ms) => {
                for m in ms {
                    m.scm = m.scm.set_kinds(kind_env)?;
                    m.scm.check_kinds(kind_env)?;
                    m.scm_via_defn = m.scm_via_defn.set_kinds(kind_env)?;
                    m.scm_via_defn.check_kinds(kind_env)?;
                }
            }
        }
        Ok(())
    }

    // Check if this value is a simple value, not a trait method.
    pub fn is_simple_value(&self) -> bool {
        matches!(self.expr, SymbolExpr::Simple(_))
    }

    // Get the document of this value.
    pub fn get_document(&self) -> Option<String> {
        // Try to get document from the source code.
        let docs = self
            .decl_src
            .as_ref()
            .and_then(|src| src.get_document().ok());

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

    // Find the minimum node which includes the specified source code position.
    // - `name`: the name of this global value (i.e., the key in `Program::global_values`).
    pub fn find_node_at(&self, name: &FullName, pos: &SourcePos) -> Option<EndNode> {
        let node = self.expr.find_node_at(name, pos);
        if node.is_some() {
            return node;
        }
        // Walk the syntactic scheme if available so that type aliases written
        // in source resolve back to the alias name (rather than the expanded
        // type, which is what `scm` carries).
        let scm_for_lookup = self.syn_scm.as_ref().unwrap_or(&self.scm);
        let node = scm_for_lookup.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        if let Some(ref span) = self.decl_src {
            if span.includes_pos_lsp(pos) {
                return Some(EndNode::ValueDecl(name.clone()));
            }
        }
        if let Some(ref span) = self.defn_src {
            if span.includes_pos_lsp(pos) {
                return Some(EndNode::ValueDecl(name.clone()));
            }
        }
        None
    }
}

// Expression of global symbol.
#[derive(Clone)]
pub enum SymbolExpr {
    Simple(TypedExpr),            // Definition such as "id : a -> a; id = \x -> x".
    Method(Vec<TraitMemberImpl>), // Trait member implementations.
}

impl SymbolExpr {
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        match self {
            SymbolExpr::Simple(_) => Ok(()),
            SymbolExpr::Method(impls) => {
                let mut errors = Errors::empty();
                for method_impl in impls {
                    errors.eat_err(method_impl.resolve_type_aliases(type_env));
                }
                errors.to_result()
            }
        }
    }

    #[allow(dead_code)]
    pub fn source(&self) -> Option<Span> {
        match self {
            SymbolExpr::Simple(e) => e.expr.source.clone(),
            SymbolExpr::Method(ms) => ms.first().map(|m| m.expr.expr.source.clone()).flatten(),
        }
    }

    // Find the minimum expression node which includes the specified source code position.
    // - `name`: the name of the global value (e.g., `Std::ToString::to_string`), used to return `EndNode::ValueDecl` when
    //   the cursor is on the LHS of a trait member implementation.
    pub fn find_node_at(&self, name: &FullName, pos: &SourcePos) -> Option<EndNode> {
        match self {
            SymbolExpr::Simple(e) => e.find_node_at(pos),
            SymbolExpr::Method(ms) => ms.iter().filter_map(|m| m.find_node_at(name, pos)).next(),
        }
    }

    /// Visit every `Expr::Var` occurrence inside this symbol's expression(s).
    /// For `Method`, walks every per-impl expression in turn.
    pub fn walk_var_uses<F: FnMut(&Var, &Option<Span>)>(&self, f: &mut F) {
        match self {
            SymbolExpr::Simple(te) => te.expr.walk_var_uses(f),
            SymbolExpr::Method(impls) => {
                for impl_ in impls {
                    impl_.expr.expr.walk_var_uses(f);
                }
            }
        }
    }

    /// Visit every pattern (in `Let` / `Match` arms) inside this symbol's
    /// expression(s).
    pub fn walk_patterns<F: FnMut(&Arc<PatternNode>)>(&self, f: &mut F) {
        match self {
            SymbolExpr::Simple(te) => te.expr.walk_patterns(f),
            SymbolExpr::Method(impls) => {
                for impl_ in impls {
                    impl_.expr.expr.walk_patterns(f);
                }
            }
        }
    }
}

// The expression with all sub-expressions typed.
#[derive(Clone, Serialize, Deserialize)]
pub struct TypedExpr {
    // The expression.
    //
    // It and its all subexpressions has their types resolved, and these types contains only ones that appear in the context (type signature) of this expression.
    pub expr: Arc<ExprNode>,
    // Equalities to be assumed in the context of this expression.
    //
    // For example, consider the following expression:
    // ```
    // extend : [c1 : Collects, c2 : Collects, Elem c1 = e, Elem c2 = e] c1 -> c2 -> c2;
    // extend = |xs, ys| xs.to_iter.fold(ys, |ys, x| ys.insert(x));
    // ```
    // In this case, the `equalities` field consists of two equalities: `Elem c1 = e` and `Elem c2 = e`.
    //
    // In fact, this information is neccesary to instantiate the typed expression to a concrete type:
    // In the above case, the sub-expression `x` has type `e` (not `Elem c1` or `Elem c2`).
    // When instantiating this typed expression to a concrete type, e.g., `extend : Array I64 -> Array I64 -> Array I64`,
    // we need to use the equality `Elem c1 = e` to prove that `x` has type `I64`.
    pub equalities: Vec<Equality>,
    // Concrete types for opaque type constructors in this expression.
    #[serde(default)]
    pub opaque_types: Map<FullName, Vec<OpaqueTyConResolution>>,
}

impl TypedExpr {
    pub fn from_expr(expr: Arc<ExprNode>) -> Self {
        TypedExpr {
            expr,
            equalities: vec![],
            opaque_types: Map::default(),
        }
    }

    // Find the minimum expression node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        let node = self.expr.find_node_at(pos);
        if node.is_none() {
            return None;
        }
        let node = node.unwrap();
        Some(node)
    }
}

// Trait member implementation
#[derive(Clone)]
pub struct TraitMemberImpl {
    // Type of this member.
    //
    // For example, in case "impl [a: ToString, b: ToString] (a, b): ToString {...}",
    // the type of member "to_string" is "[a: ToString, b: ToString] (a, b) -> String",
    //
    // Users can give type signatures in each trait member implementation.
    // In this case, the `scm` field contains the type signature given by users.
    pub scm: Arc<Scheme>,
    // This field holds the type scheme obtained from the trait member definition.
    pub scm_via_defn: Arc<Scheme>,
    // Expression of this implementation
    pub expr: TypedExpr,
    // Module where this implmentation is given.
    // NOTE:
    // For trait member, `define_module` may differ to the first component of namespace of the function.
    // For example, if `Main` module implements `SomeType : Eq`, then implementation of `eq` for `SomeType` is defined in `Main` module,
    // but its name as a function is still `Std::Eq::eq`.
    pub define_module: Name,
    // The source spans of the left-hand side names in the trait member implementation.
    // For example, in `impl MyType : ToString { to_string : MyType -> String; to_string = ...; }`,
    // this contains spans of both `to_string` occurrences (type signature and definition).
    pub lhs_srcs: Vec<Span>,
}

impl TraitMemberImpl {
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.scm = self.scm.resolve_type_aliases(type_env)?;
        self.scm_via_defn = self.scm_via_defn.resolve_type_aliases(type_env)?;
        Ok(())
    }

    // Find the minimum expression node which includes the specified source code position.
    // - `name`: the name of the global value (e.g., `Std::ToString::to_string`), used to return
    //   `EndNode::ValueDecl` when the cursor is on the LHS of this trait member implementation.
    pub fn find_node_at(&self, name: &FullName, pos: &SourcePos) -> Option<EndNode> {
        let node = self.expr.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        for span in &self.lhs_srcs {
            if span.includes_pos_lsp(pos) {
                return Some(EndNode::ValueDecl(name.clone()));
            }
        }
        None
    }
}

/// A module of the program, and the sources it is made of.
#[derive(Clone)]
pub struct ModuleInfo {
    /// The name of the module.
    pub name: Name,
    /// The `module` declaration the module is defined by.
    pub source: Span,
    /// The sources that extend the module beyond the one it is declared in, in the order they were
    /// linked. Shared, so that cloning a module — which `TypeCheckContext` does for every
    /// speculative check — copies none of them.
    ///
    /// A source reaches this list by being linked with `Program::link`'s `extend` set, which is how
    /// the compiler adds the definitions it writes itself to `Std`: the trait implementations for
    /// the tuple sizes the program uses (`make_tuple_traits_mod`) and the traits that convert
    /// between numeric types (`make_numeric_cast_traits_mod`). `module_dependency_hash` folds them
    /// beside the module's own source, because a value defined in one of them is as much a function
    /// of that source as a value defined in the file the module is declared in.
    pub extending_sources: Arc<Vec<SourceFile>>,
}

impl ModuleInfo {
    /// Every source the module is made of: the one it is declared in, then the ones that extend it.
    pub fn sources(&self) -> impl Iterator<Item = &SourceFile> {
        std::iter::once(&self.source.input).chain(self.extending_sources.iter())
    }

    /// A hash of each source the module is made of, in the order `sources` gives them. A hash naming
    /// what a module is made of is a list of these, so that a source belongs to the module it
    /// extends.
    pub fn source_hashes(&self) -> Result<Vec<String>, Errors> {
        collect_results(self.sources().map(|source| source.hash()))
    }
}

// Program of fix a collection of modules.
// A program can link another program which consists of a single module.
pub struct Program {
    /* AST */
    // Global values.
    pub global_values: Map<FullName, GlobalValue>,
    // Type definitions.
    pub type_defns: Vec<TypeDefn>,
    // Type environment, which is calculated from `type_defns` once and cached.
    pub type_env: TypeEnv,
    // Trait environment.
    pub trait_env: TraitEnv,
    // Entry point value of the program.
    // - Instantiation of `Main::main` when run or build mode.
    // - Instantiation of `Main::test` when test mode.
    // - None when library mode.
    pub entry_io_value: Option<Arc<ExprNode>>,
    // Export statements.
    pub export_statements: Vec<ExportStatement>,
    /// `DEPRECATED[...]` pragmas, accumulated at parse time and consumed in
    /// elaboration to set per-symbol `deprecation` fields.
    pub deprecation_statements: Vec<DeprecationStatement>,
    // List of tuple sizes used in this program.
    pub used_tuple_sizes: Vec<u32>,
    // Import statements.
    // Key is the name of the importer module.
    // Each module implicitly imports itself.
    // This is used to namespace resolution and overloading resolution.
    pub mod_to_import_stmts: Map<Name, Vec<ImportStatement>>,

    /* Instantiated symbols */
    // Opaque types instantiated in this program, keyed by opaque TyCon name.
    pub opaque_types: Map<FullName, Vec<OpaqueTyConResolution>>,
    // Instantiated symbols.
    pub symbols: Map<FullName, Symbol>,
    // Deferred instantiations.
    // This is a state variable for the instantiation process.
    pub deferred_instantiation: Vec<Symbol>,

    /* Dependency information */
    pub modules: Vec<ModuleInfo>,

    /* Diagnostic */
    // Deferred errors.
    // Errors that should be displayed in the diagnostic information.
    pub deferred_errors: Errors,
    // Names required to be imported in each module.
    pub import_required: Map<Name, Set<FullName>>,

    /* Optimization */
    // Number of optimization steps.
    // This is used to name the symbol files when outputting them at each optimization step.
    pub optimization_step: usize,
}

impl Program {
    pub fn merge_import_required(&mut self, other: Map<Name, Vec<FullName>>) {
        for (mod_name, names) in other {
            let entry = self
                .import_required
                .entry(mod_name)
                .or_insert_with(Set::default);
            for name in names {
                entry.insert(name);
            }
        }
    }

    pub fn find_mod(&self, mod_name: &Name) -> Option<ModuleInfo> {
        for mod_info in &self.modules {
            if &mod_info.name == mod_name {
                return Some(mod_info.clone());
            }
        }
        None
    }

    /// The expressions the entry point and the exported functions were instantiated as.
    pub fn root_value_exprs(&self) -> Vec<&Arc<ExprNode>> {
        self.entry_io_value
            .iter()
            .chain(
                self.export_statements
                    .iter()
                    .filter_map(|stmt| stmt.value_expr.as_ref()),
            )
            .collect()
    }

    /// The module defined by the source file at `path`, compared by
    /// absolute path.
    pub fn module_of_file(&self, path: &Path) -> Option<&ModuleInfo> {
        let path = to_absolute_path(path).ok()?;
        self.modules
            .iter()
            .find(|mi| to_absolute_path(&mi.source.input.file_path).ok().as_ref() == Some(&path))
    }

    /// The names of the entry point and the exported functions.
    pub fn root_value_names(&self) -> Vec<FullName> {
        self.root_value_exprs()
            .iter()
            .map(|expr| expr.get_var().name.clone())
            .collect()
    }

    // Get the list of module names from a list of files.
    pub fn modules_from_files(&self, files: &[PathBuf]) -> Result<Vec<Name>, Errors> {
        let mut abs_files = vec![];
        for f in files {
            abs_files.push(to_absolute_path(f)?);
        }
        let mut mod_names = vec![];
        for mod_info in &self.modules {
            let mod_file = to_absolute_path(&mod_info.source.input.file_path)?;
            if abs_files.contains(&mod_file) {
                mod_names.push(mod_info.name.clone());
            }
        }
        Ok(mod_names)
    }

    // Create a program consists of single module.
    pub fn single_module(mod_info: ModuleInfo) -> Program {
        let mut fix_mod = Program {
            mod_to_import_stmts: Default::default(),
            type_defns: Default::default(),
            global_values: Default::default(),
            symbols: Default::default(),
            deferred_instantiation: Default::default(),
            trait_env: Default::default(),
            type_env: Default::default(),
            used_tuple_sizes: (0..=TUPLE_SIZE_BASE).collect(),
            modules: Default::default(),
            entry_io_value: None,
            export_statements: vec![],
            deprecation_statements: vec![],
            deferred_errors: Errors::empty(),
            import_required: Default::default(),
            optimization_step: 0,
            opaque_types: Map::default(),
        };
        fix_mod.add_import_statement_no_verify(ImportStatement::implicit_self_import(
            mod_info.name.clone(),
        ));
        fix_mod.add_import_statement_no_verify(ImportStatement::implicit_std_import(
            mod_info.name.clone(),
        ));
        fix_mod.modules.push(mod_info);
        fix_mod
    }

    // Add `Std::TupleN` type
    fn add_tuple_defn(&mut self, tuple_size: u32) {
        self.type_defns.push(tuple_defn(tuple_size));
    }

    // Add `Std::TupleN` type for each `n` in `used_tuple_sizes`.
    pub fn add_tuple_defns(&mut self) {
        // Make elements of used_tuple_sizes unique.
        self.used_tuple_sizes.sort();
        self.used_tuple_sizes.dedup();
        let used_tuple_sizes = replace(&mut self.used_tuple_sizes, vec![]);
        for tuple_size in &used_tuple_sizes {
            self.add_tuple_defn(*tuple_size);
        }
        self.used_tuple_sizes = used_tuple_sizes;
    }

    // If this program consists of single module, returns its name.
    pub fn get_name_if_single_module(&self) -> Name {
        let linked_mods = self.linked_mods();
        if linked_mods.len() == 1 {
            return linked_mods.into_iter().next().unwrap();
        }
        panic!("")
    }

    pub fn is_linked(&self, mod_name: &Name) -> bool {
        self.mod_to_import_stmts.contains_key(mod_name)
    }

    // Add import statements.
    pub fn add_import_statements(&mut self, imports: Vec<ImportStatement>) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for stmt in imports {
            errors.eat_err(self.add_import_statement(stmt));
        }
        errors.to_result()
    }

    // Add an import statement.
    pub fn add_import_statement(
        &mut self,
        import_statement: ImportStatement,
    ) -> Result<(), Errors> {
        // Refuse importing the module itself.
        if import_statement.module.0 == import_statement.importer {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Module `{}` cannot import itself.",
                    import_statement.module.0.to_string()
                ),
                &[&import_statement.source],
            ));
        }

        // When user imports `Std` explicitly, remove implicit `Std` import statement.
        if import_statement.module.0 == STD_NAME {
            let stmts = self
                .mod_to_import_stmts
                .get_mut(&import_statement.importer)
                .unwrap();
            *stmts = replace(stmts, vec![])
                .into_iter()
                .filter(|stmt| !(stmt.module.0 == STD_NAME && stmt.implicit))
                .collect();
        }

        self.add_import_statement_no_verify(import_statement);

        Ok(())
    }

    pub fn add_import_statement_no_verify(&mut self, import_statement: ImportStatement) {
        let importer = &import_statement.importer;
        if let Some(stmts) = self.mod_to_import_stmts.get_mut(importer) {
            stmts.push(import_statement);
        } else {
            self.mod_to_import_stmts
                .insert(importer.clone(), vec![import_statement]);
        }
    }

    /// Materialize implicit imports for every absolute-path `FullName`
    /// (`::Mod::Ns::name`, value or type position) the parser collected
    /// in `abs_path_uses`. Each becomes an
    /// `ImportStatement { implicit: true, ... }` carrying the span of
    /// each path token, mirroring the shape a user-written
    /// `import Mod::Ns::name;` would produce — so the rest of the
    /// pipeline sees the dependency without the user having to write
    /// the import.
    pub fn inject_abs_path_implicit_imports(
        &mut self,
        current_module: &Name,
        abs_path_uses: Vec<(FullName, Vec<Span>)>,
    ) {
        for (abs_path, path_spans) in abs_path_uses {
            if abs_path.module() == *current_module {
                // Self-imports are already added unconditionally.
                continue;
            }
            // `current_module`'s entry is established by
            // `Program::single_module` (implicit self/std imports), so
            // a missing entry would be a bug, not a normal path.
            let existing = self.mod_to_import_stmts.get(current_module).unwrap();
            if is_accessible(existing, &abs_path) {
                continue;
            }
            let mut stmt = ImportStatement::import_to_use_with_spans(
                current_module.clone(),
                abs_path,
                &path_spans,
            );
            stmt.implicit = true;
            self.add_import_statement_no_verify(stmt);
        }
    }

    pub fn import_statements(&self) -> Vec<ImportStatement> {
        self.mod_to_import_stmts
            .values()
            .flat_map(|stmts| stmts.iter())
            .cloned()
            .collect()
    }

    // Add traits.
    pub fn add_traits(
        &mut self,
        trait_infos: Vec<TraitDefn>,
        trait_impls: Vec<TraitImpl>,
        trait_aliases: Vec<TraitAlias>,
    ) -> Result<(), Errors> {
        self.trait_env.add(trait_infos, trait_impls, trait_aliases)
    }

    // Register declarations of user-defined types.
    pub fn add_type_defns(&mut self, mut type_defns: Vec<TypeDefn>) {
        self.type_defns.append(&mut type_defns);
    }

    // Calculate list of type constructors including user-defined types.
    pub fn calculate_type_env(&mut self) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        let mut tycons = bulitin_tycons();
        let mut aliases: Map<TyCon, TyAliasInfo> = Map::default();
        for type_decl in &mut self.type_defns {
            // Set kinds of type variables in the right hand side of type definition.
            type_decl.set_kinds_in_value()?;

            // Check duplicate type definition.
            let tycon = type_decl.tycon();
            if tycons.contains_key(&tycon) || aliases.contains_key(&tycon) {
                let other_src = if tycons.contains_key(&tycon) {
                    let tc = tycons.get(&tycon).unwrap();
                    tc.source.clone()
                } else {
                    let ta = aliases.get(&tycon).unwrap();
                    ta.source.clone()
                };
                errors.append(Errors::from_msg_srcs(
                    format!("Duplicate definitions of type `{}`.", tycon.to_string()),
                    &[
                        &type_decl.source.as_ref().map(|s| s.to_head_character()),
                        &other_src.as_ref().map(|s| s.to_head_character()),
                    ],
                ));
                continue;
            }
            if type_decl.is_alias() {
                aliases.insert(tycon.clone(), type_decl.alias_info());
            } else {
                tycons.insert(tycon.clone(), type_decl.tycon_info(&[]));
            }
            // A struct also gets, per field, the type constructor of the struct with that field
            // punched out, which is what `act_` and `mod_` hold the rest of the struct in while the
            // field is out.
            if let TypeDeclValue::Struct(s) = &type_decl.value {
                for i in 0..s.fields.len() {
                    let mut punched_tycon = tycon.clone();
                    punched_tycon.into_punched_type_name(i);
                    tycons.insert(punched_tycon, type_decl.tycon_info(&[i]));
                }
            }
        }
        // Create type environment.
        self.type_env = TypeEnv::new(tycons, aliases);

        errors.to_result()
    }

    // Get list of type constructors including user-defined types.
    pub fn type_env(&self) -> TypeEnv {
        self.type_env.clone()
    }

    /// The type of every top-level symbol of the program, by name. Compiling one unit under separated
    /// compilation needs the types of the symbols the other units define as well, since this unit's
    /// code refers to them, so this covers the whole program rather than any one unit.
    pub fn global_types(&self) -> Map<FullName, Arc<TypeNode>> {
        self.symbols
            .iter()
            .map(|(name, symbol)| (name.clone(), symbol.ty.clone()))
            .collect()
    }

    // Get of list of tycons that can be used for namespace resolution.
    pub fn tycon_names_with_aliases(&self) -> Set<FullName> {
        let mut res: Set<FullName> = Default::default();
        for (k, _) in self.type_env().tycons.iter() {
            res.insert(k.name.clone());
        }
        for (k, _) in self.type_env().aliases.iter() {
            res.insert(k.name.clone());
        }
        res
    }

    pub fn assoc_ty_to_arity(&self) -> Map<FullName, usize> {
        self.trait_env.assoc_ty_to_arity()
    }

    // Get of list of traits that can be used for namespace resolution.
    pub fn trait_names_with_aliases(&self) -> Set<FullName> {
        self.trait_env.trait_names()
    }

    pub fn traits_with_aliases(&self) -> Vec<TraitId> {
        self.trait_env.traits_with_aliases()
    }

    // Add a global value.
    pub fn add_global_value(
        &mut self,
        name: FullName,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
        decl_src: Option<Span>,
        defn_src: Option<Span>,
        document: Option<String>,
    ) -> Result<(), Errors> {
        self.add_global_value_common(name, (expr, scm), decl_src, defn_src, document, false)
    }

    /// Programmatically register a `DEPRECATED[...]` pragma for a global
    /// value, attaching the given user-facing message to `target`. The pragma
    /// is processed during elaboration just like a source-level pragma.
    pub fn add_deprecation(&mut self, target: FullName, message: String) {
        self.deprecation_statements.push(DeprecationStatement {
            target_path: target,
            target_name_src: None,
            origin_namespace: NameSpace::local(),
            message,
            src: None,
        });
    }

    // Add a compiler-defined method.
    pub fn add_compiler_defined_method(
        &mut self,
        name: FullName,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
        document: Option<String>,
    ) -> Result<(), Errors> {
        // When the compiler automatically adds functions to user-defined modules,
        // the symbol names and type names used in those functions should not require the user to import them.
        // Therefore, convert all global names to absolute names before registering.
        let expr = expr.global_to_absolute();
        let scm = scm.global_to_absolute();
        self.add_global_value_common(name, (expr, scm), None, None, document, true)
    }

    /// Registers a global value whose body is `expr` and whose type is `scm`.
    ///
    /// # Arguments
    /// * `decl_src` — where the value's type signature is written.
    /// * `defn_src` — where the left hand side of the value's definition is written.
    /// * `document` — the documentation of the value, for a value whose `decl_src` is
    ///   unavailable; otherwise the documentation is read from the source code.
    /// * `compiler_defined_method` — marks a method the compiler generates for a type, such as
    ///   `@{field}` or `set_{field}`, which `fix docs` leaves out.
    fn add_global_value_common(
        &mut self,
        name: FullName,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
        decl_src: Option<Span>,
        defn_src: Option<Span>,
        document: Option<String>,
        compiler_defined_method: bool,
    ) -> Result<(), Errors> {
        let gv = GlobalValue {
            scm: scm.clone(),
            syn_scm: None,
            expr: SymbolExpr::Simple(TypedExpr::from_expr(expr)),
            decl_src,
            defn_src,
            document,
            compiler_defined_method,
            deprecation: None,
        };
        self.add_global_value_gv(name, gv)
    }

    /// Registers an already-built global value under `name`, reporting an error that points at
    /// both declarations when the name is taken.
    pub fn add_global_value_gv(&mut self, name: FullName, gv: GlobalValue) -> Result<(), Errors> {
        // Check duplicate definition.
        if self.global_values.contains_key(&name) {
            let this = gv.decl_src.map(|s| s.to_head_character());
            let other = self
                .global_values
                .get(&name)
                .unwrap()
                .decl_src
                .as_ref()
                .map(|s| s.to_head_character());
            return Err(Errors::from_msg_srcs(
                format!(
                    "Duplicated definition for global value: `{}`",
                    name.to_string()
                ),
                &[&this, &other],
            ));
        }
        self.global_values.insert(name, gv);
        Ok(())
    }

    /// Pairs each definition with the type signature carrying the same name and registers the
    /// pairs as global values. A name that carries two definitions, two signatures, a definition
    /// without a signature, or a signature without a definition is reported as an error.
    pub fn add_global_values(
        &mut self,
        defns: Vec<GlobalValueDefn>,
        decls: Vec<GlobalValueDecl>,
    ) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        /// The two halves of one global value, collected while pairing them up by name. A half
        /// stays `None` until it is met.
        struct GlobalValue {
            /// The definition, e.g. `main = println("Hello World");`.
            defn: Option<GlobalValueDefn>,
            /// The type signature, e.g. `main : IO ();`.
            decl: Option<GlobalValueDecl>,
        }
        let mut global_values: Map<FullName, GlobalValue> = Default::default();

        // Register definitions checking duplication.
        for defn in defns {
            if !global_values.contains_key(&defn.name) {
                global_values.insert(
                    defn.name.clone(),
                    GlobalValue {
                        defn: Some(defn),
                        decl: None,
                    },
                );
            } else {
                let gv = global_values.get_mut(&defn.name).unwrap();
                if gv.defn.is_some() {
                    errors.append(Errors::from_msg_srcs(
                        format!(
                            "Duplicate definition for global value: `{}`.",
                            defn.name.to_string()
                        ),
                        &[
                            &defn.src.map(|s| s.to_head_character()),
                            &gv.defn
                                .as_ref()
                                .unwrap()
                                .src
                                .as_ref()
                                .map(|s| s.to_head_character()),
                        ],
                    ));
                } else {
                    gv.defn = Some(defn);
                }
            }
        }

        // Register definitions checking duplication.
        for decl in decls {
            if !global_values.contains_key(&decl.name) {
                global_values.insert(
                    decl.name.clone(),
                    GlobalValue {
                        decl: Some(decl),
                        defn: None,
                    },
                );
            } else {
                let gv = global_values.get_mut(&decl.name).unwrap();
                if gv.decl.is_some() {
                    errors.append(Errors::from_msg_srcs(
                        format!("Duplicate declaration for `{}`.", decl.name.to_string()),
                        &[
                            &decl.src.map(|s| s.to_head_character()),
                            &gv.decl
                                .as_ref()
                                .unwrap()
                                .src
                                .as_ref()
                                .map(|s| s.to_head_character()),
                        ],
                    ));
                } else {
                    gv.decl = Some(decl);
                }
            }
        }

        // Check that declarations and definitions are paired.
        for (name, gv) in global_values {
            if gv.defn.is_none() {
                errors.append(Errors::from_msg_srcs(
                    format!("Global value `{}` lacks its definition.", name.to_string()),
                    &[&gv.decl.unwrap().src.as_ref().map(|s| s.to_head_character())],
                ));
            } else if gv.decl.is_none() {
                errors.append(Errors::from_msg_srcs(
                    format!(
                        "Global value `{}` lacks its type signature.",
                        name.to_string()
                    ),
                    &[&gv.defn.unwrap().src.as_ref().map(|s| s.to_head_character())],
                ));
            } else {
                let decl_src = gv.decl.as_ref().unwrap().src.clone();
                let defn_src = gv.defn.as_ref().unwrap().src.clone();
                errors.eat_err(self.add_global_value(
                    name,
                    (gv.defn.unwrap().expr, gv.decl.unwrap().ty),
                    decl_src,
                    defn_src,
                    None,
                ));
            }
        }

        errors.to_result()
    }

    /// Resolve namespaces, resolve type aliases, and run the type
    /// checker for a single value's expression.
    ///
    /// # Arguments
    /// * `te` — the expression to be namespace-resolved and type-checked.
    /// * `req_scm` — the type scheme that the expression should have.
    /// * `val_name` — the name of the expression, e.g., `Std::ToString::to_string`.
    /// * `def_mod` — the module where the expression is defined. For a
    ///   trait method implementation this may differ from `val_name.module()`.
    /// * `nrctx` — the name resolution context. Pass one created by
    ///   `program.create_name_resolution_context(define_module)`.
    /// * `ver_hash` — hash of the source code `te` depends on, used
    ///   to detect or invalidate the cache file. Pass one created by
    ///   `program.module_dependency_hash(define_module, config)`.
    ///
    /// # Returns
    /// * `Ok((te, errors))` — namespace resolution and substitution
    ///   completed; `te` is the typed expression. `errors` may still
    ///   carry tolerated diagnostics from `check_type` (holes,
    ///   cannot-infer, unsatisfied predicates, disjoint equalities).
    ///   The caller should always save `te` (so the LSP can hover on
    ///   it) and propagate `errors`. The cache is written only for a
    ///   strict check that produced no `errors`.
    /// * `Err(errs)` — a hard failure happened before substitution
    ///   completed (e.g. resolve_namespace, resolve_type_aliases, or
    ///   the substitution itself blew up). No useful typed expression
    ///   to propagate.
    fn resolve_namespace_and_check_type_sub(
        mut te: TypedExpr,
        req_scm: &Arc<Scheme>,
        val_name: &FullName,
        def_mod: &ModuleInfo,
        nrctx: &mut NameResolutionContext,
        ver_hash: &str,
        mut tc: TypeCheckContext,
    ) -> Result<(TypedExpr, Errors), Errors> {
        // Load type-checking cache file.
        let cached_te = tc.cache.load_cache(val_name, req_scm, ver_hash);
        if cached_te.is_some() {
            // If cache is available,
            te = cached_te.unwrap();
            return Ok((te, Errors::empty()));
        }

        // Perform namespace inference.
        te.expr = te.expr.resolve_namespace(nrctx)?;

        // Resolve type aliases in expression.
        te.expr = te.expr.resolve_type_aliases(&tc.type_env)?;

        // Perform type-checking.
        tc.current_module = Some(def_mod.clone());
        let (typed_expr, check_errors) = tc.check_type(te.expr.clone(), req_scm.clone())?;
        te.expr = typed_expr;
        // Fill in the concrete rhs for opaque type resolutions set up during desugaring.
        tc.fill_opaque_concrete_types(&mut te.opaque_types);
        te.equalities = tc.local_assumed_eqs;

        // A run whose `load_cache` finds the expression returns it without checking it, so the
        // cache may hold only what a strict check accepted. Two things disqualify a result:
        //
        // - a tolerated diagnostic, which the next run owes the user and would not produce again;
        // - an `error_tolerant` check, which swallows every type error and reports none, so its
        //   result would enter as a clean entry and the value would be published as type-correct.
        if !tc.error_tolerant && !check_errors.has_diagnostics() {
            tc.cache.save_cache(&te, val_name, req_scm, ver_hash);
        }

        // Add names required to be imported found in type-checking to NameResolutionContext's import_required.
        nrctx.add_import_required(tc.import_required);

        Ok((te, check_errors))
    }

    /// Builds the program-wide table that name resolution reads: every type constructor, trait
    /// and associated type a capitalized name can resolve to, plus each module's import
    /// statements. A `NameResolutionContext` fixes the module a name is written in and shares
    /// this table.
    pub fn create_name_resolution_env(&self) -> Arc<NameResolutionEnv> {
        Arc::new(NameResolutionEnv::new(
            &self.tycon_names_with_aliases(),
            &self.trait_names_with_aliases(),
            self.assoc_ty_to_arity(),
            self.mod_to_import_stmts.clone(),
            self.modules.clone(),
        ))
    }

    /// Resolve namespaces in, and typecheck the bodies of, global
    /// values declared in `modules`.
    ///
    /// # Arguments
    /// * `target_symbols` — when `Some`, restrict checking to globals
    ///   whose name is in the slice; when `None`, check every global
    ///   declared in `modules`.
    pub fn resolve_namespace_and_check_type_in_modules(
        &mut self,
        tc: &TypeCheckContext,
        modules: &[Name],
        target_symbols: Option<&[FullName]>,
        config: &Configuration,
    ) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        let target_set: Option<Set<&FullName>> = target_symbols.map(|s| s.iter().collect());

        // Names of global values to be checked.
        let mut names_to_check: Vec<FullName> = vec![];
        for (name, gv) in self.global_values.iter() {
            if let Some(set) = target_set.as_ref() {
                if !set.contains(name) {
                    continue;
                }
            }
            match gv.expr {
                SymbolExpr::Simple(_) => {
                    // Check simple values only if they are in `modules`.
                    if modules.contains(&name.module()) {
                        names_to_check.push(name.clone());
                    }
                }
                SymbolExpr::Method(_) => {
                    // We filter methods by `method_impl_filter`.
                    names_to_check.push(name.clone());
                }
            }
        }

        // Method implementations to be checked.
        let modules = modules.to_vec();
        let method_impl_filter =
            |method: &TraitMemberImpl| Ok(modules.contains(&method.define_module));

        errors.eat_err(self.resolve_namespace_and_check_type(
            tc,
            &names_to_check,
            method_impl_filter,
            config,
        ));

        // The concrete types of opaque types are known only once the values are checked, and a
        // cycle among them has to be reported before instantiation puts each in the other's place.
        errors.eat_err(self.validate_opaque_types_are_acyclic(tc));

        errors.to_result()
    }

    /// Resolves namespaces in, and type-checks, the global values named in `val_names`, updating
    /// their `TypedExpr` in `self.global_values` in place.
    ///
    /// # Arguments
    /// * `method_impl_filter` — decides, for each implementation of a trait method the names cover,
    ///   whether this run checks it.
    pub fn resolve_namespace_and_check_type(
        &mut self,
        tc: &TypeCheckContext,
        val_names: &[FullName],
        method_impl_filter: impl Fn(&TraitMemberImpl) -> Result<bool, Errors>,
        config: &Configuration,
    ) -> Result<(), Errors> {
        let nrenv = self.create_name_resolution_env();

        /// What checking one value produced. The typed expression and any tolerated diagnostics
        /// come together so the caller can save the typed expression for the LSP even when the
        /// value didn't type-check cleanly. See `check_type` for the rules on what counts as
        /// "tolerated".
        struct CheckTaskOutput {
            /// The value's expression, with namespaces resolved and types assigned.
            te: TypedExpr,
            /// The names the expression referred to that the module it is defined in has to
            /// import, keyed by that module.
            import_required: Map<Name, Vec<FullName>>,
            /// The tolerated diagnostics the check produced.
            errors: Errors,
        }
        /// One value to check, as a closure a worker thread can run.
        struct CheckTask {
            /// The global value to check.
            val_name: FullName,
            /// Resolves namespaces in the value's expression and type-checks it.
            task: Box<dyn FnOnce() -> Result<CheckTaskOutput, Errors> + Send>,
            /// Which implementation of the trait method to check; `None` for a simple value.
            method_impl_idx: Option<usize>,
        }
        let mut tasks: Vec<CheckTask> = vec![];

        // Create tasks.
        for val_name in val_names {
            let gv = self.global_values.get(&val_name).unwrap();
            match &gv.expr {
                SymbolExpr::Simple(e) => {
                    // Create a task for simple value.
                    let te = e.clone();
                    let scm = gv.scm.clone();
                    let val_name_clone = val_name.clone(); // For move into closure.
                    let def_mod = self.find_mod(&val_name.module()).unwrap().clone();
                    let mut nrctx = NameResolutionContext::new(def_mod.name.clone(), nrenv.clone());
                    let ver_hash = self.module_dependency_hash(&def_mod.name, config)?;
                    let tc = tc.clone();
                    let task = Box::new(move || -> Result<CheckTaskOutput, Errors> {
                        // Perform type-checking.
                        let (te, errors) = Program::resolve_namespace_and_check_type_sub(
                            te,
                            &scm,
                            &val_name_clone,
                            &def_mod,
                            &mut nrctx,
                            &ver_hash,
                            tc,
                        )?;
                        let output = CheckTaskOutput {
                            te,
                            import_required: nrctx.import_required,
                            errors,
                        };
                        Ok(output)
                    });

                    tasks.push(CheckTask {
                        val_name: val_name.clone(),
                        task,
                        method_impl_idx: None,
                    });
                }
                SymbolExpr::Method(impls) => {
                    for (i, member) in impls.iter().enumerate() {
                        // Select method implementation.
                        if !method_impl_filter(member)? {
                            continue;
                        }

                        // Create a task for method implementation.
                        let te = member.expr.clone();
                        let scm = member.scm.clone();
                        let scm_via_defn = member.scm_via_defn.clone();
                        let impl_src = member.expr.expr.source.clone();
                        let decl_src = gv.decl_src.clone();
                        let val_name_clone = val_name.clone(); // For move into closure.
                        let def_mod = self.find_mod(&member.define_module).unwrap().clone();
                        let mut nrctx =
                            NameResolutionContext::new(def_mod.name.clone(), nrenv.clone());
                        let ver_hash = self.module_dependency_hash(&def_mod.name, config)?;
                        let tc = tc.clone();
                        let task = Box::new(move || -> Result<CheckTaskOutput, Errors> {
                            // Check that the type signature given by implementor is equivalent to
                            // the type scheme obtained from the trait member definition.
                            if tc.check_scheme_equivalent(&scm, &scm_via_defn).is_err() {
                                return Err(Errors::from_msg_srcs(
                                    format!(
                                        "Type signature in implementation does not match trait definition.\nExpected: `{}`\nFound: `{}`",
                                        scm_via_defn.to_string(),
                                        scm.to_string(),
                                    ),
                                    &[
                                        &impl_src
                                            .as_ref()
                                            .map(|s| s.to_head_character()),
                                        &decl_src
                                            .as_ref()
                                            .map(|s| s.to_head_character()),
                                    ],
                                ));
                            }
                            // Perform type-checking.
                            let (te, errors) = Program::resolve_namespace_and_check_type_sub(
                                te,
                                &scm,
                                &val_name_clone,
                                &def_mod,
                                &mut nrctx,
                                &ver_hash,
                                tc,
                            )?;
                            let output = CheckTaskOutput {
                                te,
                                import_required: nrctx.import_required,
                                errors,
                            };
                            Ok(output)
                        });

                        tasks.push(CheckTask {
                            val_name: val_name.clone(),
                            task,
                            method_impl_idx: Some(i),
                        });
                    }
                }
            };
        }

        // Run all tasks.
        /// What one task produced, together with the place in `self` it belongs to.
        struct CheckResult {
            /// The global value the task checked.
            val_name: FullName,
            /// The typed expression the check produced, or the errors that stopped it.
            output: Result<CheckTaskOutput, Errors>,
            /// Which implementation of the trait method the task checked; `None` for a simple
            /// value.
            method_impl_idx: Option<usize>,
        }
        /// Runs `task` and pairs its output with the value the task checked, so that the typed
        /// expression can be stored back where it came from.
        fn run_check_task(task: CheckTask) -> CheckResult {
            let output = (task.task)();
            CheckResult {
                val_name: task.val_name,
                output,
                method_impl_idx: task.method_impl_idx,
            }
        }
        let results: Vec<CheckResult> = if tc.num_worker_threads <= 1 || tasks.len() <= 1 {
            // Run tasks in the main thread.
            tasks.into_iter().map(run_check_task).collect()
        } else {
            // Run tasks in parallel via a shared work queue: every
            // worker thread pops the next task from the same `Vec`
            // until it is empty, so an idle worker immediately picks
            // up whatever is left.
            //
            // Per-gv typecheck cost is highly uneven (cache hits cost
            // ~30 μs, misses ~300 ms), so a static per-thread shard
            // would leave cores idle whenever the misses cluster on
            // one shard.
            let queue = Arc::new(Mutex::new(tasks));
            let mut threads = vec![];
            for _ in 0..tc.num_worker_threads {
                let queue = queue.clone();
                let thread = spawn_compiler_thread(move || {
                    let mut results = vec![];
                    loop {
                        let task = match queue.lock().unwrap().pop() {
                            Some(task) => task,
                            None => break,
                        };
                        results.push(run_check_task(task));
                    }
                    results
                });
                threads.push(thread);
            }
            join_compiler_threads(threads)
                .into_iter()
                .flatten()
                .collect()
        };

        // Store the results into members of `self`.
        let mut errors = Errors::empty();
        for result in results {
            if result.output.is_err() {
                errors.append(result.output.err().unwrap());
                continue;
            }
            let mut output = result.output.ok().unwrap();
            // Carry tolerated diagnostics (holes, cannot-infer, etc.)
            // forward, but still install the typed expression below so
            // the LSP can hover on its sub-expressions.
            errors.append(output.errors);
            for (k, mut v) in output.te.opaque_types.drain() {
                self.opaque_types.entry(k).or_default().append(&mut v);
            }
            self.merge_import_required(output.import_required);
            let gv = self.global_values.get_mut(&result.val_name).unwrap();
            match &mut gv.expr {
                SymbolExpr::Simple(e) => {
                    *e = output.te;
                }
                SymbolExpr::Method(impls) => {
                    let impl_idx = result.method_impl_idx.unwrap();
                    impls[impl_idx].expr = output.te;
                }
            };
        }

        errors.to_result()
    }

    /// Fills `sym.expr` with the typed expression of `sym.generic_name`
    /// specialized to `sym.ty`, picking the matching implementation when the
    /// symbol is a trait method.
    ///
    /// Assumes that `resolve_namespace_and_check_type_in_modules` has already
    /// run over all global values.
    fn instantiate_symbol(
        &mut self,
        sym: &mut Symbol,
        tc: &TypeCheckContext,
    ) -> Result<(), Errors> {
        assert!(sym.expr.is_none());
        // Resolve opaque types in sym.ty before method selection,
        // so that trait method implementations can be matched against concrete types.
        sym.ty = resolve_opaque_type_in_type(&sym.ty, &self.opaque_types);
        // Select method implementation whose type unifies with the required type `sym.ty`.
        // Also resolve opaque types in method_ty so both sides use concrete types.
        let opaque_types = &self.opaque_types;
        let method_type_matches = |method: &TraitMemberImpl| -> Result<bool, Errors> {
            let method_ty = resolve_opaque_type_in_type(&method.scm_via_defn.ty, opaque_types);
            tc.are_unifiable(&method_ty, &sym.ty)
        };

        // Select the typed expression to specialize.
        let global_sym = self.global_values.get(&sym.generic_name).unwrap();
        let te = match &global_sym.expr {
            SymbolExpr::Simple(e) => e,
            SymbolExpr::Method(impls) => {
                let method = impls
                    .iter()
                    .find(|method| method_type_matches(method).unwrap_or(false))
                    .unwrap();
                &method.expr
            }
        };

        // Specialize the typed expression to `sym.ty` and resolve opaque types.
        let mut tc = tc.clone();
        tc.assert_freshness();
        let expr_ty =
            resolve_opaque_type_in_type(te.expr.type_.as_ref().unwrap(), &self.opaque_types);
        tc.unify(&expr_ty, &sym.ty).ok().unwrap();
        for eq in &te.equalities {
            tc.unify(&eq.lhs(), &eq.value).ok().unwrap();
        }
        let expr = tc.fix_types(te.expr.clone())?;
        // Reject indeterminate types here so instantiation never
        // proceeds with one (`fix_types` only substitutes; it does not
        // verify that every type variable was solved).
        tc.check_types_are_fixed(&expr)?;
        let expr = remove_opaque_wrapper_func(expr);
        let expr = resolve_opaque_tycon_in_expr(&expr, &self.opaque_types);
        // Reduce associated types newly exposed by opaque-tycon resolution:
        // an opaque rhs may carry an `AssocTy` whose arguments only become
        // concrete here (e.g. `Item it` with `it := RangeIterator` becomes
        // `Item RangeIterator`, reducible to `I64` via the trait instance).
        let expr = tc.fix_types(expr)?;
        sym.expr = Some(self.instantiate_expr(&expr)?);
        Ok(())
    }

    /// Instantiates every symbol queued in `deferred_instantiation`, moving each
    /// into `symbols`. Instantiation queues further symbols, so this runs until
    /// the queue drains.
    pub fn instantiate_symbols(&mut self, tc: &TypeCheckContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        while !self.deferred_instantiation.is_empty() {
            let sym = self.deferred_instantiation.pop().unwrap();
            let name = sym.name.clone();
            let mut sym = sym.clone();
            errors.eat_err(self.instantiate_symbol(&mut sym, tc));
            self.symbols.insert(name, sym);
        }
        errors.to_result()
    }

    /// Instantiates the program's entry point at type `IO ()` and stores it in
    /// `entry_io_value`.
    ///
    /// # Arguments
    /// * `test_mode` — when true the entry point is `Test::test`, as `fix test`
    ///   runs it; otherwise it is `Main::main`.
    pub fn instantiate_entry_io_value(
        &mut self,
        tc: &TypeCheckContext,
        test_mode: bool,
    ) -> Result<(), Errors> {
        let main_func_name = if test_mode {
            FullName::from_strs(&[TEST_MODULE_NAME], TEST_FUNCTION_NAME)
        } else {
            FullName::from_strs(&[MAIN_MODULE_NAME], MAIN_FUNCTION_NAME)
        };
        let main_ty = make_io_unit_ty();
        let (expr, _ty) =
            self.instantiate_exported_value(&main_func_name, Some(main_ty), &None, tc)?;
        self.entry_io_value = Some(expr);
        Ok(())
    }

    /// Instantiates the value named by each export statement, recording the
    /// instantiated expression and its exported function type back into the
    /// statement.
    pub fn instantiate_exported_values(&mut self, tc: &TypeCheckContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        let mut export_stmts = replace(&mut self.export_statements, vec![]);
        for stmt in &mut export_stmts {
            errors.eat_err_or(
                self.instantiate_exported_value(&stmt.value_name, None, &stmt.src, tc),
                |(instantiated_expr, eft)| {
                    stmt.function_type = Some(eft);
                    stmt.value_expr = Some(instantiated_expr);
                },
            );
        }
        errors.to_result()?;
        self.export_statements = export_stmts;
        Ok(())
    }

    /// Instantiates the global value `value_name` for export, returning the
    /// instantiated expression together with the function type it is exported at.
    ///
    /// # Arguments
    /// * `required_ty` — the type the value is required to have, e.g. `IO ()`
    ///   for `Main::main`; `None` accepts the type the user declared.
    /// * `required_src` — the source location of the export, used to place the
    ///   error message.
    pub fn instantiate_exported_value(
        &mut self,
        value_name: &FullName,
        required_ty: Option<Arc<TypeNode>>,
        required_src: &Option<Span>,
        tc: &TypeCheckContext,
    ) -> Result<(Arc<ExprNode>, ExportedFunctionType), Errors> {
        // Check if the value is defined.
        let gv = self.global_values.get(value_name);
        if gv.is_none() {
            return Err(Errors::from_msg_srcs(
                format!("Value `{}` is not found.", value_name.to_string()),
                &[required_src],
            ));
        }

        // Validate the type of the value.
        let gv: &GlobalValue = gv.unwrap();
        let (required_ty, eft) = if let Some(required_ty) = required_ty {
            // If the type of the value is specified, check if it matches the required type.
            if gv.scm.to_string_normalize() != required_ty.to_string() {
                let gv_src = gv.scm.ty.get_source();
                return Err(Errors::from_msg_srcs(
                    format!(
                        "The value `{}` should have type `{}`.",
                        value_name.to_string(),
                        required_ty.to_string()
                    ),
                    &[gv_src, required_src],
                ));
            }
            let eft = ExportedFunctionType {
                doms: vec![],
                codom: make_unit_ty(),
                io_type: IOType::IO,
            };
            (required_ty, eft)
        } else {
            // If the type of the value is not specified, check if it is good as the type of an exported value.
            let err_msg_prefix = format!(
                "The type of the value `{}` is not suitable for export: ",
                value_name.to_string(),
            );
            let eft = ExportedFunctionType::validate(
                gv.scm.clone(),
                &tc.type_env,
                err_msg_prefix,
                required_src,
            )?;
            (gv.scm.ty.clone(), eft)
        };
        let symbol_name = self.require_instantiation(&value_name, &required_ty)?;
        self.instantiate_symbols(tc)?;
        let expr = expr_var(symbol_name, None).set_type(required_ty);
        Ok((expr, eft))
    }

    // Instantiate expression.
    fn instantiate_expr(&mut self, expr: &Arc<ExprNode>) -> Result<Arc<ExprNode>, Errors> {
        let ret = match &*expr.expr {
            Expr::Var(v) => {
                if v.name.is_local() {
                    expr.clone()
                } else {
                    let instance =
                        self.require_instantiation(&v.name, &expr.type_.as_ref().unwrap())?;
                    let v = v.set_name(instance);
                    expr.set_var_var(v)
                }
            }
            Expr::LLVM(_) => expr.clone(),
            Expr::App(fun, args) => {
                let fun = self.instantiate_expr(fun)?;
                let args = collect_results(args.iter().map(|arg| self.instantiate_expr(arg)))?;
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_, body) => expr.set_lam_body(self.instantiate_expr(body)?),
            Expr::Let(_, bound, val) => {
                let bound = self.instantiate_expr(bound)?;
                let val = self.instantiate_expr(val)?;
                expr.set_let_bound(bound).set_let_value(val)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.instantiate_expr(cond)?;
                let then_expr = self.instantiate_expr(then_expr)?;
                let else_expr = self.instantiate_expr(else_expr)?;
                expr.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::Match(cond, pat_vals) => {
                let cond = self.instantiate_expr(cond)?;
                let mut new_pat_vals = vec![];
                for (pat, e) in pat_vals {
                    let e = self.instantiate_expr(e)?;
                    new_pat_vals.push((pat.clone(), e));
                }
                expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals)
            }
            Expr::TyAnno(e, _) => {
                let e = self.instantiate_expr(e)?;
                expr.set_tyanno_expr(e)
            }
            Expr::MakeStruct(_, fields) => {
                let mut expr = expr.clone();
                for (field_name, _, field_expr) in fields {
                    let field_expr = self.instantiate_expr(field_expr)?;
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                expr
            }
            Expr::ArrayLit(elems) => {
                let mut expr = expr.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.instantiate_expr(e)?;
                    expr = expr.set_array_lit_elem(e, i);
                }
                expr
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                let mut expr = expr.clone();
                for (i, e) in args.iter().enumerate() {
                    let e = self.instantiate_expr(e)?;
                    expr = expr.set_ffi_call_arg(e, i);
                }
                expr
            }
            Expr::Eval(side, main) => {
                let side = self.instantiate_expr(side)?;
                let main = self.instantiate_expr(main)?;
                expr.set_eval_side(side).set_eval_main(main)
            }
        };
        // If the type of an expression contains indeterminate type variable after instantiation, raise an error.
        //
        // NOTE: This check is a precaution, as we are determining whether there are any indeterminate type variables during the type inference phase.
        let ret_ty = ret.type_.as_ref().unwrap();
        if !ret_ty.is_ground() {
            let (fv_name, _) = ret_ty.free_vars().into_iter().next().unwrap();
            // Must stay in sync with the same message in typecheck.rs (check_is_type_fixed).
            return Err(Errors::from_msg_srcs(
                format!(
                    "Cannot infer the type of this expression: inferred as `{}`, but the type variable `{}` is unresolved.\nHint: add a type annotation to this expression.",
                    ret_ty.to_string(),
                    fv_name,
                ),
                &[&ret.source],
            ));
        }
        Ok(ret)
    }

    /// Ask that the generic value `name` be instantiated at type `ty`, and return the name that
    /// instantiation is known by. Asking twice for the same name and type yields that one name and
    /// queues one symbol, whose expression is filled in when the queue is drained.
    pub fn require_instantiation(
        &mut self,
        name: &FullName,
        ty: &Arc<TypeNode>,
    ) -> Result<FullName, Errors> {
        let inst_name = self.determine_symbol_name(name, ty)?;
        if !self.symbols.contains_key(&inst_name)
            && self
                .deferred_instantiation
                .iter()
                .all(|symbol| symbol.name != inst_name)
        {
            self.deferred_instantiation.push(Symbol {
                name: inst_name.clone(),
                generic_name: name.clone(),
                ty: ty.clone(),
                expr: None,
                inline_into_callers: false,
            });
        }
        Ok(inst_name)
    }

    /// The name the instantiation of the generic value `name` at type `ty` is known by: `name` with
    /// a hash of `ty` appended. Two requests for the same type therefore name one symbol, and
    /// requests for different types name different ones.
    fn determine_symbol_name(
        &self,
        name: &FullName,
        ty: &Arc<TypeNode>,
    ) -> Result<FullName, Errors> {
        let ty = ty.resolve_type_aliases(&self.type_env())?;
        let hash = ty.hash();
        let mut name = name.clone();
        name.name += INSTANCIATED_NAME_SEPARATOR;
        name.name += &hash;
        Ok(name)
    }

    // Create symbols of trait members from TraitEnv.
    pub fn create_trait_member_symbols(&mut self) {
        for (trait_id, trait_) in &self.trait_env.traits {
            for member in &trait_.members {
                let member_scm = trait_.member_scheme(&member.name, false);
                let syntactic_member_scm = trait_.member_scheme(&member.name, true);
                let mut member_impls: Vec<TraitMemberImpl> = vec![];
                let instances = self.trait_env.impls.get(trait_id);
                if let Some(insntances) = instances {
                    for trait_impl in insntances {
                        let scm = trait_impl.member_scheme(&member.name, trait_);
                        let scm_via_defn = trait_impl.member_scheme_by_defn(&member.name, trait_);
                        let expr = trait_impl.member_expr(&member.name);
                        let lhs_srcs = trait_impl
                            .member_lhs_srcs
                            .get(&member.name)
                            .cloned()
                            .unwrap_or_default();
                        member_impls.push(TraitMemberImpl {
                            scm,
                            scm_via_defn,
                            expr: TypedExpr::from_expr(expr),
                            define_module: trait_impl.define_module.clone(),
                            lhs_srcs,
                        });
                    }
                }
                let member_name = FullName::new(&trait_id.name.to_namespace(), &member.name);
                self.global_values.insert(
                    member_name,
                    GlobalValue {
                        scm: member_scm,
                        syn_scm: Some(syntactic_member_scm),
                        expr: SymbolExpr::Method(member_impls),
                        decl_src: member.decl_src.clone(),
                        defn_src: member.decl_src.clone(),
                        document: member.document.clone(),
                        compiler_defined_method: false,
                        deprecation: member.deprecation.clone(),
                    },
                );
            }
        }
    }

    /// Report every constraint written in a global value's type signature, and in the signature of
    /// each implementation of a trait method, that `Scheme::validate_constraints` rejects.
    pub fn validate_global_value_type_constraints(&self) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for (_name, gv) in &self.global_values {
            if let Err(e) = gv.scm.validate_constraints(&self.trait_env) {
                errors.append(e);
                continue;
            };
            match gv.expr {
                SymbolExpr::Simple(ref _e) => {}
                SymbolExpr::Method(ref impls) => {
                    for impl_ in impls {
                        errors.eat_err(impl_.scm.validate_constraints(&self.trait_env));
                        errors.eat_err(impl_.scm_via_defn.validate_constraints(&self.trait_env));
                    }
                }
            }
        }
        errors.to_result()
    }

    /// Report every value of the instantiated program whose type has no size, at the expression the
    /// value appears as.
    ///
    /// A field of an unboxed type is laid out in place, so a value the unboxed fields reach again
    /// would have to be larger than itself, and a type reached from itself at a larger type argument
    /// needs endlessly many layouts. `no_size_reason` decides the first and bounds the second. Code
    /// generation would meet either as a descent through the fields that never ends, so this runs
    /// once the program's types are instantiated and before any of them is laid out.
    pub fn validate_layouts(&self) -> Result<(), Errors> {
        let type_env = self.type_env();

        // The entry point and the exported values come first, so that a type they carry is reported
        // in the program's own code rather than in a library function instantiated at it. The
        // symbols follow, the standard library last: a library function instantiated at a type the
        // program declared would otherwise take the report into the library's own source, which
        // says nothing about the program. Within each group the order is by name, so that a program
        // rejected twice is rejected the same way.
        let mut roots = self.root_value_exprs();
        let mut symbol_names: Vec<&FullName> = self.symbols.keys().collect();
        symbol_names.sort_by_key(|name| {
            let in_std = name.namespace.names.first().map(String::as_str) == Some(STD_NAME);
            (in_std, *name)
        });
        roots.extend(
            symbol_names
                .iter()
                .filter_map(|name| self.symbols[*name].expr.as_ref()),
        );

        let mut walk = LayoutWalk::default();
        let mut errors = Errors::empty();
        // A node the compiler built carries no source location, so a round that reports only at
        // located nodes runs first; the second round takes the types that appear at no located node.
        for located_only in [true, false] {
            for expr in &roots {
                expr.walk_nodes(&mut |node| {
                    if located_only && node.source.is_none() {
                        return;
                    }
                    let ty = node.type_.as_ref().unwrap_or_else(|| {
                        panic!(
                            "Instantiation left an expression with no type: `{}`.",
                            node.expr.stringify().to_string()
                        )
                    });
                    if let Some(msg) = no_size_reason(ty, &type_env, &mut walk) {
                        errors.append(Errors::from_msg_srcs(msg, &[&node.source]));
                    }
                })
            }
        }
        // Every instantiated symbol is compiled, so its own type is laid out whether or not an
        // expression carries it — a compiler-generated accessor has none. These come last because
        // such a symbol has no source location of its own to report at.
        for name in &symbol_names {
            let symbol = &self.symbols[*name];
            if let Some(msg) = no_size_reason(&symbol.ty, &type_env, &mut walk) {
                let source = symbol.expr.as_ref().and_then(|expr| expr.source.clone());
                errors.append(Errors::from_msg_srcs(msg, &[&source]));
            }
        }
        errors.to_result()
    }

    /// Report every `FFI_EXPORT` statement that names its value by an absolute path, that gives a
    /// C function name C cannot spell, or that takes a C function name another statement took.
    pub fn validate_export_statements(&self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Reject absolute-path forms (`FFI_EXPORT[::Foo::bar, c];`) and
        // run per-statement name validation.
        for stmt in &self.export_statements {
            if stmt.value_name.namespace.is_absolute {
                errors.append(Errors::from_msg_srcs(
                    "`FFI_EXPORT` cannot take an absolute path. Use a path relative to the surrounding namespace.".to_string(),
                    &[&stmt.src],
                ));
            }
            errors.eat_err(stmt.validate_names(&stmt.src));
        }

        // Throw errors if any.
        errors.to_result()?;

        // Check if there are multiple export statements having the same `c_function_name`.
        let mut c_function_names: Vec<(String, Option<Span>)> = Default::default();
        for stmt in &self.export_statements {
            if let Some((_, span)) = c_function_names
                .iter()
                .find(|(name, _)| *name == stmt.function_name)
            {
                errors.append(Errors::from_msg_srcs(
                    format!(
                        "Multiple export statements have the same C function name `{}`.",
                        stmt.function_name
                    ),
                    &[&stmt.src, span],
                ));
                continue;
            }
            c_function_names.push((stmt.function_name.clone(), stmt.src.clone()));
        }

        errors.to_result()?;
        Ok(())
    }

    /// Write the warning-severity items of `deferred_errors` to stderr and take them out of it,
    /// leaving the error-severity items in place. Warnings reach the terminal this way even where
    /// compilation succeeds.
    pub fn flush_warnings_to_stderr(&mut self) {
        let warnings = self.deferred_errors.take_warnings();
        if warnings.has_diagnostics() {
            eprint!("{}", warnings.to_string());
        }
    }

    /// Reports the calls of `Std::mark_threaded` this program makes when multi-threading is off.
    ///
    /// Multi-threading is what gives an object a mode to be put into, so `Std::mark_threaded` has
    /// nothing to work with without it. The setting belongs to the program being built, so a library
    /// that needs multi-threading reaches the user through this: the calls reported are the ones
    /// asking for the setting, in the files they were written in.
    ///
    /// Only the calls a program reaches are reported, so the symbols have to be instantiated by the
    /// time this runs. Run it before the program is optimized, while each expression still carries
    /// the source it came from.
    pub fn check_multi_threading_requirement(&self, config: &Configuration) -> Result<(), Errors> {
        if config.threaded {
            return Ok(());
        }
        let mark_threaded = FullName::from_strs(&[STD_NAME], MARK_THREADED_NAME);
        // A generic value is instantiated once per type it is used at, and every instance answers to
        // the name it was written as.
        let instances = self
            .symbols
            .values()
            .filter(|symbol| symbol.generic_name == mark_threaded)
            .map(|symbol| symbol.name.clone())
            .collect::<Set<_>>();
        if instances.is_empty() {
            return Ok(());
        }
        let mut uses: Vec<(&FullName, Option<Span>)> = vec![];
        for symbol in self.symbols.values() {
            let expr = symbol.expr.as_ref().unwrap();
            expr.walk_var_uses(&mut |var, src| {
                if instances.contains(&var.name) {
                    uses.push((&symbol.name, src.clone()));
                }
            });
        }
        // The symbols are held in a map, so an order is chosen here to keep the report the same from
        // one build to the next.
        uses.sort_by(|a, b| a.0.cmp(b.0));
        let srcs = uses.iter().map(|(_, src)| src).collect::<Vec<_>>();
        Err(Errors::from_msg_srcs(
            format!(
                "`{}` requires multi-threading. Enable it by `threaded = true` in the project file \
                 of the program being built, or by the `--threaded` compiler option.",
                mark_threaded.to_string()
            ),
            &srcs,
        ))
    }

    /// The uses of items marked `DEPRECATED[...]` that this program makes, as warnings, or as
    /// errors where `Configuration.deprecation_mode` is `Deny`.
    ///
    /// The diagnostics are scoped to the user's own code by `Configuration.root_source_files`: a
    /// use is reported where its source span lies in one of those files, so that what is reported
    /// is what the user can edit.
    pub fn collect_deprecation_diagnostics(&self, config: &Configuration) -> Errors {
        let mut diagnostics = Errors::empty();
        // Exhaustive match: a new `DeprecationMode` variant must be handled here.
        let promote_to_error = match config.deprecation_mode {
            DeprecationMode::Allow => return diagnostics,
            DeprecationMode::Warn => false,
            DeprecationMode::Deny => true,
        };

        // Canonicalize the user-code file set once so per-use comparisons
        // are simple `HashSet` lookups. Paths that fail to canonicalize
        // (e.g. they no longer exist) are dropped; a missing file just
        // won't match any use site, which is the safe direction.
        let user_files: Set<PathBuf> = config
            .root_source_files
            .iter()
            .filter_map(|p| to_absolute_path(p).ok())
            .collect();

        for (_gv_name, gv) in &self.global_values {
            // Skip uses inside an item that is itself deprecated. This is the
            // standard "deprecated context" rule shared by Rust, Java, C# and
            // Swift: a deprecated helper calling another deprecated helper
            // (or recursing into itself) shouldn't spam warnings — the
            // migration burden lives at the boundary, not in the cluster.
            if gv.deprecation.is_some() {
                continue;
            }
            let mut uses: Vec<(FullName, Option<Span>)> = vec![];
            gv.expr.walk_var_uses(&mut |var, src| {
                if var.name.is_global() {
                    uses.push((var.name.clone(), src.clone()));
                }
            });
            for (used_name, use_src) in uses {
                let target = match self.global_values.get(&used_name) {
                    Some(t) => t,
                    None => continue,
                };
                let info = match &target.deprecation {
                    Some(i) => i,
                    None => continue,
                };
                // Without a span we can't tell which file the use lives
                // in, so we drop it: a use without a source location is
                // synthetic (compiler-generated wrappers, builtin bridges)
                // and not something the user can act on.
                let span = match &use_src {
                    Some(s) => s,
                    None => continue,
                };
                let abs = match to_absolute_path(&span.input.file_path) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                if !user_files.contains(&abs) {
                    continue;
                }
                let msg = format!(
                    "`{}` is deprecated: {}",
                    used_name.to_string(),
                    info.message
                );
                let mut err = if promote_to_error {
                    Error::from_msg_srcs(msg, &[&use_src])
                } else {
                    Error::warning_from_msg_srcs(msg, &[&use_src])
                };
                err.code = Some(WARN_DEPRECATED);
                diagnostics.append(Errors::from_err(err));
            }
        }
        diagnostics
    }

    /// Resolve `DEPRECATED[...]` pragmas against the program and attach
    /// `DeprecationInfo` to the matching `GlobalValue` or `TraitMember`.
    ///
    /// Must run before `create_trait_member_symbols` so that trait-member
    /// deprecation is propagated naturally into the per-impl `GlobalValue`s.
    pub fn identify_deprecation_targets(&mut self) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        // Clone so the originals stay on `self.deprecation_statements` for
        // later phases that look up pragma spans by name.
        let stmts = self.deprecation_statements.clone();

        // Track which targets have already been deprecated, to detect duplicates.
        let mut already_deprecated: Map<FullName, Option<Span>> = Map::default();

        for stmt in stmts {
            // Reject absolute-path forms with a friendly error.
            if stmt.target_path.namespace.is_absolute {
                errors.append(Errors::from_msg_srcs(
                    "`DEPRECATED` cannot take an absolute path. Use a path relative to where the pragma is written.".to_string(),
                    &[&stmt.src],
                ));
                continue;
            }

            // Reject duplicate pragmas pointing at the same target.
            if let Some(prev_src) = already_deprecated.get(&stmt.target_path) {
                errors.append(Errors::from_msg_srcs(
                    format!(
                        "Multiple `DEPRECATED` pragmas for the same target `{}`.",
                        stmt.target_path.to_string()
                    ),
                    &[&stmt.src, prev_src],
                ));
                continue;
            }

            let info = DeprecationInfo {
                message: stmt.message.clone(),
            };

            // Try direct global value lookup.
            if let Some(gv) = self.global_values.get_mut(&stmt.target_path) {
                gv.deprecation = Some(info);
                already_deprecated.insert(stmt.target_path.clone(), stmt.src.clone());
                continue;
            }

            // Try trait member lookup: split target into (trait, member).
            if let Some((trait_id, member_name)) = TraitId::split_member_fullname(&stmt.target_path)
            {
                if let Some(trait_defn) = self.trait_env.traits.get_mut(&trait_id) {
                    if let Some(member) = trait_defn
                        .members
                        .iter_mut()
                        .find(|m| m.name == member_name)
                    {
                        member.deprecation = Some(info);
                        already_deprecated.insert(stmt.target_path.clone(), stmt.src.clone());
                        continue;
                    }
                }
            }

            // Not found.
            let container = if stmt.origin_namespace.names.is_empty() {
                "the project root".to_string()
            } else {
                format!("`{}`", stmt.origin_namespace.to_string())
            };
            errors.append(Errors::from_msg_srcs(
                format!(
                    "`DEPRECATED` target `{}` was not found under {}. Targets must be a global value or a trait member declared as a child of where the pragma is written.",
                    stmt.target_path.to_string(),
                    container,
                ),
                &[&stmt.src],
            ));
        }

        errors.to_result()
    }

    pub fn set_kinds(&mut self) -> Result<(), Errors> {
        self.trait_env.set_kinds_in_trait_and_alias_defns()?;
        let kind_env = self.kind_env();
        self.trait_env.set_kinds_in_trait_instances(&kind_env)?;
        let mut errors = Errors::empty();
        for (_name, sym) in &mut self.global_values {
            errors.eat_err(sym.set_kinds(&kind_env));
        }
        errors.to_result()
    }

    pub fn kind_env(&self) -> KindEnv {
        KindEnv {
            tycons: self.type_env().kinds(),
            assoc_tys: self.trait_env.assoc_ty_kind_info(),
            traits_and_aliases: self.trait_env.trait_kind_map_with_aliases(),
        }
    }

    // Infer namespaces of traits and types that appear in declarations and associated type implementations.
    // NOTE: names in the lhs of definition of types/traits/global_values have to be full-named already when this function called.
    pub fn resolve_namespace_not_in_expr(&mut self) -> Result<(), Errors> {
        let env = self.create_name_resolution_env();
        let mut ctx = NameResolutionContext::new("NA".to_string(), env.clone());

        // Resolve namespaces in type constructors.
        {
            let mut tycons = (*self.type_env.tycons).clone();
            for (tc, ti) in &mut tycons {
                let module = tc.name.module();
                ctx.set_current_module(module);
                ti.resolve_namespace(&mut ctx)?;
            }
            self.type_env.tycons = Arc::new(tycons);
        }
        // Resolve namespaces in type aliases.
        {
            let mut aliases = (*self.type_env.aliases).clone();
            for (tc, ta) in &mut aliases {
                let module = tc.name.module();
                ctx.set_current_module(module);
                ta.resolve_namespace(&mut ctx)?;
            }
            self.type_env.aliases = Arc::new(aliases);
        }

        self.trait_env.resolve_namespace(&mut ctx)?;

        for decl in &mut self.type_defns {
            let module = decl.name.module();
            ctx.set_current_module(module);
            decl.resolve_namespace(&mut ctx)?;
        }

        for (name, sym) in &mut self.global_values {
            ctx.set_current_module(name.module());
            sym.resolve_namespace_in_declaration(&mut ctx)?;
        }

        self.merge_import_required(ctx.import_required);
        Ok(())
    }

    // Resolve type aliases in types that appear NOT in expressions.
    pub fn resolve_type_aliases_not_in_expr(&mut self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Resolve aliases in type constructors.
        errors.eat_err(self.type_env.resolve_type_aliases_in_tycons());
        errors.to_result()?;

        // Get the updated type env.
        let type_env = self.type_env();

        // Resolve aliases in trait env.
        errors.eat_err(self.trait_env.resolve_type_aliases(&type_env));

        // Resolve aliases in type definitions.
        for decl in &mut self.type_defns {
            errors.eat_err(decl.resolve_type_aliases(&type_env));
        }

        // Resolve aliases in type signatures of global values.
        for (_, sym) in &mut self.global_values {
            errors.eat_err(sym.resolve_type_aliases(&type_env));
        }

        errors.to_result()
    }

    // Validate user-defined types
    pub fn validate_type_defns(&self) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for type_defn in &self.type_defns {
            // Check for opaque type variables in type definitions.
            for tv in &type_defn.tyvars {
                if is_opaque_tyvar(&tv.name) {
                    errors.append(Errors::from_msg_srcs(
                        format!(
                            "Opaque type variable `{}` is not allowed in a type definition.",
                            tv.name,
                        ),
                        &[&type_defn.source.as_ref().map(|s| s.to_head_character())],
                    ));
                }
            }
            errors.eat_err(type_defn.validate_tyvars());
            if errors.has_error() {
                continue;
            }
            let type_name = &type_defn.name;
            match &type_defn.value {
                TypeDeclValue::Struct(str) => {
                    for field in &str.fields {
                        if !field.ty.is_assoc_ty_free() {
                            errors.append(Errors::from_msg_srcs(
                                "Associated type is not allowed in the field type of a struct."
                                    .to_string(),
                                &[&type_defn.source.as_ref().map(|s| s.to_head_character())],
                            ));
                        }
                    }
                    match Field::check_duplication(&str.fields) {
                        Some(field_name) => {
                            errors.append(Errors::from_msg_srcs(
                                format!(
                                    "Duplicate field `{}` in the definition of struct `{}`.",
                                    field_name,
                                    type_name.to_string()
                                ),
                                &[&type_defn.source.as_ref().map(|s| s.to_head_character())],
                            ));
                        }
                        _ => {}
                    }
                }
                TypeDeclValue::Union(union) => {
                    for field in &union.fields {
                        if !field.ty.is_assoc_ty_free() {
                            errors.append(Errors::from_msg_srcs(
                                "Associated type is not allowed in the field type of a union."
                                    .to_string(),
                                &[&type_defn.source.as_ref().map(|s| s.to_head_character())],
                            ));
                        }
                    }
                    match Field::check_duplication(&union.fields) {
                        Some(field_name) => {
                            errors.append(Errors::from_msg_srcs(
                                format!(
                                    "Duplicate field `{}` in the definition of union `{}`.",
                                    field_name,
                                    type_name.to_string()
                                ),
                                &[&type_defn.source.as_ref().map(|s| s.to_head_character())],
                            ));
                        }
                        _ => {}
                    }
                }
                TypeDeclValue::Alias(ta) => {
                    if !ta.value.is_assoc_ty_free() {
                        errors.append(Errors::from_msg_srcs(
                            "Associated type is not allowed in the right-hand side of a type alias.".to_string(),
                            &[&type_defn.source.as_ref().map(|s| s.to_head_character())],
                        ));
                    }
                } // Nothing to do.
            }
        }
        errors.to_result()
    }

    /// Validates the traits, the trait aliases and the trait implementations, structurally.
    pub fn validate_trait_env_structure(&self) -> Result<(), Errors> {
        self.trait_env.validate_structure()
    }

    /// Reports each pair of implementations of one trait whose heads can denote the same type.
    pub fn validate_overlapping_instances(&self) -> Result<(), Errors> {
        self.trait_env
            .validate_overlapping_instances(self.kind_env())
    }

    /// Reports each name that is used by more than one of the types, the traits and the associated
    /// types, aliases included.
    pub fn validate_capital_name_confliction(&self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        let types = self.tycon_names_with_aliases();
        let traits = self.trait_names_with_aliases();
        let assc_tys = self.assoc_ty_to_arity();

        // Check if there is a name confliction between types and traits.
        for name in types.iter() {
            if traits.contains(name) {
                errors.append(Errors::from_msg(format!(
                    "Name confliction: `{}` is both a type and a trait.",
                    name.to_string()
                )));
            }
        }

        // Check if there is a name confliction between types and traits.
        for name in types.iter() {
            if assc_tys.contains_key(name) {
                errors.append(Errors::from_msg(format!(
                    "Name confliction: `{}` is both a type and an associated type.",
                    name.to_string()
                )));
            }
        }

        // Check if there is a name confliction between traits and associated types.
        for name in traits.iter() {
            if assc_tys.contains_key(name) {
                errors.append(Errors::from_msg(format!(
                    "Name confliction: `{}` is both a trait and an associated type.",
                    name.to_string()
                )));
            }
        }

        errors.to_result()
    }

    pub fn add_methods(self: &mut Program) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for defn in &self.type_defns.clone() {
            match &defn.value {
                TypeDeclValue::Struct(str) => {
                    let struct_name = defn.name.clone();
                    for field in &str.fields {
                        // Add getter function
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", STRUCT_GETTER_SYMBOL, &field.name),
                            ),
                            struct_get(defn, &field.name),
                            Some(format!(
                                "Retrieves the field `{}` from a value of `{}`.",
                                &field.name, struct_name.name
                            )),
                        ));
                        // Add setter function
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", STRUCT_SETTER_SYMBOL, &field.name),
                            ),
                            struct_set(&struct_name, defn, &field.name),
                            Some(format!(
                                "Updates a value of `{}` by setting field `{}` to a specified one.",
                                struct_name.name, &field.name,
                            )),
                        ));
                        // Add modifier functions.
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", STRUCT_MODIFIER_SYMBOL, &field.name,),
                            ),
                            struct_mod(defn, &field.name),
                            Some(format!(
                                "Updates a value of `{}` by applying a function to field `{}`.",
                                struct_name.name, &field.name,
                            )),
                        ));
                        // Add act functions
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", STRUCT_ACT_SYMBOL, &field.name),
                            ),
                            struct_act(&struct_name, defn, &field.name),
                            Some(format!(
                                "Updates a value of `{}` by applying a functorial action to field `{}`.",
                                struct_name.name, &field.name,
                            )),
                        ));
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("_{}{}_identity", STRUCT_ACT_SYMBOL, &field.name),
                            ),
                            struct_act_identity(&struct_name, defn, &field.name),
                            Some(format!(
                                "Optimized implementation of `act_{{field}}` function for `Identity` functor."
                            )),
                        ));
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("_{}{}_const", STRUCT_ACT_SYMBOL, &field.name),
                            ),
                            struct_act_const(&struct_name, defn, &field.name),
                            Some(format!(
                                "Optimized implementation of `act_{{field}}` function for `Const r` functor."
                            )),
                        ));
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("_{}{}_tuple2", STRUCT_ACT_SYMBOL, &field.name),
                            ),
                            struct_act_tuple2(&struct_name, defn, &field.name),
                            Some(format!(
                                "Optimized implementation of `act_{{field}}` function for `Tuple2 x` functor."
                            )),
                        ));
                        // Add punch functions.
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", STRUCT_PUNCH_SYMBOL, &field.name),
                            ),
                            struct_punch(defn, &field.name, false),
                            None,
                        ));
                        // Add plug-in functions.
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", STRUCT_PLUG_IN_SYMBOL, &field.name),
                            ),
                            struct_plug_in(defn, &field.name, false),
                            None,
                        ));
                        // Add punch functions (force-unique version)
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL, &field.name),
                            ),
                            struct_punch(defn, &field.name, true),
                            None,
                        ));
                        // Add plug-in functions (force-unique version)
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", STRUCT_PLUG_IN_FORCE_UNIQUE_SYMBOL, &field.name),
                            ),
                            struct_plug_in(defn, &field.name, true),
                            None,
                        ));
                    }
                }
                TypeDeclValue::Union(union) => {
                    let union_name = &defn.name;
                    for field in &union.fields {
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(&defn.name.to_namespace(), &field.name),
                            union_new(&union_name, &field.name, defn),
                            Some(format!(
                                "Constructs a value of union `{}` taking the variant `{}`.",
                                union_name.name, &field.name
                            )),
                        ));
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", UNION_AS_SYMBOL, field.name),
                            ),
                            union_as(&field.name, defn),
                            Some(format!(
                                "Unwraps a union value of `{}` as the variant `{}`.\nIf the value is not the variant `{}`, this function aborts the program.",
                                union_name.name, &field.name, &field.name,
                            )),
                        ));
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", UNION_IS_SYMBOL, field.name),
                            ),
                            union_is(&field.name, defn),
                            Some(format!(
                                "Checks if a union value of `{}` is the variant `{}`.",
                                union_name.name, &field.name,
                            )),
                        ));
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("{}{}", UNION_MOD_SYMBOL, field.name),
                            ),
                            union_mod_function(&union_name, &field.name, defn),
                            Some(format!(
                                "Updates a value of union `{}` by applying a function if it is the variant `{}`, or doing nothing otherwise.",
                                union_name.name, &field.name,
                            )),
                        ));
                    }
                }
                TypeDeclValue::Alias(_) => {} // Nothing to do
            }
        }
        errors.to_result()?;
        Ok(())
    }

    // Add `Std::Boxed` implementations for all user-defined boxed types.
    pub fn add_boxed_impls(&mut self) -> Result<(), Errors> {
        for defn in &self.type_defns {
            match &defn.value {
                TypeDeclValue::Struct(str) => {
                    if str.is_boxed() {
                        let ty = defn.applied_type();
                        self.trait_env.add_instance(boxed_trait_instance(&ty))?;
                    }
                }
                TypeDeclValue::Union(union) => {
                    if union.is_boxed() {
                        let ty = defn.applied_type();
                        self.trait_env.add_instance(boxed_trait_instance(&ty))?;
                    }
                }
                TypeDeclValue::Alias(_) => {} // Nothing to do
            }
        }
        Ok(())
    }

    pub fn linked_mods(&self) -> Set<Name> {
        self.mod_to_import_stmts.keys().cloned().collect()
    }

    // Link an module.
    // * extend - If true, the module defined in `other` allowed to conflict with a module already in `self`.
    //            This is used for extending implementation of a module already linked to `self`.
    pub fn link(&mut self, mut other: Program, extend: bool) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Merge `module_to_files`.
        // Also, check if there is a module defined in multiple files.
        for mod_info in &other.modules {
            let file = mod_info.source.input.file_path.clone();
            if let Some(defined_at) = self.modules.iter().position(|mi| mi.name == mod_info.name) {
                // If the module is already defined,
                if extend {
                    // If extending mode, this is not a problem: every source the module here is
                    // made of joins the ones the module is made of already, and
                    // `module_dependency_hash` reads them all.
                    let joining = mod_info.sources().cloned().collect::<Vec<_>>();
                    Arc::make_mut(&mut self.modules[defined_at].extending_sources).extend(joining);
                    continue;
                }
                let other_file = self.modules[defined_at].source.input.file_path.clone();
                if to_absolute_path(&other_file)? == to_absolute_path(&file)? {
                    // If the module is defined in the same file, this is not a problem.
                    continue;
                }
                let msg = format!(
                    "Module `{}` is defined in two files: \"{}\" and \"{}\".",
                    mod_info.name,
                    other_file.to_string_lossy().to_string(),
                    file.to_string_lossy().to_string()
                );
                errors.append(Errors::from_msg(msg));
                continue;
            }
            self.modules.push(mod_info.clone());
        }

        // Throw an error if necessary.
        errors.to_result()?;

        // If already linked, do nothing.
        if !extend
            && self
                .linked_mods()
                .contains(&other.get_name_if_single_module())
        {
            return Ok(());
        }

        // Merge `mod_to_import_stmts`.
        for (importer, stmts) in &other.mod_to_import_stmts {
            insert_to_map_vec_many(&mut self.mod_to_import_stmts, importer, stmts.clone());
        }

        // Merge types.
        self.add_type_defns(other.type_defns);

        // Merge traits and instances.
        errors.eat_err(self.trait_env.import(other.trait_env));

        // Merge global values. A trait member's symbol is built by `create_trait_member_symbols`,
        // which runs after every link, so each value here is a simple one.
        for (name, gv) in other.global_values {
            assert!(
                gv.is_simple_value(),
                "`{}` is a trait member before its symbols are created.",
                name.to_string()
            );
            errors.eat_err(self.add_global_value_gv(name, gv));
        }

        // Merge export statements.
        self.export_statements.append(&mut other.export_statements);

        // Merge deprecation statements.
        self.deprecation_statements
            .append(&mut other.deprecation_statements);

        // Merge used_tuple_sizes.
        self.used_tuple_sizes.append(&mut other.used_tuple_sizes);

        errors.to_result()
    }

    // Check that all imported modules are linked.
    pub fn check_imports(&mut self) -> Result<(), Errors> {
        let mut unresolved_imports = self.import_statements();

        loop {
            if unresolved_imports.is_empty() {
                break Ok(());
            }
            let import_stmt = unresolved_imports.pop().unwrap();
            let module = &import_stmt.module.0;

            // If import is already resolved, do nothing.
            if self.is_linked(&module) {
                continue;
            }

            // `module.1` carries the span of the `Mod` token wherever
            // the user wrote it — inside an `import Mod;` line for
            // user imports, or inside a `::Mod::name` expression for
            // the parser-synthesised per-absolute-path imports — so a
            // single field gives us a good error location for both.
            return Err(Errors::from_msg_srcs(
                format!("Cannot find module `{}`.", module),
                &[&import_stmt.module.1],
            ));
        }
    }

    // Create a graph of modules. If module A imports module B, an edge from A to B is added.
    pub fn importing_module_graph(&self) -> (Graph<Name>, Map<Name, usize>) {
        let (mut graph, elem_to_idx) = Graph::from_set(self.linked_mods());
        for (importer, stmts) in &self.mod_to_import_stmts {
            let importer_idx = *elem_to_idx.get(importer).unwrap();
            for stmt in stmts {
                graph.connect_idx(importer_idx, *elem_to_idx.get(&stmt.module.0).unwrap());
            }
        }
        (graph, elem_to_idx)
    }

    // Calculate a set of modules on which a module depends.
    pub fn dependent_modules(&self, module: &Name) -> Set<Name> {
        let (importing_graph, mod_to_node) = self.importing_module_graph();
        importing_graph
            .reachable_nodes(*mod_to_node.get(module).unwrap())
            .iter()
            .map(|idx| importing_graph.get(*idx).clone())
            .collect()
    }

    // Calculate a map from a module to a set of modules on which the module depends.
    pub fn module_dependency_map(&self) -> Map<Name, Set<Name>> {
        // TODO: Improve time complexity.
        let mods = self.linked_mods();
        let mut dependency = Map::default();
        for module in &mods {
            dependency.insert(module.clone(), self.dependent_modules(&module));
        }
        dependency
    }

    /// A hash naming everything a value defined in `module` is type-checked from.
    ///
    /// It keys the type-checking cache, of the batch compiler and of the LSP alike, so it covers
    /// every input that decides what the check produces:
    ///
    /// - **Every source each module `module` depends on is made of.** A module is made of the file
    ///   it is declared in, and of the sources linked to extend it — the definitions the compiler
    ///   writes itself, which vary with the program (see `ModuleInfo::extending_sources`).
    /// - **The settings that decide what the elaborated program is**, which reach it without
    ///   passing through any source (`Configuration::elaboration_hash`).
    /// - **The build of the compiler.** A cached typed expression is serialized in a format the
    ///   compiler defines, and a differently-built compiler may define it differently — reading such
    ///   a cache back would misinterpret it. `build_time_utc!()` changes with every compiler build.
    ///
    /// Every value goes in through `HashSource`, which gives it a length of its own, so where one
    /// value ends and the next begins never depends on what the values are.
    pub fn module_dependency_hash(
        &self,
        module: &Name,
        config: &Configuration,
    ) -> Result<String, Errors> {
        let mut dependent_module_names = self
            .dependent_modules(module)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        dependent_module_names.sort(); // To remove randomness introduced by HashSet, we sort it.
        let mut hash_source = HashSource::default();
        for mod_name in &dependent_module_names {
            hash_source.push_list(&self.find_mod(mod_name).unwrap().source_hashes()?);
        }
        hash_source.push_text(&config.elaboration_hash());
        hash_source.push_text(build_time::build_time_utc!());
        Ok(hash_source.finish())
    }

    // Calculate a map from a module to a hash value of the module which is affected by source codes of all dependent modules.
    pub fn module_dependency_hash_map(&self, config: &Configuration) -> Map<Name, String> {
        // TODO: Improve time complexity.
        let mods = self.linked_mods();
        let mut mod_to_hash = Map::default();
        for module in &mods {
            mod_to_hash.insert(
                module.clone(),
                panic_if_err(self.module_dependency_hash(&module, config)),
            );
        }
        mod_to_hash
    }

    // Check if all items referred in import statements are defined.
    pub fn validate_import_statements(&self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        let stmts = self.import_statements();
        let items = stmts.iter().map(|stmt| stmt.referred_items()).flatten();

        let values = self.global_values.keys().collect::<Set<_>>();
        let types = self.tycon_names_with_aliases();
        let traits = self.trait_names_with_aliases();
        let assoc_tys = self.assoc_ty_to_arity();

        for item in items {
            match item {
                ImportItem::Symbol(name, src) => {
                    if values.contains(&name) {
                        continue;
                    }
                    errors.append(Errors::from_msg_srcs(
                        format!("Cannot find value named `{}`.", name.to_string()),
                        &[&src],
                    ));
                }
                ImportItem::TypeOrTrait(name, src) => {
                    if types.contains(&name)
                        || traits.contains(&name)
                        || assoc_tys.contains_key(&name)
                    {
                        continue;
                    }
                    errors.append(Errors::from_msg_srcs(
                        format!("Cannot find entity named `{}`.", name.to_string()),
                        &[&src],
                    ));
                }
                ImportItem::NameSpace(namespace, src) => {
                    // Search for an entity that is in the namespace.
                    if values.iter().any(|name| name.is_in_namespace(&namespace)) {
                        continue;
                    }
                    if types.iter().any(|name| name.is_in_namespace(&namespace)) {
                        continue;
                    }
                    if traits.iter().any(|name| name.is_in_namespace(&namespace)) {
                        continue;
                    }
                    errors.append(Errors::from_msg_srcs(
                        format!(
                            "Namespace `{}` is not defined or empty.",
                            namespace.to_string()
                        ),
                        &[&src],
                    ));
                }
            }
        }
        errors.to_result()
    }

    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        for (name, gv) in &self.global_values {
            let node = gv.find_node_at(name, pos);
            if node.is_some() {
                return node;
            }
        }
        for td in &self.type_defns {
            let node = td.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        let node = self.trait_env.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        // Cursor on the name token of an `FFI_EXPORT[...]` pragma.
        for stmt in &self.export_statements {
            if let Some(span) = &stmt.value_name_src {
                if span.includes_pos_lsp(pos) {
                    return Some(EndNode::Expr(Var::create(stmt.value_name.clone()), None));
                }
            }
        }
        // Cursor on the name token of a `DEPRECATED[...]` pragma.
        for stmt in &self.deprecation_statements {
            if let Some(span) = &stmt.target_name_src {
                if span.includes_pos_lsp(pos) {
                    return Some(EndNode::Expr(Var::create(stmt.target_path.clone()), None));
                }
            }
        }
        let mod_name = self
            .modules_from_files(&vec![pos.input.file_path.clone()])
            .ok()?
            .pop()?;
        for stmt in self
            .mod_to_import_stmts
            .get(&mod_name)
            .unwrap_or(&vec![])
            .iter()
        {
            let node = stmt.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }

        None
    }

    pub fn stringify_symbols(&self) -> Text {
        let mut sym_texts: Vec<(String, Text)> = vec![];
        for sym in self.symbols.values() {
            let mut sym_text = Text::empty();

            let type_sgn_str = format!("{} : {};", sym.name.to_string(), sym.ty.to_string());
            let type_sgn = Text::from_str(&type_sgn_str);
            sym_text = sym_text.append(type_sgn);

            let code = Text::from_str(&format!("{} = ", sym.name.to_string()))
                .append_nobreak(
                    sym.expr
                        .as_ref()
                        .unwrap()
                        .expr
                        .stringify()
                        .brace_if_multiline(),
                )
                .append_to_last_line(";");
            sym_text = sym_text.append(code);

            sym_texts.push((type_sgn_str, sym_text));
        }
        sym_texts.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut text = Text::empty();
        for (_, sym_text) in sym_texts {
            text = text.append(sym_text);
            text = text.append(Text::from_str(""));
        }

        text
    }

    pub fn emit_symbols(&self, step_name: &str) {
        let file_name = format!("{}/{}.symbols.fix", DOT_FIXLANG, step_name);
        let file_path = PathBuf::from(file_name);

        let text = self.stringify_symbols().to_string();
        let mut file = File::create(&file_path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }

    pub fn create_typechecker(&self, config: &Configuration) -> TypeCheckContext {
        // Error tolerance is opt-in via the diagnostics-mode config;
        // every other subcommand stays strict.
        let error_tolerant = matches!(
            &config.subcommand,
            SubCommand::Diagnostics(d) if d.error_tolerant
        );
        let mut typechecker = TypeCheckContext::new(
            self.trait_env.clone(),
            self.type_env(),
            self.kind_env(),
            self.mod_to_import_stmts.clone(),
            config.type_check_cache.clone(),
            config.num_worker_thread,
            error_tolerant,
        );

        // Register type declarations of global symbols to typechecker.
        let globals = self
            .global_values
            .iter()
            .map(|(name, defn)| (name.clone(), defn.scm.clone()))
            .collect::<Vec<_>>();
        typechecker.scope.set_globals(globals);

        typechecker
    }
}

#[derive(Serialize, Deserialize)]
pub enum EndNode {
    Expr(Var, Option<Arc<TypeNode>>),
    Pattern(Var, Option<Arc<TypeNode>>),
    Type(TyCon),
    Trait(TraitId),
    TypeOrTrait(FullName), // Unknown whether Type or Trait
    Module(Name),
    // The definition name (left-hand side) of a global value declaration.
    ValueDecl(FullName),
    // An associated type name (e.g., `Item` in `Item iter`).
    AssocType(AssocType),
    // A struct field name; the cursor is on the bare name in the type
    // definition or on a MakeStruct / Pattern::Struct field-name use.
    Field(TyCon, Name),
    // A union variant name; the cursor is on the bare name in the type
    // definition or on a Pattern::Union variant-name use.
    Variant(TyCon, Name),
    // The type inferred for a `_` type wildcard; the cursor is on the wildcard
    // in a type annotation. Carries the resolved type so hover can display it.
    InferredType(Arc<TypeNode>),
}
