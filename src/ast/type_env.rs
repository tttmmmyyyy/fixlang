use crate::ast::name::{FullName, Name};
use crate::ast::types::{Kind, TyAliasInfo, TyCon, TyConInfo, TyConVariant};
use crate::constants::STRUCT_ACT_SYMBOL;
use crate::elaboration::name_resolution::NameResolutionContext;
use crate::error::Errors;
use crate::misc::{Map, Set};
use crate::rc_ir::leaf_map::BoxedLeafPathsMemo;
use std::sync::Arc;

/// What a program declares about its types: the type constructors and the type aliases it can name,
/// and which of the newtypes among them a value has stopped being built at.
#[derive(Clone)]
pub struct TypeEnv {
    /// The declaration of every type constructor, built-in and user-defined, by its name.
    ///
    /// Private, because the field types held here answer what a value of a type is laid out as, and
    /// `unwrap_newtypes` puts them in a form the rest of the compiler relies on. A declaration
    /// enters through `add_tycons`, which puts it in that same form.
    ///
    /// Replaced through `set_tycons` alone, which keeps `boxed_leaf_paths_memo` in step with it.
    /// This module holds nothing but `TypeEnv`, so the field's privacy is what enforces that.
    tycons: Arc<Map<TyCon, TyConInfo>>,
    /// The declaration of every type alias, by its name.
    pub aliases: Arc<Map<TyCon, TyAliasInfo>>,
    /// The newtypes a value of which has become a value of its one field. Empty until the pass that
    /// unwraps newtypes runs.
    ///
    /// A field type this environment reports has none of these saturated in it, so a value is never
    /// built at the struct one of them was declared as. The declarations stay, because a newtype
    /// carried by a higher-kinded type variable occurs without its arguments, and such an occurrence
    /// still names one.
    ///
    /// Replaced through `set_unwrapped_newtypes` alone, which keeps `boxed_leaf_paths_memo` in step
    /// with it.
    unwrapped_newtypes: Arc<Set<TyCon>>,
    /// The boxed leaf paths `boxed_leaf_paths` has worked out under this environment.
    ///
    /// The paths follow from `tycons` and `unwrapped_newtypes`, and the setter of each puts an empty
    /// memo here. The clones of an environment share the memo until one of them replaces either
    /// field.
    boxed_leaf_paths_memo: Arc<BoxedLeafPathsMemo>,
}

impl Default for TypeEnv {
    /// An environment in which no type constructor and no type alias is declared.
    fn default() -> Self {
        Self {
            tycons: Arc::new(Default::default()),
            aliases: Arc::new(Default::default()),
            unwrapped_newtypes: Arc::new(Default::default()),
            boxed_leaf_paths_memo: Arc::new(Default::default()),
        }
    }
}

impl TypeEnv {
    /// An environment holding `tycons` and `aliases` as declared, with every newtype among them
    /// still a type values are built at.
    pub fn new(tycons: Map<TyCon, TyConInfo>, aliases: Map<TyCon, TyAliasInfo>) -> TypeEnv {
        TypeEnv {
            tycons: Arc::new(tycons),
            aliases: Arc::new(aliases),
            unwrapped_newtypes: Arc::new(Default::default()),
            boxed_leaf_paths_memo: Arc::new(Default::default()),
        }
    }

    /// Replaces the declarations of the type constructors with `tycons`, and forgets what was worked
    /// out from the old ones.
    fn set_tycons(&mut self, tycons: Map<TyCon, TyConInfo>) {
        self.tycons = Arc::new(tycons);
        self.boxed_leaf_paths_memo = Arc::new(Default::default());
    }

    /// Replaces the set of unwrapped newtypes with `newtypes`, and forgets what was worked out under
    /// the old one.
    fn set_unwrapped_newtypes(&mut self, newtypes: Set<TyCon>) {
        self.unwrapped_newtypes = Arc::new(newtypes);
        self.boxed_leaf_paths_memo = Arc::new(Default::default());
    }

    /// The boxed leaf paths `boxed_leaf_paths` has worked out under this environment.
    pub fn boxed_leaf_paths_memo(&self) -> &BoxedLeafPathsMemo {
        &self.boxed_leaf_paths_memo
    }

    /// Makes a value of each newtype in `newtypes` a value of its one field: records them, then
    /// rewrites the stored declarations so that no field type this environment reports has one of
    /// them saturated in it.
    ///
    /// The declarations are read as they stand while they are rewritten, so each one is unwrapped
    /// from the same starting point.
    ///
    /// Every newtype recorded is one this environment declares, which is what lets
    /// `unwrapped_newtype_info` answer with a declaration rather than with the possibility of one.
    pub fn unwrap_newtypes(&mut self, newtypes: Set<TyCon>) {
        for tycon in &newtypes {
            assert!(
                self.tycons.contains_key(tycon),
                "`{}` is unwrapped, though this environment holds no declaration of it.",
                tycon.to_string()
            );
        }
        self.set_unwrapped_newtypes(newtypes);
        let declared_type_env = self.clone();
        let mut rewritten = self.tycons.as_ref().clone();
        for (_tycon, tycon_info) in &mut rewritten {
            for field in &mut tycon_info.fields {
                field.ty = field.ty.unwrap_newtypes(&declared_type_env);
            }
        }
        self.set_tycons(rewritten);
    }

    /// The declaration of `tycon` if a value of it has become a value of its one field, and `None`
    /// otherwise. `unwrap_newtypes` asserts that every newtype it records is declared here.
    pub fn unwrapped_newtype_info(&self, tycon: &TyCon) -> Option<&TyConInfo> {
        if !self.unwrapped_newtypes.contains(tycon) {
            return None;
        }
        Some(self.tycon_info(tycon))
    }

    /// Whether a value of `tycon` has become a value of its one field.
    pub fn is_unwrapped_newtype(&self, tycon: &TyCon) -> bool {
        self.unwrapped_newtypes.contains(tycon)
    }

    /// Adds each declaration of `new_tycons` to this environment, replacing the one already held
    /// under the same name. The field types of each are unwrapped as `unwrap_newtypes` unwraps those
    /// already held, so a declaration added after that pass reads the same as one that was there
    /// before it.
    pub fn add_tycons(&mut self, new_tycons: Map<TyCon, TyConInfo>) {
        let declared_type_env = self.clone();
        let mut tycons = self.tycons.as_ref().clone();
        for (tycon, mut tycon_info) in new_tycons.into_iter() {
            for field in &mut tycon_info.fields {
                field.ty = field.ty.unwrap_newtypes(&declared_type_env);
            }
            tycons.insert(tycon, tycon_info);
        }
        self.set_tycons(tycons);
    }

    /// The declaration of every type constructor this environment holds, by its name.
    pub fn tycons(&self) -> &Map<TyCon, TyConInfo> {
        &self.tycons
    }

    /// The `TyConInfo` this environment holds for `tycon`. Panics, naming `tycon`, if it holds none.
    pub fn tycon_info(&self, tycon: &TyCon) -> &TyConInfo {
        self.tycons
            .get(tycon)
            .unwrap_or_else(|| panic!("the type constructor `{}` is undeclared", tycon.to_string()))
    }

    /// The kind of every name this environment gives a meaning to, type constructors and type
    /// aliases together in one table.
    pub fn kinds(&self) -> Map<TyCon, Arc<Kind>> {
        let mut res = Map::default();
        for (tc, ti) in self.tycons.as_ref().iter() {
            res.insert(tc.clone(), ti.kind.clone());
        }
        for (tc, ta) in self.aliases.as_ref().iter() {
            res.insert(tc.clone(), ta.kind.clone());
        }
        res
    }

    /// The struct and the field that `name` is the `act_{field}` function of: `name` is a global
    /// name whose namespace is a struct this environment declares, and whose last component names
    /// one of that struct's fields.
    ///
    /// The answer comes from the name alone, so ask it while a value of the struct is still built as
    /// that struct. A newtype keeps its declaration after `unwrap_newtypes` records it, so this
    /// still names the struct of a newtype whose values have become values of its one field.
    pub fn is_struct_act(&self, name: &FullName) -> Option<(TyCon, Name)> {
        if name.is_local() {
            return None;
        }
        let struct_name = name.namespace.clone().to_fullname();
        let struct_tycon = TyCon { name: struct_name };
        match self.tycons.get(&struct_tycon) {
            Some(tycon_info) => {
                if tycon_info.variant != TyConVariant::Struct {
                    return None;
                }
                for field in &tycon_info.fields {
                    let act_func_name = format!("{}{}", STRUCT_ACT_SYMBOL, field.name);
                    if act_func_name == name.name {
                        return Some((struct_tycon, field.name.clone()));
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Resolves every type name written in the definitions of the type constructors and the type
    /// aliases of this environment to its full name, in the module that declares each definition.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        let mut tycons = (*self.tycons).clone();
        for (tc, ti) in &mut tycons {
            ctx.set_current_module(tc.name.module());
            ti.resolve_namespace(ctx)?;
        }
        self.set_tycons(tycons);
        let mut aliases = (*self.aliases).clone();
        for (tc, ta) in &mut aliases {
            ctx.set_current_module(tc.name.module());
            ta.resolve_namespace(ctx)?;
        }
        self.aliases = Arc::new(aliases);
        Ok(())
    }

    /// Replace every type alias written in the definition of a type constructor of this environment
    /// by the type it stands for, so that a stage reading a field or variant type meets no alias.
    pub fn resolve_type_aliases_in_tycons(&mut self) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        let type_env = self.clone();
        let mut tycons = (*self.tycons).clone();
        for (_, ti) in &mut tycons {
            errors.eat_err(ti.resolve_type_aliases(&type_env));
        }
        errors.to_result()?; // Throw errors if any.
        self.set_tycons(tycons);
        Ok(())
    }
}
