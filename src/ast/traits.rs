use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::*;

// Identifier to spacify trait.
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TraitId {
    pub name: FullName,
}

impl TraitId {
    pub fn from_fullname(name: FullName) -> TraitId {
        TraitId { name }
    }

    pub fn to_string(&self) -> String {
        self.namespaced_name().to_string()
    }

    pub fn namespaced_name(&self) -> FullName {
        self.name.clone()
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), String> {
        self.name = ctx.resolve(&self.name, &[NameResolutionType::Trait])?;
        Ok(())
    }
}

// Definition of associated type.
#[derive(Clone)]
pub struct AssocTypeDefn {
    // The local name of the associated type.
    pub name: Name,
    // Kind predicates on the definition of the associated type.
    pub kind_signs: Vec<KindSignature>,
    // Type parameters of the associated type.
    // Includes `impl_type`.
    pub params: Vec<Arc<TyVar>>,
    // The kind of the application of the associated type.
    pub kind_applied: Arc<Kind>,
    // Source location of associated type definition.
    pub src: Option<Span>,
}

impl AssocTypeDefn {
    pub fn param_kinds(&self) -> Vec<Arc<Kind>> {
        self.params.iter().map(|p| p.kind.clone()).collect()
    }

    pub fn set_kinds(&mut self, impl_type_kind: Arc<Kind>) {
        // Set `impl_type_kind` to `parms[0]`.
        self.params[0] = self.params[0].set_kind(impl_type_kind.clone());
        // Set `kind_signs` to `self.params`.
        for param in &mut self.params[1..] {
            // Skip `self`.
            for kind_sign in &self.kind_signs {
                if param.name == kind_sign.tyvar {
                    *param = param.set_kind(kind_sign.kind.clone());
                }
            }
        }
    }
}

// Implementation of associated type.
#[derive(Clone)]
pub struct AssocTypeImpl {
    pub name: Name,
    // Type parameters of the associated type implementation.
    // Includes `impl_type`.
    pub params: Vec<Arc<TyVar>>,
    pub value: Arc<TypeNode>,
    pub source: Option<Span>,
}

impl AssocTypeImpl {
    pub fn set_kinds(&mut self, trait_inst: &TraitInstance, kind_env: &KindEnv) {
        let assoc_ty_name = TyAssoc {
            name: FullName::new(&trait_inst.trait_id().name.to_namespace(), &self.name),
        };
        let param_kinds = &kind_env.assoc_tys.get(&assoc_ty_name).unwrap().param_kinds;
        if self.params.len() != param_kinds.len() {
            error_exit_with_src(
                &format!(
                    "Invalid number of parameters for associated type `{}`. Expect: {}, found: {}.",
                    self.name,
                    param_kinds.len(),
                    self.params.len()
                ),
                &self.source,
            )
        }
        let mut tvs_in_value = vec![];
        trait_inst.impl_type().free_vars_to_vec(&mut tvs_in_value);
        for (param, kind) in &mut self.params[1..].iter_mut().zip(param_kinds[1..].iter()) {
            *param = param.set_kind(kind.clone());
            tvs_in_value.push(param.clone());
        }
        let mut tv_to_kind = HashMap::new();
        for tv_in_value in tvs_in_value {
            tv_to_kind.insert(tv_in_value.name.clone(), tv_in_value.kind.clone());
        }
        self.value = self.value.set_kinds(&tv_to_kind);
    }
}

// #[derive(Clone, Serialize, Deserialize)]
// pub struct AssocTypeInfo {
//     pub name: FullName,
//     pub arity: usize,
//     pub kind: Arc<Kind>,
//     pub source: Option<Span>,
// }

#[derive(Clone)]
pub struct AssocTypeKindInfo {
    pub name: TyAssoc,
    pub param_kinds: Vec<Arc<Kind>>, // Includes `self`.
    pub value_kind: Arc<Kind>,
}

// Traits definitions.
#[derive(Clone)]
pub struct TraitInfo {
    // Identifier of this trait (i.e. the name).
    pub id: TraitId,
    // Type variable used in trait definition.
    pub type_var: Arc<TyVar>,
    // Methods of this trait.
    // Here, for example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String",
    // and not "a -> String for a : Show".
    pub methods: HashMap<Name, QualType>,
    // Associated type synonyms.
    pub assoc_types: HashMap<Name, AssocTypeDefn>,
    // Kind signatures at the trait declaration, e.g., "f: *->*" in "trait [f:*->*] f: Functor {}".
    pub kind_signs: Vec<KindSignature>,
    // Source location of trait definition.
    pub source: Option<Span>,
}

impl TraitInfo {
    // Resolve namespace.
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for (_name, qt) in &mut self.methods {
            qt.resolve_namespace(ctx);
        }
    }

    // Resolve type aliases
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        for (_name, qt) in &mut self.methods {
            qt.resolve_type_aliases(type_env);
        }
    }

    // Get type-scheme of a method.
    // Here, for example, in case "trait a: ToString { to_string : a -> String }",
    // this function returns "[a: ToString] a -> String" as type of "to_string" method.
    pub fn method_scheme(&self, name: &Name) -> Arc<Scheme> {
        let mut qual_ty = self.methods.get(name).unwrap().clone();
        let mut vars = vec![];
        qual_ty.free_vars_vec(&mut vars);
        let mut preds = vec![Predicate::make(
            self.id.clone(),
            type_from_tyvar(self.type_var.clone()),
        )];
        preds.append(&mut qual_ty.preds);
        Scheme::generalize(vars, preds, qual_ty.eqs, qual_ty.ty)
    }

    // Get the type of a method.
    // Here, for example, in case "trait a: ToString { to_string: a -> String }",
    // this function returns "a -> String" as type of "to_string" method.
    pub fn method_ty(&self, name: &Name) -> QualType {
        self.methods.get(name).unwrap().clone()
    }

    // Validate kind_signs and set it to self.type_var.
    // Also, set kinds of parameters of associated type definition.
    pub fn set_trait_kind(&mut self) {
        if self.kind_signs.len() >= 2 {
            let span = Span::unite_opt(&self.kind_signs[0].source, &self.kind_signs[1].source);
            error_exit_with_src(
                "You can specify at most one constraint of the form `{type-variable} : {kind}` as the assumption of trait definition.",
                &span,
            );
        }
        if self.kind_signs.len() > 0 {
            if self.kind_signs[0].tyvar != self.type_var.name {
                error_exit_with_src(
                    &format!(
                        "The type variable used in the assumption of trait `{}` has to be `{}`.",
                        self.id.to_string(),
                        self.type_var.name,
                    ),
                    &self.kind_signs[0].source,
                );
            }
            self.type_var = self.type_var.set_kind(self.kind_signs[0].kind.clone());
        }
        for (_, assoc_ty_defn) in &mut self.assoc_types {
            assoc_ty_defn.set_kinds(self.type_var.kind.clone());
        }
    }

    // pub fn assoc_type_info(&self, assoc_type_name: &Name) -> AssocTypeInfo {
    //     let assoc_type = self.assoc_types.get(assoc_type_name).unwrap();
    //     AssocTypeInfo {
    //         name: FullName::new(&self.id.name.to_namespace(), assoc_type_name),
    //         kind: kind_arrow(self.type_var.kind.clone(), assoc_type.kind_applied.clone()),
    //         source: assoc_type.src.clone(),
    //     }
    // }
}

// Trait instance.
#[derive(Clone)]
pub struct TraitInstance {
    // Statement such as "[a: Show, b: Show] (a, b): Show".
    pub qual_pred: QualPredicate,
    // Method implementation.
    pub methods: HashMap<Name, Arc<ExprNode>>,
    // Associated type synonym implementation.
    pub assoc_types: HashMap<Name, AssocTypeImpl>,
    // Module where this instance is defined.
    pub define_module: Name,
    // Source location where this instance is defined.
    pub source: Option<Span>,
}

impl TraitInstance {
    pub fn set_kinds_in_qual_pred(&mut self, kind_env: &KindEnv) {
        let mut scope = HashMap::new();
        let preds = &self.qual_pred.pred_constraints;
        let eqs = &self.qual_pred.eq_constraints;
        let kind_signs = &self.qual_pred.kind_constraints;
        let res = QualPredicate::extend_kind_scope(&mut scope, preds, eqs, kind_signs, kind_env);
        if res.is_err() {
            error_exit_with_src(&res.unwrap_err(), &self.source);
        }
        self.qual_pred.predicate.set_kinds(&scope);
        for pred in &mut self.qual_pred.pred_constraints {
            pred.set_kinds(&scope);
        }
        for eq in &mut self.qual_pred.eq_constraints {
            eq.set_kinds(&scope);
        }
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        self.qual_pred.resolve_namespace(ctx);

        // This function is called only by resolve_namespace_in_declaration, so we don't need to see into expression.

        // for (_name, expr) in &mut self.methods {
        //     *expr = expr.resolve_namespace(ctx);
        // }
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        self.qual_pred.resolve_type_aliases(type_env);
    }

    // Get trait id.
    fn trait_id(&self) -> TraitId {
        self.qual_pred.predicate.trait_id.clone()
    }

    // Get mutable trait id.
    fn trait_id_mut(&mut self) -> &mut TraitId {
        &mut self.qual_pred.predicate.trait_id
    }

    // Get type-scheme of a method implementation.
    // Here, for example, in case "impl [a: ToString, b: ToString] (a, b): ToString",
    // this function returns "[a: ToString, b: ToString] (a, b) -> String" as the type of "to_string".
    pub fn method_scheme(&self, method_name: &Name, trait_info: &TraitInfo) -> Arc<Scheme> {
        // Create qualtype. ex. `[] (a, b) -> String`.
        let trait_tyvar = &trait_info.type_var.name; // ex. tyvar == `t`
        let impl_type = self.impl_type(); // ex. impl_type == `(a, b)`
        let s = Substitution::single(&trait_tyvar, impl_type);
        let mut method_qualty = trait_info.method_ty(method_name); // ex. method_qualty == `[] t -> String`
        s.substitute_qualtype(&mut method_qualty); // ex. method_qualty == `[] (a, b) -> String`

        // Prepare `vars`, `ty`, `preds`, and `eqs` to be generalized.
        let ty = method_qualty.ty.clone();
        let mut vars = vec![];
        method_qualty.free_vars_vec(&mut vars);
        self.qual_pred.free_vars_vec(&mut vars);
        let mut preds = self.qual_pred.pred_constraints.clone();
        preds.append(&mut method_qualty.preds);
        let mut eqs = self.qual_pred.eq_constraints.clone();
        eqs.append(&mut method_qualty.eqs);

        // Set source location of the type to the location where the method is implemented.
        let source = self
            .method_expr(method_name)
            .source
            .as_ref()
            .map(|src| src.to_single_character());
        let ty = ty.set_source(source);

        Scheme::generalize(vars, preds, eqs, ty)
    }

    // Get expression that implements a method.
    pub fn method_expr(&self, name: &Name) -> Arc<ExprNode> {
        self.methods.get(name).unwrap().clone()
    }

    // Get the type implementing the trait.
    pub fn impl_type(&self) -> Arc<TypeNode> {
        self.qual_pred.predicate.ty.clone()
    }
}

// Trait Aliases
#[derive(Clone)]
pub struct TraitAlias {
    // Identifier of this trait (i.e., the name).
    pub id: TraitId,
    // Aliased traits.
    pub value: Vec<TraitId>,
    // Source location of alias definition.
    pub source: Option<Span>,
    // Kind of this trait alias.
    pub kind: Arc<Kind>,
}

impl TraitAlias {
    // Resolve namespace of trait names in value.
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for trait_id in &mut self.value {
            let res = trait_id.resolve_namespace(ctx);
            if res.is_err() {
                error_exit_with_src(&res.unwrap_err(), &self.source);
            }
        }
    }
}

// Qualified predicate. Statement such as "[a : Eq] Array a : Eq".
// Constraints in `[...]` can be trait bound and equality.
#[derive(Clone)]
pub struct QualPredicate {
    pub pred_constraints: Vec<Predicate>,
    pub eq_constraints: Vec<Equality>,
    pub kind_constraints: Vec<KindSignature>,
    pub predicate: Predicate,
}

impl QualPredicate {
    // pub fn free_vars(&self) -> HashMap<Name, Arc<TyVar>> {
    //     let mut vars = self.predicate.free_vars();
    //     for pred in &self.pred_constraints {
    //         vars.extend(pred.free_vars());
    //     }
    //     for eq in &self.eq_constraints {
    //         vars.extend(eq.free_vars());
    //     }
    //     vars
    // }

    pub fn free_vars_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        for pred in &self.pred_constraints {
            pred.ty.free_vars_to_vec(buf);
        }
        for eq in &self.eq_constraints {
            eq.free_vars_vec(buf);
        }
        self.predicate.ty.free_vars_to_vec(buf);
        // Apply kind predicates.
        for tv in buf {
            for kind_sign in &self.kind_constraints {
                if tv.name == kind_sign.tyvar {
                    *tv = tv.set_kind(kind_sign.kind.clone());
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        let mut s = String::default();
        if self.pred_constraints.len() > 0 || self.kind_constraints.len() > 0 {
            s += "[";
        }
        let mut preds = vec![];
        preds.extend(self.kind_constraints.iter().map(|p| p.to_string()));
        preds.extend(self.pred_constraints.iter().map(|p| p.to_string()));
        s += &preds.join(", ");
        if self.pred_constraints.len() > 0 || self.kind_constraints.len() > 0 {
            s += "] ";
        }
        s += &self.predicate.to_string();
        s
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for p in &mut self.pred_constraints {
            p.resolve_namespace(ctx);
        }
        self.predicate.resolve_namespace(ctx);
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        for p in &mut self.pred_constraints {
            p.resolve_type_aliases(type_env);
        }
        self.predicate.resolve_type_aliases(type_env);
    }

    pub fn extend_kind_scope(
        scope: &mut HashMap<Name, Arc<Kind>>,
        preds: &Vec<Predicate>,
        eqs: &Vec<Equality>,
        kind_signs: &Vec<KindSignature>,
        kind_env: &KindEnv,
    ) -> Result<(), String> {
        fn insert(
            scope: &mut HashMap<Name, Arc<Kind>>,
            tyvar: String,
            kind: Arc<Kind>,
        ) -> Result<(), String> {
            if scope.contains_key(&tyvar) {
                if scope[&tyvar] != kind {
                    return Err(format!("Kind mismatch on type variable `{}`.", tyvar));
                }
            } else {
                scope.insert(tyvar, kind);
            }
            Ok(())
        }
        fn extend_by_assoc_ty_application(
            scope: &mut HashMap<Name, Arc<Kind>>,
            assoc_ty_app: Arc<TypeNode>,
            kind_env: &KindEnv,
        ) -> Result<(), String> {
            match &assoc_ty_app.ty {
                Type::AssocTy(assoc_ty, args) => {
                    let kind_info = kind_env.assoc_tys.get(assoc_ty).unwrap();
                    if args.len() != kind_info.param_kinds.len() {
                        return Err(format!(
                            "Invalid number of arguments for associated type `{}`. Expect: {}, found: {}.",
                            assoc_ty.name.to_string(),
                            kind_info.param_kinds.len(),
                            args.len()
                        ));
                    }
                    for (arg, kind) in args.iter().zip(kind_info.param_kinds.iter()) {
                        match &arg.ty {
                            Type::TyVar(tv) => {
                                insert(scope, tv.name.clone(), kind.clone())?;
                            }
                            Type::AssocTy(_, _) => {
                                extend_by_assoc_ty_application(scope, arg.clone(), kind_env)?;
                            }
                            _ => {}
                        }
                    }
                }
                _ => unreachable!("Associated type application expected."),
            }
            Ok(())
        }

        for kp in kind_signs {
            let tyvar = kp.tyvar.clone();
            let kind = kp.kind.clone();
            insert(scope, tyvar, kind)?;
        }
        for pred in preds {
            match &pred.ty.ty {
                Type::TyVar(tv) => {
                    let trait_id = &pred.trait_id;
                    if !kind_env.traits_and_aliases.contains_key(trait_id) {
                        panic!("Unknown trait: {}", trait_id.to_string());
                    }
                    let kind = kind_env.traits_and_aliases[trait_id].clone();
                    insert(scope, tv.name.clone(), kind)?;
                }
                Type::AssocTy(_, _) => {
                    extend_by_assoc_ty_application(scope, pred.ty.clone(), kind_env)?;
                }
                _ => {
                    // Do nothing.
                }
            }
        }
        for eq in eqs {
            extend_by_assoc_ty_application(scope, eq.lhs(), kind_env)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct QualPredScheme {
    pub gen_vars: Vec<Arc<TyVar>>,
    pub qual_pred: QualPredicate,
}

#[derive(Clone)]
pub struct QualType {
    pub preds: Vec<Predicate>,
    pub eqs: Vec<Equality>,
    pub kind_signs: Vec<KindSignature>,
    pub ty: Arc<TypeNode>,
}

impl QualType {
    // Resolve namespace.
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for pred in &mut self.preds {
            pred.resolve_namespace(ctx);
        }
        for eq in &mut self.eqs {
            eq.resolve_namespace(ctx);
        }
        self.ty = self.ty.resolve_namespace(ctx);
    }

    // Resolve type aliases
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        for pred in &mut self.preds {
            pred.resolve_type_aliases(type_env);
        }
        for eq in &mut self.eqs {
            eq.resolve_type_aliases(type_env);
        }
        self.ty = self.ty.resolve_type_aliases(type_env);
    }

    // Calculate free type variables.
    // pub fn free_vars(&self) -> HashMap<Name, Arc<TyVar>> {
    //     let mut ret = self.ty.free_vars();
    //     for pred in &self.preds {
    //         ret.extend(pred.free_vars());
    //     }
    //     for eq in &self.eqs {
    //         ret.extend(eq.free_vars());
    //     }
    //     ret
    // }

    pub fn free_vars_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        for pred in &self.preds {
            pred.ty.free_vars_to_vec(buf);
        }
        for eq in &self.eqs {
            eq.free_vars_vec(buf);
        }
        self.ty.free_vars_to_vec(buf);
        // Apply kind predicates.
        for tv in buf {
            for kind_sign in &self.kind_signs {
                if tv.name == kind_sign.tyvar {
                    *tv = tv.set_kind(kind_sign.kind.clone());
                }
            }
        }
    }
}

// Statement such as "String: Show" or "a: Eq".
#[derive(Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub trait_id: TraitId,
    pub ty: Arc<TypeNode>,
    pub source: Option<Span>,
}

impl Predicate {
    pub fn validate_freeness(&self) {
        if !self.ty.is_free() {
            error_exit_with_src(
                "Trait constraint is only allowed on a free type. \
                Here, free type is either a type variable or an associated type applied to free types where all type variable that appear are distinct.",
                &self.source,
            );
        }
    }

    pub fn free_vars(&self) -> HashMap<Name, Arc<TyVar>> {
        self.ty.free_vars()
    }

    pub fn set_source(&mut self, source: Span) {
        self.source = Some(source);
    }

    pub fn make(trait_id: TraitId, ty: Arc<TypeNode>) -> Self {
        Predicate {
            trait_id,
            ty,
            source: None,
        }
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        let resolve_result = self.trait_id.resolve_namespace(ctx);
        if resolve_result.is_err() {
            error_exit_with_src(&resolve_result.unwrap_err(), &self.source)
        }
        self.ty = self.ty.resolve_namespace(ctx);
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        self.ty = self.ty.resolve_type_aliases(type_env);
    }

    pub fn to_string_normalize(&self) -> String {
        format!(
            "{} : {}",
            self.ty.to_string_normalize(),
            self.trait_id.to_string()
        )
    }

    pub fn to_string(&self) -> String {
        format!("{} : {}", self.ty.to_string(), self.trait_id.to_string())
    }

    pub fn set_kinds(&mut self, scope: &HashMap<Name, Arc<Kind>>) {
        self.ty = self.ty.set_kinds(scope);
    }

    pub fn check_kinds(&self, kind_env: &KindEnv) {
        let expected = &kind_env.traits_and_aliases[&self.trait_id];
        let found = self.ty.kind(kind_env);
        if *expected != found {
            error_exit_with_src(
                &format!(
                    "Kind mismatch in `{}`. Expect: {}, found: {}.",
                    self.to_string_normalize(),
                    expected.to_string(),
                    found.to_string()
                ),
                &self.source,
            )
        }
    }

    // If the trait used in this predicate is a trait alias, resolve it to a set of predicates that are not using trait aliases.
    pub fn resolve_trait_aliases(&self, trait_env: &TraitEnv) -> Vec<Predicate> {
        if !trait_env.is_alias(&self.trait_id) {
            return vec![self.clone()];
        }
        let trait_ids = trait_env.resolve_aliases(&self.trait_id);
        let mut res = vec![];
        for trait_id in trait_ids {
            let mut p = self.clone();
            p.trait_id = trait_id;
            res.push(p);
        }
        res
    }
}

// Statement such as "f: * -> *".
#[derive(Clone)]
pub struct KindSignature {
    pub tyvar: Name,
    pub kind: Arc<Kind>,
    pub source: Option<Span>,
}

impl KindSignature {
    pub fn to_string(&self) -> String {
        format!("{} : {}", self.tyvar, self.kind.to_string())
    }
}

// Equality predicate `AssociateType args = value`.
#[derive(Clone, Serialize, Deserialize)]
pub struct Equality {
    pub assoc_type: TyAssoc,
    pub args: Vec<Arc<TypeNode>>,
    pub value: Arc<TypeNode>,
    pub source: Option<Span>,
}

impl Equality {
    pub fn validate_freeness(&self) {
        if !self.lhs().is_free() {
            error_exit_with_src(
                "Equality constraint is only allowed on free types. \
                Here, free type is either a type variable or an associated type applied to free types where all type variable that appear are distinct.",
                &self.source,
            );
        }
    }

    pub fn check_kinds(&self, kind_env: &KindEnv) {
        let kind_info = kind_env.assoc_tys.get(&self.assoc_type).unwrap();
        if self.args.len() != kind_info.param_kinds.len() {
            error_exit_with_src(
                &format!(
                    "Invalid number of arguments for associated type `{}`. Expect: {}, found: {}.",
                    self.assoc_type.name.to_string(),
                    kind_info.param_kinds.len(),
                    self.args.len()
                ),
                &self.source,
            )
        }
        for (arg, expect_kind) in self.args.iter().zip(kind_info.param_kinds.iter()) {
            let found_kind = arg.kind(kind_env);
            if *expect_kind != found_kind {
                error_exit_with_src(
                    &format!(
                        "Kind mismatch in `{}`. Expect: {}, found: {}.",
                        arg.to_string(),
                        expect_kind.to_string(),
                        found_kind.to_string()
                    ),
                    &self.source,
                )
            }
        }
        let found_kind = self.value.kind(kind_env);
        if kind_info.value_kind != found_kind {
            error_exit_with_src(
                &format!(
                    "Kind mismatch in `{}`. Expect: {}, found: {}.",
                    self.value.to_string(),
                    kind_info.value_kind.to_string(),
                    found_kind.to_string()
                ),
                &self.source,
            )
        }
    }

    pub fn set_kinds(&mut self, scope: &HashMap<Name, Arc<Kind>>) {
        for arg in &mut self.args {
            *arg = arg.set_kinds(scope);
        }
        self.value = self.value.set_kinds(scope);
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        for arg in &mut self.args {
            *arg = arg.resolve_type_aliases(type_env);
        }
        self.value = self.value.resolve_type_aliases(type_env);
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        let result = self.assoc_type.resolve_namespace(ctx);
        if result.is_err() {
            error_exit_with_src(&result.unwrap_err(), &self.source)
        }
        for arg in &mut self.args {
            *arg = arg.resolve_namespace(ctx);
        }
        self.value = self.value.resolve_namespace(ctx);
    }

    // pub fn is_all_args_tyvar(&self) -> bool {
    //     self.args.iter().all(|arg| arg.is_tyvar())
    // }

    pub fn to_string(&self) -> String {
        format!("{} = {}", self.lhs().to_string(), self.value.to_string())
    }

    pub fn free_vars(&self) -> HashMap<Name, Arc<TyVar>> {
        let mut vars = self.value.free_vars();
        for arg in &self.args {
            vars.extend(arg.free_vars());
        }
        vars
    }

    pub fn free_vars_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        for arg in &self.args {
            arg.free_vars_to_vec(buf);
        }
        self.value.free_vars_to_vec(buf);
    }

    // Get the type of the left-hand side of the equality.
    pub fn lhs(&self) -> Arc<TypeNode> {
        type_assocty(self.assoc_type.clone(), self.args.clone())
    }

    // pub fn trait_id(&self) -> TraitId {
    //     let mut names = self.assoc_type.name.namespace.names.clone();
    //     let name = names.pop().unwrap();
    //     let namespace = NameSpace::new(names);
    //     TraitId::from_fullname(FullName::new(&namespace, &name))
    // }

    // Returns the predicate to reduce this equality.
    // pub fn predicate(&self) -> Predicate {
    //     // Get trait name from the name of the associated type.
    //     let trait_name = self.assoc_type.namespace;
    //     let trait_name = FullName::new(
    //         &NameSpace::new(trait_name.names[0..trait_name.names.len() - 1].to_vec()),
    //         trait_name.names.last().unwrap(),
    //     );
    //     Predicate::make(TraitId::from_fullname(trait_name), self.impl_type.clone())
    // }

    pub fn generalize(&self) -> EqualityScheme {
        let mut tyvars = vec![];
        for arg in &self.args {
            arg.free_vars_to_vec(&mut tyvars);
        }
        self.value.free_vars_to_vec(&mut tyvars);
        EqualityScheme {
            gen_vars: tyvars,
            equality: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct EqualityScheme {
    pub gen_vars: Vec<Arc<TyVar>>,
    pub equality: Equality,
}

// Trait environments.
#[derive(Clone, Default)]
pub struct TraitEnv {
    pub traits: HashMap<TraitId, TraitInfo>,
    pub instances: HashMap<TraitId, Vec<TraitInstance>>,
    pub aliases: HashMap<TraitId, TraitAlias>,
}

impl TraitEnv {
    // Get of list of trait names including aliases.
    pub fn trait_names(&self) -> HashSet<FullName> {
        let mut res: HashSet<FullName> = Default::default();
        for (k, _v) in &self.traits {
            res.insert(k.name.clone());
        }
        for (k, _v) in &self.aliases {
            res.insert(k.name.clone());
        }
        res
    }

    pub fn validate(&mut self, kind_env: KindEnv) {
        // Check name confliction of traits and aliases.
        fn create_conflicting_error(env: &TraitEnv, trait_id: &TraitId) {
            let this_src = &env.traits.get(trait_id).unwrap().source;
            let other_src = &env.aliases.get(trait_id).unwrap().source;
            error_exit_with_srcs(
                &format!("Duplicate definition for `{}`", trait_id.to_string()),
                &[&this_src, &other_src],
            );
        }
        for (trait_id, _) in &self.traits {
            if self.aliases.contains_key(trait_id) {
                create_conflicting_error(self, trait_id);
            }
        }
        for (trait_id, _) in &self.aliases {
            if self.traits.contains_key(trait_id) {
                create_conflicting_error(self, trait_id);
            }
        }

        // Check that values of trait aliases are defined.
        for (_, ta) in &self.aliases {
            for v in &ta.value {
                if !self.traits.contains_key(v) && !self.aliases.contains_key(v) {
                    error_exit_with_src(&format!("Unknown trait `{}`.", v.to_string()), &ta.source);
                }
            }
        }
        // Circular aliasing will be detected in `TraitEnv::resolve_aliases`.

        for (_trait_id, trait_info) in &self.traits {
            for (method_name, method_ty) in &trait_info.methods {
                // Forbid unrelated trait method:
                // Check that the type variable in trait definition appears each of the methods' type.
                // This assumption is used in `InstanciatedSymbol::dependent_modules`.
                if !method_ty.ty.contains_tyvar(&trait_info.type_var) {
                    error_exit_with_src (
                        &format!(
                            "Type variable `{}` used in trait definition has to appear in the type of a method `{}`.",
                            trait_info.type_var.name,
                            method_name,
                        ),
                        method_ty.ty.get_source()
                    );
                }
                // Validate constraints.
                for pred in &method_ty.preds {
                    pred.validate_freeness();
                }
                for eq in &method_ty.eqs {
                    eq.validate_freeness();
                }
            }
        }
        let aliases: HashSet<_> = self.aliases.keys().collect();
        // Prepare TypeCheckContext to use `unify`.
        let tc = TypeCheckContext::new(
            TraitEnv::default(),
            TypeEnv::default(),
            kind_env,
            HashMap::new(),
        );
        // Validate trait instances.
        for (trait_id, insts) in &mut self.instances {
            for inst in insts.iter_mut() {
                // check implementation is given for trait, not for trait alias.
                if aliases.contains(trait_id) {
                    error_exit_with_src(
                        "A trait alias cannot be implemented directly. Implement each aliased trait instead.",
                        &inst.qual_pred.predicate.source,
                    )
                }

                *inst.trait_id_mut() = trait_id.clone();

                // Check instance head.
                let implemented_ty = &inst.qual_pred.predicate.ty;
                if !implemented_ty.is_implementable() {
                    error_exit_with_src(
                        &format!("Implementing trait for type `{}` is not allowed. \
                        The head (in this case, `{}`) of a type for which trait is implemented should be a type constructor.", implemented_ty.to_string(), implemented_ty.get_head_string()),&implemented_ty.get_source()
                    );
                }

                // Validate instance context.
                for pred in &inst.qual_pred.pred_constraints {
                    pred.validate_freeness();
                }
                for eq in &inst.qual_pred.eq_constraints {
                    eq.validate_freeness();
                }

                // Check whether all trait methods are implemented.
                let trait_methods = &self.traits[trait_id]
                    .methods
                    .iter()
                    .map(|s| s.0)
                    .collect::<HashSet<_>>();
                let impl_methods = inst.methods.iter().map(|s| s.0).collect::<HashSet<_>>();
                for trait_method in trait_methods {
                    if !impl_methods.contains(trait_method) {
                        let pred = inst.qual_pred.predicate.to_string_normalize();
                        error_exit_with_src(
                            &format!(
                                "Lacking implementation of method `{}` for `{}`",
                                trait_method, pred,
                            ),
                            &inst.source,
                        )
                    }
                }
                for impl_method in impl_methods {
                    if !trait_methods.contains(impl_method) {
                        error_exit_with_src(
                            &format!(
                                "`{}` is not a method of trait `{}`.",
                                impl_method,
                                trait_id.to_string(),
                            ),
                            &inst
                                .methods
                                .get(impl_method)
                                .unwrap()
                                .source
                                .as_ref()
                                .map(|s| s.to_single_character()),
                        )
                    }
                }

                // Check Orphan rules.
                let instance_def_mod = &inst.define_module;
                let trait_def_id = trait_id.name.module();
                let ty = &inst.qual_pred.predicate.ty;
                let type_def_id = if ty.is_funty() {
                    STD_NAME.to_string()
                } else {
                    ty.toplevel_tycon().unwrap().name.module()
                };
                if trait_def_id != *instance_def_mod && type_def_id != *instance_def_mod {
                    error_exit_with_src(
                        &format!(
                            "Implementing trait `{}` for type `{}` in module `{}` is illegal; it is not allowed to implement an external trait for an external type.",
                            trait_id.to_string(),
                            ty.to_string_normalize(),
                            instance_def_mod.to_string(),
                        ),
                        &inst.source.as_ref().map(|s| s.to_single_character()),
                    );
                }
            }

            // Check overlapping instance.
            for i in 0..insts.len() {
                for j in (i + 1)..insts.len() {
                    let inst_i = &insts[i];
                    let inst_j = &insts[j];
                    let mut tc = tc.clone();
                    if tc.unify(&inst_i.impl_type(), &inst_j.impl_type()).is_err() {
                        continue;
                    }
                    error_exit_with_srcs(
                        &format!(
                            "Two trait implementations for `{}` are overlapping.",
                            trait_id.to_string()
                        ),
                        &[
                            &inst_i.source.as_ref().map(|s| s.to_single_character()),
                            &inst_j.source.as_ref().map(|s| s.to_single_character()),
                        ],
                    );
                }
            }
        }
    }

    pub fn resolve_namespace(
        &mut self,
        ctx: &mut NameResolutionContext,
        imported_modules: &HashMap<Name, Vec<ImportStatement>>,
    ) {
        // Resolve names in trait aliases.
        for (trait_id, alias_info) in &mut self.aliases {
            ctx.import_statements = imported_modules[&trait_id.name.module()].clone();
            alias_info.resolve_namespace(ctx);
        }

        // Resolve names in trait definitions.
        for (trait_id, trait_info) in &mut self.traits {
            ctx.import_statements = imported_modules[&trait_id.name.module()].clone();
            // Keys in self.traits should already be resolved.
            assert!(
                trait_id.name
                    == ctx
                        .resolve(&trait_id.name, &[NameResolutionType::Trait])
                        .unwrap()
            );
            trait_info.resolve_namespace(ctx);
        }

        // Resolve names in trait implementations.
        let insntaces = std::mem::replace(&mut self.instances, Default::default());
        let mut instances_resolved: HashMap<TraitId, Vec<TraitInstance>> = Default::default();
        for (trait_id, insts) in insntaces {
            for mut inst in insts {
                // Set up NameResolutionContext.
                ctx.import_statements = imported_modules[&inst.define_module].clone();

                // Resolve trait_id's namespace.
                let mut trait_id = trait_id.clone();
                let resolve_result = trait_id.resolve_namespace(ctx);
                if resolve_result.is_err() {
                    let src = inst.qual_pred.predicate.source.clone();
                    error_exit_with_src(&resolve_result.unwrap_err(), &src)
                }

                // Resolve names in TrantInstance.
                inst.resolve_namespace(ctx);

                // Insert to instances_resolved
                if !instances_resolved.contains_key(&trait_id) {
                    instances_resolved.insert(trait_id.clone(), vec![]);
                }
                instances_resolved.get_mut(&trait_id).unwrap().push(inst);
            }
        }
        self.instances = instances_resolved;
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        // Resolve aliases in trait definitions.
        for (_, trait_info) in &mut self.traits {
            trait_info.resolve_type_aliases(type_env);
        }

        // Resolve aliases in trait implementations.
        let insntaces = std::mem::replace(&mut self.instances, Default::default());
        let mut instances_resolved: HashMap<TraitId, Vec<TraitInstance>> = Default::default();
        for (trait_id, insts) in insntaces {
            for mut inst in insts {
                // Resolve names in TrantInstance.
                inst.resolve_type_aliases(type_env);

                // Insert to instances_resolved
                if !instances_resolved.contains_key(&trait_id) {
                    instances_resolved.insert(trait_id.clone(), vec![]);
                }
                instances_resolved.get_mut(&trait_id).unwrap().push(inst);
            }
        }
        self.instances = instances_resolved;
    }

    // Add traits.
    pub fn add(
        &mut self,
        trait_infos: Vec<TraitInfo>,
        trait_impls: Vec<TraitInstance>,
        trait_aliases: Vec<TraitAlias>,
    ) {
        for trait_info in trait_infos {
            self.add_trait(trait_info);
        }
        for trait_impl in trait_impls {
            self.add_instance(trait_impl);
        }
        for trait_alias in trait_aliases {
            self.add_alias(trait_alias);
        }
    }

    // Add a trait to TraitEnv.
    pub fn add_trait(&mut self, info: TraitInfo) {
        // Check duplicate definition.
        if self.traits.contains_key(&info.id) {
            let info1 = self.traits.get(&info.id).unwrap();
            error_exit_with_srcs(
                &format!("Duplicate definition for trait {}.", info.id.to_string()),
                &[&info1.source, &info.source],
            );
        }
        self.traits.insert(info.id.clone(), info);
    }

    // Add an instance.
    pub fn add_instance(&mut self, inst: TraitInstance) {
        let trait_id = inst.trait_id();
        if !self.instances.contains_key(&trait_id) {
            self.instances.insert(trait_id.clone(), vec![]);
        }
        self.instances.get_mut(&trait_id).unwrap().push(inst);
    }

    // Add an trait alias.
    fn add_alias(&mut self, alias: TraitAlias) {
        // Check duplicate definition.
        if self.aliases.contains_key(&alias.id) {
            let alias1 = self.aliases.get(&alias.id).unwrap();
            error_exit_with_srcs(
                &format!("Duplicate definition for trait {}.", alias.id.to_string()),
                &[&alias1.source, &alias.source],
            );
        }
        self.aliases.insert(alias.id.clone(), alias);
    }

    pub fn qualified_predicates(&self) -> HashMap<TraitId, Vec<QualPredScheme>> {
        let mut qps = HashMap::default();
        for (trait_id, insts) in &self.instances {
            for inst in insts {
                let mut vars = vec![];
                inst.qual_pred.free_vars_vec(&mut vars);
                misc::insert_to_hashmap_vec(
                    &mut qps,
                    trait_id,
                    QualPredScheme {
                        gen_vars: vars,
                        qual_pred: inst.qual_pred.clone(),
                    },
                );
            }
        }
        qps
    }

    // From implementation of associated types, get generalized type equalities.
    pub fn type_equalities(&self) -> HashMap<TyAssoc, Vec<EqualityScheme>> {
        let mut eq_scms = HashMap::default();
        for (trait_id, insts) in &self.instances {
            for inst in insts {
                for (assoc_type_name, assoc_type_impl) in &inst.assoc_types {
                    let assoc_type_namespace = trait_id.name.to_namespace();
                    let assoc_type_fullname = FullName::new(&assoc_type_namespace, assoc_type_name);
                    let impl_type = inst.impl_type();
                    let mut args = vec![impl_type];
                    for tv in &assoc_type_impl.params[1..] {
                        args.push(type_from_tyvar(tv.clone()));
                    }
                    let equality = Equality {
                        assoc_type: TyAssoc {
                            name: assoc_type_fullname,
                        },
                        args,
                        value: assoc_type_impl.value.clone(),
                        source: assoc_type_impl.source.clone(),
                    };
                    misc::insert_to_hashmap_vec(
                        &mut eq_scms,
                        &equality.assoc_type,
                        equality.generalize(),
                    );
                }
            }
        }
        eq_scms
    }

    // pub fn assoc_ty_names(&self) -> HashSet<FullName> {
    //     let mut names = vec![];
    //     for (trait_id, trait_info) in &self.traits {
    //         for (assoc_ty_name, _assoc_ty_info) in &trait_info.assoc_types {
    //             let assoc_type_namespace = trait_id.name.to_namespace();
    //             let assoc_type_fullname = FullName::new(&assoc_type_namespace, &assoc_ty_name);
    //             names.push(assoc_type_fullname)
    //         }
    //     }
    //     names.into_iter().collect::<HashSet<_>>()
    // }

    pub fn assoc_ty_to_arity(&self) -> HashMap<FullName, usize> {
        let mut assoc_ty_arity = HashMap::new();
        for (trait_id, trait_info) in &self.traits {
            for (assoc_ty_name, assoc_ty_info) in &trait_info.assoc_types {
                let assoc_type_namespace = trait_id.name.to_namespace();
                let assoc_type_fullname = FullName::new(&assoc_type_namespace, &assoc_ty_name);
                let arity = assoc_ty_info.params.len();
                assoc_ty_arity.insert(assoc_type_fullname, arity);
            }
        }
        assoc_ty_arity
    }

    pub fn assoc_ty_kind_info(&self) -> HashMap<TyAssoc, AssocTypeKindInfo> {
        let mut assoc_ty_kind_info = HashMap::new();
        for (trait_id, trait_info) in &self.traits {
            for (assoc_ty_name, assoc_ty_info) in &trait_info.assoc_types {
                let assoc_type_namespace = trait_id.name.to_namespace();
                let assoc_type = TyAssoc {
                    name: FullName::new(&assoc_type_namespace, &assoc_ty_name),
                };
                assoc_ty_kind_info.insert(
                    assoc_type.clone(),
                    AssocTypeKindInfo {
                        name: assoc_type,
                        param_kinds: assoc_ty_info.param_kinds(),
                        value_kind: assoc_ty_info.kind_applied.clone(),
                    },
                );
            }
        }
        assoc_ty_kind_info
    }

    // // Reduce a predicate p to a context of trait instance.
    // // For example, reduce `Array a : Eq` to `a : Eq` using instance `impl [a : Eq] Array a : Eq`.
    // // Returns None when p cannot be reduced more.
    // pub fn reduce_to_context_of_instance(
    //     &self,
    //     p: &Predicate,
    //     kind_map: &HashMap<TyCon, Arc<Kind>>,
    // ) -> Option<Vec<Predicate>> {
    //     let insntances = self.instances.get(&p.trait_id);
    //     if let Some(instances) = insntances {
    //         for inst in instances {
    //             match Substitution::matching(&inst.qual_pred.predicate.ty, &p.ty) {
    //                 Some(s) => {
    //                     let ps = inst.qual_pred.context.iter().map(|c| {
    //                         let mut c = c.clone();
    //                         s.substitute_predicate(&mut c);
    //                         c
    //                     });
    //                     let mut ret = vec![];
    //                     for p in ps {
    //                         ret.append(&mut p.resolve_trait_aliases(self));
    //                     }
    //                     return Some(ret);
    //                 }
    //                 None => {}
    //             }
    //         }
    //     }
    //     return None;
    // }

    // // Judge whether a predicate p is entailed by a set of predicates ps.
    // pub fn entail(
    //     &self,
    //     ps: &Vec<Predicate>,
    //     p: &Predicate,
    //     kind_map: &HashMap<TyCon, Arc<Kind>>,
    // ) -> bool {
    //     // Resolve trait aliases in ps.
    //     let mut resolved_ps = vec![];
    //     for p in ps {
    //         resolved_ps.append(&mut p.resolve_trait_aliases(self));
    //     }
    //     let ps = resolved_ps;

    //     p.resolve_trait_aliases(self)
    //         .iter()
    //         .all(|p| self.entail_inner(&ps, p, kind_map))
    // }

    // // Judge whether a predicate p is entailed by a set of predicates ps.
    // // p and ps cannot contain trait aliases.
    // fn entail_inner(
    //     &self,
    //     ps: &Vec<Predicate>,
    //     p: &Predicate,
    //     kind_map: &HashMap<TyCon, Arc<Kind>>,
    // ) -> bool {
    //     // If p is in ps, then ok.
    //     for q in ps {
    //         if q.to_string() == p.to_string() {
    //             return true;
    //         }
    //     }
    //     // Try reducing p by instances.
    //     match self.reduce_to_context_of_instance(p, kind_map) {
    //         Some(ctxs) => {
    //             let mut all_ok = true;
    //             for ctx in ctxs {
    //                 if !self.entail(ps, &ctx, kind_map) {
    //                     all_ok = false;
    //                     break;
    //                 }
    //             }
    //             all_ok
    //         }
    //         None => false,
    //     }
    // }

    // // Reduce a predicate to head normal form.
    // // Returns Err(p) if reduction failed due to predicate p.
    // fn reduce_to_hnfs(
    //     &self,
    //     p: &Predicate,
    //     kind_map: &HashMap<TyCon, Arc<Kind>>,
    // ) -> Result<Vec<Predicate>, Predicate> {
    //     if p.ty.is_hnf() {
    //         return Ok(vec![p.clone()]);
    //     }
    //     match self.reduce_to_context_of_instance(p, kind_map) {
    //         Some(ps) => self.reduce_to_hnfs_many(&ps, kind_map),
    //         None => Err(p.clone()),
    //     }
    // }

    // // Reduce predicates to head normal form.
    // // Returns Err(p) if reduction failed due to predicate p.
    // fn reduce_to_hnfs_many(
    //     &self,
    //     ps: &Vec<Predicate>,
    //     kind_map: &HashMap<TyCon, Arc<Kind>>,
    // ) -> Result<Vec<Predicate>, Predicate> {
    //     let mut ret: Vec<Predicate> = Default::default();
    //     for p in ps {
    //         ret.append(&mut self.reduce_to_hnfs(p, kind_map)?)
    //     }
    //     Ok(ret)
    // }

    // // Simplify a set of predicates by entail.
    // fn reduce_predicates_by_entail(
    //     &self,
    //     ps: &Vec<Predicate>,
    //     kind_map: &HashMap<TyCon, Arc<Kind>>,
    // ) -> Vec<Predicate> {
    //     let mut ps = ps.clone();
    //     let mut i = 0 as usize;
    //     while i < ps.len() {
    //         let qs: Vec<Predicate> = ps
    //             .iter()
    //             .enumerate()
    //             .filter_map(|(j, p)| if i == j { None } else { Some(p.clone()) })
    //             .collect();
    //         if self.entail(&qs, &ps[i], kind_map) {
    //             ps.remove(i);
    //         } else {
    //             i += 1;
    //         }
    //     }
    //     ps
    //     // TODO: Improve performance. See scEntail in "Typing Haskell in Haskell".
    // }

    // // Context reduction.
    // // Returns qs when satisfaction of ps are reduced to qs.
    // // In particular, returns empty when ps are satisfied.
    // // Returns Err(p) if reduction failed due to predicate p.
    // pub fn reduce_to_hnf(
    //     &self,
    //     ps: &Vec<Predicate>,
    //     kind_map: &HashMap<TyCon, Arc<Kind>>,
    // ) -> Result<Vec<Predicate>, Predicate> {
    //     // Resolve trait aliases in ps.
    //     let mut resolved_ps = vec![];
    //     for p in ps {
    //         resolved_ps.append(&mut p.resolve_trait_aliases(self));
    //     }
    //     let ps = resolved_ps;

    //     let ret = self.reduce_to_hnfs_many(&ps, kind_map)?;
    //     let ret = self.reduce_predicates_by_entail(&ret, kind_map);
    //     // Every predicate has to be hnf.
    //     assert!(ret.iter().all(|p| p.ty.is_hnf()));
    //     Ok(ret)
    // }

    // Resolve trait aliases.
    fn resolve_aliases(&self, trait_id: &TraitId) -> Vec<TraitId> {
        fn resolve_aliases_inner(
            env: &TraitEnv,
            trait_id: &TraitId,
            res: &mut Vec<TraitId>,
            visited: &mut HashSet<TraitId>,
        ) {
            if visited.contains(trait_id) {
                error_exit(&format!(
                    "Circular aliasing detected in trait alias `{}`.",
                    trait_id.to_string()
                ));
            }
            visited.insert(trait_id.clone());
            if env.traits.contains_key(trait_id) {
                res.push(trait_id.clone());
                return;
            }
            for v in &env.aliases.get(trait_id).unwrap().value {
                resolve_aliases_inner(env, v, res, visited);
            }
        }

        let mut res = vec![];
        let mut visited = HashSet::new();
        resolve_aliases_inner(self, trait_id, &mut res, &mut visited);
        res
    }

    // Check if a trait name is an alias.
    pub fn is_alias(&self, trait_id: &TraitId) -> bool {
        self.aliases.contains_key(trait_id)
    }

    // Set kinds in TraitInfo, TraitAlias and TraitInstances.
    pub fn set_kinds_in_trait_and_alias_defns(&mut self) {
        for (_id, ti) in &mut self.traits {
            ti.set_trait_kind();
        }
        let mut resolved_aliases: HashMap<TraitId, Vec<TraitId>> = HashMap::new();
        for (id, _) in &self.aliases {
            resolved_aliases.insert(id.clone(), self.resolve_aliases(id));
        }
        for (id, ta) in &mut self.aliases {
            let mut kinds = resolved_aliases
                .get(id)
                .unwrap()
                .iter()
                .map(|id| self.traits.get(id).unwrap().type_var.kind.clone());
            let kind = kinds.next().unwrap();
            for k in kinds {
                if k != kind {
                    error_exit_with_src(
                        &format!(
                            "Kind mismatch in the definition of trait alias `{}`",
                            id.to_string()
                        ),
                        &ta.source,
                    )
                }
            }
            ta.kind = kind;
        }
    }

    pub fn set_kinds_in_trait_instances(&mut self, kind_env: &KindEnv) {
        for (_trait_id, trait_impls) in &mut self.instances {
            for inst in trait_impls {
                inst.set_kinds_in_qual_pred(kind_env);
                let mut assoc_tys = std::mem::replace(&mut inst.assoc_types, HashMap::default());
                for (_, assoc_ty_impl) in &mut assoc_tys {
                    assoc_ty_impl.set_kinds(&inst, kind_env);
                }
                inst.assoc_types = assoc_tys;
            }
        }
    }

    pub fn trait_kind_map_with_aliases(&self) -> HashMap<TraitId, Arc<Kind>> {
        let mut res: HashMap<TraitId, Arc<Kind>> = HashMap::default();
        for (id, ti) in &self.traits {
            res.insert(id.clone(), ti.type_var.kind.clone());
        }
        for (id, ta) in &self.aliases {
            res.insert(id.clone(), ta.kind.clone());
        }
        res
    }

    pub fn import(&mut self, other: TraitEnv) {
        for (_, ti) in other.traits {
            self.add_trait(ti);
        }
        for (_, insts) in other.instances {
            for inst in insts {
                self.add_instance(inst);
            }
        }
        for (_, alias) in other.aliases {
            self.add_alias(alias);
        }
    }
}
