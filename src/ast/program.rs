use build_time::build_time_utc;
use serde::{Deserialize, Serialize};
use std::{io::Write, sync::Arc, vec};

use super::*;

#[derive(Clone)]
pub struct TypeEnv {
    // List of type constructors including user-defined types.
    pub tycons: Arc<HashMap<TyCon, TyConInfo>>,
    // List of type aliases.
    pub aliases: Arc<HashMap<TyCon, TyAliasInfo>>,
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
    pub fn new(tycons: HashMap<TyCon, TyConInfo>, aliases: HashMap<TyCon, TyAliasInfo>) -> TypeEnv {
        TypeEnv {
            tycons: Arc::new(tycons),
            aliases: Arc::new(aliases),
        }
    }

    pub fn kinds(&self) -> HashMap<TyCon, Arc<Kind>> {
        let mut res = HashMap::default();
        for (tc, ti) in self.tycons.as_ref().iter() {
            res.insert(tc.clone(), ti.kind.clone());
        }
        for (tc, ta) in self.aliases.as_ref().iter() {
            res.insert(tc.clone(), ta.kind.clone());
        }
        res
    }
}

#[derive(Clone)]
pub struct InstantiatedSymbol {
    pub instantiated_name: FullName,
    pub generic_name: FullName,
    pub ty: Arc<TypeNode>,
    pub expr: Option<Arc<ExprNode>>,
}

impl InstantiatedSymbol {
    // The set of modules that this symbol depends on.
    // If any of these modules, or any of their importee are changed, then they are required to be re-compiled.
    // Note that this set may not be fully spanned in the importing graph.
    pub fn dependent_modules(&self) -> HashSet<Name> {
        let mut dep_mods = HashSet::default();
        dep_mods.insert(self.instantiated_name.module());
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

pub struct GlobalValue {
    // Type of this symbol.
    // For example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String for a: Show",
    pub scm: Arc<Scheme>,
    pub expr: SymbolExpr,
    // TODO: add ty_src: Span
    // TODO: add expr_src: Span
}

impl GlobalValue {
    pub fn resolve_namespace_in_declaration(&mut self, ctx: &NameResolutionContext) {
        // If this function is called for methods, we must call resolve_namespace on MethodImpl.ty.
        assert!(matches!(self.expr, SymbolExpr::Simple(_)));
        self.scm = self.scm.resolve_namespace(ctx);
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        self.scm = self.scm.resolve_type_aliases(type_env);
        self.expr.resolve_type_aliases(type_env);
    }

    pub fn set_kinds(&mut self, kind_env: &KindEnv) {
        self.scm = self.scm.set_kinds(kind_env);
        self.scm.check_kinds(kind_env);
        match &mut self.expr {
            SymbolExpr::Simple(_) => {}
            SymbolExpr::Method(ms) => {
                for m in ms {
                    m.ty = m.ty.set_kinds(kind_env);
                    m.ty.check_kinds(kind_env);
                }
            }
        }
    }
}

// Expression of global symbol.
#[derive(Clone)]
pub enum SymbolExpr {
    Simple(TypedExpr),       // Definition such as "id : a -> a; id = \x -> x".
    Method(Vec<MethodImpl>), // Trait method implementations.
}

impl SymbolExpr {
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        match self {
            SymbolExpr::Simple(_) => {}
            SymbolExpr::Method(impls) => {
                for method_impl in impls {
                    method_impl.resolve_type_aliases(type_env);
                }
            }
        }
    }
}

// Pair of expression and type resolver for it.
#[derive(Clone, Serialize, Deserialize)]
pub struct TypedExpr {
    pub expr: Arc<ExprNode>,
    pub substitution: Substitution,
}

impl TypedExpr {
    pub fn from_expr(expr: Arc<ExprNode>) -> Self {
        TypedExpr {
            expr,
            substitution: Substitution::default(),
        }
    }

    pub fn calculate_free_vars(&mut self) {
        self.expr = calculate_free_vars(self.expr.clone());
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
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        self.ty = self.ty.resolve_type_aliases(type_env);
    }
}

pub struct NameResolutionContext {
    pub candidates: HashMap<FullName, NameResolutionType>,
    pub assoc_ty_to_arity: HashMap<FullName, usize>,
    pub import_statements: Vec<ImportStatement>,
}

impl<'a> NameResolutionContext {
    pub fn new(
        tycon_names_with_aliases: &HashSet<FullName>,
        trait_names_with_aliases: &HashSet<FullName>,
        assoc_ty_to_arity: HashMap<FullName, usize>,
        import_statements: Vec<ImportStatement>,
    ) -> Self {
        let mut candidates: HashMap<FullName, NameResolutionType> = HashMap::new();
        fn insert_or_err(
            candidates: &mut HashMap<FullName, NameResolutionType>,
            name: FullName,
            nrt: NameResolutionType,
        ) {
            if candidates.contains_key(&name) && candidates[&name] != nrt {
                // If there is confliction between type names, trait names and associated type names, raise an error.
                // This restriction is necessary for namespace resolution and avoiding ambiguous import statement.
                error_exit(&format!(
                    "There are two entities named as `{}`: one is a {} and one is a {}.",
                    name.to_string(),
                    candidates[&name].to_string(),
                    nrt.to_string()
                ))
            }
            candidates.insert(name, nrt);
        }
        for name in tycon_names_with_aliases {
            insert_or_err(&mut candidates, name.clone(), NameResolutionType::TyCon);
        }
        for name in trait_names_with_aliases {
            insert_or_err(&mut candidates, name.clone(), NameResolutionType::Trait);
        }
        for (name, _arity) in &assoc_ty_to_arity {
            insert_or_err(&mut candidates, name.clone(), NameResolutionType::AssocTy);
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
    ) -> Result<FullName, String> {
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
            Err(format!(
                "Unknown {} name `{}`.",
                accept_type_string,
                short_name.to_string()
            ))
        } else if candidates.len() == 1 {
            Ok(candidates[0].clone())
        } else {
            // candidates.len() >= 2
            let mut candidates = candidates
                .iter()
                .map(|id| "`".to_string() + &id.to_string() + "`")
                .collect::<Vec<_>>();
            candidates.sort(); // Sort for deterministic error message.
            let candidates = candidates.join(", ");
            let msg = format!(
                "Name `{}` is ambiguous. There are {}.",
                short_name.to_string(),
                candidates
            );
            Err(msg)
        }
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

// Program of fix a collection of modules.
// A program can link another program which consists of a single module.
pub struct Program {
    /* AST */
    // Global values.
    pub global_values: HashMap<FullName, GlobalValue>,
    // Type definitions.
    pub type_defns: Vec<TypeDefn>,
    // Type environment, which is calculated from `type_defns` once and cached.
    pub type_env: TypeEnv,
    // Trait environment.
    pub trait_env: TraitEnv,
    // List of tuple sizes used in this program.
    pub used_tuple_sizes: Vec<u32>,
    // Import statements.
    // Key is the name of the importer module.
    // Each module implicitly imports itself.
    // This is used to namespace resolution and overloading resolution.
    pub mod_to_import_stmts: HashMap<Name, Vec<ImportStatement>>,

    /* Instantiation */
    // Instantiated symbols.
    pub instantiated_symbols: HashMap<FullName, InstantiatedSymbol>,
    // Deferred instantiation, which is a state variable for the instantiation process.
    pub deferred_instantiation: Vec<InstantiatedSymbol>,

    /* Dependency information */
    // A map from module name to source file.
    pub module_to_files: HashMap<Name, SourceFile>,
}

impl Program {
    // Create a program consists of single module.
    pub fn single_module(module_name: Name, src: &SourceFile) -> Program {
        let mut fix_mod = Program {
            mod_to_import_stmts: Default::default(),
            type_defns: Default::default(),
            global_values: Default::default(),
            instantiated_symbols: Default::default(),
            deferred_instantiation: Default::default(),
            trait_env: Default::default(),
            type_env: Default::default(),
            used_tuple_sizes: (0..=TUPLE_SIZE_BASE).filter(|i| *i != 1).collect(),
            module_to_files: Default::default(),
        };
        fix_mod.add_import_statement_no_verify(ImportStatement::implicit_self_import(
            module_name.clone(),
        ));
        fix_mod.add_import_statement_no_verify(ImportStatement::implicit_std_import(
            module_name.clone(),
        ));
        fix_mod.module_to_files.insert(module_name, src.clone());
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
    pub fn add_import_statements(&mut self, imports: Vec<ImportStatement>) {
        for stmt in imports {
            self.add_import_statement(stmt);
        }
    }

    pub fn add_import_statement(&mut self, import_statement: ImportStatement) {
        // Refuse importing the module itself.
        if import_statement.module == import_statement.importer {
            error_exit_with_src(
                &format!(
                    "Module `{}` cannot import itself.",
                    import_statement.module.to_string()
                ),
                &import_statement.source,
            );
        }

        // When user imports `Std` explicitly, remove implicit `Std` import statement.
        if import_statement.module == STD_NAME {
            let stmts = self
                .mod_to_import_stmts
                .get_mut(&import_statement.importer)
                .unwrap();
            *stmts = std::mem::replace(stmts, vec![])
                .into_iter()
                .filter(|stmt| !(stmt.module == STD_NAME && stmt.implicit))
                .collect();
        }

        self.add_import_statement_no_verify(import_statement);
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
    ) {
        self.trait_env.add(trait_infos, trait_impls, trait_aliases);
    }

    // Register declarations of user-defined types.
    pub fn add_type_defns(&mut self, mut type_defns: Vec<TypeDefn>) {
        self.type_defns.append(&mut type_defns);
    }

    // Calculate list of type constructors including user-defined types.
    pub fn calculate_type_env(&mut self) {
        let mut tycons = bulitin_tycons();
        let mut aliases: HashMap<TyCon, TyAliasInfo> = HashMap::new();
        for type_decl in &mut self.type_defns {
            type_decl.set_kinds_in_value();
            let tycon = type_decl.tycon();
            if tycons.contains_key(&tycon) || aliases.contains_key(&tycon) {
                let other_src = if tycons.contains_key(&tycon) {
                    let tc = tycons.get(&tycon).unwrap();
                    tc.source.clone()
                } else {
                    let ta = aliases.get(&tycon).unwrap();
                    ta.source.clone()
                };
                error_exit_with_srcs(
                    &format!("Duplicate definition of type `{}`.", tycon.to_string()),
                    &[
                        &type_decl.source.as_ref().map(|s| s.to_single_character()),
                        &other_src.as_ref().map(|s| s.to_single_character()),
                    ],
                );
            }
            if type_decl.is_alias() {
                aliases.insert(tycon, type_decl.alias_info());
            } else {
                tycons.insert(tycon, type_decl.tycon_info());
            }
        }
        self.type_env = TypeEnv::new(tycons, aliases);
    }

    // Get list of type constructors including user-defined types.
    pub fn type_env(&self) -> TypeEnv {
        self.type_env.clone()
    }

    // Get of list of tycons that can be used for namespace resolution.
    pub fn tycon_names_with_aliases(&self) -> HashSet<FullName> {
        let mut res: HashSet<FullName> = Default::default();
        for (k, _) in self.type_env().tycons.iter() {
            res.insert(k.name.clone());
        }
        for (k, _) in self.type_env().aliases.iter() {
            res.insert(k.name.clone());
        }
        res
    }

    // pub fn assoc_ty_names(&self) -> HashSet<FullName> {
    //     self.trait_env.assoc_ty_names()
    // }

    pub fn assoc_ty_to_arity(&self) -> HashMap<FullName, usize> {
        self.trait_env.assoc_ty_to_arity()
    }

    // Get of list of traits that can be used for namespace resolution.
    pub fn trait_names_with_aliases(&self) -> HashSet<FullName> {
        self.trait_env.trait_names()
    }

    // Add a global value.
    pub fn add_global_value(&mut self, name: FullName, (expr, scm): (Arc<ExprNode>, Arc<Scheme>)) {
        if self.global_values.contains_key(&name) {
            error_exit_with_src(
                &format!(
                    "Duplicated definition for global value: `{}`",
                    name.to_string()
                ),
                &Span::unite_opt(scm.ty.get_source(), &expr.source),
            );
        }
        self.global_values.insert(
            name,
            GlobalValue {
                scm,
                expr: SymbolExpr::Simple(TypedExpr::from_expr(expr)),
            },
        );
    }

    // Add global values.
    pub fn add_global_values(&mut self, exprs: Vec<GlobalValueDefn>, types: Vec<GlobalValueDecl>) {
        struct GlobalValue {
            defn: Option<GlobalValueDefn>,
            decl: Option<GlobalValueDecl>,
        }

        let mut global_values: HashMap<FullName, GlobalValue> = Default::default();
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
                    error_exit_with_srcs(
                        &format!(
                            "Duplicate definition for global value: `{}`.",
                            defn.name.to_string()
                        ),
                        &[
                            &defn.src.map(|s| s.to_single_character()),
                            &gv.defn
                                .as_ref()
                                .unwrap()
                                .src
                                .as_ref()
                                .map(|s| s.to_single_character()),
                        ],
                    );
                } else {
                    gv.defn = Some(defn);
                }
            }
        }
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
                    error_exit_with_srcs(
                        &format!("Duplicate declaration for `{}`.", decl.name.to_string()),
                        &[
                            &decl.src.map(|s| s.to_single_character()),
                            &gv.decl
                                .as_ref()
                                .unwrap()
                                .src
                                .as_ref()
                                .map(|s| s.to_single_character()),
                        ],
                    );
                } else {
                    gv.decl = Some(decl);
                }
            }
        }

        for (name, gv) in global_values {
            if gv.defn.is_none() {
                error_exit_with_src(
                    &format!("Global value `{}` lacks declaration.", name.to_string()),
                    &gv.decl
                        .unwrap()
                        .src
                        .as_ref()
                        .map(|s| s.to_single_character()),
                )
            }
            if gv.decl.is_none() {
                error_exit_with_src(
                    &format!("Global value `{}` lacks definition.", name.to_string()),
                    &gv.defn
                        .unwrap()
                        .src
                        .as_ref()
                        .map(|s| s.to_single_character()),
                )
            }
            self.add_global_value(name, (gv.defn.unwrap().expr, gv.decl.unwrap().ty))
        }
    }

    // - Resolve namespace of type and trats in expression,
    // - resolve type aliases, and
    // - perform typechecking.
    // The result will be written to `te`.
    fn resolve_and_check_type(
        &self,
        te: &mut TypedExpr,
        required_scheme: &Arc<Scheme>,
        name: &FullName,
        define_module: &Name,
        tc: &TypeCheckContext,
    ) {
        fn cache_file_name(
            name: &FullName,
            hash_of_dependent_codes: &str,
            scheme: &Arc<Scheme>,
        ) -> String {
            let data = format!(
                "{}_{}_{}_{}",
                name.to_string(),
                hash_of_dependent_codes,
                scheme.to_string(),
                build_time_utc!()
            );
            format!("{:x}", md5::compute(data))
        }
        fn load_cache(
            name: &FullName,
            hash_of_dependent_codes: &str,
            required_scheme: &Arc<Scheme>,
        ) -> Option<TypedExpr> {
            let cache_file_name = cache_file_name(name, hash_of_dependent_codes, required_scheme);
            let cache_dir = touch_directory(TYPE_CHECK_CACHE_PATH);
            let cache_file = cache_dir.join(cache_file_name);
            let cache_file_display = cache_file.display();
            if !cache_file.exists() {
                return None;
            }
            let mut cache_file = match File::open(&cache_file) {
                Err(_) => {
                    return None;
                }
                Ok(file) => file,
            };
            let mut cache_bytes = vec![];
            match cache_file.read_to_end(&mut cache_bytes) {
                Ok(_) => {}
                Err(why) => {
                    eprintln!(
                        "warning: Failed to read cache file {}: {}.",
                        cache_file_display, why
                    );
                    return None;
                }
            }
            let expr: TypedExpr = match serde_pickle::from_slice(&cache_bytes, Default::default()) {
                Ok(res) => res,
                Err(why) => {
                    eprintln!(
                        "warning: Failed to parse content of cache file {}: {}.",
                        cache_file_display, why
                    );
                    return None;
                }
            };
            Some(expr)
        }

        fn save_cache(
            te: &TypedExpr,
            required_scheme: &Arc<Scheme>,
            name: &FullName,
            hash_of_dependent_codes: &str,
        ) {
            let cache_file_name = cache_file_name(name, hash_of_dependent_codes, required_scheme);
            let cache_dir = touch_directory(TYPE_CHECK_CACHE_PATH);
            let cache_file = cache_dir.join(cache_file_name);
            let cache_file_display = cache_file.display();
            let mut cache_file = match File::create(&cache_file) {
                Err(_) => {
                    eprintln!(
                        "warning: Failed to create cache file {}.",
                        cache_file_display
                    );
                    return;
                }
                Ok(file) => file,
            };
            let serialized = serde_pickle::to_vec(&te, Default::default()).unwrap();
            match cache_file.write_all(&serialized) {
                Ok(_) => {}
                Err(_) => {
                    eprintln!(
                        "warning: Failed to write cache file {}.",
                        cache_file_display
                    );
                }
            }
        }

        // Load type-checking cache file.
        let hash_of_dependent_codes = self.module_dependency_hash(define_module);
        let cache = load_cache(name, &hash_of_dependent_codes, required_scheme);
        if cache.is_some() {
            // If cache is available,
            *te = cache.unwrap();
            return;
        }

        // Perform namespace inference.
        let nrctx = NameResolutionContext::new(
            &self.tycon_names_with_aliases(),
            &self.trait_names_with_aliases(),
            self.assoc_ty_to_arity(),
            self.mod_to_import_stmts[define_module].clone(),
        );
        te.expr = te.expr.resolve_namespace(&nrctx);

        // Resolve type aliases in expression.
        te.expr = te.expr.resolve_type_aliases(&tc.type_env);

        // Perform type-checking.
        let mut tc = tc.clone();
        tc.current_module = Some(define_module.clone());
        te.expr = tc.check_type(te.expr.clone(), required_scheme.clone());
        te.substitution = tc.substitution;

        // Save the result to cache file.
        save_cache(te, required_scheme, name, &hash_of_dependent_codes);
    }

    // Instantiate symbol.
    fn instantiate_symbol(&mut self, sym: &mut InstantiatedSymbol, tc: &TypeCheckContext) {
        assert!(sym.expr.is_none());
        if !sym.ty.free_vars().is_empty() {
            error_exit_with_src(&format!("Cannot instantiate global value `{}` of type `{}` since the type contains undetermined type variable. Maybe you need to add type annotation.", sym.generic_name.to_string(), sym.ty.to_string_normalize()), &sym.expr.as_ref().unwrap().source);
        }
        let global_sym = self.global_values.get(&sym.generic_name).unwrap();
        let expr = match &global_sym.expr {
            SymbolExpr::Simple(e) => {
                // Perform type-checking.
                let define_module = sym.generic_name.module();
                let mut e = e.clone();
                self.resolve_and_check_type(
                    &mut e,
                    &global_sym.scm,
                    &sym.generic_name,
                    &define_module,
                    tc,
                );
                // Calculate free vars.
                e.calculate_free_vars();
                // Specialize e's type to the required type `sym.ty`.
                let mut tc = tc.clone();
                assert!(tc.substitution.is_empty());
                tc.substitution = std::mem::replace(&mut e.substitution, Substitution::default());
                tc.unify(e.expr.ty.as_ref().unwrap(), &sym.ty).ok().unwrap();
                tc.finish_inferred_types(e.expr)
            }
            SymbolExpr::Method(impls) => {
                let mut opt_e: Option<Arc<ExprNode>> = None;
                for method in impls {
                    // Check if the type of this implementation unify with the required type `sym.ty`.
                    // NOTE: Since overlapping implementations and unrelated methods are forbidden,
                    // we only need to check the unifiability here,
                    // and we do not need to check whether predicates or equality constraints are satisfiable or not.
                    {
                        let mut tc0 = tc.clone();
                        if tc0.unify(&method.ty.ty, &sym.ty).is_err() {
                            continue;
                        }
                    }
                    // Perform type-checking.
                    let define_module = method.define_module.clone();
                    let mut e = method.expr.clone();
                    self.resolve_and_check_type(
                        &mut e,
                        &method.ty,
                        &sym.generic_name,
                        &define_module,
                        tc,
                    );
                    // Calculate free vars.
                    e.calculate_free_vars();
                    // Specialize e's type to the required type `sym.ty`.
                    let mut tc = tc.clone();
                    assert!(tc.substitution.is_empty());
                    tc.substitution =
                        std::mem::replace(&mut e.substitution, Substitution::default());
                    tc.unify(e.expr.ty.as_ref().unwrap(), &sym.ty).ok().unwrap();
                    opt_e = Some(tc.finish_inferred_types(e.expr));
                    break;
                }
                opt_e.unwrap()
            }
        };
        sym.expr = Some(self.instantiate_expr(&expr));
    }

    // Instantiate all symbols.
    pub fn instantiate_symbols(&mut self, tc: &TypeCheckContext) {
        while !self.deferred_instantiation.is_empty() {
            let sym = self.deferred_instantiation.pop().unwrap();
            let name = sym.instantiated_name.clone();
            let mut sym = sym.clone();
            self.instantiate_symbol(&mut sym, tc);
            self.instantiated_symbols.insert(name, sym);
        }
    }

    // Instantiate main function.
    pub fn instantiate_main_function(&mut self, tc: &TypeCheckContext) -> Arc<ExprNode> {
        let main_func_name = FullName::from_strs(&[MAIN_MODULE_NAME], MAIN_FUNCTION_NAME);
        if !self.global_values.contains_key(&main_func_name) {
            error_exit(&format!("{} not found.", main_func_name.to_string()));
        }
        let main_ty = make_io_unit_ty();
        let inst_name = self.require_instantiated_symbol(&main_func_name, &main_ty);
        self.instantiate_symbols(tc);
        expr_var(inst_name, None).set_inferred_type(main_ty)
    }

    // Instantiate expression.
    fn instantiate_expr(&mut self, expr: &Arc<ExprNode>) -> Arc<ExprNode> {
        let ret = match &*expr.expr {
            Expr::Var(v) => {
                if v.name.is_local() {
                    expr.clone()
                } else {
                    let instance =
                        self.require_instantiated_symbol(&v.name, &expr.ty.as_ref().unwrap());
                    let v = v.set_name(instance);
                    expr.set_var_var(v)
                }
            }
            Expr::LLVM(_) => expr.clone(),
            Expr::App(fun, args) => {
                let fun = self.instantiate_expr(fun);
                let args = args
                    .iter()
                    .map(|arg| self.instantiate_expr(arg))
                    .collect::<Vec<_>>();
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_, body) => expr.set_lam_body(self.instantiate_expr(body)),
            Expr::Let(_, bound, val) => {
                let bound = self.instantiate_expr(bound);
                let val = self.instantiate_expr(val);
                expr.set_let_bound(bound).set_let_value(val)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.instantiate_expr(cond);
                let then_expr = self.instantiate_expr(then_expr);
                let else_expr = self.instantiate_expr(else_expr);
                expr.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::TyAnno(e, _) => {
                let e = self.instantiate_expr(e);
                expr.set_tyanno_expr(e)
            }
            Expr::MakeStruct(_, fields) => {
                let mut expr = expr.clone();
                for (field_name, field_expr) in fields {
                    let field_expr = self.instantiate_expr(field_expr);
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                expr
            }
            Expr::ArrayLit(elems) => {
                let mut expr = expr.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.instantiate_expr(e);
                    expr = expr.set_array_lit_elem(e, i);
                }
                expr
            }
            Expr::CallC(_, _, _, _, args) => {
                let mut expr = expr.clone();
                for (i, e) in args.iter().enumerate() {
                    let e = self.instantiate_expr(e);
                    expr = expr.set_call_c_arg(e, i);
                }
                expr
            }
        };
        // If the type of an expression contains undetermied type variable after instantiation, raise an error.
        if !ret.ty.as_ref().unwrap().free_vars().is_empty() {
            error_exit_with_src(
                "The type of an expression cannot be determined. You need to add type annotation to help type inference.",
                &expr.source,
            );
        }
        calculate_free_vars(ret)
    }

    // Require instantiate generic symbol such that it has a specified type.
    pub fn require_instantiated_symbol(&mut self, name: &FullName, ty: &Arc<TypeNode>) -> FullName {
        let inst_name = self.determine_instantiated_symbol_name(name, ty);
        if !self.instantiated_symbols.contains_key(&inst_name)
            && self
                .deferred_instantiation
                .iter()
                .all(|symbol| symbol.instantiated_name != inst_name)
        {
            self.deferred_instantiation.push(InstantiatedSymbol {
                instantiated_name: inst_name.clone(),
                generic_name: name.clone(),
                ty: ty.clone(),
                expr: None,
            });
        }
        inst_name
    }

    // Determine the name of instantiated generic symbol so that it has a specified type.
    // tc: a typechecker (substituion) under which ty should be interpreted.
    fn determine_instantiated_symbol_name(&self, name: &FullName, ty: &Arc<TypeNode>) -> FullName {
        let ty = ty.resolve_type_aliases(&self.type_env());
        let hash = ty.hash();
        let mut name = name.clone();
        name.name += INSTANCIATED_NAME_SEPARATOR;
        name.name += &hash;
        name
    }

    // Create symbols of trait methods from TraitEnv.
    pub fn create_trait_method_symbols(&mut self) {
        for (trait_id, trait_info) in &self.trait_env.traits {
            for (method_name, _) in &trait_info.methods {
                let method_ty = trait_info.method_scheme(method_name);
                let mut method_impls: Vec<MethodImpl> = vec![];
                let instances = self.trait_env.instances.get(trait_id);
                if let Some(insntances) = instances {
                    for trait_impl in insntances {
                        let scm = trait_impl.method_scheme(method_name, trait_info);
                        let expr = trait_impl.method_expr(method_name);
                        method_impls.push(MethodImpl {
                            ty: scm,
                            expr: TypedExpr::from_expr(expr),
                            define_module: trait_impl.define_module.clone(),
                        });
                    }
                }
                let method_name = FullName::new(&trait_id.name.to_namespace(), &method_name);
                self.global_values.insert(
                    method_name,
                    GlobalValue {
                        scm: method_ty,
                        expr: SymbolExpr::Method(method_impls),
                    },
                );
            }
        }
    }

    pub fn validate_global_value_types(&self) {
        for (_name, gv) in &self.global_values {
            gv.scm.validate_constraints(&self.trait_env);
            match gv.expr {
                SymbolExpr::Simple(ref _e) => {}
                SymbolExpr::Method(ref impls) => {
                    for impl_ in impls {
                        impl_.ty.validate_constraints(&self.trait_env);
                    }
                }
            }
        }
    }

    pub fn set_kinds(&mut self) {
        self.trait_env.set_kinds_in_trait_and_alias_defns();
        let kind_env = self.kind_env();
        self.trait_env.set_kinds_in_trait_instances(&kind_env);
        for (_name, sym) in &mut self.global_values {
            sym.set_kinds(&kind_env);
        }
    }

    pub fn kind_env(&self) -> KindEnv {
        KindEnv {
            tycons: self.type_env().kinds(),
            assoc_tys: self.trait_env.assoc_ty_kind_info(),
            traits_and_aliases: self.trait_env.trait_kind_map_with_aliases(),
        }
    }

    // Infer namespaces of traits and types that appear in declarations and associated type implementations.
    // NOTE: names in the definition of types/traits/global_values have to be full-named already when this function called.
    pub fn resolve_namespace_capital_names_not_in_expression(&mut self) {
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
                ti.resolve_namespace(&ctx);
            }
            self.type_env.tycons = Arc::new(tycons);
        }
        // Resolve namespaces in type aliases.
        {
            let mut aliases = (*self.type_env.aliases).clone();
            for (tc, ta) in &mut aliases {
                ctx.import_statements = self.mod_to_import_stmts[&tc.name.module()].clone();
                ta.resolve_namespace(&ctx);
            }
            self.type_env.aliases = Arc::new(aliases);
        }

        self.trait_env
            .resolve_namespace(&mut ctx, &self.mod_to_import_stmts);
        for decl in &mut self.type_defns {
            ctx.import_statements = self.mod_to_import_stmts[&decl.name.module()].clone();
            decl.resolve_namespace(&ctx);
        }
        for (name, sym) in &mut self.global_values {
            ctx.import_statements = self.mod_to_import_stmts[&name.module()].clone();
            sym.resolve_namespace_in_declaration(&ctx);
        }
    }

    // Resolve type aliases that appear in declarations and associated type implementations.
    pub fn resolve_type_aliases_in_declaration(&mut self) {
        // Resolve in type constructors.
        {
            let type_env = self.type_env();
            let mut tycons = (*self.type_env.tycons).clone();
            for (_, ti) in &mut tycons {
                ti.resolve_type_aliases(&type_env);
            }
            self.type_env.tycons = Arc::new(tycons);
        }
        let type_env = self.type_env();
        self.trait_env.resolve_type_aliases(&type_env);
        for decl in &mut self.type_defns {
            decl.resolve_type_aliases(&type_env);
        }
        for (_, sym) in &mut self.global_values {
            sym.resolve_type_aliases(&type_env);
        }
    }

    // Validate user-defined types
    pub fn validate_type_defns(&self) {
        for type_defn in &self.type_defns {
            type_defn.check_tyvars();
            let type_name = &type_defn.name;
            match &type_defn.value {
                TypeDeclValue::Struct(str) => {
                    for field in &str.fields {
                        if !field.ty.is_assoc_ty_free() {
                            error_exit_with_src(
                                "Associated type is not allowed in the field type of a struct.",
                                &type_defn.source.as_ref().map(|s| s.to_single_character()),
                            );
                        }
                    }
                    match Field::check_duplication(&str.fields) {
                        Some(field_name) => {
                            error_exit_with_src(
                                &format!(
                                    "Duplicate field `{}` in the definition of struct `{}`.",
                                    field_name,
                                    type_name.to_string()
                                ),
                                &type_defn.source.as_ref().map(|s| s.to_single_character()),
                            );
                        }
                        _ => {}
                    }
                }
                TypeDeclValue::Union(union) => {
                    for field in &union.fields {
                        if !field.ty.is_assoc_ty_free() {
                            error_exit_with_src(
                                "Associated type is not allowed in the field type of a union.",
                                &type_defn.source.as_ref().map(|s| s.to_single_character()),
                            );
                        }
                    }
                    match Field::check_duplication(&union.fields) {
                        Some(field_name) => {
                            error_exit_with_src(
                                &format!(
                                    "Duplicate field `{}` in the definition of union `{}`",
                                    field_name,
                                    type_name.to_string()
                                ),
                                &type_defn.source.as_ref().map(|s| s.to_single_character()),
                            );
                        }
                        _ => {}
                    }
                }
                TypeDeclValue::Alias(ta) => {
                    if !ta.value.is_assoc_ty_free() {
                        error_exit_with_src(
                            "Associated type is not allowed in the right-hand side of a type alias.",
                            &type_defn.source.as_ref().map(|s| s.to_single_character()),
                        );
                    }
                } // Nothing to do.
            }
        }
    }

    pub fn validate_trait_env(&mut self) {
        self.trait_env.validate(self.kind_env());
    }

    pub fn add_methods(self: &mut Program) {
        for decl in &self.type_defns.clone() {
            match &decl.value {
                TypeDeclValue::Struct(str) => {
                    let struct_name = decl.name.clone();
                    for field in &str.fields {
                        self.add_global_value(
                            FullName::new(
                                &decl.name.to_namespace(),
                                &format!("{}{}", STRUCT_GETTER_SYMBOL, &field.name),
                            ),
                            struct_get(&struct_name, decl, &field.name),
                        );
                        for is_unique in [false, true] {
                            self.add_global_value(
                                FullName::new(
                                    &decl.name.to_namespace(),
                                    &format!(
                                        "mod_{}{}",
                                        &field.name,
                                        if is_unique { "!" } else { "" }
                                    ),
                                ),
                                struct_mod(&struct_name, decl, &field.name, is_unique),
                            );
                            self.add_global_value(
                                FullName::new(
                                    &decl.name.to_namespace(),
                                    &format!(
                                        "{}{}{}",
                                        STRUCT_SETTER_SYMBOL,
                                        &field.name,
                                        if is_unique { "!" } else { "" }
                                    ),
                                ),
                                struct_set(&struct_name, decl, &field.name, is_unique),
                            )
                        }
                    }
                }
                TypeDeclValue::Union(union) => {
                    let union_name = &decl.name;
                    for field in &union.fields {
                        self.add_global_value(
                            FullName::new(&decl.name.to_namespace(), &field.name),
                            union_new(&union_name, &field.name, decl),
                        );
                        self.add_global_value(
                            FullName::new(&decl.name.to_namespace(), &format!("as_{}", field.name)),
                            union_as(&union_name, &field.name, decl),
                        );
                        self.add_global_value(
                            FullName::new(&decl.name.to_namespace(), &format!("is_{}", field.name)),
                            union_is(&union_name, &field.name, decl),
                        );
                        self.add_global_value(
                            FullName::new(
                                &decl.name.to_namespace(),
                                &format!("mod_{}", field.name),
                            ),
                            union_mod_function(&union_name, &field.name, decl),
                        );
                    }
                }
                TypeDeclValue::Alias(_) => {} // Nothing to do
            }
        }
    }

    pub fn linked_mods(&self) -> HashSet<Name> {
        self.mod_to_import_stmts.keys().cloned().collect()
    }

    // Link an module.
    // * extend - If true, the module defined in `other` allowed to conflict with a module already in `self`.
    //            This is used for extending implementation of a module already linked to `self` afterwards.
    pub fn link(&mut self, mut other: Program, extend: bool) {
        // Merge module file paths.
        for (mod_name, file) in &other.module_to_files {
            let file = file.clone();
            if self.module_to_files.contains_key(mod_name) {
                let another = self.module_to_files.get(mod_name).unwrap();
                if extend {
                    break;
                }
                error_exit(&format!(
                    "Module `{}` is defined in two files: \"{}\" and \"{}\".",
                    mod_name,
                    another.file_path.to_str().unwrap(),
                    file.file_path.to_str().unwrap()
                ));
            }
            self.module_to_files.insert(mod_name.clone(), file);
        }

        // If already linked, do nothing.
        if !extend
            && self
                .linked_mods()
                .contains(&other.get_name_if_single_module())
        {
            return;
        }

        // Merge visible_mods.
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
        self.trait_env.import(other.trait_env);

        // Merge global values.
        for (name, gv) in other.global_values {
            let ty = gv.scm;
            if let SymbolExpr::Simple(expr) = gv.expr {
                self.add_global_value(name, (expr.expr, ty));
            }
        }

        // Merge used_tuple_sizes.
        self.used_tuple_sizes.append(&mut other.used_tuple_sizes);
    }

    // Link built-in modules following unsolved import statements.
    // This function may mutate config to add dynamically linked libraries.
    pub fn resolve_imports(&mut self, config: &mut Configuration) {
        let mut unresolved_imports = self.import_statements();

        loop {
            if unresolved_imports.is_empty() {
                break;
            }
            let import_stmt = unresolved_imports.pop().unwrap();
            let module = import_stmt.module;

            // If import is already resolved, do nothing.
            if self.is_linked(&module) {
                continue;
            }

            let mut imported = false;
            // Search for bulit-in modules.
            for (mod_name, source_content, file_name, config_modifier, mod_modifier) in
                STANDARD_LIBRARIES
            {
                if module == *mod_name {
                    let mut fixmod = parse_and_save_to_temporary_file(source_content, file_name);
                    if let Some(mod_modifier) = mod_modifier {
                        mod_modifier(&mut fixmod);
                    }
                    unresolved_imports.append(&mut fixmod.import_statements());
                    self.link(fixmod, false);
                    if let Some(config_modifier) = config_modifier {
                        config_modifier(config);
                    }
                    imported = true;
                    break;
                }
            }
            if imported {
                continue;
            }

            error_exit_with_src(
                &format!("Cannot find module `{}`", module),
                &import_stmt.source,
            );
        }
    }

    // Create a graph of modules. If module A imports module B, an edge from A to B is added.
    pub fn importing_module_graph(&self) -> (Graph<Name>, HashMap<Name, usize>) {
        let (mut graph, elem_to_idx) = Graph::from_set(self.linked_mods());
        for (importer, stmts) in &self.mod_to_import_stmts {
            for stmt in stmts {
                graph.connect(
                    *elem_to_idx.get(importer).unwrap(),
                    *elem_to_idx.get(&stmt.module).unwrap(),
                );
            }
        }
        (graph, elem_to_idx)
    }

    // Calculate a set of modules on which a module depends.
    pub fn dependent_modules(&self, module: &Name) -> HashSet<Name> {
        let (importing_graph, mod_to_node) = self.importing_module_graph();
        importing_graph
            .reachable_nodes(*mod_to_node.get(module).unwrap())
            .iter()
            .map(|idx| importing_graph.get(*idx).clone())
            .collect()
    }

    // Calculate a map from a module to a set of modules on which the module depends.
    pub fn module_dependency_map(&self) -> HashMap<Name, HashSet<Name>> {
        // TODO: Improve time complexity.
        let mods = self.linked_mods();
        let mut dependency = HashMap::new();
        for module in &mods {
            dependency.insert(module.clone(), self.dependent_modules(&module));
        }
        dependency
    }

    // Calculate a hash value of a module which is affected by source codes of all dependent modules.
    pub fn module_dependency_hash(&self, module: &Name) -> String {
        let mut dependent_module_names = self
            .dependent_modules(module)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        dependent_module_names.sort(); // To remove randomness introduced by HashSet, we sort it.
        let concatenated_source_hashes = dependent_module_names
            .iter()
            .map(|mod_name| self.module_to_files.get(mod_name).unwrap().hash())
            .collect::<Vec<_>>()
            .join("");
        format!("{:x}", md5::compute(concatenated_source_hashes))
    }

    // Calculate a map from a module to a hash value of the module which is affected by source codes of all dependent modules.
    pub fn module_dependency_hash_map(&self) -> HashMap<Name, String> {
        // TODO: Improve time complexity.
        let mods = self.linked_mods();
        let mut mod_to_hash = HashMap::new();
        for module in &mods {
            mod_to_hash.insert(module.clone(), self.module_dependency_hash(&module));
        }
        mod_to_hash
    }

    // Check if all items referred in import statements are defined.
    pub fn validate_import_statements(&self) {
        let stmts = self.import_statements();
        let items = stmts.iter().map(|stmt| stmt.referred_items()).flatten();

        let values = self.global_values.keys().collect::<HashSet<_>>();
        let types = self.tycon_names_with_aliases();
        let traits = self.trait_names_with_aliases();

        for item in items {
            match item {
                ImportItem::Symbol(name, src) => {
                    if values.contains(&name) {
                        continue;
                    }
                    error_exit_with_src(
                        &format!("Cannot find value named `{}`.", name.to_string()),
                        &src,
                    );
                }
                ImportItem::TypeOrTrait(name, src) => {
                    if types.contains(&name) || traits.contains(&name) {
                        continue;
                    }
                    error_exit_with_src(
                        &format!("Cannot find entity named `{}`.", name.to_string()),
                        &src,
                    );
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
                    error_exit_with_src(
                        &format!(
                            "Namespace `{}` is not defined or empty.",
                            namespace.to_string()
                        ),
                        &src,
                    );
                }
            }
        }
    }
}
