use build_time::build_time_utc;
use inkwell::{debug_info::AsDIScope, module::Linkage};
use serde::{Deserialize, Serialize};
use std::io::Write;

use self::stopwatch::StopWatch;

use super::*;

#[derive(Clone)]
pub struct TypeEnv {
    // List of type constructors including user-defined types.
    pub tycons: Rc<HashMap<TyCon, TyConInfo>>,
    // List of type aliases.
    pub aliases: Rc<HashMap<TyCon, TyAliasInfo>>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self {
            tycons: Rc::new(Default::default()),
            aliases: Rc::new(Default::default()),
        }
    }
}

impl TypeEnv {
    pub fn new(tycons: HashMap<TyCon, TyConInfo>, aliases: HashMap<TyCon, TyAliasInfo>) -> TypeEnv {
        TypeEnv {
            tycons: Rc::new(tycons),
            aliases: Rc::new(aliases),
        }
    }

    pub fn kinds(&self) -> HashMap<TyCon, Rc<Kind>> {
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
    pub template_name: FullName,
    pub ty: Rc<TypeNode>,
    pub expr: Option<Rc<ExprNode>>,
    pub type_resolver: TypeResolver, // type resolver for types in expr.
}

// Declaration (name and type signature) of global value.
// `main : IO()`
pub struct GlobalValueDecl {
    pub name: FullName,
    pub ty: Rc<Scheme>,
    pub src: Option<Span>,
}

// Definition (name and expression)
// `main = println("Hello World")`
pub struct GlobalValueDefn {
    pub name: FullName,
    pub expr: Rc<ExprNode>,
    pub src: Option<Span>,
}

pub struct GlobalValue {
    // Type of this symbol.
    // For example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String for a: Show",
    pub scm: Rc<Scheme>,
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

    pub fn set_kinds(
        &mut self,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
        trait_kind_map: &HashMap<TraitId, Rc<Kind>>,
    ) {
        self.scm = self.scm.set_kinds(trait_kind_map);
        self.scm.check_kinds(kind_map, trait_kind_map);
        match &mut self.expr {
            SymbolExpr::Simple(_) => {}
            SymbolExpr::Method(ms) => {
                for m in ms {
                    m.ty = m.ty.set_kinds(trait_kind_map);
                    m.ty.check_kinds(kind_map, trait_kind_map);
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
    pub expr: Rc<ExprNode>,
    pub type_resolver: TypeResolver,
}

impl TypedExpr {
    pub fn from_expr(expr: Rc<ExprNode>) -> Self {
        TypedExpr {
            expr,
            type_resolver: TypeResolver::default(),
        }
    }

    pub fn calculate_free_vars(&mut self) {
        self.expr = calculate_free_vars(self.expr.clone());
    }

    // When unification fails, it has no side effect to self.
    pub fn unify_to(&mut self, target_ty: &Rc<TypeNode>) -> bool {
        return self
            .type_resolver
            .unify(&self.expr.ty.as_ref().unwrap(), target_ty);
    }
}

// Trait method implementation
#[derive(Clone)]
pub struct MethodImpl {
    // Type of this method.
    // For example, in case "impl [a: Show, b: Show] (a, b): Show {...}",
    // the type of method "show" is "[a: Show, b: Show] (a, b) -> String",
    pub ty: Rc<Scheme>,
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
    pub types: HashSet<FullName>,
    pub traits: HashSet<FullName>,
    pub imported_modules: HashSet<Name>,
}

#[derive(PartialEq)]
pub enum NameResolutionType {
    Type,
    Trait,
}

impl<'a> NameResolutionContext {
    pub fn resolve(
        &self,
        ns: &FullName,
        type_or_trait: NameResolutionType,
    ) -> Result<FullName, String> {
        let candidates = if type_or_trait == NameResolutionType::Type {
            &self.types
        } else {
            &self.traits
        };
        let candidates = candidates
            .iter()
            .filter(|name| self.imported_modules.contains(&name.module()))
            .filter_map(|id| {
                if ns.is_suffix(id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if candidates.len() == 0 {
            let msg = match type_or_trait {
                NameResolutionType::Type => {
                    format!("Unknown type name `{}`.", ns.to_string())
                }
                NameResolutionType::Trait => {
                    format!("Unknown trait name `{}`.", ns.to_string())
                }
            };
            Err(msg)
        } else if candidates.len() == 1 {
            Ok(candidates[0].clone())
        } else {
            // candidates.len() >= 2
            let msg = if type_or_trait == NameResolutionType::Type {
                format!("Type name `{}` is ambiguous.", ns.to_string())
            } else {
                format!("Trait name `{}` is ambiguous.", ns.to_string())
            };
            Err(msg)
        }
    }
}

// Program of fix a collection of modules.
// A program can link another program which consists of a single module.
pub struct Program {
    // List of tuple sizes used in this program.
    pub used_tuple_sizes: Vec<u32>,

    pub type_defns: Vec<TypeDefn>,
    pub global_values: HashMap<FullName, GlobalValue>,
    pub instantiated_global_symbols: HashMap<FullName, InstantiatedSymbol>,
    pub deferred_instantiation: HashMap<FullName, InstantiatedSymbol>,
    pub trait_env: TraitEnv,
    pub type_env: TypeEnv,

    // Import statements to be resolved.
    pub unresolved_imports: Vec<ImportStatement>,
    // For each linked module `m`, `visible_mods[m]` is the set of modules imported by `m`.
    // Each module imports itself.
    // This is used to namespace resolution and overloading resolution.
    pub visible_mods: HashMap<Name, HashSet<Name>>,
    // For each module, the path to the source file.
    pub module_to_files: HashMap<Name, SourceFile>,
}

impl Program {
    // Create a program consists of single module.
    pub fn single_module(module_name: Name, src: &SourceFile) -> Program {
        let mut fix_mod = Program {
            unresolved_imports: vec![],
            visible_mods: Default::default(),
            type_defns: Default::default(),
            global_values: Default::default(),
            instantiated_global_symbols: Default::default(),
            deferred_instantiation: Default::default(),
            trait_env: Default::default(),
            type_env: Default::default(),
            used_tuple_sizes: Vec::from_iter(0..=TUPLE_SIZE_BASE),
            module_to_files: Default::default(),
        };
        fix_mod.add_visible_mod(&module_name, &module_name);
        fix_mod.add_visible_mod(&module_name, &STD_NAME.to_string());
        fix_mod.module_to_files.insert(module_name, src.clone());
        fix_mod
    }

    // Add `Std::TupleN` type if not exists.
    pub fn add_tuple_defn(&mut self, tuple_size: u32) {
        self.type_defns.push(tuple_defn(tuple_size));
    }

    // If this program consists of single module, returns its name.
    pub fn get_name_if_single_module(&self) -> Name {
        let linked_mods = self.linked_mods();
        if linked_mods.len() == 1 {
            return linked_mods.into_iter().next().unwrap();
        }
        panic!("")
    }

    // Add import statements.
    pub fn add_import_statements(&mut self, mut imports: Vec<ImportStatement>) {
        for import in &imports {
            self.add_visible_mod(&import.source_module, &import.target_module);
        }
        self.unresolved_imports.append(&mut imports);
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
        for type_decl in &self.type_defns {
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

    // Get of list of traits that can be used for namespace resolution.
    pub fn trait_names_with_aliases(&self) -> HashSet<FullName> {
        self.trait_env.trait_names()
    }

    // Add a global value.
    pub fn add_global_value(&mut self, name: FullName, (expr, scm): (Rc<ExprNode>, Rc<Scheme>)) {
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

    // Generate codes of global symbols.
    pub fn generate_code(&self, gc: &mut GenerationContext) {
        let _sw = StopWatch::new("generate_code", gc.config.show_build_times);

        // First, declare accessor function (a function that returns a pointer to the global value) for a global value, or function for global function value.
        let global_objs = self
            .instantiated_global_symbols
            .iter()
            .map(|(name, sym)| {
                gc.typeresolver = sym.type_resolver.clone();
                let obj_ty = sym.type_resolver.substitute_type(&sym.ty);
                if obj_ty.is_funptr() {
                    // Declare lambda function.
                    let lam = sym.expr.as_ref().unwrap().clone();
                    let lam = lam.set_inferred_type(obj_ty.clone());
                    let lam_fn = gc.declare_lambda_function(lam, Some(name));
                    gc.add_global_object(name.clone(), lam_fn, obj_ty.clone());
                    (name, lam_fn, sym.clone(), obj_ty)
                } else {
                    // Declare accessor function.
                    let acc_fn_name = format!("Get#{}", name.to_string());
                    let acc_fn_type = ptr_to_object_type(gc.context).fn_type(&[], false);
                    let acc_fn =
                        gc.module
                            .add_function(&acc_fn_name, acc_fn_type, Some(Linkage::Internal));

                    // Create debug info subprogram
                    if gc.has_di() {
                        acc_fn.set_subprogram(gc.create_debug_subprogram(
                            &acc_fn_name,
                            sym.expr.as_ref().unwrap().source.clone(),
                        ));
                    }

                    // Register the accessor function to gc.
                    gc.add_global_object(name.clone(), acc_fn, obj_ty.clone());

                    // Return global variable and accessor function.
                    (name, acc_fn, sym.clone(), obj_ty)
                }
            })
            .collect::<Vec<_>>();

        // Implement functions.
        for (name, acc_fn, sym, obj_ty) in global_objs {
            gc.typeresolver = sym.type_resolver;
            if obj_ty.is_funptr() {
                // Implement lambda function.
                let lam_fn = acc_fn;
                let lam = sym.expr.as_ref().unwrap().clone();
                let lam = lam.set_inferred_type(obj_ty);
                gc.implement_lambda_function(lam, lam_fn, None);
            } else {
                // Prepare global variable to store the initialized global value.
                let obj_embed_ty = obj_ty.get_embedded_type(gc, &vec![]);
                let global_var_name = format!("GlobalVar#{}", name.to_string());
                let global_var = gc.module.add_global(obj_embed_ty, None, &global_var_name);
                global_var.set_initializer(&obj_embed_ty.const_zero());
                let global_var = global_var.as_basic_value_enum().into_pointer_value();

                // Prepare initialized flag.
                let flag_name = format!("InitFlag#{}", name.to_string());
                let (flag_ty, flag_init_val) = if gc.config.threaded {
                    (
                        pthread_once_init_flag_type(gc.context),
                        pthread_once_init_flag_value(gc.context),
                    )
                } else {
                    let ty = gc.context.i8_type();
                    (ty, ty.const_zero())
                };
                let init_flag = gc.module.add_global(flag_ty, None, &flag_name);
                init_flag.set_initializer(&flag_init_val);
                let init_flag = init_flag.as_basic_value_enum().into_pointer_value();

                // Start to implement accessor function.
                let entry_bb = gc.context.append_basic_block(acc_fn, "entry");
                gc.builder().position_at_end(entry_bb);

                // Push debug info scope.
                let _di_scope_guard: Option<PopDebugScopeGuard<'_>> = if gc.has_di() {
                    Some(gc.push_debug_scope(
                        acc_fn.get_subprogram().map(|sp| sp.as_debug_info_scope()),
                    ))
                } else {
                    None
                };

                let (init_bb, end_bb, mut init_fun_di_scope_guard) = if !gc.config.threaded {
                    // In single-threaded mode, we implement `call_once` logic by hand.
                    let flag = gc
                        .builder()
                        .build_load(init_flag, "load_init_flag")
                        .into_int_value();
                    let is_zero = gc.builder().build_int_compare(
                        IntPredicate::EQ,
                        flag,
                        flag.get_type().const_zero(),
                        "flag_is_zero",
                    );
                    let init_bb = gc.context.append_basic_block(acc_fn, "flag_is_zero");
                    let end_bb = gc.context.append_basic_block(acc_fn, "flag_is_nonzero");
                    gc.builder()
                        .build_conditional_branch(is_zero, init_bb, end_bb);

                    (init_bb, end_bb, None)
                } else {
                    // In threaded mode, we add a function for initialization and call it by `pthread_once`.

                    // Add initialization function.
                    let init_fn_name = format!("InitOnce#{}", name.to_string());
                    let init_fn_type = gc.context.void_type().fn_type(&[], false);
                    let init_fn = gc.module.add_function(
                        &init_fn_name,
                        init_fn_type,
                        Some(Linkage::Internal),
                    );

                    // Create debug info subprgoram
                    if gc.has_di() {
                        init_fn.set_subprogram(gc.create_debug_subprogram(
                            &init_fn_name,
                            sym.expr.as_ref().unwrap().source.clone(),
                        ));
                    }

                    // In the accessor function, call `init_fn` by `pthread_once`.
                    gc.call_runtime(
                        RuntimeFunctions::PthreadOnce,
                        &[
                            init_flag.into(),
                            init_fn.as_global_value().as_pointer_value().into(),
                        ],
                    );
                    // The end block of the accessor function.
                    let end_bb = gc.context.append_basic_block(acc_fn, "end_bb");
                    gc.builder().build_unconditional_branch(end_bb);

                    // The entry block for the initialization function.
                    let init_bb = gc.context.append_basic_block(init_fn, "init_bb");

                    // Push debug info scope for initialization function.
                    let init_fn_di_scope_guard: Option<PopDebugScopeGuard<'_>> = if gc.has_di() {
                        Some(gc.push_debug_scope(
                            init_fn.get_subprogram().map(|sp| sp.as_debug_info_scope()),
                        ))
                    } else {
                        None
                    };
                    (init_bb, end_bb, init_fn_di_scope_guard)
                };

                // Implement initialization code.
                {
                    // Evaluate object value and store it to the global variable.
                    gc.builder().position_at_end(init_bb);

                    // Prepare memory space for rvo.
                    let rvo = if obj_ty.is_unbox(gc.type_env()) {
                        Some(Object::new(global_var, obj_ty.clone()))
                    } else {
                        None
                    };
                    // Execute expression.
                    let obj = gc.eval_expr(sym.expr.unwrap().clone(), rvo.clone());

                    // Mark the object and all object reachable from it as global.
                    gc.mark_global(obj.clone());

                    // If we didn't rvo, then store the result to global_ptr.
                    if rvo.is_none() {
                        let obj_val = obj.value(gc);
                        gc.builder().build_store(global_var, obj_val);
                    }
                }

                // After initialization,
                if !gc.config.threaded {
                    // In unthreaded mode, set the initialized flag 1 by hand.
                    gc.builder()
                        .build_store(init_flag, gc.context.i8_type().const_int(1, false));

                    // And jump to the end of accessor function.
                    gc.builder().build_unconditional_branch(end_bb);
                } else {
                    // In threaded mode, simply return from the initialization function.
                    gc.builder().build_return(None);

                    // Drop di_scope_guard for initialization function.
                    init_fun_di_scope_guard.take();
                    gc.set_debug_location(None);
                }

                // In the end of the accessor function, merely return the object.
                gc.builder().position_at_end(end_bb);
                let ret = if obj_ty.is_box(gc.type_env()) {
                    gc.builder()
                        .build_load(global_var, "ptr_to_obj")
                        .into_pointer_value()
                } else {
                    global_var
                };
                let ret = gc.cast_pointer(ret, ptr_to_object_type(gc.context));
                gc.builder().build_return(Some(&ret));
            }
        }
    }

    // - Resolve namespace of type and trats in expression,
    // - resolve type aliases, and
    // - perform typechecking.
    // The result will be written to `te`.
    fn resolve_and_check_type(
        &self,
        te: &mut TypedExpr,
        required_scheme: &Rc<Scheme>,
        name: &FullName,
        define_module: &Name,
        tc: &TypeCheckContext,
    ) {
        fn cache_file_name(
            name: &FullName,
            hash_of_dependent_codes: &str,
            scheme: &Rc<Scheme>,
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
            required_scheme: &Rc<Scheme>,
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
            required_scheme: &Rc<Scheme>,
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
        let hash_of_dependent_codes = self.hash_of_dependent_codes(define_module);
        let opt_cache = load_cache(name, &hash_of_dependent_codes, required_scheme);
        if opt_cache.is_some() {
            // If cache is available,
            *te = opt_cache.unwrap();
            te.type_resolver.kind_map = tc.type_env.kinds();
            return;
        }

        // Perform namespace inference.
        let nrctx = NameResolutionContext {
            types: self.tycon_names_with_aliases(),
            traits: self.trait_names_with_aliases(),
            imported_modules: self.visible_mods[define_module].clone(),
        };
        te.expr = te.expr.resolve_namespace(&nrctx);

        // Resolve type aliases in expression.
        te.expr = te.expr.resolve_type_aliases(&tc.type_env);

        // Perform type-checking.
        let mut tc = tc.clone();
        tc.current_module = Some(define_module.clone());
        te.expr = tc.check_type(te.expr.clone(), required_scheme.clone());
        te.type_resolver = tc.resolver;

        // Save the result to cache file.
        save_cache(te, required_scheme, name, &hash_of_dependent_codes);
    }

    // Instantiate symbol.
    fn instantiate_symbol(&mut self, sym: &mut InstantiatedSymbol, tc: &TypeCheckContext) {
        assert!(sym.expr.is_none());
        if !sym.ty.free_vars().is_empty() {
            error_exit_with_src(&format!("Cannot instantiate global value `{}` of type `{}` since the type contains undetermined type variable. Maybe you need to add type annotation.", sym.template_name.to_string(), sym.ty.to_string_normalize()), &sym.expr.as_ref().unwrap().source);
        }
        let global_sym = self.global_values.get(&sym.template_name).unwrap();
        let typed_expr = match &global_sym.expr {
            SymbolExpr::Simple(e) => {
                // Perform type-checking.
                let define_module = sym.template_name.module();
                let mut e = e.clone();
                self.resolve_and_check_type(
                    &mut e,
                    &global_sym.scm,
                    &sym.template_name,
                    &define_module,
                    tc,
                );
                // Calculate free vars.
                e.calculate_free_vars();
                // Specialize e's type to the required type `sym.ty`.
                let ok = e.unify_to(&sym.ty);
                assert!(ok);
                e
            }
            SymbolExpr::Method(impls) => {
                let mut opt_e: Option<TypedExpr> = None;
                for method in impls {
                    // Check if the type of this implementation unify with the required type `sym.ty`.
                    let mut tc0 = tc.clone();
                    let (_, method_ty) = tc0.instantiate_scheme(&method.ty, false);
                    if Substitution::unify(&tc.type_env.kinds(), &method_ty, &sym.ty).is_none() {
                        continue;
                    }
                    // Perform type-checking.
                    let define_module = method.define_module.clone();
                    let mut e = method.expr.clone();
                    self.resolve_and_check_type(
                        &mut e,
                        &method.ty,
                        &sym.template_name,
                        &define_module,
                        tc,
                    );
                    // Calculate free vars.
                    e.calculate_free_vars();
                    // Specialize e's type to required type `sym.ty`
                    assert!(e.unify_to(&sym.ty));
                    opt_e = Some(e);
                    break;
                }
                opt_e.unwrap()
            }
        };
        sym.expr = Some(self.instantiate_expr(&typed_expr.type_resolver, &typed_expr.expr));
        sym.type_resolver = typed_expr.type_resolver;
    }

    // Instantiate all symbols.
    pub fn instantiate_symbols(&mut self, tc: &TypeCheckContext) {
        while !self.deferred_instantiation.is_empty() {
            let (name, sym) = self.deferred_instantiation.iter().next().unwrap();
            let name = name.clone();
            let mut sym = sym.clone();
            self.instantiate_symbol(&mut sym, tc);
            self.deferred_instantiation.remove(&name);
            self.instantiated_global_symbols.insert(name, sym);
        }
    }

    // Instantiate main function.
    pub fn instantiate_main_function(&mut self, tc: &TypeCheckContext) -> Rc<ExprNode> {
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
    fn instantiate_expr(&mut self, tr: &TypeResolver, expr: &Rc<ExprNode>) -> Rc<ExprNode> {
        let ret = match &*expr.expr {
            Expr::Var(v) => {
                if v.name.is_local() {
                    expr.clone()
                } else {
                    let ty = tr.substitute_type(&expr.ty.as_ref().unwrap());
                    let instance = self.require_instantiated_symbol(&v.name, &ty);
                    let v = v.set_name(instance);
                    expr.set_var_var(v)
                }
            }
            Expr::LLVM(_) => expr.clone(),
            Expr::App(fun, args) => {
                let fun = self.instantiate_expr(tr, fun);
                let args = args
                    .iter()
                    .map(|arg| self.instantiate_expr(tr, arg))
                    .collect::<Vec<_>>();
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_, body) => expr.set_lam_body(self.instantiate_expr(tr, body)),
            Expr::Let(_, bound, val) => {
                let bound = self.instantiate_expr(tr, bound);
                let val = self.instantiate_expr(tr, val);
                expr.set_let_bound(bound).set_let_value(val)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.instantiate_expr(tr, cond);
                let then_expr = self.instantiate_expr(tr, then_expr);
                let else_expr = self.instantiate_expr(tr, else_expr);
                expr.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::TyAnno(e, _) => {
                let e = self.instantiate_expr(tr, e);
                expr.set_tyanno_expr(e)
            }
            Expr::MakeStruct(_, fields) => {
                let mut expr = expr.clone();
                for (field_name, field_expr) in fields {
                    let field_expr = self.instantiate_expr(tr, field_expr);
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                expr
            }
            Expr::ArrayLit(elems) => {
                let mut expr = expr.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.instantiate_expr(tr, e);
                    expr = expr.set_array_lit_elem(e, i);
                }
                expr
            }
            Expr::CallC(_, _, _, _, args) => {
                let mut expr = expr.clone();
                for (i, e) in args.iter().enumerate() {
                    let e = self.instantiate_expr(tr, e);
                    expr = expr.set_call_c_arg(e, i);
                }
                expr
            }
        };
        // If the type of an expression contains undetermied type variable after instantiation, raise an error.
        if !tr
            .substitute_type(ret.ty.as_ref().unwrap())
            .free_vars()
            .is_empty()
        {
            error_exit_with_src(
                "The type of an expression cannot be determined. You need to add type annotation to help type inference.",
                &expr.source,
            );
        }
        calculate_free_vars(ret)
    }

    // Require instantiate generic symbol such that it has a specified type.
    pub fn require_instantiated_symbol(&mut self, name: &FullName, ty: &Rc<TypeNode>) -> FullName {
        let inst_name = self.determine_instantiated_symbol_name(name, ty);
        if !self.instantiated_global_symbols.contains_key(&inst_name)
            && !self.deferred_instantiation.contains_key(&inst_name)
        {
            self.deferred_instantiation.insert(
                inst_name.clone(),
                InstantiatedSymbol {
                    template_name: name.clone(),
                    ty: ty.clone(),
                    expr: None,
                    type_resolver: TypeResolver::default(), // This field will be set in the end of instantiation.
                },
            );
        }
        inst_name
    }

    // Determine the name of instantiated generic symbol so that it has a specified type.
    // tc: a typechecker (substituion) under which ty should be interpreted.
    fn determine_instantiated_symbol_name(&self, name: &FullName, ty: &Rc<TypeNode>) -> FullName {
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

    pub fn set_kinds(&mut self) {
        self.trait_env.set_kinds();
        let kind_map = &self.type_env().kinds();
        let trait_kind_map = self.trait_env.trait_kind_map();
        for (_name, sym) in &mut self.global_values {
            sym.set_kinds(kind_map, &trait_kind_map);
        }
    }

    // Infer namespaces to traits and types that appear in declarations (not in expressions).
    // NOTE: names of in the definition of types/traits/global_values have to be full-named already when this function called.
    pub fn resolve_namespace_in_declaration(&mut self) {
        let mut ctx = NameResolutionContext {
            types: self.tycon_names_with_aliases(),
            traits: self.trait_names_with_aliases(),
            imported_modules: HashSet::default(),
        };
        // Resolve namespaces in type constructors.
        {
            let mut tycons = (*self.type_env.tycons).clone();
            for (tc, ti) in &mut tycons {
                ctx.imported_modules = self.visible_mods[&tc.name.module()].clone();
                ti.resolve_namespace(&ctx);
            }
            self.type_env.tycons = Rc::new(tycons);
        }
        // Resolve namespaces in type aliases.
        {
            let mut aliases = (*self.type_env.aliases).clone();
            for (tc, ta) in &mut aliases {
                ctx.imported_modules = self.visible_mods[&tc.name.module()].clone();
                ta.resolve_namespace(&ctx);
            }
            self.type_env.aliases = Rc::new(aliases);
        }

        self.trait_env
            .resolve_namespace(&mut ctx, &self.visible_mods);
        for decl in &mut self.type_defns {
            ctx.imported_modules = self.visible_mods[&decl.name.module()].clone();
            decl.resolve_namespace(&ctx);
        }
        for (name, sym) in &mut self.global_values {
            ctx.imported_modules = self.visible_mods[&name.module()].clone();
            sym.resolve_namespace_in_declaration(&ctx);
        }
    }

    // Resolve type aliases that appear in declarations (not in expressions).
    pub fn resolve_type_aliases_in_declaration(&mut self) {
        // Resolve in type constructors.
        {
            let type_env = self.type_env();
            let mut tycons = (*self.type_env.tycons).clone();
            for (_, ti) in &mut tycons {
                ti.resolve_type_aliases(&type_env);
            }
            self.type_env.tycons = Rc::new(tycons);
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
                TypeDeclValue::Struct(str) => match Field::check_duplication(&str.fields) {
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
                },
                TypeDeclValue::Union(union) => match Field::check_duplication(&union.fields) {
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
                },
                TypeDeclValue::Alias(_) => {} // Nothing to do.
            }
        }
    }

    pub fn validate_trait_env(&mut self) {
        let kind_map = self.type_env.kinds();
        self.trait_env.validate(&kind_map);
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
        self.visible_mods.keys().cloned().collect()
    }

    // Link an module.
    pub fn link(&mut self, mut other: Program) {
        // Morge module file paths.
        for (mod_name, file) in &other.module_to_files {
            let file = file.clone();
            if self.module_to_files.contains_key(mod_name) {
                let another = self.module_to_files.get(mod_name).unwrap();
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
        if self
            .linked_mods()
            .contains(&other.get_name_if_single_module())
        {
            return;
        }

        // Merge imported_mod_map.
        for (importer, importee) in &other.visible_mods {
            if let Some(known_importee) = self.visible_mods.get(importer) {
                assert_eq!(known_importee, importee);
            } else {
                self.visible_mods.insert(importer.clone(), importee.clone());
            }
        }

        // Merge unresolved_imports.
        self.unresolved_imports
            .append(&mut other.unresolved_imports);

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
        while self.unresolved_imports.len() > 0 {
            let import = self.unresolved_imports.pop().unwrap();

            // If import is already resolved, do nothing.
            if self.visible_mods.contains_key(&import.target_module) {
                continue;
            }

            let mut imported = false;
            // Search for bulit-in modules.
            for (mod_name, source_content, file_name, config_modifier, mod_modifier) in
                STANDARD_LIBRARIES
            {
                if import.target_module == *mod_name {
                    let mut fixmod = parse_and_save_to_temporary_file(source_content, file_name);
                    if let Some(mod_modifier) = mod_modifier {
                        mod_modifier(&mut fixmod);
                    }
                    self.link(fixmod);
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
                &format!("Cannot find module `{}`", import.target_module),
                &import.source,
            );
        }
    }

    pub fn add_visible_mod(&mut self, importer: &Name, imported: &Name) {
        if !self.visible_mods.contains_key(importer) {
            self.visible_mods
                .insert(importer.clone(), Default::default());
        }
        self.visible_mods
            .get_mut(importer)
            .unwrap()
            .insert(imported.clone());
    }

    // Create a graph of modules. If module A imports module B, an edge from A to B is added.
    pub fn importing_module_graph(&self) -> (Graph<Name>, HashMap<Name, usize>) {
        let (mut graph, elem_to_idx) = Graph::from_set(self.linked_mods());
        for (importer, importees) in &self.visible_mods {
            for importee in importees {
                graph.connect(
                    *elem_to_idx.get(importer).unwrap(),
                    *elem_to_idx.get(importee).unwrap(),
                );
            }
        }
        (graph, elem_to_idx)
    }

    // Calculate a hash value of a module which is affected by source codes of all dependent modules.
    pub fn hash_of_dependent_codes(&self, module: &Name) -> String {
        let (importing_graph, mod_to_node) = self.importing_module_graph();
        let mut dependent_module_names = importing_graph
            .reachable_nodes(*mod_to_node.get(module).unwrap())
            .iter()
            .map(|idx| importing_graph.get(*idx))
            .collect::<Vec<_>>();
        dependent_module_names.sort(); // To remove randomness introduced by HashSet, we sort it.
        let concatenated_source_hashes = dependent_module_names
            .iter()
            .map(|mod_name| self.module_to_files.get(*mod_name).unwrap().hash())
            .collect::<Vec<_>>()
            .join("");
        format!("{:x}", md5::compute(concatenated_source_hashes))
    }
}
