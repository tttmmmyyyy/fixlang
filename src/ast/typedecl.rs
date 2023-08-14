use super::*;

// Declaration of user-defind types.
#[derive(Clone)]
pub struct TypeDefn {
    pub name: FullName,
    pub value: TypeDeclValue,
    pub tyvars: Vec<Name>,
    pub source: Option<Span>,
}

impl TypeDefn {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        assert!(self.name == ctx.resolve(&self.name, NameResolutionType::Type).unwrap());
        self.value.resolve_namespace(ctx);
    }

    pub fn tycon(&self) -> TyCon {
        TyCon::new(self.name.clone())
    }

    pub fn tycon_info(&self) -> TyConInfo {
        let kind = self.kind();
        let (variant, is_unbox, fields) = match &self.value {
            TypeDeclValue::Struct(s) => (TyConVariant::Struct, s.is_unbox, s.fields.clone()),
            TypeDeclValue::Union(u) => (TyConVariant::Union, u.is_unbox, u.fields.clone()),
            TypeDeclValue::Alias(_) => panic!("Try to get TyConInfo of a type alias."),
        };
        TyConInfo {
            kind,
            variant,
            is_unbox,
            tyvars: self.tyvars.clone(),
            fields,
            source: self.source.clone(),
        }
    }

    // Calculate kind of tycon defined by this type definition.
    // NOTE: Currently, all type variables appear in type definition have kind "*"".
    pub fn kind(&self) -> Rc<Kind> {
        let mut kind = kind_star();
        for _ in &self.tyvars {
            kind = kind_arrow(kind_star(), kind);
        }
        kind
    }

    pub fn alias_info(&self) -> TyAliasInfo {
        let kind = self.kind();
        let value = match &self.value {
            TypeDeclValue::Alias(a) => a.value.clone(),
            TypeDeclValue::Struct(_) => panic!("Try to get TyAliasInfo of a struct."),
            TypeDeclValue::Union(_) => panic!("Try to get TyAliasInfo of an union."),
        };
        TyAliasInfo {
            kind,
            value,
            tyvars: self.tyvars.clone(),
            source: self.source.clone(),
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
            TypeDeclValue::Alias(_) => panic!("Try to get fields of a type alias."),
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

    // Get free type variables that appear in the right hand side of type definition.
    pub fn free_variables_in_definition(&self) -> Vec<Name> {
        let mut ret = vec![];
        if self.is_alias() {
            match &self.value {
                TypeDeclValue::Alias(ta) => ta.value.free_vars_vec(&mut ret),
                _ => unreachable!(),
            }
        } else {
            for field in self.fields() {
                field.ty.free_vars_vec(&mut ret);
            }
        }
        ret
    }

    // Check if all of type variables in field types appear in lhs of type definition.
    pub fn check_tyvars(&self) {
        let tyvars = HashSet::<String>::from_iter(self.tyvars.iter().map(|s| s.clone()));
        for v in self.free_variables_in_definition() {
            if !tyvars.contains(&v) {
                error_exit_with_src(
                    &format!(
                        "Unknown type variable `{}` in the definition of type `{}`",
                        v,
                        self.name.to_string()
                    ),
                    &self.source.as_ref().map(|s| s.to_single_character()),
                )
            }
        }
    }

    pub fn is_alias(&self) -> bool {
        self.value.is_alias()
    }
}

// Right hand side of type declaration.
#[derive(Clone)]
pub enum TypeDeclValue {
    Struct(Struct),
    Union(Union),
    Alias(TypeAlias),
}

impl TypeDeclValue {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        match self {
            TypeDeclValue::Struct(s) => s.resolve_namespace(ctx),
            TypeDeclValue::Union(u) => u.resolve_namespace(ctx),
            TypeDeclValue::Alias(a) => a.resolve_namespace(ctx),
        }
    }

    pub fn is_alias(&self) -> bool {
        match self {
            TypeDeclValue::Alias(_) => true,
            _ => false,
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
pub struct TypeAlias {
    pub value: Rc<TypeNode>,
}

impl TypeAlias {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        self.value = self.value.resolve_namespace(ctx);
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
