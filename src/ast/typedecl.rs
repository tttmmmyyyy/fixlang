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

impl Field {
    pub fn check_duplication(fields: &Vec<Field>) -> Option<Name> {
        let mut names: HashSet<Name> = Default::default();
        for field in fields {
            if names.contains(&field.name) {
                return Some(field.name.clone());
            } else {
                names.insert(field.name.clone());
            }
        }
        return None;
    }
}
