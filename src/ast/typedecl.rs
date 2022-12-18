use super::*;

// Declaration of user-defind types.
#[derive(Clone)]
pub struct TypeDecl {
    pub name: NameSpacedName,
    pub value: TypeDeclValue,
    pub tyvars: Vec<Name>,
}

impl TypeDecl {
    pub fn tycon(&self) -> TyCon {
        TyCon::new(self.name.clone())
    }

    pub fn kind(&self) -> Arc<Kind> {
        let mut kind = kind_star();
        for _ in &self.tyvars {
            kind = kind_arrow(kind_star(), kind);
        }
        kind
    }

    pub fn ty(&self) -> Arc<TypeNode> {
        let mut ty = type_tycon(&Arc::new(self.tycon()));
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

    // Check if all of type variables in field types appear in lhs of type definition.
    pub fn check_tyvars(&self) {
        let tyvars = HashSet::<String>::from_iter(self.tyvars.iter().map(|s| s.clone()));
        for field in self.fields() {
            let free_vars = field.ty.free_vars();
            for (v, _) in &free_vars {
                if !tyvars.contains(v) {
                    error_exit(&format!(
                        "unknown type variable `{}` in the definition of field `{}` of type `{}`",
                        v,
                        field.name,
                        self.name.to_string()
                    ))
                }
            }
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
    // Check if fields are duplicated. If duplication found, it returns the duplicated field.
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
