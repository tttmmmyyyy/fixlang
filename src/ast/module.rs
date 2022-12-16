use inkwell::module::Linkage;

use super::*;

// Module of fix-lang.
// Avoiding name confliction with "Module" of inkwell.

const MAIN_FUNCTION_NAME: &str = "main";

pub struct FixModule {
    pub name: Name,
    pub type_decls: Vec<TypeDecl>,
    pub global_symbols: HashMap<NameSpacedName, GlobalSymbol>,
    pub instantiated_global_symbols: HashMap<NameSpacedName, InstantiatedSymbol>,
    pub deferred_instantiation: HashMap<NameSpacedName, InstantiatedSymbol>,
    pub trait_env: TraitEnv,
    pub type_env: TypeEnv,
}

#[derive(Clone)]
pub struct TypeEnv {
    // List of type constructors including user-defined types.
    pub tycons: Arc<HashMap<TyCon, Arc<Kind>>>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self {
            tycons: Arc::new(Default::default()),
        }
    }
}

impl TypeEnv {
    pub fn new(tycons: HashMap<TyCon, Arc<Kind>>) -> TypeEnv {
        TypeEnv {
            tycons: Arc::new(tycons),
        }
    }

    pub fn kind(&self, tycon: &TyCon) -> Arc<Kind> {
        self.tycons.get(tycon).unwrap().clone()
    }

    pub fn infer_namespace(&self, ns: &NameSpacedName, module_name: &Name) -> NameSpacedName {
        let candidates = self
            .tycons
            .iter()
            .filter_map(|(id, _)| {
                if ns.is_suffix(&id.name) {
                    Some(id.name.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if candidates.len() == 0 {
            error_exit(&format!("unknown type: {}", ns.to_string()))
        } else if candidates.len() == 1 {
            candidates[0].clone()
        } else {
            // candidates.len() >= 2
            let candidates = candidates
                .iter()
                .filter(|name| name.namespace.len() >= 1 && name.namespace.module() == *module_name)
                .collect::<Vec<_>>();
            if candidates.len() == 1 {
                candidates[0].clone()
            } else {
                error_exit("Type name `{}` is ambiguous.");
            }
        }
    }
}

#[derive(Clone)]
pub struct InstantiatedSymbol {
    template_name: NameSpacedName,
    ty: Arc<TypeNode>,
    expr: Option<Arc<ExprNode>>,
}

pub struct GlobalSymbol {
    // Type of this symbol.
    // For example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String for a: Show",
    pub ty: Arc<Scheme>,
    pub expr: SymbolExpr,
    // TODO: add ty_src: Span
    // TODO: add expr_src: Span
}

impl GlobalSymbol {
    pub fn set_namespace_of_tycons_and_traits(
        &mut self,
        type_env: &TypeEnv,
        trait_env: &TraitEnv,
        module_name: &Name,
    ) {
        self.ty = self
            .ty
            .set_namespace_of_tycons_and_traits(type_env, trait_env, module_name);
        self.expr
            .set_namespace_of_tycons_and_traits(type_env, trait_env, module_name);
    }

    pub fn set_kinds(&mut self, type_env: &TypeEnv, trait_kind_map: &HashMap<TraitId, Arc<Kind>>) {
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
    Simple(Arc<ExprNode>),   // Definition such as "id : a -> a; id = \x -> x".
    Method(Vec<MethodImpl>), // Trait method implementations.
}

impl SymbolExpr {
    pub fn set_namespace_of_tycons_and_traits(
        &mut self,
        type_env: &TypeEnv,
        trait_env: &TraitEnv,
        module_name: &Name,
    ) {
        match self {
            SymbolExpr::Simple(e) => {
                *self = SymbolExpr::Simple(e.set_namespace_of_tycons(type_env, module_name));
            }
            SymbolExpr::Method(mis) => {
                for mi in mis {
                    mi.set_namespace_of_tycons_and_traits(type_env, trait_env, module_name);
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
    pub ty: Arc<Scheme>,
    // Expression of this implementation
    pub expr: Arc<ExprNode>,
}

impl MethodImpl {
    pub fn set_namespace_of_tycons_and_traits(
        &mut self,
        type_env: &TypeEnv,
        trait_env: &TraitEnv,
        module_name: &Name,
    ) {
        self.ty = self
            .ty
            .set_namespace_of_tycons_and_traits(type_env, trait_env, module_name);
        self.expr = self.expr.set_namespace_of_tycons(type_env, module_name);
    }
}

impl FixModule {
    // Create empty module.
    pub fn new(name: Name) -> FixModule {
        FixModule {
            name,
            type_decls: Default::default(),
            global_symbols: Default::default(),
            instantiated_global_symbols: Default::default(),
            deferred_instantiation: Default::default(),
            trait_env: Default::default(),
            type_env: Default::default(),
        }
    }

    // Set traits.
    pub fn set_traits(&mut self, trait_infos: Vec<TraitInfo>, trait_impls: Vec<TraitInstance>) {
        self.trait_env
            .set(trait_infos, trait_impls, &self.type_env, &self.name);
    }

    // Register declarations of user-defined types.
    pub fn set_type_decls(&mut self, type_decls: Vec<TypeDecl>) {
        self.type_decls = type_decls;
    }

    // Calculate list of type constructors including user-defined types.
    pub fn calculate_type_env(&mut self) {
        let mut tycons: HashMap<TyCon, Arc<Kind>> = bulitin_type_to_kind_map();
        for type_decl in &self.type_decls {
            let tycon = TyCon::new(NameSpacedName::from_strs(&[&self.name], &type_decl.name));
            if tycons.contains_key(&tycon) {
                error_exit_with_src(
                    &format!("Type `{}` is already defined.", type_decl.name),
                    &None,
                );
            }
            tycons.insert(tycon, kind_star());
        }
        self.type_env = TypeEnv::new(tycons);
    }

    // Get list of type constructors including user-defined types.
    pub fn type_env(&self) -> TypeEnv {
        self.type_env.clone()
    }

    // Get this module's namespace.
    pub fn get_namespace(&self) -> NameSpace {
        NameSpace::new(vec![self.name.clone()])
    }

    // Get this module's namespace with a name.
    pub fn get_namespaced_name(&self, name: &Name) -> NameSpacedName {
        NameSpacedName {
            namespace: self.get_namespace(),
            name: name.clone(),
        }
    }

    // Add a global symbol.
    pub fn add_global_object(
        &mut self,
        name: NameSpacedName,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
    ) {
        if self.global_symbols.contains_key(&name) {
            error_exit(&format!("duplicated global object: `{}`", name.to_string()));
        }
        self.global_symbols.insert(
            name,
            GlobalSymbol {
                ty: scm,
                expr: SymbolExpr::Simple(expr),
            },
        );
    }

    // Generate codes of global symbols.
    pub fn generate_code(&self, gc: &mut GenerationContext) {
        // Create global symbols, global variable and accessor function.
        let global_objs = self
            .instantiated_global_symbols
            .iter()
            .map(|(name, sym)| {
                let ptr_to_obj_ty = ptr_to_object_type(&gc.context);
                let ptr_name = format!("PtrTo{}", name.to_string());
                let acc_fn_name = format!("Get{}", name.to_string());

                // Add global variable.
                let global_var = gc.module.add_global(ptr_to_obj_ty, None, &ptr_name);
                let null = ptr_to_obj_ty.const_null().as_basic_value_enum();
                global_var.set_initializer(&null);
                let global_var = global_var.as_basic_value_enum().into_pointer_value();

                // Add accessor function.
                let acc_fn_type = ptr_to_obj_ty.fn_type(&[], false);
                let acc_fn =
                    gc.module
                        .add_function(&acc_fn_name, acc_fn_type, Some(Linkage::External));

                // Register the accessor function to gc.
                gc.add_global_object(name.clone(), acc_fn);

                // Return global variable and accessor.
                (global_var, acc_fn, sym.clone())
            })
            .collect::<Vec<_>>();

        // Implement global accessor function.
        for (global_var, acc_fn, sym) in global_objs {
            let entry_bb = gc.context.append_basic_block(acc_fn, "entry");
            gc.builder().position_at_end(entry_bb);
            let ptr_to_obj = gc
                .builder()
                .build_load(global_var, "load_global_var")
                .into_pointer_value();
            let is_null = gc.builder().build_is_null(ptr_to_obj, "PtrToObjIsNull");
            let init_bb = gc.context.append_basic_block(acc_fn, "ptr_is_null");
            let end_bb = gc.context.append_basic_block(acc_fn, "ptr_is_non_null");
            gc.builder()
                .build_conditional_branch(is_null, init_bb, end_bb);

            // If ptr is null, then create object and initialize the pointer.
            gc.builder().position_at_end(init_bb);
            let obj = gc.eval_expr(sym.expr.unwrap().clone());
            gc.builder().build_store(global_var, obj);
            if SANITIZE_MEMORY {
                // Mark this object as global.
                let obj_id = gc.get_obj_id(obj);
                gc.call_runtime(RuntimeFunctions::MarkGlobal, &[obj_id.into()]);
            }
            gc.builder().build_unconditional_branch(end_bb);

            // Return object.
            gc.builder().position_at_end(end_bb);
            let ret = gc
                .builder()
                .build_load(global_var, "PtrToObj")
                .into_pointer_value();
            gc.builder().build_return(Some(&ret));
        }
    }

    // Instantiate symbol.
    fn instantiate_symbol(&mut self, tc: &TypeCheckContext, sym: &mut InstantiatedSymbol) {
        assert!(sym.expr.is_none());
        let mut tc = tc.clone();
        let global_sym = self.global_symbols.get(&sym.template_name).unwrap();
        let template_expr = match &global_sym.expr {
            SymbolExpr::Simple(e) => {
                tc.unify(&e.inferred_ty.as_ref().unwrap(), &sym.ty);
                e.clone()
            }
            SymbolExpr::Method(impls) => {
                // Find method implementation that unifies to "sym.ty".
                let mut e: Option<Arc<ExprNode>> = None;
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
    }

    // Instantiate all symbols.
    pub fn instantiate_symbols(&mut self, tc: &TypeCheckContext) {
        while !self.deferred_instantiation.is_empty() {
            let (name, sym) = self.deferred_instantiation.iter().next().unwrap();
            let name = name.clone();
            let mut sym = sym.clone();
            self.instantiate_symbol(tc, &mut sym);
            self.deferred_instantiation.remove(&name);
            self.instantiated_global_symbols.insert(name, sym);
        }
    }

    // Instantiate main function.
    pub fn instantiate_main_function(&mut self, tc: &TypeCheckContext) -> Arc<ExprNode> {
        let main_func_name = self.get_namespaced_name(&MAIN_FUNCTION_NAME.to_string());
        if !self.global_symbols.contains_key(&main_func_name) {
            error_exit("main function not found.")
        } else {
            let inst_name = self.require_instantiated_symbol(tc, &main_func_name, &int_lit_ty());
            self.instantiate_symbols(tc);
            expr_var(inst_name, None)
        }
    }

    // Instantiate expression.
    fn instantiate_expr(&mut self, tc: &TypeCheckContext, expr: &Arc<ExprNode>) -> Arc<ExprNode> {
        let ret = match &*expr.expr {
            Expr::Var(v) => {
                if v.name.is_local() {
                    expr.clone()
                } else {
                    let ty = tc.substitute_type(&expr.inferred_ty.as_ref().unwrap());
                    let instance = self.require_instantiated_symbol(tc, &v.name, &ty);
                    let v = v.set_name(instance);
                    expr.set_var_var(v)
                }
            }
            Expr::Lit(_) => expr.clone(),
            Expr::App(fun, arg) => {
                let fun = self.instantiate_expr(tc, fun);
                let arg = self.instantiate_expr(tc, arg);
                expr.set_app_func(fun).set_app_arg(arg)
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
        };
        calculate_free_vars(ret)
    }

    // Require instantiate generic symbol such that it has a specified type.
    fn require_instantiated_symbol(
        &mut self,
        tc: &TypeCheckContext,
        name: &NameSpacedName,
        ty: &Arc<TypeNode>,
    ) -> NameSpacedName {
        assert!(ty.free_vars().is_empty());
        let inst_name = self.determine_instantiated_symbol_name(tc, name, ty);
        if !self.instantiated_global_symbols.contains_key(&inst_name)
            && !self.deferred_instantiation.contains_key(&inst_name)
        {
            self.deferred_instantiation.insert(
                inst_name.clone(),
                InstantiatedSymbol {
                    template_name: name.clone(),
                    ty: ty.clone(),
                    expr: None,
                },
            );
        }
        inst_name
    }

    // Determine the name of instantiated generic symbol so that it has a specified type.
    fn determine_instantiated_symbol_name(
        &self,
        tc: &TypeCheckContext,
        name: &NameSpacedName,
        ty: &Arc<TypeNode>,
    ) -> NameSpacedName {
        assert!(ty.free_vars().is_empty());
        let mut tc = tc.clone();
        let gs = self.global_symbols.get(name).unwrap();

        // Calculate free variables that is instantiated. They are variables that appear in context predicates.
        let (preds, generic_ty) = tc.instantiate_scheme(&gs.ty, false);
        let mut inst_fvs: HashMap<Name, Arc<Kind>> = Default::default();
        for pred in preds {
            for (k, v) in pred.ty.free_vars() {
                inst_fvs.insert(k, v);
            }
        }

        // Calculate instantiation of free variables.
        let mut sub = Substitution::default();
        tc.unify(&generic_ty, &ty);
        for (name, kind) in inst_fvs {
            let tyvar = type_tyvar(&name, &kind);
            let inst_ty = tc.substitute_type(&tyvar);
            sub.add_substitution(&Substitution::single(&name, inst_ty))
        }

        // Return the name.
        let inst_ty = sub.substitute_type(&generic_ty);
        let type_string = inst_ty.to_string_normalize();
        let hash = format!("{:x}", md5::compute(type_string));
        let mut name = name.clone();
        name.name += "@";
        name.name += &hash;
        name
    }

    // Create symbols of trait methods from TraitEnv.
    pub fn create_trait_method_symbols(&mut self) {
        for (trait_id, trait_info) in &self.trait_env.traits {
            for (method_name, _) in &trait_info.methods {
                let method_ty = trait_info.method_scheme(method_name);
                let mut method_impls: Vec<MethodImpl> = vec![];
                for trait_impl in &trait_info.instances {
                    let ty = trait_impl.method_scheme(method_name, trait_info);
                    let expr = trait_impl.method_expr(method_name);
                    method_impls.push(MethodImpl { ty, expr });
                }
                let method_name = NameSpacedName::new(&trait_id.name.to_namespace(), &method_name);
                self.global_symbols.insert(
                    method_name,
                    GlobalSymbol {
                        ty: method_ty,
                        expr: SymbolExpr::Method(method_impls),
                    },
                );
            }
        }
    }

    pub fn set_kinds(&mut self) {
        self.trait_env.set_kinds();
        let type_env = &self.type_env();
        let trait_kind_map = self.trait_env.trait_kind_map();
        for (_name, sym) in &mut self.global_symbols {
            sym.set_kinds(type_env, &trait_kind_map);
        }
    }

    pub fn set_namespace_of_tycons_and_traits(&mut self) {
        // Currently we don't need to pass trait_env to expressions, since
        // trait name appears in expression only in the namespace of a variable and it will be resolved by typechecker.
        let type_env = self.type_env();
        self.trait_env
            .set_namespace_of_tycons(&type_env, &self.name);
        let trait_env = &self.trait_env;
        for (_, sym) in &mut self.global_symbols {
            sym.set_namespace_of_tycons_and_traits(&type_env, trait_env, &self.name);
        }
    }

    // Add bult-in functions to a given ast.
    pub fn add_builtin_symbols(self: &mut FixModule) {
        fn add_global(
            program: &mut FixModule,
            ns: &[&str],
            name: &str,
            (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
        ) {
            program.add_global_object(NameSpacedName::from_strs(ns, name), (expr, scm));
        }
        add_global(self, &[STD_NAME], "add", add());
        add_global(self, &[STD_NAME], "eq", eq());
        add_global(self, &[STD_NAME], "fix", fix());
        add_global(self, &[STD_NAME, ARRAY_NAME], "new", new_array());
        add_global(self, &[STD_NAME, ARRAY_NAME], "get", read_array());
        add_global(self, &[STD_NAME, ARRAY_NAME], "set", write_array());
        add_global(self, &[STD_NAME, ARRAY_NAME], "set!", write_array_unique());
        for decl in &self.type_decls.clone() {
            match &decl.value {
                TypeDeclValue::Struct(str) => {
                    let module_name = self.name.clone();
                    let ns = vec![module_name.as_str(), decl.name.as_str()];
                    let struct_name =
                        NameSpacedName::from_strs(&[module_name.as_str()], &decl.name);
                    add_global(self, ns.as_slice(), "new", struct_new(&struct_name, str));
                    for field in &str.fields {
                        add_global(
                            self,
                            ns.as_slice(),
                            &format!("get_{}", &field.name),
                            struct_get(&struct_name, str, &field.name),
                        );
                        for is_unique in [false, true] {
                            add_global(
                                self,
                                ns.as_slice(),
                                &format!("mod_{}{}", &field.name, if is_unique { "!" } else { "" }),
                                struct_mod(&struct_name, str, &field.name, is_unique),
                            );
                        }
                    }
                }
            }
        }
    }
}
