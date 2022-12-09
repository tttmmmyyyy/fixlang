use inkwell::module::Linkage;

use super::*;

// Module of fix-lang.
// Avoiding name confliction with "Module" of inkwell.

const MAIN_FUNCTION_NAME: &str = "main";

pub struct FixModule {
    pub name: Name,
    pub type_decls: Vec<TypeDecl>,
    pub global_symbols: HashMap<NameSpacedName, GlobalSymbol>,
}

pub struct GlobalSymbol {
    pub ty: Arc<Scheme>,
    pub expr: Arc<ExprNode>,
    // TODO: add ty_src: Span
    // TODO: add expr_src: Span
}

impl FixModule {
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

    // Add a global object.
    pub fn add_global_object(
        &mut self,
        name: NameSpacedName,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
    ) {
        if self.global_symbols.contains_key(&name) {
            error_exit(&format!("duplicated global object: `{}`", name.to_string()));
        }
        self.global_symbols
            .insert(name, GlobalSymbol { ty: scm, expr });
    }

    // Get main function of this module.
    pub fn main_function(&self) -> Arc<ExprNode> {
        match self
            .global_symbols
            .get(&self.get_namespaced_name(&MAIN_FUNCTION_NAME.to_string()))
        {
            Some(gs) => {
                // TODO: Here check if gs has required type.
                expr_var(MAIN_FUNCTION_NAME, Some(self.get_namespace()), None)
            }
            None => error_exit("main function not found."),
        }
    }

    // Generate codes of global objects.
    pub fn generate_code(&self, gc: &mut GenerationContext) {
        // Create global objects, global variable and accessor function.
        let global_objs = self
            .global_symbols
            .iter()
            .map(|(name, defn)| {
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
                (global_var, acc_fn, defn.clone())
            })
            .collect::<Vec<_>>();

        // Implement global accessor function.
        for (global_var, acc_fn, defn) in global_objs {
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
            let obj = gc.eval_expr(defn.expr.clone());
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
}
