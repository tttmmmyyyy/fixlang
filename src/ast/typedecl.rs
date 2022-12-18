use super::*;

// Declaration of user-defind types.
#[derive(Clone)]
pub struct TypeDecl {
    pub name: Name,
    pub value: TypeDeclValue,
    pub tyvars: Vec<Name>,
}

impl TypeDecl {
    pub fn tycon(&self, namespace: &NameSpace) -> TyCon {
        TyCon::new(NameSpacedName::new(namespace, &self.name))
    }

    pub fn kind(&self) -> Arc<Kind> {
        let mut kind = kind_star();
        for _ in &self.tyvars {
            kind = kind_arrow(kind_star(), kind);
        }
        kind
    }

    pub fn ty(&self, namespace: &NameSpace) -> Arc<TypeNode> {
        let mut ty = type_tycon(&tycon(NameSpacedName::new(namespace, &self.name)));
        for tyvar in &self.tyvars {
            ty = type_tyapp(ty, type_tyvar(tyvar, &kind_star()));
        }
        ty
    }

    pub fn fields(&self) -> &Vec<Field> {
        match self.value {
            TypeDeclValue::Struct(ref s) => &s.fields,
            TypeDeclValue::Union(ref u) => &u.fields,
        }
    }
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
