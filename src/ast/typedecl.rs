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
    Union(Union),
}

#[derive(Clone)]
pub struct Struct {
    pub fields: Vec<Field>,
}

#[derive(Clone)]
pub struct Union {
    pub fields: Vec<Field>,
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub ty: Arc<TypeNode>,
}
