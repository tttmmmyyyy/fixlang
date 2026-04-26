use std::sync::Arc;
use crate::ast::expr::Var;
use crate::ast::name::{FullName, Name};
use crate::ast::program::{EndNode, TypeEnv};
use crate::ast::typedecl::Field;
use crate::ast::types::{TyCon, TyConInfo, TypeNode, type_from_tyvar};
use crate::elaboration::name_resolution::NameResolutionContext;
use crate::elaboration::typecheck::{TypeCheckContext, UnifOrOtherErr};
use crate::error::Errors;
use crate::fixstd::builtin::get_tuple_n;
use crate::misc::{make_set, Map, Set};
use crate::parse::sourcefile::{SourcePos, Span};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PatternNode {
    pub pattern: Pattern,
    pub info: PatternInfo,
}

impl PatternNode {
    // Assign types to the pattern so that it has the given type.
    //
    // All types must have their type aliases resolved and associated types expanded.
    //
    // Note:
    // This function ignores user-provided type annotations.
    // Specifically, if the pattern is `v : A` and the type `B` is given, then type `B` is assigned to `v` even if `A != B`.
    // Therefore, this function must not be used for type checking. It is used in a process after type checking has succeeded.
    #[allow(dead_code)]
    pub fn get_typed_matching(
        &self,
        type_: &Arc<TypeNode>,
        type_env: &TypeEnv,
    ) -> Option<Arc<PatternNode>> {
        match &self.pattern {
            Pattern::Var(_v, _ty) => {
                // IGNORES user-provided type annotation!
                // if let Some(ty) = ty {
                //     if ty.to_string_normalize() != type_.to_string_normalize() {
                //         return None;
                //     }
                // }
                let pat = self.set_type(type_.clone());
                Some(pat)
            }
            Pattern::Struct(tc, field_to_pat) => {
                let type_tc = type_.toplevel_tycon();
                if type_tc.is_none() {
                    return None;
                }
                let type_tc = type_tc.unwrap();
                if type_tc.as_ref() != tc.as_ref() {
                    return None;
                }

                let type_ti = type_env.tycons.get(&type_tc)?;
                let mut field_name_to_idx = Map::default();
                for (i, field) in type_ti.fields.iter().enumerate() {
                    field_name_to_idx.insert(field.name.clone(), i);
                }

                // Recursively match each field pattern with its expected type
                let field_types = type_.field_types(type_env);
                let mut field_to_pat = field_to_pat.clone();
                for (field_name, _, pat) in field_to_pat.iter_mut() {
                    let field_idx = *field_name_to_idx.get(field_name)?;
                    let field_ty = &field_types[field_idx];
                    let matched_pat = pat.get_typed_matching(field_ty, type_env)?;
                    *pat = matched_pat;
                }

                Some(
                    self.set_type(type_.clone())
                        .set_struct_field_to_pat(field_to_pat),
                )
            }
            Pattern::Union(variant_name, _, subpat) => {
                let tc = TyCon::new(variant_name.namespace.clone().to_fullname());
                let variant_name = &variant_name.name;

                let type_tc = type_.toplevel_tycon();
                if type_tc.is_none() {
                    return None;
                }
                let type_tc = type_tc.unwrap();
                if type_tc.as_ref() != &tc {
                    return None;
                }

                let type_ti = type_env.tycons.get(&type_tc)?;
                let mut variant_name_to_idx = Map::default();
                for (i, field) in type_ti.fields.iter().enumerate() {
                    variant_name_to_idx.insert(field.name.clone(), i);
                }

                // Recursively match each field pattern with its expected type
                let variant_types = type_.field_types(type_env);
                let variant_idx = *variant_name_to_idx.get(variant_name)?;
                let variant_ty = &variant_types[variant_idx];
                let subpat = subpat.get_typed_matching(variant_ty, type_env)?;

                Some(self.set_type(type_.clone()).set_union_pat(subpat))
            }
        }
    }

    // Set `self.info.type_`.
    // Returns the pattern itself with a map which maps variable names to their types.
    pub fn get_typed(
        self: &Arc<PatternNode>,
        typechcker: &mut TypeCheckContext,
    ) -> Result<(Arc<PatternNode>, Map<FullName, Arc<TypeNode>>), Errors> {
        match &self.pattern {
            Pattern::Var(v, ty) => {
                let var_name = v.name.clone();
                let ty = if ty.is_none() {
                    let tv = typechcker.new_tyvar_star();
                    typechcker.add_tyvar_source(tv.name.clone(), self.info.source.clone());
                    type_from_tyvar(tv)
                } else {
                    let ty = ty.as_ref().unwrap();
                    typechcker.validate_type_annotation(ty)?
                };
                let mut var_to_ty = Map::default();
                var_to_ty.insert(var_name, ty.clone());
                Ok((self.set_type(ty), var_to_ty))
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
                for (field_name, _, pat) in &mut field_to_pat {
                    let (typed_pat, var_ty) = pat.get_typed(typechcker)?;
                    *pat = typed_pat;
                    var_to_ty.extend(var_ty);
                    let unify_res = UnifOrOtherErr::extract_others(typechcker.unify(
                        &pat.info.type_.as_ref().unwrap(),
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
                    self.set_type(ty).set_struct_field_to_pat(field_to_pat),
                    var_to_ty,
                ))
            }
            Pattern::Union(variant_name, _, subpat) => {
                let (variant_idx, tc, _ti) =
                    Pattern::get_variant_info(&variant_name, &typechcker.type_env);

                // Get the union type and variant type.
                let union_ty = tc.get_struct_union_value_type(typechcker);
                let variant_ty = union_ty.field_types(&typechcker.type_env)[variant_idx].clone();

                // Infer the type of the subpattern.
                let (subpat, var_ty) = subpat.get_typed(typechcker)?;

                // Unify the type of the subpattern with the type of the variant.
                let unify_res = UnifOrOtherErr::extract_others(
                    typechcker.unify(&subpat.info.type_.as_ref().unwrap(), &variant_ty),
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
                Ok((self.set_type(union_ty).set_union_pat(subpat), var_ty))
            }
        }
    }

    // For every `Pattern::Var` in this pattern tree, return its name and
    // the `PatternInfo` of the `PatternNode` wrapping it — so callers get
    // the binder's type and source span alongside the name. Sub-patterns
    // of `Struct` / `Union` are walked left-to-right.
    pub fn var_infos(&self) -> Vec<(FullName, PatternInfo)> {
        let mut out = vec![];
        self.collect_var_infos(&mut out);
        out
    }

    fn collect_var_infos(&self, out: &mut Vec<(FullName, PatternInfo)>) {
        match &self.pattern {
            Pattern::Var(v, _) => out.push((v.name.clone(), self.info.clone())),
            Pattern::Struct(_, fields) => {
                for (_, _, sub) in fields {
                    sub.collect_var_infos(out);
                }
            }
            Pattern::Union(_, _, sub) => sub.collect_var_infos(out),
        }
    }

    // Find the node at the specified position.
    pub fn find_node_at_pos(self: &Arc<PatternNode>, pos: &SourcePos) -> Option<EndNode> {
        if self.info.source.is_none() {
            return None;
        }
        let span = self.info.source.as_ref().unwrap();
        if !span.includes_pos_lsp(pos) {
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
                    self.info.type_.clone(),
                ))
            }
            Pattern::Struct(tc, field_to_pat) => {
                // Check if cursor is on any field-name span first.
                for (name, name_src, pat) in field_to_pat {
                    if let Some(ns) = name_src {
                        if ns.includes_pos_lsp(pos) {
                            return Some(EndNode::Field(
                                tc.as_ref().clone(),
                                name.clone(),
                            ));
                        }
                    }
                    let res = pat.find_node_at_pos(pos);
                    if res.is_some() {
                        return res;
                    }
                }
                Some(EndNode::Type(tc.as_ref().clone()))
            }
            Pattern::Union(variant, variant_src, subpat) => {
                if let Some(vs) = variant_src {
                    if vs.includes_pos_lsp(pos) && !variant.namespace.names.is_empty() {
                        let tc = TyCon::new(variant.namespace.clone().to_fullname());
                        return Some(EndNode::Variant(tc, variant.name.clone()));
                    }
                }
                let node = subpat.find_node_at_pos(pos);
                if node.is_some() {
                    return node;
                }
                None
            }
        }
    }

    pub fn resolve_namespace(
        self: &PatternNode,
        ctx: &mut NameResolutionContext,
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
                let mut field_to_pat = field_to_pat.clone();
                for (_, _, pat) in &mut field_to_pat {
                    *pat = pat.resolve_namespace(ctx)?;
                }
                Ok(self
                    .set_struct_tycon(Arc::new(tc))
                    .set_struct_field_to_pat(field_to_pat))
            }
            Pattern::Union(_, _, subpat) => {
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
                let mut field_to_pat = field_to_pat.clone();
                for (_, _, pat) in &mut field_to_pat {
                    *pat = pat.resolve_type_aliases(type_env)?;
                }
                Ok(self.set_struct_field_to_pat(field_to_pat))
            }
            Pattern::Union(_, _, subpat) => {
                let subpat = subpat.resolve_type_aliases(type_env)?;
                Ok(self.set_union_pat(subpat))
            }
        }
    }

    // Convert all global FullNames to absolute paths.
    pub fn global_to_absolute(&self) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &self.pattern {
            Pattern::Var(v, ty) => {
                let new_v = v.global_to_absolute();
                let new_ty = ty.as_ref().map(|t| t.global_to_absolute());
                node.pattern = Pattern::Var(new_v, new_ty);
            }
            Pattern::Struct(tc, field_to_pat) => {
                let new_tc = tc.global_to_absolute();
                let mut field_to_pat = field_to_pat.clone();
                for (_, _, pat) in &mut field_to_pat {
                    *pat = pat.global_to_absolute();
                }
                node.pattern = Pattern::Struct(new_tc, field_to_pat);
            }
            Pattern::Union(variant_name, variant_src, subpat) => {
                let mut new_variant_name = variant_name.clone();
                new_variant_name.global_to_absolute();
                let new_subpat = subpat.global_to_absolute();
                node.pattern =
                    Pattern::Union(new_variant_name, variant_src.clone(), new_subpat);
            }
        }
        Arc::new(node)
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
        field_to_pat: Vec<(Name, Option<Span>, Arc<PatternNode>)>,
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
            Pattern::Union(variant, variant_src, _) => {
                node.pattern = Pattern::Union(variant.clone(), variant_src.clone(), pat);
            }
            _ => panic!(),
        }
        Arc::new(node)
    }

    pub fn get_union_variant(&self) -> &FullName {
        match &self.pattern {
            Pattern::Union(variant, _, _) => variant,
            _ => panic!(),
        }
    }

    pub fn is_union(&self) -> bool {
        matches!(&self.pattern, Pattern::Union(_, _, _))
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

    pub fn set_aux_src(self: &PatternNode, src: Span) -> Arc<PatternNode> {
        let mut node = self.clone();
        node.info.aux_src = Some(src);
        Arc::new(node)
    }

    pub fn set_type(self: &PatternNode, ty: Arc<TypeNode>) -> Arc<PatternNode> {
        let mut node = self.clone();
        node.info.type_ = Some(ty);
        Arc::new(node)
    }

    pub fn make_var(var: Arc<Var>, anno_ty: Option<Arc<TypeNode>>) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Var(var, anno_ty),
            info: PatternInfo::default(),
        })
    }

    // Construct a struct destructuring pattern from `(field name,
    // sub-pattern)` pairs, with no per-field-name source spans (the
    // entries get `None` spans).
    pub fn make_struct(
        tycon: Arc<TyCon>,
        fields: Vec<(Name, Arc<PatternNode>)>,
    ) -> Arc<PatternNode> {
        let fields = fields.into_iter().map(|(n, p)| (n, None, p)).collect();
        Arc::new(PatternNode {
            pattern: Pattern::Struct(tycon, fields),
            info: PatternInfo::default(),
        })
    }

    // Construct a struct destructuring pattern from `(field name,
    // optional field-name source span, sub-pattern)` triples.
    pub fn make_struct_with_spans(
        tycon: Arc<TyCon>,
        fields: Vec<(Name, Option<Span>, Arc<PatternNode>)>,
    ) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Struct(tycon, fields),
            info: PatternInfo::default(),
        })
    }

    // Construct a union match pattern with the variant name's source span.
    pub fn make_union_with_span(
        variant: FullName,
        variant_src: Option<Span>,
        subpat: Arc<PatternNode>,
    ) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Union(variant, variant_src, subpat),
            info: PatternInfo::default(),
        })
    }

    // Validate the variant name of a `Union` pattern against the union
    // type being matched, and return a normalized copy of the pattern.
    //
    // Validation: the parsed variant name's namespace must be a suffix of
    // the union's namespace, and the bare variant name must be one of the
    // union's variants.
    //
    // Normalization (the reason this returns a new pattern rather than
    // just `Result<(), Errors>`): the user may have written the variant
    // unqualified (e.g. `some(v)`) or partially qualified (e.g.
    // `Maybe::some(v)`); we replace its namespace with the union's full
    // namespace so downstream code can treat the variant as a single,
    // fully-qualified `FullName` without re-resolving it.
    pub fn validate_variant_name(
        self: &PatternNode,
        cond_tycon: &TyCon,
        cond_ti: &TyConInfo,
    ) -> Result<Arc<PatternNode>, Errors> {
        let name_space = cond_tycon.name.to_namespace();
        match &self.pattern {
            Pattern::Union(variant, variant_src, subpat) => {
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
                    pattern: Pattern::Union(variant, variant_src.clone(), subpat.clone()),
                    info: self.info.clone(),
                }))
            }
            _ => panic!(),
        }
    }

    // Rename names in the pattern.
    pub fn rename_by_map(&self, rename: &Map<FullName, FullName>) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &mut node.pattern {
            Pattern::Var(v, _) => {
                if let Some(new_name) = rename.get(&v.name) {
                    let new_v = v.set_name(new_name.clone());
                    *v = new_v;
                }
            }
            Pattern::Struct(_, fields) => {
                for (_, _, pat) in fields {
                    *pat = pat.rename_by_map(rename);
                }
            }
            Pattern::Union(_, _, pat) => {
                *pat = pat.rename_by_map(rename);
            }
        }
        Arc::new(node)
    }

    fn to_string_internal(&self, with_type: bool) -> String {
        let pat_str = self.pattern.to_string_internal(with_type);
        if with_type {
            format!(
                "{}<{}>",
                pat_str,
                self.info
                    .type_
                    .as_ref()
                    .map_or("na".to_string(), |t| t.to_string())
            )
        } else {
            pat_str
        }
    }

    pub fn to_string(&self) -> String {
        self.to_string_internal(false)
    }

    #[allow(dead_code)]
    pub fn to_string_with_type(&self) -> String {
        self.to_string_internal(true)
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PatternInfo {
    pub type_: Option<Arc<TypeNode>>,
    pub source: Option<Span>,
    // Auxiliary source span that depends on the pattern variant.
    // For Struct patterns: the source span of the type constructor name only.
    pub aux_src: Option<Span>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Pattern {
    Var(Arc<Var>, Option<Arc<TypeNode>>),
    // Struct destructuring pattern. Each entry is (field name, optional
    // source span of just the field name, sub-pattern).
    Struct(Arc<TyCon>, Vec<(Name, Option<Span>, Arc<PatternNode>)>),
    // Union match pattern. The optional Span is the source span of just the
    // variant name (without any namespace prefix).
    Union(FullName, Option<Span>, Arc<PatternNode>),
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
                for (_, _, pat) in field_to_pat {
                    ret += pat.pattern.count_vars();
                }
                ret
            }
            Pattern::Union(_, _, pat) => pat.pattern.count_vars(),
        }
    }

    // Calculate the set of variables that appears in this pattern.
    pub fn vars(&self) -> Set<FullName> {
        match self {
            Pattern::Var(var, _) => make_set([var.name.clone()]),
            Pattern::Struct(_, pats) => {
                let mut ret = Set::default();
                for (_, _, pat) in pats {
                    ret.extend(pat.pattern.vars());
                }
                ret
            }
            Pattern::Union(_, _, pat) => pat.pattern.vars(),
        }
    }

    pub fn to_string(&self) -> String {
        self.to_string_internal(false)
    }

    fn to_string_internal(&self, with_type: bool) -> String {
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
                        .map(|(_, _, pat)| pat.to_string_internal(with_type))
                        .collect::<Vec<_>>();
                    if n == 1 {
                        format!("({},)", pats[0])
                    } else {
                        format!("({})", pats.join(", "))
                    }
                } else {
                    let pats = fields
                        .iter()
                        .map(|(name, _, pat)| {
                            format!("{}: {}", name, pat.to_string_internal(with_type))
                        })
                        .collect::<Vec<_>>();
                    format!("{} {{{}}}", tc.to_string(), pats.join(", "))
                }
            }
            Pattern::Union(variant, _, pat) => {
                format!(
                    "{}({})",
                    variant.to_string(),
                    pat.to_string_internal(with_type)
                )
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
        let mut found_otherwise = false;
        for pat in pats {
            match &pat.pattern {
                Pattern::Union(variant, _, _) => {
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
                    found_otherwise = true;
                }
            }
        }
        if !found_otherwise && !variants.is_empty() {
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
