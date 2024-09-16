#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;

use super::*;
use ast::export_statement::ExportStatement;
use either::Either;
use error::Errors;
use num_bigint::BigInt;
use pest::error::Error;
use std::{cmp::min, mem::swap, sync::Arc};

struct ParseContext {
    // The list of sizes of tuples used in this module.
    tuple_sizes: Vec<u32>,
    // Context for parsing *-operator.
    do_context: DoContext,
    // The source code.
    source: SourceFile,
    // The module name.
    module_name: Name,
    // Curent namespace.
    namespace: NameSpace,
    // Configuration.
    config: Configuration,
}

impl ParseContext {
    fn from_source(source: SourceFile, config: &Configuration) -> Self {
        Self {
            tuple_sizes: vec![],
            do_context: DoContext::default(),
            source,
            module_name: "".to_string(),
            namespace: NameSpace::local(),
            config: config.clone(),
        }
    }
}

#[derive(Default)]
struct DoContext {
    counter: u32,
    monads: Vec<BindOperatorInfo>, // (Monadic action, the result of action).
}

struct BindOperatorInfo {
    operator_src: Span,
    operand: Arc<ExprNode>,
    result_var: Arc<Var>,
}

impl DoContext {
    // Pushes monadic value, and returns expression that represents the result of monadic action.
    fn push_monad(&mut self, monad: Arc<ExprNode>, operator_src: Span) -> Arc<ExprNode> {
        let var_name = FullName::local(&format!("#monadic_value{}", self.counter));
        let var_var = var_var(var_name.clone());
        let var_expr = expr_var(var_name, None);
        self.counter += 1;
        self.monads.push(BindOperatorInfo {
            operator_src,
            operand: monad,
            result_var: var_var,
        });
        var_expr
    }

    fn expand_binds(&mut self, mut expr: Arc<ExprNode>) -> Arc<ExprNode> {
        while !self.monads.is_empty() {
            let BindOperatorInfo {
                operator_src,
                operand: monad,
                result_var: var,
            } = self.monads.pop().unwrap();
            let bind_arg_src = expr.source.clone();
            expr = expr_abs(vec![var], expr, bind_arg_src.clone());
            let bind_function = expr_var(
                FullName::from_strs(&[STD_NAME, MONAD_NAME], MONAD_BIND_NAME),
                Some(operator_src),
            );
            expr = expr_app(bind_function, vec![expr], bind_arg_src.clone());
            let src = Span::unite_opt(&bind_arg_src, &monad.source);
            expr = expr_app(expr, vec![monad], src).set_app_order(AppSourceCodeOrderType::XDotF);
        }
        expr
    }
}

fn unite_span(lhs: &Option<Span>, rhs: &Option<Span>) -> Option<Span> {
    match lhs {
        None => rhs.clone(),
        Some(s) => rhs.clone().map(|t| s.unite(&t)),
    }
}

// Given source code, save it to temporary file (with the given file name and hash value) and the parse the program.
// This is used to parse the source code that is not saved to a file, e.g., source code embedded to the compiler or test code.
// Saving a source code to a file is necessary for:
// - Generate debug information, which requires source locations.
pub fn parse_and_save_to_temporary_file(
    source: &str,
    file_name: &str,
    config: &Configuration,
) -> Result<Program, Errors> {
    let hash = format!("{:x}", md5::compute(source));
    if !check_temporary_source(file_name, &hash) {
        save_temporary_source(source, file_name, &hash);
    }
    parse_file_path(temporary_source_path(file_name, &hash), config)
}

pub fn parse_file_path(file_path: PathBuf, config: &Configuration) -> Result<Program, Errors> {
    let source = SourceFile::from_file_path(file_path);
    let source_cloned = source.clone();
    let source_code = source.string()?;
    let file = match FixParser::parse(Rule::file, &source_code) {
        Ok(res) => res,
        Err(e) => {
            return Err(message_parse_error(e, &source));
        }
    };
    parse_file(file, source_cloned, config)
}

fn parse_file(
    mut file: Pairs<Rule>,
    src: SourceFile,
    config: &Configuration,
) -> Result<Program, Errors> {
    let pair = file.next().unwrap();
    match pair.as_rule() {
        Rule::module => return parse_module(pair, src, config),
        _ => unreachable!(),
    }
}

fn parse_module(
    pair: Pair<Rule>,
    src: SourceFile,
    config: &Configuration,
) -> Result<Program, Errors> {
    assert_eq!(pair.as_rule(), Rule::module);
    let mut errors = Errors::empty();

    let mut ctx: ParseContext = ParseContext::from_source(src.clone(), config);

    let mut pairs = pair.into_inner();
    ctx.module_name = parse_module_defn(pairs.next().unwrap());
    ctx.namespace = NameSpace::new(vec![ctx.module_name.clone()]);
    let mut fix_mod = Program::single_module(ctx.module_name.clone(), &src);

    let mut type_defns: Vec<TypeDefn> = Vec::new();
    let mut global_value_decls: Vec<GlobalValueDecl> = vec![];
    let mut global_value_defns: Vec<GlobalValueDefn> = vec![];
    let mut trait_infos: Vec<TraitInfo> = vec![];
    let mut trait_aliases: Vec<TraitAlias> = vec![];
    let mut trait_impls: Vec<TraitInstance> = vec![];
    let mut import_statements: Vec<ImportStatement> = vec![];
    let mut export_statements: Vec<ExportStatement> = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::global_defns => errors.eat_err(parse_global_defns(
                pair,
                &mut ctx,
                &mut global_value_decls,
                &mut global_value_defns,
                &mut type_defns,
                &mut trait_infos,
                &mut trait_aliases,
                &mut export_statements,
            )),
            Rule::trait_impl => {
                errors.eat_err_or(parse_trait_impl(pair, &mut ctx), |ti| trait_impls.push(ti));
            }
            Rule::import_statement => {
                import_statements.push(parse_import_statement(pair, &mut ctx));
            }
            _ => unreachable!(),
        }
    }

    errors.eat_err(fix_mod.add_global_values(global_value_defns, global_value_decls));
    fix_mod.add_type_defns(type_defns);
    errors.eat_err(fix_mod.add_traits(trait_infos, trait_impls, trait_aliases));
    errors.eat_err(fix_mod.add_import_statements(import_statements));
    fix_mod.used_tuple_sizes.append(&mut ctx.tuple_sizes);
    fix_mod.export_statements = std::mem::replace(&mut export_statements, vec![]);

    errors.to_result().map(|_| fix_mod)
}

fn parse_global_defns(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
    global_value_decls: &mut Vec<GlobalValueDecl>,
    global_value_defns: &mut Vec<GlobalValueDefn>,
    type_defns: &mut Vec<TypeDefn>,
    trait_infos: &mut Vec<TraitInfo>,
    trait_aliases: &mut Vec<TraitAlias>,
    export_statements: &mut Vec<ExportStatement>,
) -> Result<(), Errors> {
    assert_eq!(pair.as_rule(), Rule::global_defns);
    let mut errors = Errors::empty();

    let pairs = pair.into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::global_defns_in_namespace => {
                errors.eat_err(parse_global_defns_in_namespace(
                    pair,
                    ctx,
                    global_value_decls,
                    global_value_defns,
                    type_defns,
                    trait_infos,
                    trait_aliases,
                    export_statements,
                ));
            }
            Rule::type_defn => {
                errors.eat_err_or(parse_type_defn(pair, ctx), |td| type_defns.push(td));
            }
            Rule::global_name_type_sign => {
                errors.eat_err_or(parse_global_value_decl(pair, ctx), |gvd| {
                    global_value_decls.push(gvd)
                });
            }
            Rule::global_name_defn => {
                errors.eat_err_or(parse_global_name_defn(pair, ctx), |gvd| {
                    global_value_defns.push(gvd)
                });
            }
            Rule::trait_defn => {
                errors.eat_err_or(parse_trait_defn(pair, ctx), |ti| trait_infos.push(ti));
            }
            Rule::trait_alias_defn => {
                trait_aliases.push(parse_trait_alias(pair, ctx));
            }
            Rule::export_statement => {
                export_statements.push(parse_export_statement(pair, ctx));
            }
            _ => unreachable!(),
        }
    }
    errors.to_result()
}

fn parse_global_defns_in_namespace(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
    global_value_decls: &mut Vec<GlobalValueDecl>,
    global_value_defns: &mut Vec<GlobalValueDefn>,
    type_defns: &mut Vec<TypeDefn>,
    trait_infos: &mut Vec<TraitInfo>,
    trait_aliases: &mut Vec<TraitAlias>,
    export_statements: &mut Vec<ExportStatement>,
) -> Result<(), Errors> {
    assert_eq!(pair.as_rule(), Rule::global_defns_in_namespace);
    let src = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let namespace = parse_namespace(pairs.next().unwrap(), ctx);
    // Do not allow period in namepsace: it is allowed only in module name.
    if namespace.names.iter().any(|s| s.contains(MODULE_SEPARATOR)) {
        let src = src.to_head_character();
        return Err(Errors::from_msg_srcs(
            "Using \".\"  in namespace is not allowed; it is only allowed in module name."
                .to_string(),
            &[&Some(src)],
        ));
    }
    let bak_namespace = ctx.namespace.clone();
    ctx.namespace = ctx.namespace.append(namespace);
    for pair in pairs {
        parse_global_defns(
            pair,
            ctx,
            global_value_decls,
            global_value_defns,
            type_defns,
            trait_infos,
            trait_aliases,
            export_statements,
        )?;
    }
    ctx.namespace = bak_namespace;
    Ok(())
}

fn parse_trait_alias(pair: Pair<Rule>, ctx: &mut ParseContext) -> TraitAlias {
    assert_eq!(pair.as_rule(), Rule::trait_alias_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    assert_eq!(pairs.peek().unwrap().as_rule(), Rule::trait_name);
    let id = TraitId::from_fullname(FullName::new(
        &ctx.namespace,
        &pairs.next().unwrap().as_str().to_string(),
    ));
    let mut values = vec![];
    for pair in pairs {
        if pair.as_rule() != Rule::trait_fullname {
            break;
        }
        values.push(parse_trait_fullname(pair, ctx));
    }
    TraitAlias {
        id,
        value: values,
        source: Some(span),
        kind: kind_star(), // Will be set to a correct value in TraitEnv::set_kinds.
    }
}

fn parse_trait_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<TraitInfo, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let kinds = if pairs.peek().unwrap().as_rule() == Rule::constraints {
        let pair = pairs.next().unwrap();
        let (preds, eqs, kinds) = parse_constraints(pair, ctx)?;
        if !preds.is_empty() || !eqs.is_empty() {
            let one_src = if !preds.is_empty() {
                &preds.first().unwrap().source
            } else {
                &eqs.first().unwrap().source
            };
            return Err(Errors::from_msg_srcs(
                "In the constraint of trait definition, only kind signature is allowed. Fix does not support \"super-trait\".".to_string(),
                &[one_src],
            ));
        }
        kinds
    } else {
        vec![]
    };
    let trait_tyvar = pairs.next().unwrap().as_str().to_string();
    let impl_type = type_tyvar_star(&trait_tyvar);
    assert_eq!(pairs.peek().unwrap().as_rule(), Rule::trait_name);
    let trait_name = pairs.next().unwrap().as_str().to_string();
    let mut methods: Vec<MethodInfo> = vec![];
    let mut type_syns: HashMap<Name, AssocTypeDefn> = HashMap::new();
    for pair in pairs {
        match parse_trait_member_defn(pair, &impl_type, ctx)? {
            Either::Left(method_info) => {
                if methods.iter().any(|mi| mi.name == method_info.name) {
                    return Err(Errors::from_msg_srcs(
                        format!("Duplicate definitions of member `{}`.", method_info.name),
                        &[&Some(span)],
                    ));
                }
                methods.push(method_info);
            }
            Either::Right(assoc_type) => {
                if type_syns.contains_key(&assoc_type.name.to_string()) {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Duplicate definitions of associated type `{}`.",
                            assoc_type.name.to_string()
                        ),
                        &[&Some(span)],
                    ));
                }
                type_syns.insert(assoc_type.name.to_string(), assoc_type);
            }
        }
    }
    Ok(TraitInfo {
        id: TraitId::from_fullname(FullName::new(&ctx.namespace, &trait_name)),
        type_var: tyvar_from_name(&trait_tyvar, &kind_star()),
        methods,
        assoc_types: type_syns,
        kind_signs: kinds,
        source: Some(span),
    })
}

fn parse_trait_member_defn(
    pair: Pair<Rule>,
    impl_type: &Arc<TypeNode>,
    ctx: &mut ParseContext,
) -> Result<Either<MethodInfo, AssocTypeDefn>, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_defn);
    let pair = pair.into_inner().next().unwrap();
    Ok(match pair.as_rule() {
        Rule::trait_member_value_defn => Either::Left(parse_trait_member_value_defn(pair, ctx)?),
        Rule::trait_member_type_defn => {
            Either::Right(parse_trait_member_type_defn(pair, impl_type, ctx)?)
        }
        _ => unreachable!(),
    })
}

fn parse_trait_member_value_defn(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<MethodInfo, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_value_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let method_name = pairs.next().unwrap().as_str().to_string();
    let qual_type = parse_type_qualified(pairs.next().unwrap(), ctx)?;
    Ok(MethodInfo {
        name: method_name,
        qual_ty: qual_type,
        source: Some(span),
        document: None, // Document can be obtained from `source`
    })
}

fn parse_trait_member_type_defn(
    pair: Pair<Rule>,
    impl_type: &Arc<TypeNode>,
    ctx: &mut ParseContext,
) -> Result<AssocTypeDefn, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_type_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let kind_signs = if pairs.peek().unwrap().as_rule() == Rule::constraints {
        let (preds, eqs, kind_signs) = parse_constraints(pairs.next().unwrap(), ctx)?;
        if !preds.is_empty() || !eqs.is_empty() {
            let one_src = if !preds.is_empty() {
                &preds.first().unwrap().source
            } else {
                &eqs.first().unwrap().source
            };
            return Err(Errors::from_msg_srcs(
                "In the constraint of associated type definition, only kind signature is allowed."
                    .to_string(),
                &[one_src],
            ));
        }
        kind_signs
    } else {
        vec![]
    };
    let assoc_type_defn = parse_type(pairs.next().unwrap(), ctx);
    // Validate form of `assoc_type_defn`
    let (assoc_type_name, assoc_type_params) =
        assoc_type_defn.validate_as_associated_type_defn(impl_type, &Some(span.clone()), false);
    let kind_applied = if let Some(pair) = pairs.next() {
        if pair.as_rule() == Rule::kind {
            parse_kind(pair, ctx)
        } else {
            kind_star()
        }
    } else {
        kind_star()
    };
    Ok(AssocTypeDefn {
        name: assoc_type_name,
        kind_applied,
        src: Some(span),
        params: assoc_type_params,
        kind_signs,
    })
}

fn parse_trait_impl(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<TraitInstance, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_impl);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let qual_pred = parse_predicate_qualified(pairs.next().unwrap(), ctx)?;
    let impl_type = qual_pred.predicate.ty.clone();
    let mut methods: HashMap<Name, Arc<ExprNode>> = HashMap::default();
    let mut assoc_types: HashMap<Name, AssocTypeImpl> = HashMap::default();
    for pair in pairs {
        match parse_trait_member_impl(pair, &impl_type, ctx)? {
            Either::Left((name, expr)) => {
                if methods.contains_key(&name) {
                    return Err(Errors::from_msg_srcs(
                        format!("Duplicate implementation of member `{}`.", name),
                        &[&Some(span)],
                    ));
                }
                methods.insert(name, expr);
            }
            Either::Right(assoc_type_impl) => {
                let name = &assoc_type_impl.name;
                if assoc_types.contains_key(name) {
                    return Err(Errors::from_msg_srcs(
                        format!("Duplicate implementation of associated type `{}`.", name),
                        &[&Some(span)],
                    ));
                }
                assoc_types.insert(name.clone(), assoc_type_impl);
            }
        }
    }
    Ok(TraitInstance {
        qual_pred,
        methods,
        assoc_types,
        define_module: ctx.module_name.clone(),
        source: Some(span),
    })
}

fn parse_trait_member_impl(
    pair: Pair<Rule>,
    impl_type: &Arc<TypeNode>,
    ctx: &mut ParseContext,
) -> Result<Either<(Name, Arc<ExprNode>), AssocTypeImpl>, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_impl);
    let pair = pair.into_inner().next().unwrap();
    Ok(match pair.as_rule() {
        Rule::trait_member_value_impl => {
            let (name, expr) = parse_trait_member_value_impl(pair, ctx)?;
            Either::Left((name, expr))
        }
        Rule::trait_member_type_impl => {
            Either::Right(parse_trait_member_type_impl(pair, impl_type, ctx))
        }
        _ => unreachable!(),
    })
}

fn parse_trait_member_value_impl(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<(Name, Arc<ExprNode>), Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_value_impl);
    let mut pairs = pair.into_inner();
    let method_name = pairs.next().unwrap().as_str().to_string();
    let expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    Ok((method_name, expr))
}

fn parse_trait_member_type_impl(
    pair: Pair<Rule>,
    impl_type: &Arc<TypeNode>,
    ctx: &mut ParseContext,
) -> AssocTypeImpl {
    assert_eq!(pair.as_rule(), Rule::trait_member_type_impl);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let assoc_type_application = parse_type(pairs.next().unwrap(), ctx);
    let (assoc_type_name, params) = assoc_type_application.validate_as_associated_type_defn(
        impl_type,
        &Some(span.clone()),
        true,
    );
    let type_value = parse_type(pairs.next().unwrap(), ctx);
    AssocTypeImpl {
        name: assoc_type_name,
        params,
        value: type_value,
        source: Some(span),
    }
}

fn parse_export_statement(pair: Pair<Rule>, ctx: &mut ParseContext) -> ExportStatement {
    assert_eq!(pair.as_rule(), Rule::export_statement);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // Skip `FFI_EXPORT`.
    let fix_value_name = pairs.next().unwrap().as_str().to_string();
    let c_function_name = pairs.next().unwrap().as_str().to_string();
    ExportStatement::new(
        FullName::new(&ctx.namespace, &fix_value_name),
        c_function_name,
        Some(span),
    )
}

fn parse_predicate_qualified(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<QualPredicate, Errors> {
    assert_eq!(pair.as_rule(), Rule::predicate_qualified);
    let mut pairs = pair.into_inner();
    let (predicates, eqs, kinds) = if pairs.peek().unwrap().as_rule() == Rule::constraints {
        parse_constraints(pairs.next().unwrap(), ctx)?
    } else {
        (vec![], vec![], vec![])
    };
    let predicate = parse_predicate(pairs.next().unwrap(), ctx);
    let qp = QualPredicate {
        pred_constraints: predicates,
        eq_constraints: eqs,
        kind_constraints: kinds,
        predicate,
    };
    Ok(qp)
}

fn parse_global_value_decl(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<GlobalValueDecl, Errors> {
    assert_eq!(pair.as_rule(), Rule::global_name_type_sign);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let qual_type = parse_type_qualified(pairs.next().unwrap(), ctx)?;
    let kind_sings = qual_type.kind_signs.clone();
    let preds = qual_type.preds.clone();
    let eqs = qual_type.eqs.clone();
    let ty = qual_type.ty.clone();

    Ok(GlobalValueDecl {
        name: FullName::new(&ctx.namespace, &name),
        ty: Scheme::generalize(&kind_sings, preds, eqs, ty),
        src: Some(span),
    })
}

fn parse_global_name_defn(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<GlobalValueDefn, Errors> {
    assert_eq!(pair.as_rule(), Rule::global_name_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    Ok(GlobalValueDefn {
        name: FullName::new(&ctx.namespace, &name),
        expr: expr,
        src: Some(span),
    })
}

fn parse_type_qualified(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<QualType, Errors> {
    assert_eq!(pair.as_rule(), Rule::type_qualified);
    let mut pairs = pair.into_inner();
    let (preds, eqs, kinds) = if pairs.peek().unwrap().as_rule() == Rule::constraints {
        parse_constraints(pairs.next().unwrap(), ctx)?
    } else {
        (vec![], vec![], vec![])
    };
    let ty = parse_type(pairs.next().unwrap(), ctx);
    let qt = QualType {
        preds,
        ty,
        kind_signs: kinds,
        eqs,
    };
    Ok(qt)
}

fn parse_constraints(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<(Vec<Predicate>, Vec<Equality>, Vec<KindSignature>), Errors> {
    assert_eq!(pair.as_rule(), Rule::constraints);
    let pairs = pair.into_inner();
    let mut ps: Vec<Predicate> = Default::default();
    let mut ks: Vec<KindSignature> = Default::default();
    let mut es: Vec<Equality> = Default::default();
    for pair in pairs {
        if pair.as_rule() == Rule::predicate {
            ps.push(parse_predicate(pair, ctx));
        } else if pair.as_rule() == Rule::kind_signature {
            ks.push(parse_kind_signature(pair, ctx));
        } else if pair.as_rule() == Rule::equality {
            es.push(parse_equality(pair, ctx)?);
        } else {
            unreachable!()
        }
    }
    Ok((ps, es, ks))
}

fn parse_kind_signature(pair: Pair<Rule>, ctx: &mut ParseContext) -> KindSignature {
    assert_eq!(pair.as_rule(), Rule::kind_signature);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let kind = parse_kind(pairs.next().unwrap(), ctx);
    KindSignature {
        tyvar: name,
        kind,
        source: Some(span),
    }
}

fn parse_equality(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Equality, Errors> {
    assert_eq!(pair.as_rule(), Rule::equality);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let lhs = parse_type(pairs.next().unwrap(), ctx);
    let rhs = parse_type(pairs.next().unwrap(), ctx);
    let lhs_seq = lhs.flatten_type_application();
    if lhs_seq.len() < 2 || !lhs_seq[0].is_tycon() {
        return Err(Errors::from_msg_srcs(
            "The left side of an equality constraint should be the application of an associated type."
                .to_string(),
            &[lhs.get_source()],
        ));
    }
    Ok(Equality {
        assoc_type: TyAssoc {
            name: lhs_seq[0].as_tycon().name.clone(),
        },
        args: lhs_seq[1..].iter().cloned().collect(),
        value: rhs,
        source: Some(span),
    })
}

fn parse_predicate(pair: Pair<Rule>, ctx: &mut ParseContext) -> Predicate {
    assert_eq!(pair.as_rule(), Rule::predicate);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let ty = parse_type(pairs.next().unwrap(), ctx);
    let trait_id = parse_trait_fullname(pairs.next().unwrap(), ctx);
    let mut pred = Predicate::make(trait_id, ty);
    pred.set_source(span);
    pred
}

fn parse_trait_fullname(pair: Pair<Rule>, _ctx: &mut ParseContext) -> TraitId {
    assert_eq!(pair.as_rule(), Rule::trait_fullname);
    let mut pairs = pair.into_inner();
    let fullname = parse_capital_fullname(pairs.next().unwrap());
    TraitId { name: fullname }
}

fn parse_capital_fullname(pair: Pair<Rule>) -> FullName {
    assert_eq!(pair.as_rule(), Rule::capital_fullname);
    let mut pairs = pair.into_inner();
    let mut fullname = FullName::local("");
    while pairs.peek().unwrap().as_rule() == Rule::namespace_item {
        fullname
            .namespace
            .names
            .push(pairs.next().unwrap().as_str().to_string());
    }
    let pair = pairs.next().unwrap();
    assert_eq!(pair.as_rule(), Rule::capital_name);
    fullname.name = pair.as_str().to_string();
    fullname
}

fn parse_kind(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind);
    let mut pairs = pair.into_inner();
    let mut res: Arc<Kind> = parse_kind_nlr(pairs.next().unwrap(), ctx);
    for pair in pairs {
        res = kind_arrow(res, parse_kind_nlr(pair, ctx));
    }
    res
}

fn parse_kind_nlr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind_nlr);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    if pair.as_rule() == Rule::kind_star {
        parse_kind_star(pair, ctx)
    } else if pair.as_rule() == Rule::kind_braced {
        parse_kind_braced(pair, ctx)
    } else {
        unreachable!()
    }
}

fn parse_kind_star(pair: Pair<Rule>, _ctx: &mut ParseContext) -> Arc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind_star);
    kind_star()
}

fn parse_kind_braced(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind_braced);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    parse_kind(pair, ctx)
}

fn parse_module_defn(pair: Pair<Rule>) -> Name {
    pair.into_inner().next().unwrap().as_str().to_string()
}

fn parse_type_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<TypeDefn, Errors> {
    assert_eq!(pair.as_rule(), Rule::type_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();

    // Parse constraints to specify kinds of type variables.
    let mut kinds: HashMap<Name, Arc<Kind>> = HashMap::new();
    if pairs.peek().unwrap().as_rule() == Rule::constraints {
        let pair = pairs.next().unwrap();
        let (preds, eqs, kind_signs) = parse_constraints(pair, ctx)?;
        if preds.len() > 0 || eqs.len() > 0 {
            let one_src = if !preds.is_empty() {
                &preds.first().unwrap().source
            } else {
                &eqs.first().unwrap().source
            };
            return Err(Errors::from_msg_srcs(
                "In the constraint of type definition, only kind signature is allowed.".to_string(),
                &[one_src],
            ));
        }
        for kind_sign in kind_signs {
            if kinds.contains_key(&kind_sign.tyvar) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Kind of type variable `{}` is specified more than once.",
                        kind_sign.tyvar
                    ),
                    &[&kind_sign.source],
                ));
            }
            kinds.insert(kind_sign.tyvar, kind_sign.kind);
        }
    }
    assert_eq!(pairs.peek().unwrap().as_rule(), Rule::type_name);
    let name = pairs.next().unwrap().as_str();
    let mut tyvars: Vec<Arc<TyVar>> = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::type_var {
        let tyvar_name = pairs.next().unwrap().as_str();
        let kind = kinds.get(tyvar_name).unwrap_or(&kind_star()).clone();
        tyvars.push(tyvar_from_name(tyvar_name, &kind));
    }
    let pair = pairs.next().unwrap();
    let type_value = if pair.as_rule() == Rule::struct_defn {
        parse_struct_defn(pair, ctx)
    } else if pair.as_rule() == Rule::union_defn {
        parse_union_defn(pair, ctx)
    } else if pair.as_rule() == Rule::type_alias_defn {
        parse_type_alias_defn(pair, ctx)
    } else {
        unreachable!();
    };
    Ok(TypeDefn {
        name: FullName::new(&ctx.namespace, name),
        value: type_value,
        tyvars,
        source: Some(span),
    })
}

fn parse_struct_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> TypeDeclValue {
    assert_eq!(pair.as_rule(), Rule::struct_defn);
    let mut pairs = pair.into_inner();
    let mut fields: Vec<Field> = Vec::new();
    let mut is_unbox = true; // Default value
    if let Some(pair) = pairs.peek() {
        if pair.as_rule() == Rule::box_or_unbox {
            is_unbox = parse_box_unbox(pairs.next().unwrap(), ctx);
        }
    }
    for pair in pairs {
        fields.push(parse_type_field(pair, ctx));
    }
    TypeDeclValue::Struct(Struct { fields, is_unbox })
}

fn parse_union_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> TypeDeclValue {
    assert_eq!(pair.as_rule(), Rule::union_defn);
    let mut pairs = pair.into_inner();
    let mut fields: Vec<Field> = Vec::new();
    let mut is_unbox = true; // Default value
    if pairs.peek().unwrap().as_rule() == Rule::box_or_unbox {
        is_unbox = parse_box_unbox(pairs.next().unwrap(), ctx);
    }
    for pair in pairs {
        fields.push(parse_type_field(pair, ctx));
    }
    TypeDeclValue::Union(Union { fields, is_unbox })
}

fn parse_type_alias_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> TypeDeclValue {
    assert_eq!(pair.as_rule(), Rule::type_alias_defn);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    let aliased_type = parse_type(pair, ctx);
    TypeDeclValue::Alias(TypeAlias {
        value: aliased_type,
    })
}

// Return true if unbox.
fn parse_box_unbox(pair: Pair<Rule>, _ctx: &mut ParseContext) -> bool {
    assert_eq!(pair.as_rule(), Rule::box_or_unbox);
    if pair.as_str() == "box" {
        return false;
    } else if pair.as_str() == "unbox" {
        return true;
    }
    unreachable!();
}

fn parse_type_field(pair: Pair<Rule>, ctx: &mut ParseContext) -> Field {
    assert_eq!(pair.as_rule(), Rule::type_field);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let ty = parse_type(pairs.next().unwrap(), ctx);
    Field {
        name: name.to_string(),
        ty,
        is_punched: false,
    }
}

fn parse_expr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_type_annotation(pair, ctx)
}

fn parse_expr_with_new_do(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr);

    // Here use new DoContext.
    let old_doctx = std::mem::replace(&mut ctx.do_context, DoContext::default());
    let expr = parse_expr(pair, ctx)?;
    let expr = ctx.do_context.expand_binds(expr);

    // Restore old DoContext.
    ctx.do_context = old_doctx;

    Ok(expr)
}

fn parse_expr_type_annotation(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_type_annotation);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let mut expr = parse_expr_rtl_app(pairs.next().unwrap(), ctx)?;
    match pairs.next() {
        None => {}
        Some(ty) => {
            expr = expr_tyanno(expr, parse_type(ty, ctx), Some(span));
        }
    }
    Ok(expr)
}

// Parse combinator sequence, e.g., `f x y` or `x & f & g`
fn parse_combinator_sequence(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
    inner_parser: fn(Pair<Rule>, &mut ParseContext) -> Result<Arc<ExprNode>, Errors>,
) -> Result<Vec<Arc<ExprNode>>, Errors> {
    pair.into_inner()
        .map(|pair| inner_parser(pair, ctx))
        .collect()
}

#[derive(Default, Clone)]
struct BinaryOpInfo {
    trait_name: Name,
    method_name: Name,
    reverse: bool,
    post_unary: Option<UnaryOpInfo>,
}

impl BinaryOpInfo {
    fn new(trait_name: &str, method_name: &str) -> BinaryOpInfo {
        BinaryOpInfo {
            trait_name: trait_name.to_string(),
            method_name: method_name.to_string(),
            reverse: false,
            post_unary: None,
        }
    }

    fn add_post_unary(mut self, unary_op: UnaryOpInfo) -> BinaryOpInfo {
        self.post_unary = Some(unary_op);
        self
    }

    fn reverse(mut self) -> BinaryOpInfo {
        self.reverse = !self.reverse;
        return self;
    }
}

// Binary operator
fn parse_binary_operator_sequence(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
    ops: HashMap<&str, BinaryOpInfo>,
    operator_rule: Rule,
    inner_parser: fn(Pair<Rule>, &mut ParseContext) -> Result<Arc<ExprNode>, Errors>,
) -> Result<Arc<ExprNode>, Errors> {
    let mut pairs = pair.into_inner();
    let mut expr = inner_parser(pairs.next().unwrap(), ctx)?;
    let mut next_operation = BinaryOpInfo::default();
    let mut next_op_span: Option<Span> = None;
    for pair in pairs {
        if pair.as_rule() == operator_rule {
            next_operation = ops[pair.as_str()].clone();
            next_op_span = Some(Span::from_pair(&ctx.source, &pair));
        } else {
            let mut lhs = expr;
            let mut rhs = inner_parser(pair, ctx)?;
            if next_operation.reverse {
                swap(&mut lhs, &mut rhs);
            }
            let span = unite_span(&unite_span(&next_op_span, &lhs.source), &rhs.source);
            expr = expr_app(
                expr_app(
                    expr_var(
                        FullName::from_strs(
                            &[STD_NAME, &next_operation.trait_name],
                            &next_operation.method_name,
                        ),
                        next_op_span.clone(),
                    ),
                    vec![lhs],
                    span.clone(),
                ),
                vec![rhs],
                span.clone(),
            );
            match next_operation.post_unary.as_ref() {
                Some(op) => {
                    expr = expr_app(
                        expr_var(
                            FullName::from_strs(&[STD_NAME, &op.trait_name], &op.method_name),
                            next_op_span.clone(),
                        ),
                        vec![expr.clone()],
                        span,
                    );
                }
                None => {}
            }
        }
    }
    Ok(expr)
}

// comparison operators (left-associative)
fn parse_expr_cmp(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_cmp);
    parse_binary_operator_sequence(
        pair,
        ctx,
        HashMap::from([
            (
                "==",
                BinaryOpInfo::new(EQ_TRAIT_NAME, EQ_TRAIT_EQ_NAME).reverse(),
            ),
            (
                "!=",
                BinaryOpInfo::new(EQ_TRAIT_NAME, EQ_TRAIT_EQ_NAME)
                    .add_post_unary(UnaryOpInfo::new(NOT_TRAIT_NAME, NOT_TRAIT_OP_NAME)),
            ),
            (
                "<",
                BinaryOpInfo::new(LESS_THAN_TRAIT_NAME, LESS_THAN_TRAIT_LT_NAME),
            ),
            (
                ">",
                BinaryOpInfo::new(LESS_THAN_TRAIT_NAME, LESS_THAN_TRAIT_LT_NAME).reverse(),
            ),
            (
                "<=",
                BinaryOpInfo::new(
                    LESS_THAN_OR_EQUAL_TO_TRAIT_NAME,
                    LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
                ),
            ),
            (
                ">=",
                BinaryOpInfo::new(
                    LESS_THAN_OR_EQUAL_TO_TRAIT_NAME,
                    LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
                )
                .reverse(),
            ),
        ]),
        Rule::operator_cmp,
        parse_expr_plus,
    )
}

// Operator && (right-associative)
fn parse_expr_and(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_and);
    let exprs = pair
        .into_inner()
        .map(|p| parse_expr_cmp(p, ctx))
        .collect::<Result<Vec<_>, _>>()?;

    fn and_boolean_exprs(ps: &[Arc<ExprNode>]) -> Arc<ExprNode> {
        if ps.len() == 1 {
            ps[0].clone()
        } else {
            let sub = and_boolean_exprs(&ps[1..]);
            expr_if(ps[0].clone(), sub, expr_bool_lit(false, None), None)
        }
    }

    Ok(and_boolean_exprs(&exprs))
}

// Operator || (right-associative)
fn parse_expr_or(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_or);
    let exprs = pair
        .into_inner()
        .map(|p| parse_expr_and(p, ctx))
        .collect::<Result<Vec<_>, _>>()?;

    fn or_boolean_exprs(ps: &[Arc<ExprNode>]) -> Arc<ExprNode> {
        if ps.len() == 1 {
            ps[0].clone()
        } else {
            let sub = or_boolean_exprs(&ps[1..]);
            expr_if(ps[0].clone(), expr_bool_lit(true, None), sub, None)
        }
    }

    Ok(or_boolean_exprs(&exprs))
}

// Operator +/- (left associative)
fn parse_expr_plus(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_plus);
    parse_binary_operator_sequence(
        pair,
        ctx,
        HashMap::from([
            ("+", BinaryOpInfo::new(ADD_TRAIT_NAME, ADD_TRAIT_ADD_NAME)),
            (
                "-",
                BinaryOpInfo::new(SUBTRACT_TRAIT_NAME, SUBTRACT_TRAIT_SUBTRACT_NAME),
            ),
        ]),
        Rule::operator_plus,
        parse_expr_mul,
    )
}

// Operator *,/,% (left associative)
fn parse_expr_mul(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_mul);
    parse_binary_operator_sequence(
        pair,
        ctx,
        HashMap::from([
            (
                "*",
                BinaryOpInfo::new(MULTIPLY_TRAIT_NAME, MULTIPLY_TRAIT_MULTIPLY_NAME),
            ),
            (
                "/",
                BinaryOpInfo::new(DIVIDE_TRAIT_NAME, DIVIDE_TRAIT_DIVIDE_NAME),
            ),
            (
                "%",
                BinaryOpInfo::new(REMAINDER_TRAIT_NAME, REMAINDER_TRAIT_REMAINDER_NAME),
            ),
        ]),
        Rule::operator_mul,
        parse_expr_unary,
    )
}

#[derive(Default, Clone)]
struct UnaryOpInfo {
    trait_name: Name,
    method_name: Name,
}

impl UnaryOpInfo {
    fn new(trait_name: &str, method_name: &str) -> UnaryOpInfo {
        UnaryOpInfo {
            trait_name: trait_name.to_string(),
            method_name: method_name.to_string(),
        }
    }
}

// Unary opeartors
fn parse_expr_unary(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    let pairs = pair.into_inner();
    let mut ops: Vec<UnaryOpInfo> = vec![];
    let mut spans: Vec<Span> = vec![];
    for pair in pairs {
        match pair.as_rule() {
            Rule::operator_unary => {
                spans.push(Span::from_pair(&ctx.source, &pair));
                if pair.as_str() == "-" {
                    ops.push(UnaryOpInfo::new(
                        NEGATE_TRAIT_NAME,
                        NEGATE_TRAIT_NEGATE_NAME,
                    ))
                } else if pair.as_str() == "!" {
                    ops.push(UnaryOpInfo::new(NOT_TRAIT_NAME, NOT_TRAIT_OP_NAME))
                } else {
                    panic!("unknown unary operator: `{}`", pair.as_str());
                }
            }
            _ => {
                let mut expr = parse_expr_composition(pair, ctx)?;
                for (i, op) in ops.iter().enumerate().rev() {
                    let op_span = spans[i].clone();
                    expr = expr_app(
                        expr_var(
                            FullName::from_strs(&[STD_NAME, &op.trait_name], &op.method_name),
                            Some(op_span.clone()),
                        ),
                        vec![expr.clone()],
                        expr.source.as_ref().map(|s0| s0.unite(&op_span)),
                    );
                }
                return Ok(expr);
            }
        }
    }
    unreachable!()
}

// Parse right to left application sequence, e.g., `g $ f $ x`. (right-associative)
fn parse_expr_rtl_app(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_rtl_app);
    let exprs = parse_combinator_sequence(pair, ctx, parse_expr_or)?;
    let mut exprs_iter = exprs.iter().rev();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), vec![ret], span);
    }
    Ok(ret)
}

// Parse function composition operator >> and <<.
fn parse_expr_composition(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    fn unite_src_from_expr(lhs: &Arc<ExprNode>, rhs: &Arc<ExprNode>) -> Option<Span> {
        if lhs.source.is_none() {
            return None;
        }
        if rhs.source.is_none() {
            return None;
        }
        Some(
            lhs.source
                .as_ref()
                .unwrap()
                .unite(rhs.source.as_ref().clone().unwrap()),
        )
    }

    assert_eq!(pair.as_rule(), Rule::expr_composition);
    let mut pairs = pair.into_inner();
    let mut expr = parse_expr_bind(pairs.next().unwrap(), ctx)?;
    while pairs.peek().is_some() {
        let op = pairs.next().unwrap();
        assert_eq!(op.as_rule(), Rule::operator_composition);
        let op_span = Span::from_pair(&ctx.source, &op);
        let compose = expr_var(
            FullName::from_strs(&[STD_NAME], COMPOSE_FUNCTION_NAME),
            Some(op_span),
        );
        let rhs = parse_expr_bind(pairs.next().unwrap(), ctx)?;
        match op.as_str() {
            ">>" => {
                let span = unite_src_from_expr(&compose, &expr);
                expr = expr_app(compose, vec![expr], span)
                    .set_app_order(AppSourceCodeOrderType::XDotF);
                let span = unite_src_from_expr(&expr, &rhs);
                expr = expr_app(expr, vec![rhs], span).set_app_order(AppSourceCodeOrderType::FX);
            }
            "<<" => {
                let span = unite_src_from_expr(&compose, &rhs);
                let right_expr = expr_app(compose, vec![rhs.clone()], span)
                    .set_app_order(AppSourceCodeOrderType::XDotF);
                let span = unite_src_from_expr(&right_expr, &rhs);
                expr = expr_app(right_expr, vec![expr], span)
                    .set_app_order(AppSourceCodeOrderType::FX);
            }
            _ => {
                unreachable!()
            }
        }
    }
    Ok(expr)
}

// Parse monadic bind syntax `*x`.
fn parse_expr_bind(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_bind);
    let mut stars = vec![];
    let mut pairs = pair.into_inner();
    while pairs.peek().unwrap().as_rule() == Rule::operator_bind {
        let star_pair = pairs.next().unwrap();
        stars.push(Span::from_pair(&ctx.source, &star_pair));
    }
    let mut expr = parse_expr_ltr_app(pairs.next().unwrap(), ctx)?;
    while !stars.is_empty() {
        expr = ctx.do_context.push_monad(expr, stars.pop().unwrap());
    }
    Ok(expr)
}

// Parse left to right application sequence, e.g., `x.f.g`. (left-associative)
fn parse_expr_ltr_app(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_ltr_app);
    let exprs = parse_combinator_sequence(pair, ctx, parse_expr_app)?;
    let mut exprs_iter = exprs.iter();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), vec![ret], span).set_app_order(AppSourceCodeOrderType::XDotF);
    }
    Ok(ret)
}

// Parse application sequence, e.g., `f(x, y)`. (left-associative)
fn parse_expr_app(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_app);
    let mut pairs = pair.into_inner();
    let head = parse_expr_nlr(pairs.next().unwrap(), ctx)?;
    let mut args = vec![];
    if pairs.peek().is_some() {
        // If parentheses for arguments are given,
        let pair = pairs.next().unwrap();
        let args_span = Span::from_pair(&ctx.source, &pair);
        args = parse_arg_list(pair, ctx)?;
        if args.len() == 0 {
            // `f()` is interpreted as application to unit: `f $ ()`.
            args.push(
                expr_make_struct(tycon(make_tuple_name(0)), vec![]).set_source(Some(args_span)),
            )
        }
    }
    let mut ret = head;
    for expr in args {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(ret, vec![expr.clone()], span);
    }
    Ok(ret)
}

fn parse_arg_list(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Vec<Arc<ExprNode>>, Errors> {
    assert_eq!(pair.as_rule(), Rule::arg_list);
    parse_combinator_sequence(pair, ctx, parse_expr)
}

fn parse_expr_nlr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_nlr);
    let pair = pair.into_inner().next().unwrap();
    Ok(match pair.as_rule() {
        Rule::expr_lit => parse_expr_lit(pair, ctx)?,
        Rule::expr_var => parse_expr_var(pair, ctx),
        Rule::expr_let => parse_expr_let(pair, ctx)?,
        Rule::expr_eval => parse_expr_eval(pair, ctx)?,
        Rule::expr_if => parse_expr_if(pair, ctx)?,
        Rule::expr_do => parse_expr_do(pair, ctx)?,
        Rule::expr_lam => parse_expr_lam(pair, ctx)?,
        Rule::expr_tuple => parse_expr_tuple(pair, ctx)?,
        Rule::expr_make_struct => parse_expr_make_struct(pair, ctx)?,
        Rule::expr_call_c => parse_expr_call_c(pair, ctx)?,
        _ => unreachable!(),
    })
}

fn parse_expr_var(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_var);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let namespace = if pairs.peek().unwrap().as_rule() == Rule::namespace {
        parse_namespace(pairs.next().unwrap(), ctx)
    } else {
        NameSpace::local()
    };
    let var = pairs.next().unwrap().as_str().to_string();
    let name = FullName {
        namespace,
        name: var,
    };
    expr_var(name, Some(span))
}

fn parse_namespace(pair: Pair<Rule>, _ctx: &mut ParseContext) -> NameSpace {
    assert_eq!(pair.as_rule(), Rule::namespace);
    let pairs = pair.into_inner();
    let mut ret: Vec<String> = Vec::new();
    for pair in pairs {
        ret.push(pair.as_str().to_string());
    }
    NameSpace::new(ret)
}

fn parse_expr_lit(expr: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    let pair = expr.into_inner().next().unwrap();
    Ok(match pair.as_rule() {
        Rule::expr_number_lit => parse_expr_number_lit(pair, ctx)?,
        Rule::expr_bool_lit => parse_expr_bool_lit(pair, ctx),
        Rule::expr_string_lit => parse_expr_string_lit(pair, ctx)?,
        Rule::expr_array_lit => parse_expr_array_lit(pair, ctx)?,
        Rule::expr_nullptr_lit => parse_expr_nullptr_lit(pair, ctx),
        Rule::expr_u8_lit => parse_expr_u8_lit(pair, ctx),
        _ => unreachable!(),
    })
}

fn parse_expr_let(expr: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    let span = Span::from_pair(&ctx.source, &expr);
    let mut pairs = expr.into_inner();
    let pat = parse_pattern(pairs.next().unwrap(), ctx);
    let _eq_of_let = pairs.next().unwrap();
    let bound = parse_expr(pairs.next().unwrap(), ctx)?;
    let _in_of_let = pairs.next().unwrap();
    let val = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    Ok(expr_let(pat, bound, val, Some(span)))
}

fn parse_expr_eval(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_eval);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let bound = parse_expr(pairs.next().unwrap(), ctx)?;
    pairs.next().unwrap(); // Skip `Rule::semicolon`.
    let val = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    let pat = PatternNode::make_var(var_local(EVAL_VAR_NAME), Some(make_unit_ty()));
    Ok(expr_let(pat, bound, val, Some(span)))
}

fn parse_expr_lam(expr: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    let span = Span::from_pair(&ctx.source, &expr);
    let mut pairs = expr.into_inner();
    let mut pats = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::pattern {
        let pat = parse_pattern(pairs.next().unwrap(), ctx);
        pats.push(pat);
    }
    let mut expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    let mut pat_body_span = expr.source.clone();
    let var = var_local(ARG_NAME);
    for pat in pats.iter().rev() {
        pat_body_span = Span::unite_opt(&pat_body_span, &pat.info.source);
        expr = expr_abs(
            vec![var.clone()],
            expr_let(
                pat.clone(),
                expr_var(FullName::local(ARG_NAME), pat.info.source.clone()),
                expr,
                pat_body_span.clone(),
            ),
            pat_body_span.clone(),
        )
    }
    Ok(expr.set_source(Some(span)))
}

fn parse_expr_if(expr: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(expr.as_rule(), Rule::expr_if);
    let span = Span::from_pair(&ctx.source, &expr);
    let mut pairs = expr.into_inner();
    let cond = pairs.next().unwrap();
    let then_val = pairs.next().unwrap();
    pairs.next().unwrap().as_rule(); // Skip `Rule::else_of_if` or `Rule::else_of_if_with_space`.
    let else_val = pairs.next().unwrap();
    Ok(expr_if(
        parse_expr(cond, ctx)?,
        parse_expr_with_new_do(then_val, ctx)?,
        parse_expr_with_new_do(else_val, ctx)?,
        Some(span),
    ))
}

fn parse_expr_do(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert!(pair.as_rule() == Rule::expr_do);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_with_new_do(pair, ctx)
}

fn parse_expr_tuple(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_tuple);
    let span = Span::from_pair(&ctx.source, &pair);
    let pairs = pair.into_inner();
    let mut exprs = vec![];
    let mut extra_comma = false;
    for pair in pairs {
        if pair.as_rule() == Rule::extra_comma {
            extra_comma = true;
            break;
        }
        exprs.push(parse_expr(pair, ctx)?);
    }
    let is_bracketed_expr = exprs.len() == 1 && !extra_comma;
    if is_bracketed_expr {
        Ok(exprs[0].clone())
    } else {
        let tuple_size = exprs.len();
        ctx.tuple_sizes.push(tuple_size as u32);
        let expr = expr_make_struct(
            tycon(make_tuple_name(tuple_size as u32)),
            exprs
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, expr)| (i.to_string(), expr))
                .collect(),
        )
        .set_source(Some(span));
        Ok(expr)
    }
}

fn parse_expr_make_struct(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_make_struct);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let tycon = parse_tycon(pairs.next().unwrap());
    let mut fields = vec![];
    while pairs.peek().is_some() {
        let field_name = pairs.next().unwrap().as_str().to_string();
        let field_expr = parse_expr(pairs.next().unwrap(), ctx)?;
        fields.push((field_name, field_expr));
    }
    Ok(expr_make_struct(tycon, fields).set_source(Some(span)))
}

fn parse_expr_call_c(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_call_c);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let ret_ty = parse_ffi_c_fun_ty(pairs.next().unwrap(), ctx);
    let fun_name = pairs.next().unwrap().as_str().to_string();
    let param_tys = parse_ffi_param_tys(pairs.next().unwrap(), ctx);
    let args = pairs
        .map(|pair| parse_expr(pair, ctx))
        .collect::<Result<Vec<_>, _>>()?;

    // Validate number of arguments.
    if args.len() < param_tys.len() || args.len() > param_tys.len() {
        return Err(Errors::from_msg_srcs(
            "Wrong number of arguments in FFI_CALL expression.".to_string(),
            &[&Some(span)],
        ));
    }

    Ok(expr_ffi_call(fun_name, ret_ty, param_tys, args, Some(span)))
}

fn parse_ffi_c_fun_ty(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TyCon> {
    assert_eq!(pair.as_rule(), Rule::ffi_c_fun_ty);
    let name = if pair.as_str() == "()" {
        make_tuple_name(0)
    } else {
        let mut name = pair.as_str().to_string();
        for (c_type_name, sign, size) in ctx.config.c_type_sizes.get_c_types() {
            if c_type_name == pair.as_str() {
                name = format!("{}{}", sign, size);
            }
        }
        FullName::from_strs(&[STD_NAME], &name)
    };
    tycon(name)
}

fn parse_ffi_param_tys(pair: Pair<Rule>, ctx: &mut ParseContext) -> Vec<Arc<TyCon>> {
    assert_eq!(pair.as_rule(), Rule::ffi_param_tys);
    pair.into_inner()
        .map(|pair| parse_ffi_c_fun_ty(pair, ctx))
        .collect()
}

fn parse_expr_number_lit(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_number_lit);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    assert_eq!(pair.as_rule(), Rule::number_lit_body);
    let val_str = pair.as_str();
    let is_float = val_str.contains(".");
    let (ty, ty_name) = match pairs.next() {
        Some(pair) => {
            // Type of literal is explicitly specified.
            assert_eq!(pair.as_rule(), Rule::number_lit_type);
            let type_name = pair.as_str();
            let (ty, is_float_type) = make_numeric_ty(type_name);
            if is_float != is_float_type {
                return Err(Errors::from_msg_srcs(
                    "Mismatch between literal format and specified type. Note that floating point literals must contain a decimal point.".to_string(),
                    &[&Some(span)],
                ));
            }
            (ty.unwrap(), type_name)
        }
        None => {
            // Type of literal is implicit.
            if is_float {
                (make_f64_ty(), F64_NAME)
            } else {
                (make_i64_ty(), I64_NAME)
            }
        }
    };
    if is_float {
        let val = val_str.parse::<f64>();
        if val.is_err() {
            return Err(Errors::from_msg_srcs(
                format!(
                    "A literal string `{}` cannot be parsed as a floating number.",
                    val_str
                ),
                &[&Some(span)],
            ));
        }
        let val = val.unwrap();
        Ok(expr_float_lit(val, ty, Some(span)))
    } else {
        // Integral literal
        let val = parse_integral_string_lit(val_str);
        if val.is_none() {
            return Err(Errors::from_msg_srcs(
                format!(
                    "A literal string `{}` cannot be parsed as an integer.",
                    val_str
                ),
                &[&Some(span)],
            ));
        }
        let val = val.unwrap();

        // Check size.
        let (ty_min, ty_max) = integral_ty_range(ty_name);
        if !(ty_min <= val && val <= ty_max) {
            return Err(Errors::from_msg_srcs(
                format!(
                    "The value of an integer literal `{}` is out of range of `{}`.",
                    val_str, ty_name
                ),
                &[&Some(span)],
            ));
        }

        // Now stringify val and parse it again as i128.
        let val = val.to_str_radix(10).parse::<i128>().unwrap();
        Ok(expr_int_lit(val as u64, ty, Some(span)))
    }
}

fn parse_integral_string_lit(s: &str) -> Option<BigInt> {
    if s.len() == 0 {
        return None;
    }
    if s.starts_with("0") || s.starts_with("-0") {
        if s.starts_with("0x") {
            return BigInt::parse_bytes(s.trim_start_matches("0x").as_bytes(), 16);
        }
        if s.starts_with("-0x") {
            return BigInt::parse_bytes(s.trim_start_matches("-0x").as_bytes(), 16).map(|x| -x);
        }
        if s.starts_with("0o") {
            return BigInt::parse_bytes(s.trim_start_matches("0o").as_bytes(), 8);
        }
        if s.starts_with("-0o") {
            return BigInt::parse_bytes(s.trim_start_matches("-0o").as_bytes(), 8).map(|x| -x);
        }
        if s.starts_with("0b") {
            return BigInt::parse_bytes(s.trim_start_matches("0b").as_bytes(), 2);
        }
        if s.starts_with("-0b") {
            return BigInt::parse_bytes(s.trim_start_matches("-0b").as_bytes(), 2).map(|x| -x);
        }
    }
    let split = s.split('e').collect::<Vec<_>>();
    if split.len() > 2 {
        return None;
    }
    if split.len() == 1 {
        // 'e' is not contained.
        return BigInt::parse_bytes(s.as_bytes(), 10);
    }
    assert_eq!(split.len(), 2);
    let num = BigInt::parse_bytes(split[0].as_bytes(), 10);
    if num.is_none() {
        return None;
    }
    let num = num.unwrap();
    let exp = BigInt::parse_bytes(split[1].as_bytes(), 10);
    if exp.is_none() {
        return None;
    }
    let exp = exp.unwrap();
    if exp < BigInt::from(0 as i32) {
        // Negative exponent is not allowed in integral literal.
        return None;
    }
    // Return num * 10^exp.
    let mut ret = num;
    let mut i = BigInt::from(0);
    while i < exp {
        ret *= 10;
        i += 1;
    }
    Some(ret)
}

fn parse_expr_nullptr_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_nullptr_lit);
    let span = Span::from_pair(&ctx.source, &pair);
    expr_nullptr_lit(Some(span))
}

fn parse_expr_bool_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_bool_lit);
    let val = pair.as_str().parse::<bool>().unwrap();
    let span = Span::from_pair(&ctx.source, &pair);
    expr_bool_lit(val, Some(span))
}

fn parse_expr_array_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_array_lit);
    let span = Span::from_pair(&ctx.source, &pair);
    let elems = pair
        .into_inner()
        .map(|pair| parse_expr(pair, ctx))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(expr_array_lit(elems, Some(span)))
}

fn parse_expr_string_lit(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_string_lit);
    let span = Span::from_pair(&ctx.source, &pair);
    let string = pair.into_inner().next().unwrap().as_str().to_string();
    // Resolve escape sequences.
    let mut string = string.chars();
    let mut out_string: Vec<char> = vec![];
    loop {
        match string.next() {
            None => {
                break;
            }
            Some(c) => {
                if c != '\\' {
                    out_string.push(c);
                    continue;
                }
                let c = string.next().unwrap();
                if c == '\"' {
                    out_string.push('"');
                } else if c == '\\' {
                    out_string.push('\\');
                } else if c == 'n' {
                    out_string.push('\n');
                } else if c == 'r' {
                    out_string.push('\r');
                } else if c == 't' {
                    out_string.push('\t');
                } else if c == 'u' {
                    let mut code: u32 = 0;
                    for i in 0..4 {
                        let c = string.next().unwrap().to_digit(16).unwrap();
                        code += c << 4 * (3 - i);
                    }
                    let c = match char::from_u32(code) {
                        None => {
                            return Err(Errors::from_msg_srcs(
                                format!("Invalid unicode character: u{:X}", code),
                                &[&Some(span)],
                            ));
                        }
                        Some(c) => c,
                    };
                    out_string.push(c);
                }
            }
        }
    }
    let string = String::from_iter(out_string.iter());
    Ok(make_string_from_rust_string(string, Some(span)))
}

fn parse_expr_u8_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_u8_lit);
    let span = Span::from_pair(&ctx.source, &pair);
    let string = pair.into_inner().next().unwrap().as_str().to_string();
    // Resolve escape sequences.
    let mut string = string.chars();
    let byte: u8;
    loop {
        match string.next() {
            None => {
                unreachable!()
            }
            Some(c) => {
                if c != '\\' {
                    let mut buf = [0 as u8];
                    c.encode_utf8(&mut buf);
                    byte = buf[0];
                } else {
                    let c = string.next().unwrap();
                    if c == '\'' {
                        byte = 39;
                    } else if c == '\\' {
                        byte = 92;
                    } else if c == 'n' {
                        byte = 10;
                    } else if c == 'r' {
                        byte = 13;
                    } else if c == 't' {
                        byte = 9;
                    } else if c == '0' {
                        byte = 0;
                    } else if c == 'x' {
                        let mut code: u8 = 0;
                        for i in 0..2 {
                            let c = string.next().unwrap().to_digit(16).unwrap() as u8;
                            code += c << 4 * (1 - i);
                        }
                        byte = code;
                    } else {
                        unreachable!()
                    }
                }
                break;
            }
        }
    }
    expr_int_lit(byte as u64, make_u8_ty(), Some(span))
}

fn parse_type(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_expr);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_fun => parse_type_fun(pair, ctx),
        _ => unreachable!(),
    }
    .set_source(Some(span))
}

fn parse_type_fun(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_fun);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let src_ty = parse_type_tyapp(pairs.next().unwrap(), ctx);
    match pairs.next() {
        Some(pair) => {
            let dst_ty = parse_type(pair, ctx);
            type_fun(src_ty, dst_ty)
        }
        None => src_ty,
    }
    .set_source(Some(span))
}

fn parse_type_tyapp(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_tyapp);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    let mut ret = parse_type_nlr(pair, ctx);
    for pair in pairs {
        let arg = parse_type_nlr(pair, ctx);
        let span = unite_span(ret.get_source(), arg.get_source());
        ret = type_tyapp(ret, arg).set_source(span);
    }
    ret.set_source(Some(span))
}

fn parse_type_nlr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_nlr);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_tycon => parse_type_tycon(pair, ctx),
        Rule::type_var => parse_type_var(pair, ctx),
        Rule::type_tuple => parse_type_tuple(pair, ctx),
        _ => unreachable!(),
    }
    .set_source(Some(span))
}

fn parse_type_var(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_var);
    let span = Span::from_pair(&ctx.source, &pair);
    type_tyvar(pair.as_str(), &kind_star()).set_source(Some(span))
}

fn parse_type_tycon(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_tycon);
    let span = Span::from_pair(&ctx.source, &pair);
    type_tycon(&parse_tycon(pair)).set_source(Some(span))
}

fn parse_tycon(pair: Pair<Rule>) -> Arc<TyCon> {
    assert_eq!(pair.as_rule(), Rule::type_tycon);
    tycon(parse_capital_fullname(pair.into_inner().next().unwrap()))
}

fn parse_type_tuple(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_tuple);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut types = vec![];
    let mut extra_comma = false;
    for pair in pair.into_inner() {
        if pair.as_rule() == Rule::extra_comma {
            extra_comma = true;
            break;
        }
        types.push(parse_type(pair, ctx));
    }
    let is_bracketed_type = types.len() == 1 && !extra_comma;
    if is_bracketed_type {
        types[0].clone()
    } else {
        let tuple_size = types.len();
        ctx.tuple_sizes.push(tuple_size as u32);
        let mut res = type_tycon(&tycon(make_tuple_name(tuple_size as u32)));
        for ty in types {
            res = type_tyapp(res, ty);
        }
        res
    }
    .set_source(Some(span))
}

fn parse_pattern(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern);
    let span = Span::from_pair(&ctx.source, &pair);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::pattern_var => parse_pattern_var(pair, ctx),
        Rule::pattern_tuple => parse_pattern_tuple(pair, ctx),
        Rule::pattern_struct => parse_pattern_struct(pair, ctx),
        Rule::pattern_union => parse_pattern_union(pair, ctx),
        _ => unreachable!(),
    }
    .set_source(span)
}

fn parse_pattern_var(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_var);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let var_name = pairs.next().unwrap().as_str();
    let ty = pairs.next().map(|ty| parse_type(ty, ctx));
    PatternNode::make_var(var_local(var_name), ty).set_source(span)
}

fn parse_pattern_tuple(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_tuple);
    let span = Span::from_pair(&ctx.source, &pair);
    let pairs = pair.into_inner();
    let pats = pairs
        .map(|pair| parse_pattern(pair, ctx))
        .collect::<Vec<_>>();
    let tuple_size = pats.len();
    ctx.tuple_sizes.push(tuple_size as u32);
    PatternNode::make_struct(
        tycon(make_tuple_name(tuple_size as u32)),
        pats.iter()
            .enumerate()
            .map(|(i, pat)| (i.to_string(), pat.clone()))
            .collect(),
    )
    .set_source(span)
}

fn parse_pattern_struct(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_struct);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.clone().into_inner();
    let tycon = parse_tycon(pairs.next().unwrap());
    let mut field_to_pats = Vec::default();
    while pairs.peek().is_some() {
        let field_name = pairs.next().unwrap().as_str().to_string();
        let pat = parse_pattern(pairs.next().unwrap(), ctx);
        field_to_pats.push((field_name, pat));
    }
    PatternNode::make_struct(tycon, field_to_pats).set_source(span)
}

fn parse_pattern_union(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_union);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let mut names = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::capital_name {
        names.push(pairs.next().unwrap().as_str().to_string());
    }
    let union_name = names.pop().unwrap();
    let union_namespace = NameSpace::new(names);
    let union_tycon = tycon(FullName::new(&union_namespace, &union_name));
    assert_eq!(pairs.peek().unwrap().as_rule(), Rule::type_field_name);
    let field_name = pairs.next().unwrap().as_str().to_string();
    let pat = parse_pattern(pairs.next().unwrap(), ctx);
    PatternNode::make_union(union_tycon, field_name, pat).set_source(span)
}

fn parse_import_statement(pair: Pair<Rule>, ctx: &mut ParseContext) -> ImportStatement {
    assert_eq!(pair.as_rule(), Rule::import_statement);
    let span = Span::from_pair(&ctx.source, &pair);
    let pair = pair.into_inner().next().unwrap();
    assert_eq!(pair.as_rule(), Rule::importee);
    let mut importee_pairs = pair.into_inner();
    let module_pair = importee_pairs.next().unwrap();
    let module_span = Span::from_pair(&ctx.source, &module_pair);
    let module = module_pair.as_str().to_string();
    let mut stmt = ImportStatement {
        importer: ctx.module_name.clone(),
        module,
        items: vec![ImportTreeNode::Any(Some(module_span))],
        hiding: vec![],
        source: Some(span),
        implicit: false,
    };
    for pair in importee_pairs {
        match pair.as_rule() {
            Rule::import_items_positive => {
                stmt.items = parse_import_items_positive(pair, ctx);
            }
            Rule::import_items_negative => {
                stmt.hiding = parse_import_items_negative(pair, ctx);
            }
            _ => unreachable!(),
        }
    }
    stmt
}

fn parse_import_items_positive(pair: Pair<Rule>, ctx: &mut ParseContext) -> Vec<ImportTreeNode> {
    assert_eq!(pair.as_rule(), Rule::import_items_positive);
    let pair = pair.into_inner().next().unwrap();
    parse_import_items(pair, ctx)
}

fn parse_import_items_negative(pair: Pair<Rule>, ctx: &mut ParseContext) -> Vec<ImportTreeNode> {
    assert_eq!(pair.as_rule(), Rule::import_items_negative);
    let pair = pair.into_inner().next().unwrap();
    parse_import_items(pair, ctx)
}

fn parse_import_items(pair: Pair<Rule>, ctx: &mut ParseContext) -> Vec<ImportTreeNode> {
    assert!(pair.as_rule() == Rule::import_items);
    pair.into_inner()
        .map(|pair| parse_import_item_node(pair, ctx))
        .collect()
}

fn parse_import_item_node(pair: Pair<Rule>, ctx: &mut ParseContext) -> ImportTreeNode {
    assert_eq!(pair.as_rule(), Rule::import_item_node);
    let pair = pair.into_inner().next().unwrap();
    let span = Span::from_pair(&ctx.source, &pair);
    match pair.as_rule() {
        Rule::import_item_any => ImportTreeNode::Any(Some(span)),
        Rule::import_item_symbol => ImportTreeNode::Symbol(pair.as_str().to_string(), Some(span)),
        Rule::import_item_capital_item => {
            let mut pairs = pair.into_inner();
            let capital_name = pairs.next().unwrap().as_str().to_string();
            if let Some(pair) = pairs.next() {
                ImportTreeNode::NameSpace(capital_name, parse_import_items(pair, ctx), Some(span))
            } else {
                ImportTreeNode::TypeOrTrait(capital_name, Some(span))
            }
        }
        _ => unreachable!(),
    }
}

fn rule_to_string(r: &Rule) -> String {
    fn join_by_or(tokens: &[&str]) -> String {
        tokens
            .iter()
            .map(|s| "`".to_string() + s + "`")
            .collect::<Vec<_>>()
            .join(" or ")
    }
    match r {
        Rule::EOI => "end-of-input".to_string(),
        Rule::expr_number_lit => "number literal".to_string(),
        Rule::expr_bool_lit => "boolean".to_string(),
        Rule::expr_nlr => "expression".to_string(),
        Rule::var => "variable".to_string(),
        Rule::in_of_let => "`in` or `;`".to_string(),
        Rule::eq_of_let => "`=`".to_string(),
        Rule::type_expr => "type".to_string(),
        Rule::arg_list => "list of arguments".to_string(),
        Rule::operator_mul => "`*`".to_string(),
        Rule::operator_plus => "`+`".to_string(),
        Rule::operator_and => "`&&`".to_string(),
        Rule::operator_or => "`||`".to_string(),
        Rule::type_nlr => "type".to_string(),
        Rule::operator_composition => join_by_or(&["<<", ">>"]),
        Rule::operator_cmp => join_by_or(&["==", "!=", "<=", ">=", "<", ">"]),
        Rule::trait_impl => "`impl`".to_string(),
        Rule::import_statement => "`import`".to_string(),
        Rule::module_defn => "module definition".to_string(),
        Rule::import_items_positive => "`::`".to_string(),
        Rule::import_items_negative => "`hiding`".to_string(),
        Rule::semicolon => "`;`".to_string(),
        Rule::extra_comma => ",".to_string(),
        Rule::export_symbol => "FFI_EXPORT".to_string(),
        Rule::global_defns => "definitions".to_string(),
        Rule::exported_c_function_name => "C function name".to_string(),
        _ => format!("{:?}", r),
    }
}

fn message_parse_error(e: Error<Rule>, src: &SourceFile) -> Errors {
    let mut msg: String = Default::default();

    #[allow(unused)]
    let mut suggestion: Option<String> = None;

    // Show error content.
    msg += "Expected ";
    match &e.variant {
        pest::error::ErrorVariant::ParsingError {
            positives,
            negatives,
        } => {
            fn concat_words(words: Vec<String>, sep: &str) -> String {
                let mut msg = String::from("");
                for (i, word) in words.iter().enumerate() {
                    let i = i as i32;
                    msg += word;
                    if i <= words.len() as i32 - 2 {
                        msg += &format!(" {} ", sep);
                    }
                }
                msg
            }
            if positives.len() > 0 {
                let words: Vec<String> = positives.iter().map(rule_to_string).collect();
                msg += &concat_words(words, "or");
                if negatives.len() > 0 {
                    msg += " and ";
                }
                msg += ".";
            }
        }
        pest::error::ErrorVariant::CustomError { message: _ } => unreachable!(),
    };
    if suggestion.is_some() {
        msg += "\n";
        msg += &suggestion.unwrap();
    }

    let src_string = src.string();
    if let Err(e) = src_string {
        return e;
    }
    let src_string = src_string.ok().unwrap();

    // Create span (source location).
    let span = match e.location {
        pest::error::InputLocation::Pos(s) => Span {
            input: src.clone(),
            start: s,
            end: min(s + 1, src_string.len()),
        },
        pest::error::InputLocation::Span((s, e)) => Span {
            input: src.clone(),
            start: s,
            end: e,
        },
    };

    Errors::from_msg_srcs(msg, &[&Some(span)])
}
