use super::*;

// Declaration of user-defind types.
pub struct TypeDecl {
    pub name: String,
    pub value: TypeDeclValue,
}

// Right hand side of type declaration.
pub enum TypeDeclValue {
    Struct(Vec<StructField>),
}

pub struct StructField {
    pub name: String,
    pub ty: Arc<TypeNode>,
}
