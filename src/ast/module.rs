use inkwell::module::Linkage;

use super::*;

// Module of fix-lang.
// Avoiding name confliction with "Module" of inkwell.

const MAIN_FUNCTION_NAME: &str = "main";
pub const INSTANCIATED_NAME_SEPARATOR: &str = "@";

pub struct FixModule {
    pub name: Name,
    pub type_decls: Vec<TypeDecl>,
    pub global_symbols: HashMap<FullName, GlobalSymbol>,
    pub instantiated_global_symbols: HashMap<FullName, InstantiatedSymbol>,
    pub deferred_instantiation: HashMap<FullName, InstantiatedSymbol>,
    pub trait_env: TraitEnv,
    pub type_env: TypeEnv,
}

#[derive(Clone)]
pub struct TypeEnv {
    // List of type constructors including user-defined types.
    pub tycons: Arc<HashMap<TyCon, TyConInfo>>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self {
            tycons: Arc::new(Default::default()),
        }
    }
}

impl TypeEnv {
    pub fn new(tycons: HashMap<TyCon, TyConInfo>) -> TypeEnv {
        TypeEnv {
            tycons: Arc::new(tycons),
        }
    }

    pub fn kind(&self, tycon: &TyCon) -> Arc<Kind> {
        self.tycons.get(tycon).unwrap().kind.clone()
    }
}

#[derive(Clone)]
pub struct InstantiatedSymbol {
    pub template_name: FullName,
    pub ty: Arc<TypeNode>,
    pub expr: Option<Arc<ExprNode>>,
    pub typechecker: Option<TypeCheckContext>, // type checker available for resolving types in expr.
}

pub struct GlobalSymbol {
    // Type of this symbol.
    // For example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String for a: Show",
    pub ty: Arc<Scheme>,
    pub expr: SymbolExpr,
    // Result of typechecking (mainly, substitution) of this symbol.
    pub typecheck_log: Option<TypeCheckContext>,
    // TODO: add ty_src: Span
    // TODO: add expr_src: Span
}

impl GlobalSymbol {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        self.ty = self.ty.resolve_namespace(ctx);
        self.expr.resolve_namespace(ctx);
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
    pub ty: Arc<Scheme>,
    // Expression of this implementation
    pub expr: Arc<ExprNode>,
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
    pub module_name: Name,
}

#[derive(PartialEq)]
pub enum NameResolutionType {
    Type,
    Trait,
}

impl NameResolutionContext {
    pub fn resolve(&self, ns: &FullName, type_or_trait: NameResolutionType) -> FullName {
        let candidates = if type_or_trait == NameResolutionType::Type {
            &self.types
        } else {
            &self.traits
        };
        let candidates = candidates
            .iter()
            .filter_map(|id| {
                if ns.is_suffix(id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if candidates.len() == 0 {
            error_exit(&format!("unknown name: {}", ns.to_string()))
        } else if candidates.len() == 1 {
            candidates[0].clone()
        } else {
            // candidates.len() >= 2
            let candidates = candidates
                .iter()
                .filter(|name| {
                    name.namespace.len() >= 1 && name.namespace.module() == self.module_name
                })
                .collect::<Vec<_>>();
            if candidates.len() == 1 {
                candidates[0].clone()
            } else {
                error_exit("Trait name `{}` is ambiguous.");
            }
        }
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
    pub fn add_traits(&mut self, trait_infos: Vec<TraitInfo>, trait_impls: Vec<TraitInstance>) {
        self.trait_env.set(trait_infos, trait_impls);
    }

    // Register declarations of user-defined types.
    pub fn set_type_decls(&mut self, type_decls: Vec<TypeDecl>) {
        self.type_decls = type_decls;
    }

    // Calculate list of type constructors including user-defined types.
    pub fn calculate_type_env(&mut self) {
        let mut tycons = bulitin_tycons();
        for type_decl in &self.type_decls {
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

    // Get this module's namespace.
    pub fn get_namespace(&self) -> NameSpace {
        NameSpace::new(vec![self.name.clone()])
    }

    // Get this module's namespace with a name.
    pub fn get_namespaced_name(&self, name: &Name) -> FullName {
        FullName {
            namespace: self.get_namespace(),
            name: name.clone(),
        }
    }

    // Add a global symbol.
    pub fn add_global_object(&mut self, name: FullName, (expr, scm): (Arc<ExprNode>, Arc<Scheme>)) {
        if self.global_symbols.contains_key(&name) {
            error_exit(&format!("duplicated global object: `{}`", name.to_string()));
        }
        self.global_symbols.insert(
            name,
            GlobalSymbol {
                ty: scm,
                expr: SymbolExpr::Simple(expr),
                typecheck_log: None,
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
                gc.typechecker = sym.typechecker.clone();

                let flag_name = format!("InitFlag{}", name.to_string());
                let global_var_name = format!("GlobalVar{}", name.to_string());
                let acc_fn_name = format!("Get{}", name.to_string());

                let obj_ty = sym.typechecker.as_ref().unwrap().substitute_type(&sym.ty);
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

                // Return global variable and accessor.
                (global_var, init_flag, acc_fn, sym.clone(), obj_ty)
            })
            .collect::<Vec<_>>();

        // Implement global accessor function.
        for (global_var, init_flag, acc_fn, sym, obj_ty) in global_objs {
            gc.typechecker = sym.typechecker;

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

            if NOT_RETAIN_GLOBAL && obj.is_box(gc.type_env()) {
                let obj_ptr = obj.ptr(gc);
                let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
                // Pre-retain global objects (to omit retaining later).
                let infty = refcnt_type(gc.context).const_int(u64::MAX / 2, false);
                gc.builder().build_store(ptr_to_refcnt, infty);
            }
            // If we didn't rvo, then store the result to gloval_ptr.
            if rvo.is_none() {
                let obj_val = obj.value(gc);
                gc.builder().build_store(global_var, obj_val);
            }

            // Set the initialized flag 1.
            gc.builder()
                .build_store(init_flag, gc.context.i8_type().const_int(1, false));

            if SANITIZE_MEMORY && obj.is_box(gc.type_env()) {
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

    // Instantiate symbol.
    fn instantiate_symbol(&mut self, mut tc: TypeCheckContext, sym: &mut InstantiatedSymbol) {
        assert!(sym.expr.is_none());
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
        sym.typechecker = Some(tc);
    }

    // Instantiate all symbols.
    pub fn instantiate_symbols(&mut self) {
        while !self.deferred_instantiation.is_empty() {
            let (name, sym) = self.deferred_instantiation.iter().next().unwrap();
            let gs = &self.global_symbols[&sym.template_name];
            let tc = gs.typecheck_log.as_ref().unwrap().clone();
            let name = name.clone();
            let mut sym = sym.clone();
            self.instantiate_symbol(tc, &mut sym);
            self.deferred_instantiation.remove(&name);
            self.instantiated_global_symbols.insert(name, sym);
        }
    }

    // Instantiate main function.
    pub fn instantiate_main_function(&mut self) -> Arc<ExprNode> {
        let main_func_name = self.get_namespaced_name(&MAIN_FUNCTION_NAME.to_string());
        if !self.global_symbols.contains_key(&main_func_name) {
            error_exit("main function not found.");
        }
        let main_ty = int_lit_ty();
        let inst_name = self.require_instantiated_symbol(&main_func_name, &main_ty);
        self.instantiate_symbols();
        expr_var(inst_name, None).set_inferred_type(main_ty)
    }

    // Instantiate expression.
    fn instantiate_expr(&mut self, tc: &TypeCheckContext, expr: &Arc<ExprNode>) -> Arc<ExprNode> {
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
            Expr::MakeTuple(fields) => {
                let mut expr = expr.clone();
                for (i, field) in fields.iter().enumerate() {
                    let field = self.instantiate_expr(tc, field);
                    expr = expr.set_make_tuple_field(field, i);
                }
                expr
            }
        };
        calculate_free_vars(ret)
    }

    // Require instantiate generic symbol such that it has a specified type.
    pub fn require_instantiated_symbol(&mut self, name: &FullName, ty: &Arc<TypeNode>) -> FullName {
        if !ty.free_vars().is_empty() {
            error_exit(&format!("cannot instantiate global value `{}` of type `{}` since the type contains undetermined type variable. Maybe you need to add a type annotation.", name.to_string(), ty.to_string()));
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
    fn determine_instantiated_symbol_name(&self, name: &FullName, ty: &Arc<TypeNode>) -> FullName {
        /*

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
        let hash = inst_ty.hash();
        let mut name = name.clone();
        name.name += "@";
        name.name += &hash;
        name
        */

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
                for trait_impl in self.trait_env.instances.get(trait_id).unwrap() {
                    let ty = trait_impl.method_scheme(method_name, trait_info);
                    let expr = trait_impl.method_expr(method_name);
                    method_impls.push(MethodImpl { ty, expr });
                }
                let method_name = FullName::new(&trait_id.name.to_namespace(), &method_name);
                self.global_symbols.insert(
                    method_name,
                    GlobalSymbol {
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
        for (_name, sym) in &mut self.global_symbols {
            sym.set_kinds(type_env, &trait_kind_map);
        }
    }

    // Resolve namespaces of types and traits that appear in this module.
    // NOTE: names in types/traits defined in this module have to be full-names already when calling this function.
    pub fn resolve_namespace(&mut self) {
        let ctx = NameResolutionContext {
            types: self.tycon_names(),
            traits: self.trait_names(),
            module_name: self.name.clone(),
        };
        {
            let mut tycons = (*self.type_env.tycons).clone();
            for (_, ti) in &mut tycons {
                ti.resolve_namespace(&ctx);
            }
            self.type_env.tycons = Arc::new(tycons);
        }

        self.trait_env.resolve_namespace(&ctx);
        for decl in &mut self.type_decls {
            decl.resolve_namespace(&ctx);
        }
        for (_, sym) in &mut self.global_symbols {
            sym.resolve_namespace(&ctx);
        }
    }

    // Validate user-defined types
    pub fn validate_user_defined_types(&self) {
        for type_defn in &self.type_decls {
            type_defn.check_tyvars();
            let type_name = &type_defn.name;
            match &type_defn.value {
                TypeDeclValue::Struct(str) => match Field::check_duplication(&str.fields) {
                    Some(field_name) => {
                        error_exit(&format!(
                            "duplicate field `{}` for struct `{}`",
                            field_name,
                            type_name.to_string()
                        ));
                    }
                    _ => {}
                },
                TypeDeclValue::Union(union) => match Field::check_duplication(&union.fields) {
                    Some(field_name) => {
                        error_exit(&format!(
                            "duplicate field `{}` for union `{}`",
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

    // Add built-in types and traits
    pub fn add_builtin_traits_types(&mut self) {
        self.trait_env.add_trait(eq_trait());
        self.trait_env.add_trait(add_trait());
        self.trait_env.add_trait(subtract_trait());
        self.trait_env.add_trait(negate_trait());
        self.trait_env.add_trait(not_trait());
        self.trait_env.add_trait(multiply_trait());
        self.trait_env.add_trait(divide_trait());
        self.trait_env.add_trait(remainder_trait());
        self.trait_env.add_trait(and_trait());
        self.trait_env.add_trait(or_trait());
        self.trait_env.add_trait(less_than_trait());
        self.trait_env.add_trait(less_than_or_equal_to_trait());
        self.type_decls.push(loop_result_defn());
        for i in 0..=TUPLE_SIZE_MAX {
            if i != 1 {
                self.type_decls.push(tuple_defn(i));
            }
        }
    }

    // Add bult-in functions to a given ast.
    pub fn add_builtin_symbols(self: &mut FixModule) {
        fn add_global(
            program: &mut FixModule,
            name: FullName,
            (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
        ) {
            program.add_global_object(name, (expr, scm));
        }
        self.trait_env
            .add_instance(eq_trait_instance_primitive(int_lit_ty()));
        self.trait_env
            .add_instance(eq_trait_instance_primitive(bool_lit_ty()));
        self.trait_env.add_instance(add_trait_instance_int());
        self.trait_env.add_instance(subtract_trait_instance_int());
        self.trait_env.add_instance(negate_trait_instance_int());
        self.trait_env.add_instance(not_trait_instance_bool());
        self.trait_env.add_instance(multiply_trait_instance_int());
        self.trait_env.add_instance(divide_trait_instance_int());
        self.trait_env.add_instance(remainder_trait_instance_int());
        self.trait_env.add_instance(and_trait_instance_bool());
        self.trait_env.add_instance(or_trait_instance_bool());
        self.trait_env.add_instance(less_than_trait_instance_int());
        self.trait_env
            .add_instance(less_than_or_equal_to_trait_instance_int());
        add_global(self, FullName::from_strs(&[STD_NAME], FIX_NAME), fix());
        add_global(
            self,
            FullName::from_strs(&[STD_NAME, LOOP_RESULT_NAME], "loop"),
            state_loop(),
        );
        add_global(
            self,
            FullName::from_strs(&[STD_NAME, ARRAY_NAME], "new"),
            new_array(),
        );
        add_global(
            self,
            FullName::from_strs(&[STD_NAME, ARRAY_NAME], "from_map"),
            from_map_array(),
        );
        add_global(
            self,
            FullName::from_strs(&[STD_NAME, ARRAY_NAME], "get"),
            read_array(),
        );
        add_global(
            self,
            FullName::from_strs(&[STD_NAME, ARRAY_NAME], "set"),
            write_array(),
        );
        add_global(
            self,
            FullName::from_strs(&[STD_NAME, ARRAY_NAME], "set!"),
            write_array_unique(),
        );
        add_global(
            self,
            FullName::from_strs(&[STD_NAME, ARRAY_NAME], "len"),
            length_array(),
        );
        for decl in &self.type_decls.clone() {
            match &decl.value {
                TypeDeclValue::Struct(str) => {
                    let struct_name = decl.name.clone();
                    add_global(
                        self,
                        FullName::new(&decl.name.to_namespace(), STRUCT_NEW_NAME),
                        struct_new(&struct_name, decl),
                    );
                    for field in &str.fields {
                        add_global(
                            self,
                            FullName::new(
                                &decl.name.to_namespace(),
                                &format!("{}_{}", STRUCT_GETTER_NAME, &field.name),
                            ),
                            struct_get(&struct_name, decl, &field.name),
                        );
                        for is_unique in [false, true] {
                            add_global(
                                self,
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
                        }
                    }
                }
                TypeDeclValue::Union(union) => {
                    let union_name = &decl.name;
                    for field in &union.fields {
                        add_global(
                            self,
                            FullName::new(&decl.name.to_namespace(), &field.name),
                            union_new(&union_name, &field.name, decl),
                        );
                        add_global(
                            self,
                            FullName::new(&decl.name.to_namespace(), &format!("as_{}", field.name)),
                            union_as(&union_name, &field.name, decl),
                        );
                        add_global(
                            self,
                            FullName::new(&decl.name.to_namespace(), &format!("is_{}", field.name)),
                            union_is(&union_name, &field.name, decl),
                        );
                    }
                }
            }
        }
    }
}

pub const FIX_NAME: &str = "fix";
pub const STRUCT_GETTER_NAME: &str = "get";
pub const STRUCT_NEW_NAME: &str = "new";
