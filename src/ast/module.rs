use inkwell::module::Linkage;

use super::*;

// Module of fix-lang.
// Avoiding name confliction with "Module" of inkwell.

const MAIN_FUNCTION_NAME: &str = "main";
const MAIN_MODULE_NAME: &str = "Main";
pub const INSTANCIATED_NAME_SEPARATOR: &str = "%";
pub const GETTER_SYMBOL: &str = "@";
pub const SETTER_SYMBOL: &str = "=";

#[derive(Clone)]
pub struct TypeEnv {
    // List of type constructors including user-defined types.
    pub tycons: Rc<HashMap<TyCon, TyConInfo>>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self {
            tycons: Rc::new(Default::default()),
        }
    }
}

impl TypeEnv {
    pub fn new(tycons: HashMap<TyCon, TyConInfo>) -> TypeEnv {
        TypeEnv {
            tycons: Rc::new(tycons),
        }
    }

    pub fn kind(&self, tycon: &TyCon) -> Rc<Kind> {
        self.tycons.get(tycon).unwrap().kind.clone()
    }
}

#[derive(Clone)]
pub struct InstantiatedSymbol {
    pub template_name: FullName,
    pub ty: Rc<TypeNode>,
    pub expr: Option<Rc<ExprNode>>,
    pub typechecker: Option<TypeCheckContext>, // type checker available for resolving types in expr.
}

pub struct GlobalValue {
    // Type of this symbol.
    // For example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String for a: Show",
    pub ty: Rc<Scheme>,
    pub expr: SymbolExpr,
    // Result of typechecking (mainly, substitution) of this symbol.
    pub typecheck_log: Option<TypeCheckContext>,
    // TODO: add ty_src: Span
    // TODO: add expr_src: Span
}

impl GlobalValue {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        self.ty = self.ty.resolve_namespace(ctx);
        self.expr.resolve_namespace(ctx);
    }

    pub fn set_kinds(&mut self, type_env: &TypeEnv, trait_kind_map: &HashMap<TraitId, Rc<Kind>>) {
        self.ty = self.ty.set_kinds(trait_kind_map);
        self.ty.check_kinds(type_env, trait_kind_map);
        match &mut self.expr {
            SymbolExpr::Simple(_) => {}
            SymbolExpr::Method(ms) => {
                for m in ms {
                    m.ty = m.ty.set_kinds(trait_kind_map);
                    m.ty.check_kinds(type_env, trait_kind_map);
                }
            }
        }
    }
}

// Expression of global symbol.
#[derive(Clone)]
pub enum SymbolExpr {
    Simple(Rc<ExprNode>),    // Definition such as "id : a -> a; id = \x -> x".
    Method(Vec<MethodImpl>), // Trait method implementations.
}

impl SymbolExpr {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        match self {
            SymbolExpr::Simple(e) => {
                *self = SymbolExpr::Simple(e.resolve_namespace(ctx));
            }
            SymbolExpr::Method(mis) => {
                for mi in mis {
                    mi.resolve_namespace(ctx);
                }
            }
        }
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
    pub expr: Rc<ExprNode>,
    // Module where this implmentation is given.
    // NOTE:
    // For trait method, `define_module` may not differ to the first component of namespace of the function.
    // For example, if `Main` module implements `Eq : SomeType`, then implementation of `eq` for `SomeType` is defined in `Main` module,
    // but it's name as a function is still `Std::Eq::eq`.
    pub define_module: Name,
}

impl MethodImpl {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        self.ty = self.ty.resolve_namespace(ctx);
        self.expr = self.expr.resolve_namespace(ctx);
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
    // pub fn filter_by_imported_modules(&self, imported_modules: &HashSet<Name>) -> Self {
    //     fn filter(source: &HashSet<FullName>, modules: &HashSet<Name>) -> HashSet<FullName> {
    //         HashSet::from_iter(
    //             source
    //                 .iter()
    //                 .filter(|name| modules.contains(&name.module()))
    //                 .cloned(),
    //         )
    //     }

    //     NameResolutionContext {
    //         types: filter(&self.types, imported_modules),
    //         traits: filter(&self.traits, imported_modules),
    //     }
    // }

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
                    format!("Unknown type name: {}", ns.to_string())
                }
                NameResolutionType::Trait => {
                    format!("Unknown trait name: {}", ns.to_string())
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

pub struct FixModule {
    pub name: Name,
    pub import_statements: Vec<ImportStatement>,
    // A map to represent modules imported by each submodule.
    // Each module imports itself.
    // This is used to name-resolution and overloading resolution,
    pub imported_mod_map: HashMap<Name, HashSet<Name>>,
    // Modules linked to this module.
    pub linked_mods: HashSet<Name>,
    pub type_defns: Vec<TypeDefn>,
    pub global_values: HashMap<FullName, GlobalValue>,
    pub instantiated_global_symbols: HashMap<FullName, InstantiatedSymbol>,
    pub deferred_instantiation: HashMap<FullName, InstantiatedSymbol>,
    pub trait_env: TraitEnv,
    pub type_env: TypeEnv,
}

impl FixModule {
    // Create empty module.
    pub fn new(name: Name) -> FixModule {
        let mut fix_mod = FixModule {
            name: name.clone(),
            import_statements: vec![],
            imported_mod_map: Default::default(),
            linked_mods: Default::default(),
            type_defns: Default::default(),
            global_values: Default::default(),
            instantiated_global_symbols: Default::default(),
            deferred_instantiation: Default::default(),
            trait_env: Default::default(),
            type_env: Default::default(),
        };
        fix_mod.insert_imported_mod_map(&name, &name);
        fix_mod.insert_imported_mod_map(&name, &STD_NAME.to_string());
        fix_mod
    }

    // Add import statements.
    pub fn add_import_statements(&mut self, mut imports: Vec<ImportStatement>) {
        for import in &imports {
            let mod_name = self.name.clone();
            self.insert_imported_mod_map(&mod_name, &import.module);
        }
        self.import_statements.append(&mut imports);
    }

    // Add traits.
    pub fn add_traits(&mut self, trait_infos: Vec<TraitInfo>, trait_impls: Vec<TraitInstance>) {
        self.trait_env.add(trait_infos, trait_impls);
    }

    // Register declarations of user-defined types.
    pub fn add_type_defns(&mut self, mut type_defns: Vec<TypeDefn>) {
        self.type_defns.append(&mut type_defns);
    }

    // Calculate list of type constructors including user-defined types.
    pub fn calculate_type_env(&mut self) {
        let mut tycons = bulitin_tycons();
        for type_decl in &self.type_defns {
            let tycon = type_decl.tycon();
            if tycons.contains_key(&tycon) {
                error_exit_with_src(
                    &format!("Type `{}` is already defined.", tycon.to_string()),
                    &None,
                );
            }
            tycons.insert(tycon, type_decl.tycon_info());
        }
        self.type_env = TypeEnv::new(tycons);
    }

    // Get list of type constructors including user-defined types.
    pub fn type_env(&self) -> TypeEnv {
        self.type_env.clone()
    }

    // Get of list of tycons that can be used for namespace resolution.
    pub fn tycon_names(&self) -> HashSet<FullName> {
        let mut res: HashSet<FullName> = Default::default();
        for (k, _v) in self.type_env().tycons.iter() {
            res.insert(k.name.clone());
        }
        res
    }

    // Get of list of traits that can be used for namespace resolution.
    pub fn trait_names(&self) -> HashSet<FullName> {
        let mut res: HashSet<FullName> = Default::default();
        for (k, _v) in &self.trait_env.traits {
            res.insert(k.name.clone());
        }
        res
    }

    // Add a global value.
    pub fn add_global_value(&mut self, name: FullName, (expr, scm): (Rc<ExprNode>, Rc<Scheme>)) {
        if self.global_values.contains_key(&name) {
            error_exit(&format!(
                "duplicated definition for global value: `{}`",
                name.to_string()
            ));
        }
        self.global_values.insert(
            name,
            GlobalValue {
                ty: scm,
                expr: SymbolExpr::Simple(expr),
                typecheck_log: None,
            },
        );
    }

    // Add global values
    pub fn add_global_values(
        &mut self,
        exprs: Vec<(FullName, Rc<ExprNode>)>,
        types: Vec<(FullName, Rc<Scheme>)>,
    ) {
        struct GlobalValue {
            expr: Option<Rc<ExprNode>>,
            ty: Option<Rc<Scheme>>,
        }

        let mut global_values: HashMap<FullName, GlobalValue> = Default::default();
        for (name, expr) in exprs {
            if !global_values.contains_key(&name) {
                global_values.insert(
                    name,
                    GlobalValue {
                        expr: Some(expr),
                        ty: None,
                    },
                );
            } else {
                let gs = global_values.get_mut(&name).unwrap();
                if gs.expr.is_some() {
                    error_exit(&format!(
                        "duplicated definition signature for global value: `{}`",
                        name.to_string()
                    ));
                } else {
                    gs.expr = Some(expr);
                }
            }
        }
        for (name, ty) in types {
            if !global_values.contains_key(&name) {
                global_values.insert(
                    name,
                    GlobalValue {
                        ty: Some(ty),
                        expr: None,
                    },
                );
            } else {
                let gs = global_values.get_mut(&name).unwrap();
                if gs.ty.is_some() {
                    error_exit(&format!(
                        "duplicated type signature for `{}`",
                        name.to_string()
                    ));
                } else {
                    gs.ty = Some(ty);
                }
            }
        }

        for (name, gv) in global_values {
            if gv.expr.is_none() {
                error_exit(&format!(
                    "global value `{}` lacks type signature",
                    name.to_string()
                ))
            }
            if gv.ty.is_none() {
                error_exit(&format!(
                    "global value `{}` lacks definition",
                    name.to_string()
                ))
            }
            self.add_global_value(name, (gv.expr.unwrap(), gv.ty.unwrap()))
        }
    }

    // Generate codes of global symbols.
    pub fn generate_code(&self, gc: &mut GenerationContext) {
        // First,
        // - For function pointer, declare the function and register it to global variable.
        // - For others, create global variable and declare accessor function and register it to global variable.
        let global_objs = self
            .instantiated_global_symbols
            .iter()
            .map(|(name, sym)| {
                gc.typechecker = sym.typechecker.clone();
                let obj_ty = sym.typechecker.as_ref().unwrap().substitute_type(&sym.ty);
                if obj_ty.is_funptr() {
                    let lam = sym.expr.as_ref().unwrap().clone();
                    let lam = lam.set_inferred_type(obj_ty.clone());
                    let lam_fn = gc.declare_lambda_function(lam);
                    gc.add_global_object(name.clone(), lam_fn, obj_ty.clone());
                    (None, None, lam_fn, sym.clone(), obj_ty)
                } else {
                    let flag_name = format!("InitFlag{}", name.to_string());
                    let global_var_name = format!("GlobalVar{}", name.to_string());
                    let acc_fn_name = format!("Get{}", name.to_string());

                    let obj_embed_ty = obj_ty.get_embedded_type(gc, &vec![]);

                    // Add global variable.
                    let global_var = gc.module.add_global(obj_embed_ty, None, &global_var_name);
                    global_var.set_initializer(&obj_embed_ty.const_zero());
                    let global_var = global_var.as_basic_value_enum().into_pointer_value();

                    // Add initialized flag.
                    let flag_ty = gc.context.i8_type();
                    let init_flag = gc.module.add_global(flag_ty, None, &flag_name);
                    init_flag.set_initializer(&flag_ty.const_zero());
                    let init_flag = init_flag.as_basic_value_enum().into_pointer_value();

                    // Add accessor function.
                    let acc_fn_type = ptr_to_object_type(gc.context).fn_type(&[], false);
                    let acc_fn =
                        gc.module
                            .add_function(&acc_fn_name, acc_fn_type, Some(Linkage::Internal));

                    // Register the accessor function to gc.
                    gc.add_global_object(name.clone(), acc_fn, obj_ty.clone());

                    // Return global variable and accessor function.
                    (
                        Some(global_var),
                        Some(init_flag),
                        acc_fn,
                        sym.clone(),
                        obj_ty,
                    )
                }
            })
            .collect::<Vec<_>>();

        // Implement functions.
        for (global_var, init_flag, acc_fn, sym, obj_ty) in global_objs {
            gc.typechecker = sym.typechecker;
            if obj_ty.is_funptr() {
                // Implement lambda function.
                let lam_fn = acc_fn;
                let lam = sym.expr.as_ref().unwrap().clone();
                let lam = lam.set_inferred_type(obj_ty);
                gc.implement_lambda_function(lam, lam_fn, None);
            } else {
                // Implement accessor function.
                let global_var = global_var.unwrap();
                let init_flag = init_flag.unwrap();
                let entry_bb = gc.context.append_basic_block(acc_fn, "entry");
                gc.builder().position_at_end(entry_bb);
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

                // If flag is zero, then create object and store it to the global variable.
                gc.builder().position_at_end(init_bb);
                // Prepare memory space for rvo.
                let rvo = if obj_ty.is_unbox(gc.type_env()) {
                    Some(Object::new(global_var, obj_ty))
                } else {
                    None
                };
                // Execute expression.
                let obj = gc.eval_expr(sym.expr.unwrap().clone(), rvo.clone());

                if gc.config.preretain_global && obj.is_box(gc.type_env()) {
                    let obj_ptr = obj.ptr(gc);
                    let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
                    // Pre-retain global objects (to omit retaining later).
                    let infty = refcnt_type(gc.context).const_int(u64::MAX / 2, false);
                    gc.builder().build_store(ptr_to_refcnt, infty);
                }
                // If we didn't rvo, then store the result to global_ptr.
                if rvo.is_none() {
                    let obj_val = obj.value(gc);
                    gc.builder().build_store(global_var, obj_val);
                }

                // Set the initialized flag 1.
                gc.builder()
                    .build_store(init_flag, gc.context.i8_type().const_int(1, false));

                if gc.config.sanitize_memory && obj.is_box(gc.type_env()) {
                    // Mark this object as global.
                    let ptr = obj.ptr(gc);
                    let obj_id = gc.get_obj_id(ptr);
                    gc.call_runtime(RuntimeFunctions::MarkGlobal, &[obj_id.into()]);
                }
                gc.builder().build_unconditional_branch(end_bb);

                // Return object.
                gc.builder().position_at_end(end_bb);
                let ret = if obj.is_box(gc.type_env()) {
                    gc.builder()
                        .build_load(global_var, "PtrToObj")
                        .into_pointer_value()
                } else {
                    global_var
                };
                let ret = gc.cast_pointer(ret, ptr_to_object_type(gc.context));
                gc.builder().build_return(Some(&ret));
            }
        }
    }

    // Instantiate symbol.
    fn instantiate_symbol(&mut self, mut tc: TypeCheckContext, sym: &mut InstantiatedSymbol) {
        assert!(sym.expr.is_none());
        let global_sym = self.global_values.get(&sym.template_name).unwrap();
        let template_expr = match &global_sym.expr {
            SymbolExpr::Simple(e) => {
                tc.unify(&e.inferred_ty.as_ref().unwrap(), &sym.ty);
                e.clone()
            }
            SymbolExpr::Method(impls) => {
                // Find method implementation that unifies to "sym.ty".
                let mut e: Option<Rc<ExprNode>> = None;
                for method in impls {
                    if tc.unify(&method.expr.inferred_ty.as_ref().unwrap(), &sym.ty) {
                        e = Some(method.expr.clone());
                        break;
                    }
                }
                e.unwrap()
            }
        };
        sym.expr = Some(self.instantiate_expr(&tc, &template_expr));
        sym.typechecker = Some(tc);
    }

    // Instantiate all symbols.
    pub fn instantiate_symbols(&mut self) {
        while !self.deferred_instantiation.is_empty() {
            let (name, sym) = self.deferred_instantiation.iter().next().unwrap();
            let gs = &self.global_values[&sym.template_name];
            let tc = gs.typecheck_log.as_ref().unwrap().clone();
            let name = name.clone();
            let mut sym = sym.clone();
            self.instantiate_symbol(tc, &mut sym);
            self.deferred_instantiation.remove(&name);
            self.instantiated_global_symbols.insert(name, sym);
        }
    }

    // Instantiate main function.
    pub fn instantiate_main_function(&mut self) -> Rc<ExprNode> {
        let main_func_name = FullName::from_strs(&[MAIN_MODULE_NAME], MAIN_FUNCTION_NAME);
        if !self.global_values.contains_key(&main_func_name) {
            error_exit(&format!("{} not found.", main_func_name.to_string()));
        }
        let main_ty = make_io_unit_ty();
        let inst_name = self.require_instantiated_symbol(&main_func_name, &main_ty);
        self.instantiate_symbols();
        expr_var(inst_name, None).set_inferred_type(main_ty)
    }

    // Instantiate expression.
    fn instantiate_expr(&mut self, tc: &TypeCheckContext, expr: &Rc<ExprNode>) -> Rc<ExprNode> {
        let ret = match &*expr.expr {
            Expr::Var(v) => {
                if v.name.is_local() {
                    expr.clone()
                } else {
                    let ty = tc.substitute_type(&expr.inferred_ty.as_ref().unwrap());
                    let instance = self.require_instantiated_symbol(&v.name, &ty);
                    let v = v.set_name(instance);
                    expr.set_var_var(v)
                }
            }
            Expr::Lit(_) => expr.clone(),
            Expr::App(fun, args) => {
                let fun = self.instantiate_expr(tc, fun);
                let args = args
                    .iter()
                    .map(|arg| self.instantiate_expr(tc, arg))
                    .collect::<Vec<_>>();
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_, body) => expr.set_lam_body(self.instantiate_expr(tc, body)),
            Expr::Let(_, bound, val) => {
                let bound = self.instantiate_expr(tc, bound);
                let val = self.instantiate_expr(tc, val);
                expr.set_let_bound(bound).set_let_value(val)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.instantiate_expr(tc, cond);
                let then_expr = self.instantiate_expr(tc, then_expr);
                let else_expr = self.instantiate_expr(tc, else_expr);
                expr.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::TyAnno(e, _) => {
                let e = self.instantiate_expr(tc, e);
                expr.set_tyanno_expr(e)
            }
            Expr::MakeStruct(_, fields) => {
                let mut expr = expr.clone();
                for (field_name, field_expr) in fields {
                    let field_expr = self.instantiate_expr(tc, field_expr);
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                expr
            }
            Expr::ArrayLit(elems) => {
                let mut expr = expr.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.instantiate_expr(tc, e);
                    expr = expr.set_array_lit_elem(e, i);
                }
                expr
            }
            Expr::CallC(_, _, _, _, args) => {
                let mut expr = expr.clone();
                for (i, e) in args.iter().enumerate() {
                    let e = self.instantiate_expr(tc, e);
                    expr = expr.set_call_c_arg(e, i);
                }
                expr
            }
        };
        // If the type of an expression contains undetermied type variable after instantiation, raise an error.
        if !tc
            .substitute_type(ret.inferred_ty.as_ref().unwrap())
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
        if !ty.free_vars().is_empty() {
            error_exit(&format!("Cannot instantiate global value `{}` of type `{}` since the type contains undetermined type variable. Maybe you need to add a type annotation.", name.to_string(), ty.to_string_normalize()));
        }
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
                    typechecker: None, // This field will be set after instantiation.
                                       // In instantiation, typechecker carried by GlobalSymbol is used and set to this field in the end.
                },
            );
        }
        inst_name
    }

    // Determine the name of instantiated generic symbol so that it has a specified type.
    // tc: a typechecker (substituion) under which ty should be interpret.
    fn determine_instantiated_symbol_name(&self, name: &FullName, ty: &Rc<TypeNode>) -> FullName {
        assert!(ty.free_vars().is_empty());
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
                        let ty = trait_impl.method_scheme(method_name, trait_info);
                        let expr = trait_impl.method_expr(method_name);
                        method_impls.push(MethodImpl {
                            ty,
                            expr,
                            define_module: trait_impl.define_module.clone(),
                        });
                    }
                }
                let method_name = FullName::new(&trait_id.name.to_namespace(), &method_name);
                self.global_values.insert(
                    method_name,
                    GlobalValue {
                        ty: method_ty,
                        expr: SymbolExpr::Method(method_impls),
                        typecheck_log: None,
                    },
                );
            }
        }
    }

    pub fn set_kinds(&mut self) {
        self.trait_env.set_kinds();
        let type_env = &self.type_env();
        let trait_kind_map = self.trait_env.trait_kind_map();
        for (_name, sym) in &mut self.global_values {
            sym.set_kinds(type_env, &trait_kind_map);
        }
    }

    // Resolve namespaces of types and traits that appear in this module.
    // NOTE: names of in the definition of types/traits/global_values have to be full-named already when this function called.
    pub fn resolve_namespace(&mut self) {
        let mut ctx = NameResolutionContext {
            types: self.tycon_names(),
            traits: self.trait_names(),
            imported_modules: HashSet::default(),
        };
        {
            let mut tycons = (*self.type_env.tycons).clone();
            for (tc, ti) in &mut tycons {
                ctx.imported_modules = self.imported_mod_map[&tc.name.module()].clone();
                ti.resolve_namespace(&ctx);
            }
            self.type_env.tycons = Rc::new(tycons);
        }

        self.trait_env
            .resolve_namespace(&mut ctx, &self.imported_mod_map);
        for decl in &mut self.type_defns {
            ctx.imported_modules = self.imported_mod_map[&decl.name.module()].clone();
            decl.resolve_namespace(&ctx);
        }
        for (name, sym) in &mut self.global_values {
            ctx.imported_modules = self.imported_mod_map[&name.module()].clone();
            sym.resolve_namespace(&ctx);
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
                        error_exit(&format!(
                            "Duplicate field `{}` for struct `{}`",
                            field_name,
                            type_name.to_string()
                        ));
                    }
                    _ => {}
                },
                TypeDeclValue::Union(union) => match Field::check_duplication(&union.fields) {
                    Some(field_name) => {
                        error_exit(&format!(
                            "Duplicate field `{}` for union `{}`",
                            field_name,
                            type_name.to_string()
                        ));
                    }
                    _ => {}
                },
            }
        }
    }

    pub fn validate_trait_env(&mut self) {
        self.trait_env.validate(&self.type_env);
    }

    pub fn add_methods(self: &mut FixModule) {
        for decl in &self.type_defns.clone() {
            match &decl.value {
                TypeDeclValue::Struct(str) => {
                    let struct_name = decl.name.clone();
                    for field in &str.fields {
                        self.add_global_value(
                            FullName::new(
                                &decl.name.to_namespace(),
                                &format!("{}{}", GETTER_SYMBOL, &field.name),
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
                                        SETTER_SYMBOL,
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
            }
        }
    }

    // Link two modules.
    pub fn link(&mut self, other: FixModule) {
        // TODO: check if a module defined by a single source file.

        // If already linked, do nothing.
        if self.linked_mods.contains(&other.name) {
            return;
        }
        self.linked_mods.insert(other.name);

        // Merge imported_mod_map.
        for (importer, importee) in &other.imported_mod_map {
            if let Some(known_importee) = self.imported_mod_map.get(importer) {
                assert_eq!(known_importee, importee);
            } else {
                self.imported_mod_map
                    .insert(importer.clone(), importee.clone());
            }
        }

        // Merge types.
        self.add_type_defns(other.type_defns);

        // Merge traits and instances.
        self.trait_env.import(other.trait_env);

        // Merge global values.
        for (name, gv) in other.global_values {
            let ty = gv.ty;
            if let SymbolExpr::Simple(expr) = gv.expr {
                self.add_global_value(name, (expr, ty));
            }
        }
    }

    pub fn insert_imported_mod_map(&mut self, importer: &Name, imported: &Name) {
        if !self.imported_mod_map.contains_key(importer) {
            self.imported_mod_map
                .insert(importer.clone(), Default::default());
        }
        self.imported_mod_map
            .get_mut(importer)
            .unwrap()
            .insert(imported.clone());
    }
}
