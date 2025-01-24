use std::sync::Arc;

use crate::error::Errors;
use misc::{make_set, Map, Set};
use name::{FullName, Name};
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct PatternNode {
    pub pattern: Pattern,
    pub info: PatternInfo,
}

impl PatternNode {
    // Set `self.info.inferred_ty`.
    // Returns the pattern itself with a map which maps variable names to their types.
    pub fn get_typed(
        self: &Arc<PatternNode>,
        typechcker: &mut TypeCheckContext,
    ) -> Result<(Arc<PatternNode>, Map<FullName, Arc<TypeNode>>), Errors> {
        match &self.pattern {
            Pattern::Var(v, ty) => {
                let var_name = v.name.clone();
                let ty = if ty.is_none() {
                    let tv = typechcker
                        .new_tyvar_star()
                        .set_source(self.info.source.clone());
                    type_from_tyvar(tv)
                } else {
                    let ty = ty.as_ref().unwrap();
                    typechcker.validate_type_annotation(ty)?
                };
                let mut var_to_ty = Map::default();
                var_to_ty.insert(var_name, ty.clone());
                Ok((self.set_inferred_type(ty), var_to_ty))
            }
            Pattern::Struct(tc, field_to_pat) => {
                let ty = tc.get_struct_union_value_type(typechcker);
                let mut var_to_ty = Map::default();
                let field_tys = ty.field_types(&typechcker.type_env);
                let fields = &typechcker.type_env.tycons.get(&tc).unwrap().fields;
                assert_eq!(fields.len(), field_tys.len());
                let field_name_to_ty = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let ty = field_tys[i].clone();
                        (field.name.clone(), ty)
                    })
                    .collect::<Map<_, _>>();
                let mut field_to_pat = field_to_pat.clone();
                for (field_name, pat) in &mut field_to_pat {
                    let (typed_pat, var_ty) = pat.get_typed(typechcker)?;
                    *pat = typed_pat;
                    var_to_ty.extend(var_ty);
                    let unify_res = UnifOrOtherErr::extract_others(typechcker.unify(
                        &pat.info.inferred_ty.as_ref().unwrap(),
                        field_name_to_ty.get(field_name).unwrap(),
                    ))?;
                    if unify_res.is_err() {
                        return Err(Errors::from_msg_srcs(
                            format!(
                                "Inappropriate pattern `{}` for a value of field `{}` of struct `{}`.",
                                pat.pattern.to_string(),
                                field_name,
                                tc.to_string(),
                            ),
                            &[&pat.info.source],
                        ));
                    }
                }
                Ok((
                    self.set_inferred_type(ty)
                        .set_struct_field_to_pat(field_to_pat),
                    var_to_ty,
                ))
            }
            Pattern::Union(variant_name, subpat) => {
                let (variant_idx, tc, _ti) =
                    Pattern::get_variant_info(&variant_name, &typechcker.type_env);

                // Get the union type and variant type.
                let union_ty = tc.get_struct_union_value_type(typechcker);
                let variant_ty = union_ty.field_types(&typechcker.type_env)[variant_idx].clone();

                // Infer the type of the subpattern.
                let (subpat, var_ty) = subpat.get_typed(typechcker)?;

                // Unify the type of the subpattern with the type of the variant.
                let unify_res = UnifOrOtherErr::extract_others(
                    typechcker.unify(&subpat.info.inferred_ty.as_ref().unwrap(), &variant_ty),
                )?;
                if unify_res.is_err() {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Inappropriate pattern `{}` for a value of variant `{}` of union `{}`.",
                            subpat.pattern.to_string(),
                            variant_name.to_string(),
                            tc.to_string(),
                        ),
                        &[&subpat.info.source],
                    ));
                }

                // Return the typed pattern.
                Ok((
                    self.set_inferred_type(union_ty).set_union_pat(subpat),
                    var_ty,
                ))
            }
        }
    }

    // Find the node at the specified position.
    pub fn find_node_at_pos(self: &Arc<PatternNode>, pos: &SourcePos) -> Option<EndNode> {
        if self.info.source.is_none() {
            return None;
        }
        let span = self.info.source.as_ref().unwrap();
        if !span.includes_pos(pos) {
            return None;
        }
        match &self.pattern {
            Pattern::Var(v, ty) => {
                if ty.is_some() {
                    let ty = ty.as_ref().unwrap();
                    let node = ty.find_node_at(pos);
                    if node.is_some() {
                        return node;
                    }
                }
                Some(EndNode::Pattern(
                    v.as_ref().clone(),
                    self.info.inferred_ty.clone(),
                ))
            }
            Pattern::Struct(tc, field_to_pat) => {
                for (_, pat) in field_to_pat {
                    let res = pat.find_node_at_pos(pos);
                    if res.is_some() {
                        return res;
                    }
                }
                Some(EndNode::Type(tc.as_ref().clone()))
            }
            Pattern::Union(_variant, subpat) => {
                let node = subpat.find_node_at_pos(pos);
                if node.is_some() {
                    return node;
                }
                None
            }
        }
    }

    // Validate pattern and raise error if invalid,
    pub fn validate(&self, te: &TypeEnv) -> Result<(), Errors> {
        match &self.pattern {
            Pattern::Var(_, _) => {}
            Pattern::Struct(tc, pats) => {
                let ti = te.tycons.get(&tc).unwrap();
                let fields_str = ti.fields.iter().map(|f| f.name.clone()).collect::<Set<_>>();
                let fields_pat = pats
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Set<_>>();
                if fields_pat.len() < pats.len() {
                    return Err(Errors::from_msg_srcs(
                        "Duplicate field in struct pattern.".to_string(),
                        &[&self.info.source],
                    ));
                }
                for f in fields_pat {
                    if !fields_str.contains(&f) {
                        return Err(Errors::from_msg_srcs(
                            format!(
                                "Unknown field `{}` for struct `{}`.",
                                f,
                                tc.name.to_string()
                            ),
                            &[&self.info.source],
                        ));
                    }
                }
                for (_, p) in pats {
                    p.validate(te)?;
                }
            }
            Pattern::Union(_, subpat) => {
                subpat.validate(te)?;
            }
        }
        if self.pattern.has_duplicate_vars() {
            return Err(Errors::from_msg_srcs(
                "Duplicate name defined by pattern.".to_string(),
                &[&self.info.source],
            ));
        }
        Ok(())
    }

    pub fn resolve_namespace(
        self: &PatternNode,
        ctx: &NameResolutionContext,
    ) -> Result<Arc<PatternNode>, Errors> {
        match &self.pattern {
            Pattern::Var(_, ty) => {
                let ty = ty
                    .as_ref()
                    .map(|ty| ty.resolve_namespace(ctx))
                    .transpose()?;
                Ok(self.set_var_tyanno(ty))
            }
            Pattern::Struct(tc, field_to_pat) => {
                let mut tc = tc.as_ref().clone();
                tc.resolve_namespace(ctx, &self.info.source)?;
                let mut field_to_pat_res = vec![];
                for (field_name, pat) in field_to_pat {
                    field_to_pat_res.push((field_name.clone(), pat.resolve_namespace(ctx)?));
                }
                Ok(self
                    .set_struct_tycon(Arc::new(tc))
                    .set_struct_field_to_pat(field_to_pat_res))
            }
            Pattern::Union(_, subpat) => {
                // Namespace of the variant name is resolved in the type-checking phase (`validate_variant_name`).
                let subpat = subpat.resolve_namespace(ctx)?;
                Ok(self.set_union_pat(subpat))
            }
        }
    }

    pub fn resolve_type_aliases(
        self: &PatternNode,
        type_env: &TypeEnv,
    ) -> Result<Arc<PatternNode>, Errors> {
        match &self.pattern {
            Pattern::Var(_, ty) => Ok(self.set_var_tyanno(
                ty.as_ref()
                    .map(|ty| ty.resolve_type_aliases(type_env))
                    .transpose()?,
            )),
            Pattern::Struct(tc, field_to_pat) => {
                if type_env.aliases.contains_key(tc) {
                    return Err(Errors::from_msg_srcs(
                        "In struct pattern, cannot use type alias instead of struct name."
                            .to_string(),
                        &[&self.info.source],
                    ));
                }
                let mut field_to_pat_res = vec![];
                for (field_name, pat) in field_to_pat {
                    field_to_pat_res
                        .push((field_name.clone(), pat.resolve_type_aliases(type_env)?));
                }
                Ok(self.set_struct_field_to_pat(field_to_pat_res))
            }
            Pattern::Union(_, subpat) => {
                let subpat = subpat.resolve_type_aliases(type_env)?;
                Ok(self.set_union_pat(subpat))
            }
        }
    }

    pub fn set_var_tyanno(self: &PatternNode, tyanno: Option<Arc<TypeNode>>) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &self.pattern {
            Pattern::Var(v, _) => {
                node.pattern = Pattern::Var(v.clone(), tyanno);
            }
            _ => panic!(),
        }
        Arc::new(node)
    }

    pub fn set_struct_tycon(self: &PatternNode, tc: Arc<TyCon>) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &self.pattern {
            Pattern::Struct(_, field_to_pat) => {
                node.pattern = Pattern::Struct(tc, field_to_pat.clone());
            }
            _ => panic!(),
        }
        Arc::new(node)
    }

    pub fn set_struct_field_to_pat(
        self: &PatternNode,
        field_to_pat: Vec<(Name, Arc<PatternNode>)>,
    ) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &self.pattern {
            Pattern::Struct(tc, _) => {
                node.pattern = Pattern::Struct(tc.clone(), field_to_pat);
            }
            _ => panic!(),
        }
        Arc::new(node)
    }

    pub fn set_union_pat(self: &PatternNode, pat: Arc<PatternNode>) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &self.pattern {
            Pattern::Union(variant, _) => {
                node.pattern = Pattern::Union(variant.clone(), pat);
            }
            _ => panic!(),
        }
        Arc::new(node)
    }

    pub fn get_union_variant(&self) -> &FullName {
        match &self.pattern {
            Pattern::Union(variant, _) => variant,
            _ => panic!(),
        }
    }

    pub fn is_union(&self) -> bool {
        matches!(&self.pattern, Pattern::Union(_, _))
    }

    pub fn is_var(&self) -> bool {
        matches!(&self.pattern, Pattern::Var(_, _))
    }

    pub fn get_var(&self) -> Arc<Var> {
        match &self.pattern {
            Pattern::Var(v, _) => v.clone(),
            _ => panic!(),
        }
    }

    pub fn set_source(self: &PatternNode, src: Span) -> Arc<PatternNode> {
        let mut node = self.clone();
        node.info.source = Some(src);
        Arc::new(node)
    }

    pub fn set_inferred_type(self: &PatternNode, ty: Arc<TypeNode>) -> Arc<PatternNode> {
        let mut node = self.clone();
        node.info.inferred_ty = Some(ty);
        Arc::new(node)
    }

    pub fn make_var(var: Arc<Var>, ty: Option<Arc<TypeNode>>) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Var(var, ty),
            info: PatternInfo::default(),
        })
    }

    pub fn make_struct(
        tycon: Arc<TyCon>,
        fields: Vec<(Name, Arc<PatternNode>)>,
    ) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Struct(tycon, fields),
            info: PatternInfo::default(),
        })
    }

    pub fn make_union(variant: FullName, subpat: Arc<PatternNode>) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Union(variant, subpat),
            info: PatternInfo::default(),
        })
    }

    // Validate the variant name of `Union` pattern.
    pub fn validate_variant_name(
        self: &PatternNode,
        cond_tycon: &TyCon,
        cond_ti: &TyConInfo,
    ) -> Result<Arc<PatternNode>, Errors> {
        let name_space = cond_tycon.name.to_namespace();
        match &self.pattern {
            Pattern::Union(variant, subpat) => {
                // Check the variant name.
                let is_ns_ok = variant.namespace.is_suffix_of(&name_space);
                let is_name_ok = cond_ti.fields.iter().any(|f| &f.name == &variant.name);
                if !is_ns_ok || !is_name_ok {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "`{}` is not a variant of union `{}`.",
                            variant.to_string(),
                            cond_tycon.name.to_string()
                        ),
                        &[&self.info.source],
                    ));
                }

                // Then, complete the namespace of the variant name.
                let mut variant = variant.clone();
                variant.namespace = name_space;
                Ok(Arc::new(PatternNode {
                    pattern: Pattern::Union(variant, subpat.clone()),
                    info: self.info.clone(),
                }))
            }
            _ => panic!(),
        }
    }

    // Rename a variable name in the pattern.
    pub fn rename_var(&self, from: &FullName, to: &FullName) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &mut node.pattern {
            Pattern::Var(v, _) => {
                if &v.name == from {
                    let new_v = v.set_name(to.clone());
                    *v = new_v;
                }
            }
            Pattern::Struct(_, fields) => {
                for (_, pat) in fields {
                    *pat = pat.rename_var(from, to);
                }
            }
            Pattern::Union(_, pat) => {
                *pat = pat.rename_var(from, to);
            }
        }
        let ret = Arc::new(node);
        if ret.pattern.has_duplicate_vars() {
            panic!(
                "Duplicate variable names in pattern: {}",
                ret.pattern.to_string()
            );
        }
        ret
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PatternInfo {
    pub inferred_ty: Option<Arc<TypeNode>>,
    pub source: Option<Span>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Pattern {
    Var(Arc<Var>, Option<Arc<TypeNode>>),
    Struct(Arc<TyCon>, Vec<(Name, Arc<PatternNode>)>),
    Union(FullName, Arc<PatternNode>),
}

impl Pattern {
    // Make basic variable pattern.
    #[allow(dead_code)]
    pub fn var_pattern(var: Arc<Var>) -> Arc<Pattern> {
        Arc::new(Pattern::Var(var, None))
    }

    // Check if variables defined in this pattern is duplicated or not.
    // For example, pattern (x, y) is ok, but (x, x) is invalid.
    pub fn has_duplicate_vars(&self) -> bool {
        (self.vars().len() as u32) < self.count_vars()
    }

    // Count variables defined in this pattern.
    fn count_vars(&self) -> u32 {
        match self {
            Pattern::Var(_, _) => 1,
            Pattern::Struct(_, field_to_pat) => {
                let mut ret = 0;
                for (_, pat) in field_to_pat {
                    ret += pat.pattern.count_vars();
                }
                ret
            }
            Pattern::Union(_, pat) => pat.pattern.count_vars(),
        }
    }

    // Calculate the set of variables that appears in this pattern.
    pub fn vars(&self) -> Set<FullName> {
        match self {
            Pattern::Var(var, _) => make_set([var.name.clone()]),
            Pattern::Struct(_, pats) => {
                let mut ret = Set::default();
                for (_, pat) in pats {
                    ret.extend(pat.pattern.vars());
                }
                ret
            }
            Pattern::Union(_, pat) => pat.pattern.vars(),
        }
    }

    pub fn to_string(&self) -> String {
        let mut ret = "".to_string();
        match self {
            Pattern::Var(v, t) => {
                ret += &v.name.to_string();
                match t {
                    Some(t) => {
                        ret += ": ";
                        ret += &t.to_string();
                    }
                    None => {}
                }
                ret
            }
            Pattern::Struct(tc, fields) => {
                if let Some(n) = get_tuple_n(&tc.name) {
                    let pats = fields
                        .iter()
                        .map(|(_, pat)| pat.pattern.to_string())
                        .collect::<Vec<_>>();
                    if n == 1 {
                        format!("({},)", pats[0])
                    } else {
                        format!("({})", pats.join(", "))
                    }
                } else {
                    let pats = fields
                        .iter()
                        .map(|(name, pat)| format!("{}: {}", name, pat.pattern.to_string()))
                        .collect::<Vec<_>>();
                    format!("{} {{{}}}", tc.to_string(), pats.join(", "))
                }
            }
            Pattern::Union(variant, pat) => {
                format!("{}({})", variant.to_string(), pat.pattern.to_string())
            }
        }
    }

    // From a fully-resolved variant name, gets the variant index, the type constructor of the union, and the type constructor info.
    pub fn get_variant_info<'a, 'b>(
        variant_name: &'a FullName,
        type_env: &'b TypeEnv,
    ) -> (usize, TyCon, &'b TyConInfo) {
        let tc: TyCon = TyCon::new(variant_name.namespace.clone().to_fullname());
        let ti = type_env.tycons.get(&tc).unwrap();
        let variant_idx = ti
            .fields
            .iter()
            .position(|v: &Field| &v.name == &variant_name.name)
            .unwrap();
        (variant_idx, tc, ti)
    }

    // Checks if patterns which are used in `match` syntax are exhaustive.
    pub fn validate_match_cases_exhaustiveness(
        cond_tc: &TyCon,
        cond_ti: &TyConInfo,
        match_src: &Option<Span>,
        pats: impl Iterator<Item = Arc<PatternNode>>,
    ) -> Result<(), Errors> {
        let mut variants = cond_ti.fields.iter().map(|f| &f.name).collect::<Set<_>>();
        let mut otherwise: Option<Arc<PatternNode>> = None;
        for pat in pats {
            if let Some(otherwise) = otherwise {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Pattern after `{}` is unreachable.",
                        otherwise.pattern.to_string()
                    ),
                    &[&pat.info.source],
                ));
            }
            match &pat.pattern {
                Pattern::Union(variant, _) => {
                    if !variants.contains(&variant.name) {
                        return Err(Errors::from_msg_srcs(
                            format!(
                                "`{}` is not a variant of union `{}`.",
                                variant.to_string(),
                                cond_tc.to_string()
                            ),
                            &[&pat.info.source],
                        ));
                    }
                    variants.remove(&variant.name);
                }
                _ => {
                    otherwise = Some(pat.clone());
                }
            }
        }
        if otherwise.is_none() && !variants.is_empty() {
            let msg = if variants.len() == 1 {
                format!(
                    "Variant `{}` of union `{}` is not covered.",
                    variants.iter().next().unwrap(),
                    cond_tc.to_string()
                )
            } else {
                format!(
                    "Variants {} of union `{}` are not covered.",
                    variants
                        .iter()
                        .map(|var| format!("`{}`", var))
                        .collect::<Vec<_>>()
                        .join(", "),
                    cond_tc.to_string()
                )
            };
            return Err(Errors::from_msg_srcs(msg, &[&match_src]));
        }
        Ok(())
    }
}
