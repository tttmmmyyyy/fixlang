use std::sync::Arc;

use crate::error::{error_exit_with_src, Errors};

use super::*;

// Declaration of user-defind types.
#[derive(Clone)]
pub struct TypeDefn {
    pub name: FullName,
    pub value: TypeDeclValue,
    pub tyvars: Vec<Arc<TyVar>>,
    pub source: Option<Span>,
}

impl TypeDefn {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        assert!(
            self.name
                == ctx
                    .resolve(&self.name, &[NameResolutionType::TyCon], &None)
                    .ok()
                    .unwrap()
        );
        self.value.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.value.resolve_type_aliases(type_env)?;
        Ok(())
    }

    pub fn tycon(&self) -> TyCon {
        TyCon::new(self.name.clone())
    }

    pub fn tycon_info(&self, punched_struct_fields: &[usize]) -> TyConInfo {
        let kind = self.kind();
        let (variant, is_unbox, fields) = match &self.value {
            TypeDeclValue::Struct(s) => {
                let mut fields = s.fields.clone();
                for i in punched_struct_fields {
                    fields[*i].is_punched = true;
                }
                (TyConVariant::Struct, s.is_unbox, fields)
            }
            TypeDeclValue::Union(u) => {
                assert!(punched_struct_fields.is_empty());
                (TyConVariant::Union, u.is_unbox, u.fields.clone())
            }
            TypeDeclValue::Alias(_) => panic!("Try to get TyConInfo of a type alias."),
        };
        TyConInfo {
            kind,
            variant,
            is_unbox,
            tyvars: self.tyvars.clone(),
            fields,
            source: self.source.clone(),
            document: None,
        }
    }

    // Calculate kind of tycon defined by this type definition.
    pub fn kind(&self) -> Arc<Kind> {
        let mut kind = kind_star();
        for tv in self.tyvars.iter().rev() {
            kind = kind_arrow(tv.kind.clone(), kind);
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

    pub fn ty(&self) -> Arc<TypeNode> {
        let mut ty = type_tycon(&Arc::new(self.tycon()));
        for tv in &self.tyvars {
            ty = type_tyapp(ty, type_from_tyvar(tv.clone()));
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
    pub fn free_variables_in_definition(&self) -> Vec<Arc<TyVar>> {
        let mut ret = vec![];
        if self.is_alias() {
            match &self.value {
                TypeDeclValue::Alias(ta) => ta.value.free_vars_to_vec(&mut ret),
                _ => unreachable!(),
            }
        } else {
            for field in self.fields() {
                field.ty.free_vars_to_vec(&mut ret);
            }
        }
        ret
    }

    // Check if all of type variables in field types appear in lhs of type definition.
    pub fn check_tyvars(&self) {
        let tyvars = HashSet::<String>::from_iter(self.tyvars.iter().map(|tv| tv.name.clone()));
        for v in self.free_variables_in_definition() {
            if !tyvars.contains(&v.name) {
                error_exit_with_src(
                    &format!(
                        "Unknown type variable `{}` in the definition of type `{}`.",
                        v.name,
                        self.name.to_string()
                    ),
                    &self.source.as_ref().map(|s| s.to_head_character()),
                )
            }
        }
    }

    // Set kinds to type variables in `self.value` using kind information in `self.tyvars`.
    pub fn set_kinds_in_value(&mut self) {
        let kind_scope: HashMap<_, _> = self
            .tyvars
            .iter()
            .map(|tv| (tv.name.clone(), tv.kind.clone()))
            .collect();
        self.value.set_kinds(&kind_scope);
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
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        match self {
            TypeDeclValue::Struct(s) => s.resolve_namespace(ctx),
            TypeDeclValue::Union(u) => u.resolve_namespace(ctx),
            TypeDeclValue::Alias(a) => a.resolve_namespace(ctx),
        }
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        match self {
            TypeDeclValue::Struct(s) => s.resolve_type_aliases(type_env),
            TypeDeclValue::Union(u) => u.resolve_type_aliases(type_env),
            TypeDeclValue::Alias(_) => Ok(()), // Nothing to do.
        }
    }

    pub fn is_alias(&self) -> bool {
        match self {
            TypeDeclValue::Alias(_) => true,
            _ => false,
        }
    }

    pub fn set_kinds(&mut self, kinds: &HashMap<Name, Arc<Kind>>) {
        match self {
            TypeDeclValue::Struct(s) => s.set_kinds(kinds),
            TypeDeclValue::Union(u) => u.set_kinds(kinds),
            TypeDeclValue::Alias(a) => a.set_kinds(kinds),
        }
    }
}

#[derive(Clone)]
pub struct Struct {
    pub fields: Vec<Field>,
    pub is_unbox: bool,
}

impl Struct {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        for f in &mut self.fields {
            f.resolve_namespace(ctx)?;
        }
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        for f in &mut self.fields {
            f.resolve_type_aliases(type_env)?;
        }
        Ok(())
    }

    pub fn set_kinds(&mut self, kinds: &HashMap<Name, Arc<Kind>>) {
        for f in &mut self.fields {
            f.set_kinds(kinds);
        }
    }
}

#[derive(Clone)]
pub struct Union {
    pub fields: Vec<Field>,
    pub is_unbox: bool,
}

impl Union {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        for f in &mut self.fields {
            f.resolve_namespace(ctx)?;
        }
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        for f in &mut self.fields {
            f.resolve_type_aliases(type_env)?;
        }
        Ok(())
    }

    pub fn set_kinds(&mut self, kinds: &HashMap<Name, Arc<Kind>>) {
        for f in &mut self.fields {
            f.set_kinds(kinds);
        }
    }
}

#[derive(Clone)]
pub struct TypeAlias {
    pub value: Arc<TypeNode>,
}

impl TypeAlias {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.value = self.value.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn set_kinds(&mut self, scope: &HashMap<Name, Arc<Kind>>) {
        self.value = self.value.set_kinds(scope);
    }
}

#[derive(Clone)]
pub struct Field {
    pub name: Name,
    pub ty: Arc<TypeNode>,
    pub is_punched: bool,
}

impl Field {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.ty = self.ty.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.ty = self.ty.resolve_type_aliases(type_env)?;
        Ok(())
    }

    // Check if fields are duplicated. If duplication is found, it returns the duplicated field.
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

    pub fn set_kinds(&mut self, kinds: &HashMap<Name, Arc<Kind>>) {
        self.ty = self.ty.set_kinds(kinds);
    }
}
