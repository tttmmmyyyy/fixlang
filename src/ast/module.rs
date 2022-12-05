use super::*;

// Module of fix-lang.
// Avoiding name confliction with "Module" of inkwell.

pub struct FixModule {
    pub name: Name,
    pub type_decls: Vec<TypeDecl>,
    pub global_symbol: HashMap<NameSpacedName, GlobalSymbol>,
    pub expr: Arc<ExprNode>,
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
        if self.global_symbol.contains_key(&name) {
            error_exit(&format!("duplicated global object: `{}`", name.to_string()));
        }
        self.global_symbol
            .insert(name, GlobalSymbol { ty: scm, expr });
    }
}
