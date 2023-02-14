use super::*;

// Declaration of user-defind types.
#[derive(Clone)]
pub struct TypeDefn {
    pub name: FullName,
    pub value: TypeDeclValue,
    pub tyvars: Vec<Name>,
}

impl TypeDefn {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        assert!(self.name == ctx.resolve(&self.name, NameResolutionType::Type));
        self.value.resolve_namespace(ctx);
    }

    pub fn tycon(&self) -> TyCon {
        TyCon::new(self.name.clone())
    }

    pub fn tycon_info(&self) -> TyConInfo {
        let mut kind = kind_star();
        for _ in &self.tyvars {
            kind = kind_arrow(kind_star(), kind);
        }
        let (variant, is_unbox, fields) = match &self.value {
            TypeDeclValue::Struct(s) => (TyConVariant::Struct, s.is_unbox, s.fields.clone()),
            TypeDeclValue::Union(u) => (TyConVariant::Union, u.is_unbox, u.fields.clone()),
        };
        TyConInfo {
            kind,
            variant,
            is_unbox,
            tyvars: self.tyvars.clone(),
            fields,
        }
    }

    pub fn ty(&self) -> Rc<TypeNode> {
        let mut ty = type_tycon(&Rc::new(self.tycon()));
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

    // Find the index of `field_name` in the given struct.
    pub fn get_field_by_name(&self, field_name: &str) -> Option<(u32, Field)> {
        self.fields()
            .iter()
            .enumerate()
            .find(|(_i, f)| f.name == field_name)
            .map(|(i, f)| (i as u32, f.clone()))
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

impl TypeDeclValue {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        match self {
            TypeDeclValue::Struct(s) => s.resolve_namespace(ctx),
            TypeDeclValue::Union(u) => u.resolve_namespace(ctx),
        }
    }
}

#[derive(Clone)]
pub struct Struct {
    pub fields: Vec<Field>,
    pub is_unbox: bool,
}

impl Struct {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for f in &mut self.fields {
            f.resolve_namespace(ctx);
        }
    }
}

#[derive(Clone)]
pub struct Union {
    pub fields: Vec<Field>,
    pub is_unbox: bool,
}

impl Union {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for f in &mut self.fields {
            f.resolve_namespace(ctx);
        }
    }
}

#[derive(Clone)]
pub struct Field {
    pub name: Name,
    pub ty: Rc<TypeNode>,
}

impl Field {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        self.ty = self.ty.resolve_namespace(ctx);
    }

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
