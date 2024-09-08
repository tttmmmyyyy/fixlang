use std::sync::Arc;

use crate::error::error_exit_with_src;
use crate::error::Errors;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct PatternNode {
    pub pattern: Pattern,
    pub info: PatternInfo,
}

impl PatternNode {
    // Validate pattern and raise error if invalid,
    pub fn validate(&self, te: &TypeEnv) -> Result<(), Errors> {
        match &self.pattern {
            Pattern::Var(_, _) => {}
            Pattern::Struct(tc, pats) => {
                let ti = te.tycons.get(&tc).unwrap();
                let fields_str = ti
                    .fields
                    .iter()
                    .map(|f| f.name.clone())
                    .collect::<HashSet<_>>();
                let fields_pat = pats
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<HashSet<_>>();
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
            Pattern::Union(tc, field, pat) => {
                let ti = te.tycons.get(&tc).unwrap();
                if ti.fields.iter().find(|f| &f.name == field).is_none() {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Unknown variant `{}` for union `{}`.",
                            field,
                            tc.name.to_string()
                        ),
                        &[&self.info.source],
                    ));
                }
                pat.validate(te)?;
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
            Pattern::Union(tc, _, pat) => {
                let mut tc = tc.as_ref().clone();
                tc.resolve_namespace(ctx, &self.info.source)?;
                Ok(self
                    .set_union_tycon(Arc::new(tc))
                    .set_union_pat(pat.resolve_namespace(ctx)?))
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
            Pattern::Union(tc, _, pat) => {
                if type_env.aliases.contains_key(tc) {
                    return Err(Errors::from_msg_srcs(
                        "In union pattern, cannot use type alias instead of union name."
                            .to_string(),
                        &[&self.info.source],
                    ));
                }
                Ok(self.set_union_pat(pat.resolve_type_aliases(type_env)?))
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

    pub fn set_union_tycon(self: &PatternNode, tc: Arc<TyCon>) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &self.pattern {
            Pattern::Union(_, field_name, pat) => {
                node.pattern = Pattern::Union(tc, field_name.clone(), pat.clone());
            }
            _ => panic!(),
        }
        Arc::new(node)
    }

    pub fn set_union_pat(self: &PatternNode, pat: Arc<PatternNode>) -> Arc<PatternNode> {
        let mut node = self.clone();
        match &self.pattern {
            Pattern::Union(tc, field_name, _) => {
                node.pattern = Pattern::Union(tc.clone(), field_name.clone(), pat);
            }
            _ => panic!(),
        }
        Arc::new(node)
    }

    pub fn set_source(self: &PatternNode, src: Span) -> Arc<PatternNode> {
        let mut node = self.clone();
        node.info.source = Some(src);
        Arc::new(node)
    }

    pub fn make_var(var: Arc<Var>, ty: Option<Arc<TypeNode>>) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Var(var, ty),
            info: PatternInfo { source: None },
        })
    }

    pub fn make_struct(
        tycon: Arc<TyCon>,
        fields: Vec<(Name, Arc<PatternNode>)>,
    ) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Struct(tycon, fields),
            info: PatternInfo { source: None },
        })
    }

    pub fn make_union(
        tycon: Arc<TyCon>,
        field: Name,
        subpat: Arc<PatternNode>,
    ) -> Arc<PatternNode> {
        Arc::new(PatternNode {
            pattern: Pattern::Union(tycon, field, subpat),
            info: PatternInfo { source: None },
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PatternInfo {
    pub source: Option<Span>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Pattern {
    Var(Arc<Var>, Option<Arc<TypeNode>>),
    Struct(Arc<TyCon>, Vec<(Name, Arc<PatternNode>)>),
    Union(Arc<TyCon>, Name, Arc<PatternNode>),
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
            Pattern::Union(_, _, pat) => pat.pattern.count_vars(),
        }
    }

    // Returns the type of whole pattern and each variable.
    pub fn get_type(
        &self,
        typechcker: &mut TypeCheckContext,
    ) -> Result<(Arc<TypeNode>, HashMap<FullName, Arc<TypeNode>>), Errors> {
        match self {
            Pattern::Var(v, ty) => {
                let var_name = v.name.clone();
                let ty = if ty.is_none() {
                    type_tyvar_star(&typechcker.new_tyvar())
                } else {
                    let ty = ty.as_ref().unwrap();
                    if !ty.free_vars().is_empty() {
                        return Err(Errors::from_msg_srcs(
                            "Currently, cannot use type variable in type annotation.".to_string(),
                            &[ty.get_source()],
                        ));
                    }
                    ty.clone()
                };
                let mut var_to_ty = HashMap::default();
                var_to_ty.insert(var_name, ty.clone());
                Ok((ty, var_to_ty))
            }
            Pattern::Struct(tc, field_to_pat) => {
                let ty = tc.get_struct_union_value_type(typechcker);
                let mut var_to_ty = HashMap::default();
                let field_tys = ty.field_types(&typechcker.type_env);
                let fields = &typechcker.type_env.tycons.get(tc).unwrap().fields;
                assert_eq!(fields.len(), field_tys.len());
                let field_name_to_ty = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let ty = field_tys[i].clone();
                        (field.name.clone(), ty)
                    })
                    .collect::<HashMap<_, _>>();
                for (field_name, pat) in field_to_pat {
                    let (pat_ty, var_ty) = pat.pattern.get_type(typechcker)?;
                    var_to_ty.extend(var_ty);
                    let unify_res = UnifOrOtherErr::extract_others(
                        typechcker.unify(&pat_ty, field_name_to_ty.get(field_name).unwrap()),
                    )?;
                    if let Err(_) = unify_res {
                        error_exit_with_src(
                            &format!(
                                "Inappropriate pattern `{}` for a value of field `{}` of struct `{}`.",
                                pat.pattern.to_string(),
                                field_name,
                                tc.to_string(),
                            ),
                            &pat.info.source,
                        );
                    }
                }
                Ok((ty, var_to_ty))
            }
            Pattern::Union(tc, field_name, pat) => {
                let ty = tc.get_struct_union_value_type(typechcker);
                let mut var_to_ty = HashMap::default();
                let fields = &typechcker.type_env.tycons.get(tc).unwrap().fields;
                let field_tys = ty.field_types(&typechcker.type_env);
                assert_eq!(fields.len(), field_tys.len());
                let field_idx = fields
                    .iter()
                    .enumerate()
                    .find_map(|(i, f)| if &f.name == field_name { Some(i) } else { None })
                    .unwrap();
                let field_ty = field_tys[field_idx].clone();
                let (pat_ty, var_ty) = pat.pattern.get_type(typechcker)?;
                var_to_ty.extend(var_ty);
                let unify_res =
                    UnifOrOtherErr::extract_others(typechcker.unify(&pat_ty, &field_ty))?;
                if let Err(_) = unify_res {
                    error_exit_with_src(
                        &format!(
                            "Inappropriate pattern `{}` for a value of field `{}` of union `{}`.",
                            pat.pattern.to_string(),
                            field_name,
                            tc.to_string(),
                        ),
                        &pat.info.source,
                    );
                }
                Ok((ty, var_to_ty))
            }
        }
    }

    // Calculate the set of variables that appears in this pattern.
    pub fn vars(&self) -> HashSet<FullName> {
        match self {
            Pattern::Var(var, _) => HashSet::from([var.name.clone()]),
            Pattern::Struct(_, pats) => {
                let mut ret = HashSet::default();
                for (_, pat) in pats {
                    ret.extend(pat.pattern.vars());
                }
                ret
            }
            Pattern::Union(_, _, pat) => pat.pattern.vars(),
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
            Pattern::Union(tc, field, pat) => {
                format!("{}.{}({})", tc.to_string(), field, pat.pattern.to_string())
            }
        }
    }
}
