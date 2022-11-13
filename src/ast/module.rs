use super::*;

// Module of fix-lang.
// avoiding confliction with Module of inkwell.

pub struct FixModule {
    pub name: String,
    pub type_decls: Vec<TypeDecl>,
    pub expr: Arc<ExprNode>,
}
