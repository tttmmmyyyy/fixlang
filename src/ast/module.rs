use super::*;

// Module of fix-lang.
// Avoiding name confliction with "Module" of inkwell.

pub struct FixModule {
    pub name: Name,
    pub type_decls: Vec<TypeDecl>,
    pub global_symbol: HashMap<String, GlobalSymbol>,
    pub expr: Arc<ExprNode>,
}

pub struct GlobalSymbol {
    pub ty: Arc<Scheme>,
    pub expr: Arc<ExprNode>,
    // TODO: add ty_src: Span
    // TODO: add expr_src: Span
}

impl FixModule {
    // Get name as namespace.
    pub fn get_namespace(&self) -> NameSpace {
        NameSpace::new(vec![self.name.clone()])
    }
}
