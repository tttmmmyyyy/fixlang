use crate::ast::export_statement::{ExportStatement, ExportedFunctionType, IOType};
use crate::error::{Error, Errors};
use import::{ImportItem, ImportStatement};
use misc::{collect_results, to_absolute_path, Map, Set};
use name::{FullName, Name};
use printer::Text;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::{sync::Arc, vec};

use super::*;

#[derive(Clone)]
pub struct TypeEnv {
    // List of type constructors including user-defined types.
    pub tycons: Arc<Map<TyCon, TyConInfo>>,
    // List of type aliases.
    pub aliases: Arc<Map<TyCon, TyAliasInfo>>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self {
            tycons: Arc::new(Default::default()),
            aliases: Arc::new(Default::default()),
        }
    }
}

impl TypeEnv {
    pub fn new(tycons: Map<TyCon, TyConInfo>, aliases: Map<TyCon, TyAliasInfo>) -> TypeEnv {
        TypeEnv {
            tycons: Arc::new(tycons),
            aliases: Arc::new(aliases),
        }
    }

    pub fn add_tycons(&mut self, new_tycons: Map<TyCon, TyConInfo>) {
        let mut tycons = self.tycons.as_ref().clone();
        for (tc, ti) in new_tycons.into_iter() {
            tycons.insert(tc.clone(), ti);
        }
        self.tycons = Arc::new(tycons);
    }

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

    // Check if the given function is `act_{field}` function for a field of a struct.
    pub fn is_struct_act(&self, name: &FullName) -> bool {
        if name.is_local() {
            return false;
        }
        let str_name = name.namespace.clone().to_fullname();
        match self.tycons.get(&TyCon { name: str_name }) {
            Some(tycon_info) => {
                if tycon_info.variant != TyConVariant::Struct {
                    return false;
                }
                tycon_info.fields.iter().any(|f| {
                    let act_func_name = format!("{}{}", STRUCT_ACT_SYMBOL, f.name);
                    act_func_name == name.name
                })
            }
            None => false,
        }
    }
}

// Symbols are Fix values that are instantiated:
// their types are fixed to concrete types, and given unique names.
#[derive(Clone)]
pub struct Symbol {
    pub name: FullName,
    pub generic_name: FullName,
    pub ty: Arc<TypeNode>,
    pub expr: Option<Arc<ExprNode>>,
}

impl Symbol {
    // The set of modules that this symbol depends on.
    // If any of these modules, or any of their importee are changed, then they are required to be re-compiled.
    // Note that this set may not be fully spanned in the importing graph.
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
}

// Declaration (name and type signature) of global value.
// `main : IO()`
pub struct GlobalValueDecl {
    pub name: FullName,
    pub ty: Arc<Scheme>,
    pub src: Option<Span>,
}

// Definition (name and expression)
// `main = println("Hello World")`
pub struct GlobalValueDefn {
    pub name: FullName,
    pub expr: Arc<ExprNode>,
    pub src: Option<Span>,
}

// The global value, which is either a value or trait method.
pub struct GlobalValue {
    // Type of this symbol.
    // For example, in case `trait a : Show { show : a -> String; }`, the type of method `show` is `[a : Show] a -> String`.
    pub scm: Arc<Scheme>,
    // Type of this symbol, with aliases retained.
    pub syn_scm: Option<Arc<Scheme>>,
    // The expression or implementation of this value.
    pub expr: SymbolExpr,
    // Source code where this value is defined.
    // For trait methods, this is the source code where the trait method is defined.
    pub def_src: Option<Span>,
    // The document of this value.
    // If `def_src` is available, we can also get document from the source code.
    // We use this field only when document is not available in the source code.
    pub document: Option<String>,
    // Is this value compiler-defined method?
    // True for methods such as `@{field}`, `set_{field}`, etc.
    // If true, this value is not shown in the document generated by `fix docs`.
    pub compiler_defined_method: bool,
}

impl GlobalValue {
    pub fn resolve_namespace_in_declaration(
        &mut self,
        ctx: &NameResolutionContext,
    ) -> Result<(), Errors> {
        // If this function is called for methods, we must call resolve_namespace on MethodImpl.ty.
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
                    m.ty = m.ty.set_kinds(kind_env)?;
                    m.ty.check_kinds(kind_env)?;
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
            .def_src
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
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        let node = self.expr.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        self.scm.find_node_at(pos)
    }
}

// Expression of global symbol.
#[derive(Clone)]
pub enum SymbolExpr {
    Simple(TypedExpr),       // Definition such as "id : a -> a; id = \x -> x".
    Method(Vec<MethodImpl>), // Trait method implementations.
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
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        match self {
            SymbolExpr::Simple(e) => e.find_node_at(pos),
            SymbolExpr::Method(ms) => ms.iter().filter_map(|m| m.find_node_at(pos)).next(),
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
}

impl TypedExpr {
    pub fn from_expr(expr: Arc<ExprNode>) -> Self {
        TypedExpr {
            expr,
            equalities: vec![],
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

// Trait method implementation
#[derive(Clone)]
pub struct MethodImpl {
    // Type of this method.
    // For example, in case "impl [a: Show, b: Show] (a, b): Show {...}",
    // the type of method "show" is "[a: Show, b: Show] (a, b) -> String",
    pub ty: Arc<Scheme>,
    // Expression of this implementation
    pub expr: TypedExpr,
    // Module where this implmentation is given.
    // NOTE:
    // For trait method, `define_module` may differ to the first component of namespace of the function.
    // For example, if `Main` module implements `SomeType : Eq`, then implementation of `eq` for `SomeType` is defined in `Main` module,
    // but its name as a function is still `Std::Eq::eq`.
    pub define_module: Name,
}

impl MethodImpl {
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.ty = self.ty.resolve_type_aliases(type_env)?;
        Ok(())
    }

    // Find the minimum expression node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        self.expr.find_node_at(pos)
    }
}

pub struct NameResolutionContext {
    pub candidates: Map<FullName, NameResolutionType>,
    pub assoc_ty_to_arity: Map<FullName, usize>,
    pub import_statements: Vec<ImportStatement>,
}

impl<'a> NameResolutionContext {
    pub fn new(
        tycon_names_with_aliases: &Set<FullName>,
        trait_names_with_aliases: &Set<FullName>,
        assoc_ty_to_arity: Map<FullName, usize>,
        import_statements: Vec<ImportStatement>,
    ) -> Self {
        let mut candidates: Map<FullName, NameResolutionType> = Map::default();
        fn check_insert(
            candidates: &mut Map<FullName, NameResolutionType>,
            name: FullName,
            nrt: NameResolutionType,
        ) {
            assert!(!candidates.contains_key(&name) || candidates[&name] == nrt); // This is assured by `validate_capital_name_confliction`.
            candidates.insert(name, nrt);
        }
        for name in tycon_names_with_aliases {
            check_insert(&mut candidates, name.clone(), NameResolutionType::TyCon);
        }
        for name in trait_names_with_aliases {
            check_insert(&mut candidates, name.clone(), NameResolutionType::Trait);
        }
        for (name, _arity) in &assoc_ty_to_arity {
            check_insert(&mut candidates, name.clone(), NameResolutionType::AssocTy);
        }
        NameResolutionContext {
            candidates,
            import_statements,
            assoc_ty_to_arity,
        }
    }

    pub fn resolve(
        &self,
        short_name: &FullName,
        accept_types: &[NameResolutionType],
        span: &Option<Span>,
    ) -> Result<FullName, Errors> {
        let accept_type_string = accept_types
            .iter()
            .map(|nrt| nrt.to_string())
            .collect::<Vec<_>>()
            .join(" or ");
        let candidates = self
            .candidates
            .iter()
            .filter_map(|(full_name, nrt)| {
                if !import::is_accessible(&self.import_statements, full_name) {
                    None
                } else if !accept_types.contains(nrt) {
                    None
                } else if !short_name.is_suffix(full_name) {
                    None
                } else {
                    Some(full_name.clone())
                }
            })
            .collect::<Vec<_>>();
        if candidates.len() == 0 {
            let mut err = Error::from_msg_srcs(
                format!(
                    "Unknown {} name `{}`.",
                    accept_type_string,
                    short_name.to_string()
                ),
                &[span],
            );
            err.code = Some(ERR_UNKNOWN_NAME);
            err.data = Some(serde_json::Value::String(short_name.to_string()));
            Err(Errors::from_err(err))
        } else if candidates.len() == 1 {
            Ok(candidates[0].clone())
        } else {
            // candidates.len() >= 2
            let msg = NameResolutionContext::create_ambiguous_message(
                &short_name.to_string(),
                candidates,
                false,
            );
            Err(Errors::from_msg_srcs(msg, &[span]))
        }
    }

    pub fn create_ambiguous_message(
        short_name: &str,
        mut candidates: Vec<FullName>,
        add_type_annotation: bool,
    ) -> String {
        candidates.sort(); // Sort for deterministic error message.

        // Join the candidates with ", ".
        let candidates_str = candidates
            .iter()
            .map(|fullname| "`".to_string() + &fullname.to_string() + "`")
            .collect::<Vec<_>>()
            .join(", ");

        // The Error message.
        let mut msg = format!(
            "Name `{}` is ambiguous: there are {}. Add (a suffix of) its namespace{} to help overloading resolution.",
            short_name,
            candidates_str,
            if add_type_annotation { " or type annotation" } else { "" }
        );

        // Check if there is candidates (x, y) such that x is a suffix of y.
        let mut suffixes = vec![];
        for i in 0..candidates.len() {
            for j in 0..candidates.len() {
                if i != j
                    && candidates[i]
                        .namespace
                        .is_suffix_of(&candidates[j].namespace)
                {
                    suffixes.push(candidates[i].clone());
                }
            }
        }
        // If there are suffixes, notify the user that they can use absolute namespace.
        if suffixes.len() > 0 {
            msg += &format!(
                " Here, you need to use absolute namespaces to specify {}; i.e., write as {}.",
                suffixes
                    .iter()
                    .map(|fullname| format!("`{}`", fullname.to_string()))
                    .collect::<Vec<_>>()
                    .join(", "),
                suffixes
                    .iter()
                    .map(|fullname| format!("`::{}`", fullname.to_string()))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }

        msg
    }
}

#[derive(PartialEq, Eq)]
pub enum NameResolutionType {
    TyCon,
    Trait,
    AssocTy,
}

impl NameResolutionType {
    pub fn to_string(&self) -> &'static str {
        match self {
            NameResolutionType::TyCon => "type",
            NameResolutionType::Trait => "trait",
            NameResolutionType::AssocTy => "associated type",
        }
    }
}

// Module information.
#[derive(Clone)]
pub struct ModuleInfo {
    // Module name.
    pub name: Name,
    // Source code location where this module is defined.
    pub source: Span,
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
    // List of tuple sizes used in this program.
    pub used_tuple_sizes: Vec<u32>,
    // Import statements.
    // Key is the name of the importer module.
    // Each module implicitly imports itself.
    // This is used to namespace resolution and overloading resolution.
    pub mod_to_import_stmts: Map<Name, Vec<ImportStatement>>,

    /* Instantiated symbols */
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

    /* Optimization */
    // Number of optimization steps.
    // This is used to name the symbol files when outputting them at each optimization step.
    pub optimization_step: usize,
}

impl Program {
    // Get the names of entry pointes / exported functions.
    pub fn root_value_names(&self) -> Vec<FullName> {
        let mut res = vec![];
        if let Some(entry) = self.entry_io_value.as_ref() {
            res.push(entry.get_var().name.clone());
        }
        for stmt in &self.export_statements {
            if let Some(exported) = stmt.value_expr.as_ref() {
                res.push(exported.get_var().name.clone());
            }
        }
        res
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
            deferred_errors: Errors::empty(),
            optimization_step: 0,
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
        let used_tuple_sizes = std::mem::replace(&mut self.used_tuple_sizes, vec![]);
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
            *stmts = std::mem::replace(stmts, vec![])
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
        trait_infos: Vec<TraitInfo>,
        trait_impls: Vec<TraitInstance>,
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
            type_decl.set_kinds_in_value();

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
            // If the type is a boxed struct, add punched struct types to tycons.
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

    pub fn traits_with_aliases(&self) -> Vec<Trait> {
        self.trait_env.traits_with_aliases()
    }

    // Add a global value.
    pub fn add_global_value(
        &mut self,
        name: FullName,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
        def_src: Option<Span>,
        document: Option<String>,
    ) -> Result<(), Errors> {
        self.add_global_value_common(name, (expr, scm), def_src, document, false)
    }

    // Add a compiler-defined method.
    pub fn add_compiler_defined_method(
        &mut self,
        name: FullName,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
        document: Option<String>,
    ) -> Result<(), Errors> {
        self.add_global_value_common(name, (expr, scm), None, document, true)
    }

    // Add a global value.
    fn add_global_value_common(
        &mut self,
        name: FullName,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
        def_src: Option<Span>,
        document: Option<String>,
        compiler_defined_method: bool,
    ) -> Result<(), Errors> {
        let gv = GlobalValue {
            scm: scm.clone(),
            syn_scm: None,
            expr: SymbolExpr::Simple(TypedExpr::from_expr(expr)),
            def_src,
            document,
            compiler_defined_method,
        };
        self.add_global_value_gv(name, gv)
    }

    // Add a global value.
    pub fn add_global_value_gv(&mut self, name: FullName, gv: GlobalValue) -> Result<(), Errors> {
        // Check duplicate definition.
        if self.global_values.contains_key(&name) {
            let this = gv.def_src.map(|s| s.to_head_character());
            let other = self
                .global_values
                .get(&name)
                .unwrap()
                .def_src
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

    // Add global values.
    pub fn add_global_values(
        &mut self,
        exprs: Vec<GlobalValueDefn>,
        types: Vec<GlobalValueDecl>,
    ) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        struct GlobalValue {
            defn: Option<GlobalValueDefn>,
            decl: Option<GlobalValueDecl>,
        }
        let mut global_values: Map<FullName, GlobalValue> = Default::default();

        // Register definitions checking duplication.
        for defn in exprs {
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

        // Register declarations checking duplication.
        for decl in types {
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

        // Check that definitions and declarations are paired.
        for (name, gv) in global_values {
            if gv.defn.is_none() {
                errors.append(Errors::from_msg_srcs(
                    format!("Global value `{}` lacks its expression.", name.to_string()),
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
                errors.eat_err(self.add_global_value(
                    name,
                    (gv.defn.unwrap().expr, gv.decl.unwrap().ty),
                    decl_src,
                    None,
                ));
            }
        }

        errors.to_result()
    }

    // This function performs the following tasks:
    // - Resolve namespace of type and traits in the expression.
    // - Resolve type aliases in the expression.
    // - Perform typechecking.
    //
    // Parameters:
    // - `te` : The expression to be namespace-resolved and type-checked.
    // - `req_scm` : The type scheme that the expression should have.
    // - `val_name` : The name of the expression, e.g., `Std::ToString::to_string`.
    // - `def_mod` : The module where the expression is defined. Note that if `te` is a trait method implementation, this may differ from `name.module()`.
    // - `nrctx` : The name resolution context. Pass one created by `program.create_name_resolution_context(define_module)`.
    // - `ver_hash` : A hash value of source codes `te` depends on. This is used to detect or invalidate the cache file. Pass one created by `program.module_dependency_hash(define_module)`.
    fn resolve_namespace_and_check_type_sub(
        mut te: TypedExpr,
        req_scm: &Arc<Scheme>,
        val_name: &FullName,
        def_mod: &Name,
        nrctx: &NameResolutionContext,
        ver_hash: &str,
        mut tc: TypeCheckContext,
    ) -> Result<TypedExpr, Errors> {
        // Load type-checking cache file.
        let cache = tc.cache.load_cache(val_name, req_scm, ver_hash);
        if cache.is_some() {
            // If cache is available,
            te = cache.unwrap();
            return Ok(te);
        }

        // Perform namespace inference.
        te.expr = te.expr.resolve_namespace(&nrctx)?;

        // Resolve type aliases in expression.
        te.expr = te.expr.resolve_type_aliases(&tc.type_env)?;

        // Perform type-checking.
        tc.current_module = Some(def_mod.clone());
        te.expr = tc.check_type(te.expr.clone(), req_scm.clone())?;
        te.equalities = tc.local_assumed_eqs;

        // Save the result to cache file.
        tc.cache.save_cache(&te, val_name, req_scm, ver_hash);

        Ok(te)
    }

    // Create NameResolutionContext used for symbols defined in the specified module.
    pub fn create_name_resolution_context(&self, mod_name: &Name) -> NameResolutionContext {
        NameResolutionContext::new(
            &self.tycon_names_with_aliases(),
            &self.trait_names_with_aliases(),
            self.assoc_ty_to_arity(),
            self.mod_to_import_stmts[mod_name].clone(),
        )
    }

    pub fn resolve_namespace_and_check_type_in_modules(
        &mut self,
        tc: &TypeCheckContext,
        modules: &[Name],
    ) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Names of global values to be checked.
        let mut checked_names: Vec<FullName> = vec![];
        for (name, gv) in self.global_values.iter() {
            match gv.expr {
                SymbolExpr::Simple(_) => {
                    // Check simple values only if they are in `modules`.
                    if modules.contains(&name.module()) {
                        checked_names.push(name.clone());
                    }
                }
                SymbolExpr::Method(_) => {
                    // We filter methods by `method_impl_filter`.
                    checked_names.push(name.clone());
                }
            }
        }

        // Method implementations to be checked.
        let modules = modules.to_vec();
        let method_impl_filter = |method: &MethodImpl| Ok(modules.contains(&method.define_module));

        errors.eat_err(self.resolve_namespace_and_check_type(
            tc,
            &checked_names,
            method_impl_filter,
        ));
        errors.to_result()
    }

    // Perform namespace resolution and type-checking for the specified expression.
    // This function updates `TypedExpr` in `self.global_values` in-place.
    pub fn resolve_namespace_and_check_type(
        &mut self,
        tc: &TypeCheckContext,
        val_names: &[FullName],
        method_impl_filter: impl Fn(&MethodImpl) -> Result<bool, Errors>,
    ) -> Result<(), Errors> {
        struct CheckTask {
            val_name: FullName,
            task: Box<dyn FnOnce() -> Result<TypedExpr, Errors> + Send>,
            method_impl_idx: Option<usize>,
        }
        let mut tasks: Vec<CheckTask> = vec![];

        let mut mod_to_nrctx: Map<Name, Arc<NameResolutionContext>> = Map::default();
        let mut get_nrctx = |mod_name: &Name| -> Arc<NameResolutionContext> {
            if !mod_to_nrctx.contains_key(mod_name) {
                mod_to_nrctx.insert(
                    mod_name.clone(),
                    Arc::new(self.create_name_resolution_context(mod_name)),
                );
            }
            mod_to_nrctx.get(mod_name).unwrap().clone()
        };

        // Create tasks.
        for val_name in val_names {
            let gv = self.global_values.get(&val_name).unwrap();
            match &gv.expr {
                SymbolExpr::Simple(e) => {
                    // Create a task for simple value.
                    let te = e.clone();
                    let scm = gv.scm.clone();
                    let val_name_clone = val_name.clone();
                    let def_mod = val_name_clone.module();
                    let nrctx = get_nrctx(&def_mod);
                    let ver_hash = self.module_dependency_hash(&def_mod)?;
                    let tc = tc.clone();
                    let task = Box::new(move || -> Result<TypedExpr, Errors> {
                        // Perform type-checking.
                        let te = Program::resolve_namespace_and_check_type_sub(
                            te,
                            &scm,
                            &val_name_clone,
                            &def_mod,
                            &nrctx,
                            &ver_hash,
                            tc,
                        )?;
                        Ok(te)
                    });

                    tasks.push(CheckTask {
                        val_name: val_name.clone(),
                        task,
                        method_impl_idx: None,
                    });
                }
                SymbolExpr::Method(impls) => {
                    for (i, method) in impls.iter().enumerate() {
                        // Select method implementation.
                        if !method_impl_filter(method)? {
                            continue;
                        }

                        // Create a task for method implementation.
                        let te = method.expr.clone();
                        let method_ty = method.ty.clone();
                        let val_name_clone = val_name.clone();
                        let def_mod = method.define_module.clone();
                        let nrctx = get_nrctx(&def_mod);
                        let ver_hash = self.module_dependency_hash(&def_mod)?;
                        let tc = tc.clone();
                        let task = Box::new(move || -> Result<TypedExpr, Errors> {
                            // Perform type-checking.
                            let te = Program::resolve_namespace_and_check_type_sub(
                                te,
                                &method_ty,
                                &val_name_clone,
                                &def_mod,
                                &nrctx,
                                &ver_hash,
                                tc,
                            )?;
                            Ok(te)
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
        struct CheckResult {
            val_name: FullName,
            te: Result<TypedExpr, Errors>,
            method_impl_idx: Option<usize>,
        }
        let results = if tc.num_worker_threads <= 1 || tasks.len() <= 1 {
            // Run tasks in the main thread.
            let mut results = vec![];
            for task in tasks {
                let te = (task.task)();
                results.push(CheckResult {
                    val_name: task.val_name,
                    te,
                    method_impl_idx: task.method_impl_idx,
                });
            }
            results
        } else {
            // Run tasks in parallel.
            let mut threads = vec![];
            let tasks_per_thread = tasks.len() / tc.num_worker_threads;
            for i in (0..tc.num_worker_threads).rev() {
                let mut tasks = if i == 0 {
                    std::mem::take(&mut tasks)
                } else {
                    tasks.split_off(tasks.len() - tasks_per_thread)
                };
                let thread = std::thread::spawn(move || {
                    let mut results = vec![];
                    while let Some(task) = tasks.pop() {
                        let te = (task.task)();
                        results.push(CheckResult {
                            val_name: task.val_name,
                            te,
                            method_impl_idx: task.method_impl_idx,
                        });
                    }
                    results
                });
                threads.push(thread);
            }
            let mut results = vec![];
            for thread in threads {
                results.append(&mut thread.join().unwrap());
            }
            results
        };

        // Store results to `self.global_values`.
        let mut errors = Errors::empty();
        for result in results {
            if result.te.is_err() {
                errors.append(result.te.err().unwrap());
                continue;
            }
            let te = result.te.ok().unwrap();
            let gv = self.global_values.get_mut(&result.val_name).unwrap();
            match &mut gv.expr {
                SymbolExpr::Simple(e) => {
                    *e = te;
                }
                SymbolExpr::Method(impls) => {
                    let i = result.method_impl_idx.unwrap();
                    impls[i].expr = te;
                }
            };
        }

        errors.to_result()
    }

    // Instantiate symbol.
    fn instantiate_symbol(
        &mut self,
        sym: &mut Symbol,
        tc: &TypeCheckContext,
    ) -> Result<(), Errors> {
        assert!(sym.expr.is_none());

        // First, perform namespace resolution and type-checking.
        let method_selector = |method: &MethodImpl| -> Result<bool, Errors> {
            // Select method implementation whose type unifies with the required type `sym.ty`.
            //
            // NOTE: Since overlapping implementations and unrelated methods are forbidden,
            // we only need to check the unifiability here,
            // and we do not need to check whether predicates or equality constraints are satisfiable or not.
            let mut tc0 = tc.clone();
            Ok(UnifOrOtherErr::extract_others(tc0.unify(&method.ty.ty, &sym.ty))?.is_ok())
        };
        self.resolve_namespace_and_check_type(tc, &[sym.generic_name.clone()], method_selector)?;

        // Then perform instantiation.
        let global_sym = self.global_values.get(&sym.generic_name).unwrap();
        let expr = match &global_sym.expr {
            SymbolExpr::Simple(e) => {
                // Specialize e's type to the required type `sym.ty`.
                let mut tc = tc.clone();
                tc.assert_freshness();
                tc.unify(e.expr.type_.as_ref().unwrap(), &sym.ty)
                    .ok()
                    .unwrap();
                for eq in &e.equalities {
                    tc.unify(&eq.lhs(), &eq.value).ok().unwrap();
                }
                tc.finalize_types(e.expr.clone())?
            }
            SymbolExpr::Method(impls) => {
                let mut opt_e: Option<Arc<ExprNode>> = None;
                for method in impls {
                    // Select method implementation.
                    if !method_selector(method)? {
                        continue;
                    }
                    let e = method.expr.clone();

                    // Specialize e's type to the required type `sym.ty`.
                    let mut tc = tc.clone();
                    tc.assert_freshness();
                    tc.unify(e.expr.type_.as_ref().unwrap(), &sym.ty)
                        .ok()
                        .unwrap();
                    for eq in &e.equalities {
                        tc.unify(&eq.lhs(), &eq.value).ok().unwrap();
                    }
                    opt_e = Some(tc.finalize_types(e.expr)?);
                    break;
                }
                opt_e.unwrap()
            }
        };
        sym.expr = Some(self.instantiate_expr(&expr)?);
        Ok(())
    }

    // Instantiate all symbols.
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

    // Instantiate `Main::main` (or `Test::test` if `fix test` is running).
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

    // Instantiate exported values.
    // In this function, `ExportStatement`s are updated.
    pub fn instantiate_exported_values(&mut self, tc: &TypeCheckContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        let mut export_stmts = std::mem::replace(&mut self.export_statements, vec![]);
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

    // Instantiate a global value.
    // - required_ty: for `Main::main`, pass `IO ()` to check that the specified type is correct. If None, then use the type specified by user.
    // - required_src: source place where the value is exported. Used to show error message.
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
                for (field_name, field_expr) in fields {
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
            Expr::FFICall(_, _, _, args, _) => {
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
        if !ret.type_.as_ref().unwrap().free_vars().is_empty() {
            return Err(Errors::from_msg_srcs(
                "Cannot infer the type of this expression because it contains an indeterminate type variable. Hint: you may fix this by adding a type annotation.".to_string(),
                &[&ret.source],
            ));
        }
        Ok(ret)
    }

    // Require instantiating a generic value such to a specified type.
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
            });
        }
        Ok(inst_name)
    }

    // Determine the name of instantiated generic value so that it has a specified type.
    // tc: a typechecker (substituion) under which ty should be interpreted.
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

    // Create symbols of trait methods from TraitEnv.
    pub fn create_trait_method_symbols(&mut self) {
        for (trait_id, trait_info) in &self.trait_env.traits {
            for method_info in &trait_info.methods {
                let method_ty = trait_info.method_scheme(&method_info.name, false);
                let syn_method_ty = trait_info.method_scheme(&method_info.name, true);
                let mut method_impls: Vec<MethodImpl> = vec![];
                let instances = self.trait_env.instances.get(trait_id);
                if let Some(insntances) = instances {
                    for trait_impl in insntances {
                        let scm = trait_impl.method_scheme(&method_info.name, trait_info);
                        let expr = trait_impl.method_expr(&method_info.name);
                        method_impls.push(MethodImpl {
                            ty: scm,
                            expr: TypedExpr::from_expr(expr),
                            define_module: trait_impl.define_module.clone(),
                        });
                    }
                }
                let method_name = FullName::new(&trait_id.name.to_namespace(), &method_info.name);
                self.global_values.insert(
                    method_name,
                    GlobalValue {
                        scm: method_ty,
                        syn_scm: Some(syn_method_ty),
                        expr: SymbolExpr::Method(method_impls),
                        def_src: method_info.source.clone(),
                        document: method_info.document.clone(),
                        compiler_defined_method: false,
                    },
                );
            }
        }
    }

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
                        errors.eat_err(impl_.ty.validate_constraints(&self.trait_env));
                    }
                }
            }
        }
        errors.to_result()
    }

    // Validate and update export statements.
    pub fn validate_export_statements(&self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Validate each export statement.
        for stmt in &self.export_statements {
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
    pub fn resolve_namespace_in_type_signs(&mut self) -> Result<(), Errors> {
        let mut ctx = NameResolutionContext::new(
            &self.tycon_names_with_aliases(),
            &self.trait_names_with_aliases(),
            self.assoc_ty_to_arity(),
            vec![],
        );
        // Resolve namespaces in type constructors.
        {
            let mut tycons = (*self.type_env.tycons).clone();
            for (tc, ti) in &mut tycons {
                ctx.import_statements = self.mod_to_import_stmts[&tc.name.module()].clone();
                ti.resolve_namespace(&ctx)?;
            }
            self.type_env.tycons = Arc::new(tycons);
        }
        // Resolve namespaces in type aliases.
        {
            let mut aliases = (*self.type_env.aliases).clone();
            for (tc, ta) in &mut aliases {
                ctx.import_statements = self.mod_to_import_stmts[&tc.name.module()].clone();
                ta.resolve_namespace(&ctx)?;
            }
            self.type_env.aliases = Arc::new(aliases);
        }

        self.trait_env
            .resolve_namespace(&mut ctx, &self.mod_to_import_stmts)?;
        for decl in &mut self.type_defns {
            ctx.import_statements = self.mod_to_import_stmts[&decl.name.module()].clone();
            decl.resolve_namespace(&ctx)?;
        }
        for (name, sym) in &mut self.global_values {
            ctx.import_statements = self.mod_to_import_stmts[&name.module()].clone();
            sym.resolve_namespace_in_declaration(&ctx)?;
        }
        Ok(())
    }

    // Resolve type aliases that appear in declarations and associated type implementations.
    pub fn resolve_type_aliases_in_declaration(&mut self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Resolve aliases in type constructors.
        let type_env = self.type_env();
        let mut tycons = (*self.type_env.tycons).clone();
        for (_, ti) in &mut tycons {
            errors.eat_err(ti.resolve_type_aliases(&type_env));
        }
        errors.to_result()?; // Throw errors if any.
        self.type_env.tycons = Arc::new(tycons);

        // Get the updated type env.
        let type_env = self.type_env();

        // Resolve aliases in type aliases.
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
            errors.eat_err(type_defn.check_tyvars());
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

    pub fn validate_trait_env(&mut self) -> Result<(), Errors> {
        self.trait_env.validate(self.kind_env())
    }

    // Validate name confliction between types, traits and global values.
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
                            FullName::new(&defn.name.to_namespace(), &format!("as_{}", field.name)),
                            union_as(&field.name, defn),
                            Some(format!(
                                "Unwraps a union value of `{}` as the variant `{}`.\nIf the value is not the variant `{}`, this function aborts the program.",
                                union_name.name, &field.name, &field.name,
                            )),
                        ));
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(&defn.name.to_namespace(), &format!("is_{}", field.name)),
                            union_is(&field.name, defn),
                            Some(format!(
                                "Checks if a union value of `{}` is the variant `{}`.",
                                union_name.name, &field.name,
                            )),
                        ));
                        errors.eat_err(self.add_compiler_defined_method(
                            FullName::new(
                                &defn.name.to_namespace(),
                                &format!("mod_{}", field.name),
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
            if let Some(other_mod) = self.modules.iter().find(|mi| mi.name == mod_info.name) {
                // If the module is already defined,
                if extend {
                    // If extending mode, this is not a problem.
                    continue;
                }
                let other_file = other_mod.source.input.file_path.clone();
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
        for (importer, importee) in &other.mod_to_import_stmts {
            if let Some(old_importee) = self.mod_to_import_stmts.get_mut(importer) {
                old_importee.extend(importee.iter().cloned());
            } else {
                self.mod_to_import_stmts
                    .insert(importer.clone(), importee.clone());
            }
        }

        // Merge types.
        self.add_type_defns(other.type_defns);

        // Merge traits and instances.
        errors.eat_err(self.trait_env.import(other.trait_env));

        // Merge global values.
        for (name, gv) in other.global_values {
            if gv.is_simple_value() {
                errors.eat_err(self.add_global_value_gv(name, gv));
            }
        }

        // Merge export statements.
        self.export_statements.append(&mut other.export_statements);

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

            return Err(Errors::from_msg_srcs(
                format!("Cannot find module `{}`.", module),
                &[&import_stmt.source],
            ));
        }
    }

    // Create a graph of modules. If module A imports module B, an edge from A to B is added.
    pub fn importing_module_graph(&self) -> (Graph<Name>, Map<Name, usize>) {
        let (mut graph, elem_to_idx) = Graph::from_set(self.linked_mods());
        for (importer, stmts) in &self.mod_to_import_stmts {
            for stmt in stmts {
                graph.connect_idx(
                    *elem_to_idx.get(importer).unwrap(),
                    *elem_to_idx.get(&stmt.module.0).unwrap(),
                );
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

    // Calculate a hash value of a module which is affected by source codes of all dependent modules.
    pub fn module_dependency_hash(&self, module: &Name) -> Result<String, Errors> {
        let mut dependent_module_names = self
            .dependent_modules(module)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        dependent_module_names.sort(); // To remove randomness introduced by HashSet, we sort it.
        let mut concatenated_source_hashes = String::default();
        for mod_name in &dependent_module_names {
            concatenated_source_hashes += &self
                .modules
                .iter()
                .find(|mi| mi.name == *mod_name)
                .unwrap()
                .source
                .input
                .hash()?;
        }
        Ok(format!("{:x}", md5::compute(concatenated_source_hashes)))
    }

    // Calculate a map from a module to a hash value of the module which is affected by source codes of all dependent modules.
    pub fn module_dependency_hash_map(&self) -> Map<Name, String> {
        // TODO: Improve time complexity.
        let mods = self.linked_mods();
        let mut mod_to_hash = Map::default();
        for module in &mods {
            mod_to_hash.insert(
                module.clone(),
                panic_if_err(self.module_dependency_hash(&module)),
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
        for (_name, gv) in &self.global_values {
            let node = gv.find_node_at(pos);
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

    // Generate a call graph of symbols.
    //
    // Call graph is a directed graph where each node is a symbol and an edge from A to B means that A calls B.
    pub fn call_graph(&self) -> Graph<FullName> {
        let syms = self.symbols.keys().cloned().collect::<Vec<_>>();
        let mut graph = Graph::new(syms);
        for (callee, sym) in &self.symbols {
            let expr = sym.expr.as_ref().unwrap();
            let called = expr.free_vars();
            for called in called {
                graph.connect(callee, &called);
            }
        }
        graph
    }

    pub fn create_typechecker(&self, config: &Configuration) -> TypeCheckContext {
        // Create typeckecker.
        let mut typechecker = TypeCheckContext::new(
            self.trait_env.clone(),
            self.type_env(),
            self.kind_env(),
            self.mod_to_import_stmts.clone(),
            config.type_check_cache.clone(),
            config.num_worker_thread,
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
    Trait(Trait),
    TypeOrTrait(FullName), // Unknown whether Type or Trait
    Module(Name),
}
