use std::sync::Arc;

use misc::Set;
use name::{FullName, GlobalRelativeNames, Name};

use crate::ast::kind_scope::KindScope;
use crate::error::Errors;
use crate::name_resolution::NameResolutionContext;

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
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        if self.source.is_none() {
            return None;
        }
        let span = self.source.as_ref().unwrap();
        if !span.includes_pos(pos) {
            return None;
        }
        self.value.find_node_at(pos)
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
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

    // Return TypeNode defined by this type definition.
    // If the definition is higher kinded, it returns a fully applied type (i.e., returns a type of kind `*`).
    pub fn applied_type(&self) -> Arc<TypeNode> {
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

    pub fn validate_tyvars(&self) -> Result<(), Errors> {
        // Check if type variables are not duplicated.
        let mut names = Set::<String>::default();
        for tv in &self.tyvars {
            if names.contains(&tv.name) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Type variable `{}` is duplicated in the definition of type `{}`.",
                        tv.name,
                        self.name.to_string()
                    ),
                    &[&self.source.as_ref().map(|s| s.to_head_character())],
                ));
            } else {
                names.insert(tv.name.clone());
            }
        }

        // Check if all of type variables in field types appear in the left hand side of type definition.
        let tyvars = Set::<String>::from_iter(self.tyvars.iter().map(|tv| tv.name.clone()));
        for v in self.free_variables_in_definition() {
            if !tyvars.contains(&v.name) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Unknown type variable `{}` in the definition of type `{}`.",
                        v.name,
                        self.name.to_string()
                    ),
                    &[&self.source.as_ref().map(|s| s.to_head_character())],
                ));
            }
        }
        Ok(())
    }

    // Set kinds to type variables in `self.value` using kind information in `self.tyvars`.
    pub fn set_kinds_in_value(&mut self) -> Result<(), Errors> {
        let mut kind_scope = KindScope::default();
        for tv in &self.tyvars {
            kind_scope
                .insert(tv.name.clone(), tv.kind.clone())
                .map_err(|e| {
                    Errors::from_msg_srcs(
                        e,
                        &[&self.source.as_ref().map(|s| s.to_head_character())],
                    )
                })?;
        }
        self.value.set_kinds(&kind_scope);
        Ok(())
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
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        match self {
            TypeDeclValue::Struct(s) => s.find_node_at(pos),
            TypeDeclValue::Union(u) => u.find_node_at(pos),
            TypeDeclValue::Alias(a) => a.find_node_at(pos),
        }
    }

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

    pub fn set_kinds(&mut self, kinds: &KindScope) {
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
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        for f in &self.fields {
            if let Some(node) = f.find_node_at(pos) {
                return Some(node);
            }
        }
        None
    }

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

    pub fn set_kinds(&mut self, kinds: &KindScope) {
        for f in &mut self.fields {
            f.set_kinds(kinds);
        }
    }

    pub fn is_boxed(&self) -> bool {
        !self.is_unbox
    }
}

#[derive(Clone)]
pub struct Union {
    pub fields: Vec<Field>,
    pub is_unbox: bool,
}

impl Union {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        for f in &self.fields {
            if let Some(node) = f.find_node_at(pos) {
                return Some(node);
            }
        }
        None
    }

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

    pub fn set_kinds(&mut self, kinds: &KindScope) {
        for f in &mut self.fields {
            f.set_kinds(kinds);
        }
    }

    pub fn is_boxed(&self) -> bool {
        !self.is_unbox
    }
}

#[derive(Clone)]
pub struct TypeAlias {
    pub value: Arc<TypeNode>,
}

impl TypeAlias {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        self.value.find_node_at(pos)
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.value = self.value.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn set_kinds(&mut self, scope: &KindScope) {
        self.value = self.value.set_kinds(scope);
    }
}

#[derive(Clone)]
pub struct Field {
    pub name: Name,
    // Type of the field.
    //
    // This field holds the type after type alias resolution.
    pub ty: Arc<TypeNode>,
    // Syntactic type of the field.
    pub syn_ty: Arc<TypeNode>,
    pub is_punched: bool,
    pub source: Option<Span>,
}

impl Field {
    pub fn make(name: Name, syn_ty: Arc<TypeNode>, source: Option<Span>) -> Self {
        Field {
            name,
            ty: syn_ty.clone(),
            syn_ty,
            is_punched: false,
            source,
        }
    }

    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        self.ty.find_node_at(pos)
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.syn_ty = self.syn_ty.resolve_namespace(ctx)?;
        self.ty = self.ty.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.ty = self.ty.resolve_type_aliases(type_env)?;
        Ok(())
    }

    // Collect names that should be imported.
    pub fn collect_import_names(&self, names: &mut GlobalRelativeNames) {
        // Collect from the syntactic type.
        self.syn_ty.collect_import_names(names);
    }

    // Check if fields are duplicated. If duplication is found, it returns the duplicated field.
    pub fn check_duplication(fields: &Vec<Field>) -> Option<Name> {
        let mut names: Set<Name> = Default::default();
        for field in fields {
            if names.contains(&field.name) {
                return Some(field.name.clone());
            } else {
                names.insert(field.name.clone());
            }
        }
        return None;
    }

    pub fn set_kinds(&mut self, kinds: &KindScope) {
        self.ty = self.ty.set_kinds(kinds);
    }
}
