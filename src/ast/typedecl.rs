use super::*;

// Declaration of user-defind types.
#[derive(Clone)]
pub struct TypeDecl {
    pub name: String,
    pub value: TypeDeclValue,
}

// Right hand side of type declaration.
#[derive(Clone)]
pub enum TypeDeclValue {
    Struct(Struct),
}

#[derive(Clone)]
pub struct Struct {
    pub fields: Vec<StructField>,
}

#[derive(Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Arc<TypeNode>,
}
