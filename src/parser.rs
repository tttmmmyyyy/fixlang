#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;

use crate::ast::{
    equality::Equality, predicate::Predicate, qual_pred::QualPred, qual_type::QualType,
};

use super::*;
use ast::{
    export_statement::ExportStatement,
    import::{ImportStatement, ImportTreeNode},
    name::{FullName, NameSpace},
};
use either::Either;
use error::Errors;
use misc::{make_map, save_temporary_source, Map};
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
    fn push_monad(&mut self, monad: Arc<ExprNode>, star_src: Span) -> Arc<ExprNode> {
        let var_name = FullName::local(&format!("#monadic_value{}", self.counter));
        let var_var = var_var(var_name.clone());
        let var_expr = expr_var(var_name, Some(star_src.clone()));
        self.counter += 1;
        self.monads.push(BindOperatorInfo {
            operator_src: star_src,
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
            let mut bind_function = FullName::from_strs(&[STD_NAME, MONAD_NAME], MONAD_BIND_NAME);
            bind_function.global_to_absolute();
            let bind_function = expr_var(bind_function, Some(operator_src));
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
        Some(lhs) => match rhs {
            None => return Some(lhs.clone()),
            Some(rhs) => {
                return Some(Span::unite(lhs, rhs));
            }
        },
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
    let src = save_temporary_source(source, file_name)?;
    parse_source_file(src, config)
}

pub fn parse_file_path(file_path: PathBuf, config: &Configuration) -> Result<Program, Errors> {
    let source = SourceFile::from_file_path(file_path);
    parse_source_file(source, config)
}

pub fn parse_source_file(source: SourceFile, config: &Configuration) -> Result<Program, Errors> {
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

pub fn parse_str_import_statements(
    file_path: PathBuf,
    src: &str,
) -> Result<Vec<ImportStatement>, Errors> {
    parse_str_as_rule(
        file_path,
        src,
        Rule::file_only_import_statements,
        parse_import_statements,
    )
}

pub fn parse_str_module_defn(file_path: PathBuf, src: &str) -> Result<ModuleInfo, Errors> {
    parse_str_as_rule(file_path, src, Rule::file_only_module_defn, |rule, ctx| {
        let rule = rule.into_inner().next().unwrap();
        Ok(parse_module_defn(rule, ctx))
    })
}

fn parse_str_as_rule<T>(
    file_path: PathBuf,
    src: &str,
    rule: Rule,
    parser: impl Fn(Pair<Rule>, &mut ParseContext) -> Result<T, Errors>,
) -> Result<T, Errors> {
    let mut file = match FixParser::parse(rule, src) {
        Ok(res) => res,
        Err(e) => {
            return Err(Errors::from_msg(format!(
                "Failed to parse string as rule {:?}: {}",
                rule, e
            )));
        }
    };
    let source = SourceFile::from_file_path_and_content(file_path, src.to_string());
    let config = Configuration::diagnostics_mode(DiagnosticsConfig::default())?; // Use any Configuration
    let mut ctx = ParseContext::from_source(source, &config);
    parser(file.next().unwrap(), &mut ctx)
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
    let mod_info = parse_module_defn(pairs.next().unwrap(), &mut ctx);

    ctx.module_name = mod_info.name.clone();
    ctx.namespace = NameSpace::new(vec![mod_info.name.clone()]);

    let mut fix_mod = Program::single_module(mod_info);

    let mut type_defns: Vec<TypeDefn> = Vec::new();
    let mut global_value_decls: Vec<GlobalValueDecl> = vec![];
    let mut global_value_defns: Vec<GlobalValueDefn> = vec![];
    let mut trait_infos: Vec<TraitDefn> = vec![];
    let mut trait_aliases: Vec<TraitAlias> = vec![];
    let mut trait_impls: Vec<TraitImpl> = vec![];
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
                &mut trait_impls,
                &mut export_statements,
            )),
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
    trait_infos: &mut Vec<TraitDefn>,
    trait_aliases: &mut Vec<TraitAlias>,
    trait_impls: &mut Vec<TraitImpl>,
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
                    trait_impls,
                    export_statements,
                ));
            }
            Rule::type_defn => {
                errors.eat_err_or(parse_type_defn(pair, ctx), |td| type_defns.push(td));
            }
            Rule::global_name_type_sign => {
                errors.eat_err_or(parse_global_value_decl(pair, ctx), |(gvt, gvd)| {
                    global_value_decls.push(gvt);
                    if let Some(gvd) = gvd {
                        global_value_defns.push(gvd)
                    }
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
            Rule::trait_impl => {
                errors.eat_err_or(parse_trait_impl(pair, ctx), |ti| trait_impls.push(ti));
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
    trait_infos: &mut Vec<TraitDefn>,
    trait_aliases: &mut Vec<TraitAlias>,
    trait_impls: &mut Vec<TraitImpl>,
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
            trait_impls,
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
    let name_pair = pairs.next().unwrap();
    let name_span = Span::from_pair(&ctx.source, &name_pair);
    let id = TraitId::from_fullname(FullName::new(
        &ctx.namespace,
        &name_pair.as_str().to_string(),
    ));
    let mut values = vec![];
    for pair in pairs {
        if pair.as_rule() != Rule::trait_fullname {
            break;
        }
        let span = Span::from_pair(&ctx.source, &pair);
        values.push((parse_trait_fullname(pair, ctx), span));
    }
    TraitAlias {
        id,
        value: values,
        source: Some(span),
        name_src: Some(name_span),
        kind: kind_star(), // Will be set to a correct value in TraitEnv::set_kinds.
    }
}

fn parse_trait_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<TraitDefn, Errors> {
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
                "In the constraint of trait definition, only kind signature is allowed. Fix does not support \"super-traits\".".to_string(),
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
    let trait_name_pair = pairs.next().unwrap();
    let trait_name_span = Span::from_pair(&ctx.source, &trait_name_pair);
    let trait_name = trait_name_pair.as_str().to_string();
    let mut methods: Vec<TraitMember> = vec![];
    let mut type_syns: Map<Name, AssocTypeDefn> = Map::default();
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
    Ok(TraitDefn {
        trait_: TraitId::from_fullname(FullName::new(&ctx.namespace, &trait_name)),
        type_var: make_tyvar(&trait_tyvar, &kind_star()),
        members: methods,
        assoc_types: type_syns,
        kind_signs: kinds,
        source: Some(span),
        name_src: Some(trait_name_span),
        document: None,
    })
}

fn parse_trait_member_defn(
    pair: Pair<Rule>,
    impl_type: &Arc<TypeNode>,
    ctx: &mut ParseContext,
) -> Result<Either<TraitMember, AssocTypeDefn>, Errors> {
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
) -> Result<TraitMember, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_value_defn);
    let _span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let method_name_pair = pairs.next().unwrap();
    let method_name_span = Span::from_pair(&ctx.source, &method_name_pair);
    let method_name = method_name_pair.as_str().to_string();
    let qual_type = parse_type_qualified(pairs.next().unwrap(), ctx)?;
    Ok(TraitMember {
        name: method_name,
        qual_ty: qual_type,
        syn_qual_ty: None,
        decl_src: Some(method_name_span),
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
    let (assoc_type_name, assoc_type_name_src, assoc_type_params) =
        assoc_type_defn.validate_as_associated_type_defn(impl_type, &Some(span.clone()), false)?;
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
        name_src: assoc_type_name_src,
        params: assoc_type_params,
        kind_signs,
    })
}

fn parse_trait_impl(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<TraitImpl, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_impl);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let qual_pred = parse_predicate_qualified(pairs.next().unwrap(), ctx)?;
    let impl_type = qual_pred.predicate.ty.clone();
    let mut value_impls: Map<Name, Arc<ExprNode>> = Map::default();
    let mut value_lhs_srcs: Map<Name, Vec<Span>> = Map::default();
    let mut value_type_sigs: Map<Name, QualType> = Map::default();
    let mut assoc_types: Map<Name, AssocTypeImpl> = Map::default();
    for pair in pairs {
        match parse_trait_member_impl(pair, &impl_type, ctx)? {
            TraitMemberImpl::Value { name, expr, name_span } => {
                if value_impls.contains_key(&name) {
                    return Err(Errors::from_msg_srcs(
                        format!("Duplicate implementation of member `{}`.", name),
                        &[&Some(span)],
                    ));
                }
                value_lhs_srcs.entry(name.clone()).or_default().push(name_span);
                value_impls.insert(name, expr);
            }
            TraitMemberImpl::TypeSig { name, type_sign, opt_expr, name_span } => {
                if value_type_sigs.contains_key(&name) {
                    return Err(Errors::from_msg_srcs(
                        format!("Duplicate the type signature of member `{}`.", name),
                        &[&Some(span)],
                    ));
                }
                value_lhs_srcs.entry(name.clone()).or_default().push(name_span);
                value_type_sigs.insert(name.clone(), type_sign);
                if let Some(expr) = opt_expr {
                    if value_impls.contains_key(&name) {
                        return Err(Errors::from_msg_srcs(
                            format!("Duplicate implementation of member `{}`.", name),
                            &[&Some(span)],
                        ));
                    }
                    value_impls.insert(name, expr);
                }
            }
            TraitMemberImpl::Type(assoc_type_impl) => {
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
    Ok(TraitImpl {
        qual_pred,
        members: value_impls,
        member_lhs_srcs: value_lhs_srcs,
        member_sigs: value_type_sigs,
        assoc_types,
        define_module: ctx.module_name.clone(),
        source: Some(span),
        is_user_defined: true,
    })
}

enum TraitMemberImpl {
    Value {
        name: Name,
        expr: Arc<ExprNode>,
        name_span: Span,
    },
    TypeSig {
        name: Name,
        type_sign: QualType,
        opt_expr: Option<Arc<ExprNode>>,
        name_span: Span,
    },
    Type(AssocTypeImpl),
}

fn parse_trait_member_impl(
    pair: Pair<Rule>,
    impl_type: &Arc<TypeNode>,
    ctx: &mut ParseContext,
) -> Result<TraitMemberImpl, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_impl);
    let pair = pair.into_inner().next().unwrap();
    Ok(match pair.as_rule() {
        Rule::trait_member_value_impl => {
            let (name, expr, name_span) = parse_trait_member_value_impl(pair, ctx)?;
            TraitMemberImpl::Value { name, expr, name_span }
        }
        Rule::trait_member_value_type_sign => {
            let (name, type_sign, opt_expr, name_span) = parse_trait_member_value_type_sign(pair, ctx)?;
            TraitMemberImpl::TypeSig { name, type_sign, opt_expr, name_span }
        }
        Rule::trait_member_type_impl => {
            TraitMemberImpl::Type(parse_trait_member_type_impl(pair, impl_type, ctx)?)
        }
        _ => unreachable!(),
    })
}

fn parse_trait_member_value_impl(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<(Name, Arc<ExprNode>, Span), Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_value_impl);
    let mut pairs = pair.into_inner();
    let name_pair = pairs.next().unwrap();
    let name_span = Span::from_pair(&ctx.source, &name_pair);
    let method_name = name_pair.as_str().to_string();
    let expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    Ok((method_name, expr, name_span))
}

fn parse_trait_member_value_type_sign(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<(Name, QualType, Option<Arc<ExprNode>>, Span), Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_value_type_sign);
    let mut pairs = pair.into_inner();
    let name_pair = pairs.next().unwrap();
    let name_span = Span::from_pair(&ctx.source, &name_pair);
    let method_name = name_pair.as_str().to_string();
    let qual_type = parse_type_qualified(pairs.next().unwrap(), ctx)?;
    let mut opt_expr = None;
    if let Some(pair) = pairs.peek() {
        if pair.as_rule() == Rule::expr {
            opt_expr = Some(parse_expr_with_new_do(pairs.next().unwrap(), ctx)?);
        }
    }
    Ok((method_name, qual_type, opt_expr, name_span))
}

fn parse_trait_member_type_impl(
    pair: Pair<Rule>,
    impl_type: &Arc<TypeNode>,
    ctx: &mut ParseContext,
) -> Result<AssocTypeImpl, Errors> {
    assert_eq!(pair.as_rule(), Rule::trait_member_type_impl);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let assoc_type_application = parse_type(pairs.next().unwrap(), ctx);
    let (assoc_type_name, name_src, params) = assoc_type_application.validate_as_associated_type_defn(
        impl_type,
        &Some(span.clone()),
        true,
    )?;
    let type_value = parse_type(pairs.next().unwrap(), ctx);
    Ok(AssocTypeImpl {
        name: assoc_type_name,
        params,
        value: type_value,
        source: Some(span),
        name_src,
    })
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

fn parse_predicate_qualified(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<QualPred, Errors> {
    assert_eq!(pair.as_rule(), Rule::predicate_qualified);
    let mut pairs = pair.into_inner();
    let (predicates, eqs, kinds) = if pairs.peek().unwrap().as_rule() == Rule::constraints {
        parse_constraints(pairs.next().unwrap(), ctx)?
    } else {
        (vec![], vec![], vec![])
    };
    let predicate = parse_predicate(pairs.next().unwrap(), ctx);
    let qp = QualPred {
        pred_constraints: predicates,
        eq_constraints: eqs,
        kind_constraints: kinds,
        predicate,
    };
    Ok(qp)
}

// Parse `name : Type;` or `name : Type = expr;`
fn parse_global_value_decl(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<(GlobalValueDecl, Option<GlobalValueDefn>), Errors> {
    assert_eq!(pair.as_rule(), Rule::global_name_type_sign);
    let _span = Span::from_pair(&ctx.source, &pair);

    // Parse name.
    let mut pairs = pair.into_inner();
    let name_pair = pairs.next().unwrap();
    let name_src = Span::from_pair(&ctx.source, &name_pair);
    let name = name_pair.as_str().to_string();
    let name = FullName::new(&ctx.namespace, &name);

    // Parse type.
    let qual_type = parse_type_qualified(pairs.next().unwrap(), ctx)?;
    let kind_sings = qual_type.kind_signs.clone();
    let preds = qual_type.preds.clone();
    let eqs = qual_type.eqs.clone();
    let ty = qual_type.ty.clone();
    let ty = Scheme::generalize(&kind_sings, preds, eqs, ty);

    // Parse expression (if exists).
    let mut gvd = None;
    if let Some(pair) = pairs.peek() {
        if pair.as_rule() == Rule::expr {
            let expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
            gvd = Some(GlobalValueDefn {
                name: name.clone(),
                expr,
                src: Some(name_src.clone()),
            });
        }
    }

    Ok((
        GlobalValueDecl {
            name: name,
            ty,
            src: Some(name_src),
        },
        gvd,
    ))
}

fn parse_global_name_defn(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<GlobalValueDefn, Errors> {
    assert_eq!(pair.as_rule(), Rule::global_name_defn);
    let _span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let name_pair = pairs.next().unwrap();
    let name_src = Span::from_pair(&ctx.source, &name_pair);
    let name = name_pair.as_str().to_string();
    let expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    Ok(GlobalValueDefn {
        name: FullName::new(&ctx.namespace, &name),
        expr: expr,
        src: Some(name_src),
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
        assoc_type: AssocType {
            name: lhs_seq[0].as_tycon().name.clone(),
            source: lhs_seq[0].get_source().clone(),
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

fn parse_kind(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind);
    let pairs = pair.into_inner();
    let mut kinds = pairs
        .map(|pair| parse_kind_nlr(pair, ctx))
        .collect::<Vec<_>>();
    let mut res: Arc<Kind> = kinds.pop().unwrap();
    while kinds.len() > 0 {
        let pair = kinds.pop().unwrap();
        res = kind_arrow(pair, res);
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

fn parse_module_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> ModuleInfo {
    assert_eq!(pair.as_rule(), Rule::module_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mod_name = pair.into_inner().next().unwrap().as_str().to_string();
    ModuleInfo {
        name: mod_name,
        source: span,
    }
}

fn parse_type_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<TypeDefn, Errors> {
    assert_eq!(pair.as_rule(), Rule::type_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();

    // Parse constraints to specify kinds of type variables.
    let mut kinds: Map<Name, Arc<Kind>> = Map::default();
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
    let name_pair = pairs.next().unwrap();
    let name_span = Span::from_pair(&ctx.source, &name_pair);
    let name = name_pair.as_str();
    let mut tyvars: Vec<Arc<TyVar>> = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::type_var {
        let tyvar_name_pair = pairs.next().unwrap();
        let tyvar_name = tyvar_name_pair.as_str();
        let kind = kinds.get(tyvar_name).unwrap_or(&kind_star()).clone();
        let tv = make_tyvar(tyvar_name, &kind);
        tyvars.push(tv);
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
        name_src: Some(name_span),
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
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let ty = parse_type(pairs.next().unwrap(), ctx);
    Field::make(name.to_string(), ty, Some(span))
}

fn parse_expr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_and_then_sequence(pair, ctx)
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

fn parse_expr_and_then_sequence(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    let mut exprs = vec![];
    let mut op_spans = vec![];
    let mut pairs = pair.into_inner();
    while pairs.peek().is_some() {
        let pair = pairs.next().unwrap();
        let expr = if exprs.len() == 0 {
            parse_expr_type_annotation(pair, ctx)?
        } else {
            parse_expr_with_new_do(pair, ctx)?
        };
        exprs.push(expr);
        if pairs.peek().is_some() {
            let pair = pairs.next().unwrap();
            assert!(pair.as_rule() == Rule::operator_and_then);
            op_spans.push(Span::from_pair(&ctx.source, &pair));
        }
    }
    exprs.reverse();
    op_spans.reverse();
    let mut expr = exprs.pop().unwrap();
    while exprs.len() > 0 {
        let next_expr = exprs.pop().unwrap();
        let next_expr_span = next_expr.source.clone();
        let op_span = op_spans.pop().unwrap();
        let mut bind_function = FullName::from_strs(&[STD_NAME, MONAD_NAME], MONAD_BIND_NAME);
        bind_function.global_to_absolute();
        let bind_function = expr_var(bind_function, Some(op_span.clone()));
        let bind_next_expr_span = unite_span(&Some(op_span), &next_expr_span);
        let lazy_next_expr = expr_abs(vec![var_local(PARAM_NAME)], next_expr, next_expr_span);
        let bind_next_expr = expr_app(
            bind_function,
            vec![lazy_next_expr],
            bind_next_expr_span.clone(),
        )
        .set_app_order(AppSourceCodeOrderType::FX);
        let expr_span = expr.source.clone();
        expr = expr_app(
            bind_next_expr,
            vec![expr],
            unite_span(&bind_next_expr_span, &expr_span),
        )
        .set_app_order(AppSourceCodeOrderType::XDotF);
    }
    Ok(expr)
}

// Parse combinator sequence, e.g., `f $ x $ y`
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

    fn method_fullname(&self) -> FullName {
        let mut fullname = FullName::from_strs(&[STD_NAME, &self.trait_name], &self.method_name);
        fullname.global_to_absolute();
        fullname
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
    ops: Map<&str, BinaryOpInfo>,
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
                    expr_var(next_operation.method_fullname(), next_op_span.clone()),
                    vec![lhs],
                    span.clone(),
                ),
                vec![rhs],
                span.clone(),
            );
            match next_operation.post_unary.as_ref() {
                Some(op) => {
                    expr = expr_app(
                        expr_var(op.method_fullname(), next_op_span.clone()),
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
        make_map([
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
        make_map([
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
        make_map([
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

    fn method_fullname(&self) -> FullName {
        let mut fullname = FullName::from_strs(&[STD_NAME, &self.trait_name], &self.method_name);
        fullname.global_to_absolute();
        fullname
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
                        expr_var(op.method_fullname(), Some(op_span.clone())),
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
        let mut compose_function = FullName::from_strs(&[STD_NAME], COMPOSE_FUNCTION_NAME);
        compose_function.global_to_absolute();
        let compose = expr_var(compose_function, Some(op_span));
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
    let mut expr = parse_expr_dot_sequence_of_index(pairs.next().unwrap(), ctx)?;
    while !stars.is_empty() {
        let star_src = stars.pop().unwrap();
        expr = ctx.do_context.push_monad(expr, star_src);
    }
    Ok(expr)
}

// Parse sequence of dot application and index syntax, e.g., `a.b[c].d[e][^f]`.
//
// Such sequence is should be parsed left-to-right: `(((a.b)[c]).d)([e]<<[^f])` where `<<` represents lens composition.
fn parse_expr_dot_sequence_of_index(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_dot_seq);
    let mut expr: Option<Arc<ExprNode>> = None;
    for pair in pair.into_inner() {
        let expr_index_src = Span::from_pair(&ctx.source, &pair);
        // Parse one component of dot sequence, e.g., `a`, `b[c]`, `d[e][^f]`
        let (x, is) = parse_expr_index(pair, ctx)?;
        // Example1: `expr` = `Some(obj)`, `x` = `arr`, `is` = `vec![ [i], [^field] ]`
        // In this case, we need to create expression like `(obj.arr)([i]<<[^field])` where `<<` represents lens composition.
        //
        // Example2: `expr` = `None`, `x` = `arr`, `is` = `vec![ [i], [^field] ]`
        // In this case, we need to create expression like `arr([i]<<[^field])
        //
        // Set `x` = `obj.x` in case of Example1.
        let x = match expr {
            Some(expr) => {
                let span = unite_span(&x.source, &expr.source);
                expr_app(x, vec![expr], span).set_app_order(AppSourceCodeOrderType::XDotF)
            }
            None => x,
        };

        // Then, construct index syntax `x([i]<<[^field])` from `x` and `is`.
        expr = Some(construct_expr_index(x, is, Some(expr_index_src))?);
    }

    Ok(expr.unwrap())
}

// `[i]` or `[^field]`
struct IndexSyntax {
    accessor: IndexAccessor,
    source: Option<Span>,
}

// `i` or `field` in IndexSyntax
enum IndexAccessor {
    Expr(Arc<ExprNode>),
    Field(FullName, Option<Span>),
}

// Parse expression with index syntax, e.g., `arr[expr][^field][^0]`
fn parse_expr_index(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<(Arc<ExprNode>, Vec<IndexSyntax>), Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_index);
    let mut pairs = pair.into_inner();
    let expr = parse_expr_app(pairs.next().unwrap(), ctx)?;
    let mut indices = vec![];
    for pair in pairs {
        indices.push(parse_index_syntax(pair, ctx)?);
    }
    Ok((expr, indices))
}

// Given expression `x` and possibly empty list of index syntax `vec![[i], [^field]]`,
// construct expression `x([i]<<[^field])` where `<<` represents lens composition.
//
// More specifically, construct expression like:
// ```
// |#a| x.(act_at_index(i) $ act_field $ #a)
// ```
fn construct_expr_index(
    expr: Arc<ExprNode>,
    indices: Vec<IndexSyntax>,
    expr_index_src: Option<Span>, // span covering expr and indices
) -> Result<Arc<ExprNode>, Errors> {
    if indices.len() == 0 {
        return Ok(expr);
    }
    const ACTION_NAME: &str = "#a";
    let action_arg = var_local(ACTION_NAME);
    let action_var = expr_var(FullName::local(ACTION_NAME), None);
    let mut action = action_var;
    for index in indices.into_iter().rev() {
        match index.accessor {
            IndexAccessor::Expr(index_expr) => {
                let mut act_at_index = FullName::from_strs(
                    &[STD_NAME, INDEXABLE_TRAIT_NAME],
                    INDEXABLE_TRAIT_ACT_NAME,
                );
                act_at_index.set_absolute();
                let act_func = expr_app(
                    expr_var(act_at_index, index.source.clone()),
                    vec![index_expr],
                    index.source.clone(),
                );
                let new_action_span = unite_span(&index.source, &action.source);
                action = expr_app(act_func, vec![action], new_action_span);
            }
            IndexAccessor::Field(field_name, field_span) => {
                let mut act_func_name = field_name.clone();
                // field_name = `Std::Box::value` => act_func_name = `Std::Box::act_value`
                *act_func_name.name_as_mut() = format!("{}{}", STRUCT_ACT_SYMBOL, field_name.name);
                let act_func = expr_var(act_func_name, field_span.clone())
                    .set_struct_act_func_in_index_syntax(true);
                let new_action_span = unite_span(&field_span, &action.source);
                action = expr_app(act_func, vec![action], new_action_span);
            }
        }
    }
    let expr = expr_abs(
        vec![action_arg],
        expr_app(action, vec![expr], expr_index_src.clone())
            .set_app_order(AppSourceCodeOrderType::XDotF),
        expr_index_src,
    );
    Ok(expr)
}

// Parse index syntax, e.g., `[expr]`, `[^field]`, `[^0]`
fn parse_index_syntax(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<IndexSyntax, Errors> {
    // Parse index accessor inside `[...]`
    fn parse_index_accessor(
        pair: Pair<Rule>,
        ctx: &mut ParseContext,
    ) -> Result<IndexAccessor, Errors> {
        assert_eq!(pair.as_rule(), Rule::index_accessor);
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        let src = Span::from_pair(&ctx.source, &pair);
        let accessor = match pair.as_rule() {
            Rule::expr => {
                let expr = parse_expr(pair, ctx)?;
                IndexAccessor::Expr(expr)
            }
            Rule::field_accessor => {
                let pair = pair.into_inner().next().unwrap();
                let field_name = parse_fullname(pair);
                IndexAccessor::Field(field_name, Some(src))
            }
            Rule::tuple_accessor => {
                let pair = pair.into_inner().next().unwrap();
                let field_name = parse_number_fullname(pair);
                IndexAccessor::Field(field_name, Some(src))
            }
            _ => unreachable!(),
        };
        Ok(accessor)
    }

    assert_eq!(pair.as_rule(), Rule::index_syntax);
    let pairs = pair.into_inner();
    let pair = pairs.into_iter().next().unwrap();
    let span = Span::from_pair(&ctx.source, &pair);
    Ok(IndexSyntax {
        accessor: parse_index_accessor(pair, ctx)?,
        source: Some(span),
    })
}

// Parse application sequence, e.g., `f(x, y)`. (left-associative)
fn parse_expr_app(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_app);
    let mut pairs = pair.into_inner();
    let head = parse_expr_nlr(pairs.next().unwrap(), ctx)?;
    let mut args = vec![];
    while pairs.peek().is_some() {
        // If parentheses for arguments are given,
        let pair = pairs.next().unwrap();
        let args_span = Span::from_pair(&ctx.source, &pair);
        let mut args_local = parse_arg_list(pair, ctx)?;
        if args_local.len() == 0 {
            // `f()` is interpreted as application to unit: `f $ ()`.
            args_local.push(
                expr_make_struct(tycon(make_tuple_name_abs(0)), vec![]).set_source(Some(args_span)),
            )
        }
        args.append(&mut args_local);
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
        Rule::expr_match => parse_expr_match(pair, ctx)?,
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
    let pair = pair.into_inner().next().unwrap();
    let name = parse_fullname(pair);
    expr_var(name, Some(span))
}

fn parse_fullname(pair: Pair<Rule>) -> FullName {
    assert_eq!(pair.as_rule(), Rule::fullname);
    parse_fullname_or_capital_fullname(pair)
}

fn parse_capital_fullname(pair: Pair<Rule>) -> FullName {
    assert_eq!(pair.as_rule(), Rule::capital_fullname);
    parse_fullname_or_capital_fullname(pair)
}

fn parse_number_fullname(pair: Pair<Rule>) -> FullName {
    assert_eq!(pair.as_rule(), Rule::number_fullname);
    parse_fullname_or_capital_fullname(pair)
}

fn parse_fullname_or_capital_fullname(pair: Pair<Rule>) -> FullName {
    assert!(
        pair.as_rule() == Rule::fullname
            || pair.as_rule() == Rule::capital_fullname
            || pair.as_rule() == Rule::number_fullname
    );
    let mut pairs = pair.into_inner();
    let mut fullname = FullName::local("");
    while let Some(pair) = pairs.next() {
        if pair.as_rule() == Rule::namespace_item {
            fullname.namespace.names.push(pair.as_str().to_string());
        } else if pair.as_rule() == Rule::double_colon {
            if fullname.namespace.names.is_empty() {
                // If the namespace starts with `::`, it is an absolute namespace.
                fullname.namespace.set_absolute();
            }
        } else {
            assert!(
                pair.as_rule() == Rule::name
                    || pair.as_rule() == Rule::capital_name
                    || pair.as_rule() == Rule::number_name
            );
            fullname.name = pair.as_str().to_string();
            break;
        }
    }
    fullname
}

fn parse_namespace(pair: Pair<Rule>, _ctx: &mut ParseContext) -> NameSpace {
    assert_eq!(pair.as_rule(), Rule::namespace);
    let pairs = pair.into_inner();
    let mut ret: Vec<String> = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::namespace_item {
            ret.push(pair.as_str().to_string());
        } else {
            assert_eq!(pair.as_rule(), Rule::double_colon);
        }
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

// Parse the expression `let {pat1} = {bound1}; let {pat2} = {bound2}; ...; {value}`.
fn parse_expr_let(expr: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    let pairs = expr.into_inner();
    parse_expr_let_recursively(pairs, ctx)
}

// Parse the expression `let {pat1} = {bound1}; let {pat2} = {bound2}; ...; {value}`.
fn parse_expr_let_recursively(
    mut pairs: Pairs<Rule>,
    ctx: &mut ParseContext,
) -> Result<Arc<ExprNode>, Errors> {
    let pair = pairs.next().unwrap();
    if pair.as_rule() != Rule::keyword_let {
        // `pair` is `{value}`.
        let val = parse_expr(pair, ctx)?;
        Ok(val)
    } else {
        // `pair` is `let` keyword.
        assert_eq!(pair.as_rule(), Rule::keyword_let);
        let let_span = Span::from_pair(&ctx.source, &pair);

        // Parse `{pat}` part.
        let pat = parse_pattern_nounion(pairs.next().unwrap(), ctx);
        let _eq = pairs.next().unwrap(); // Skip "=".
        assert_eq!(_eq.as_rule(), Rule::eq_of_let);

        // Parse `{bound}` part.
        let bound = parse_expr(pairs.next().unwrap(), ctx)?;
        let _semicolon = pairs.next().unwrap(); // Skip ";" or "in".
        assert_eq!(_semicolon.as_rule(), Rule::in_of_let);

        // Parse the rest of the expression recursively.
        // Here we create a new DoContext.
        let old_doctx = std::mem::replace(&mut ctx.do_context, DoContext::default());
        let value = parse_expr_let_recursively(pairs, ctx)?;
        let value = ctx.do_context.expand_binds(value);
        ctx.do_context = old_doctx; // Restore old DoContext.

        // Create the let expression.
        let span = unite_span(&value.source, &Some(let_span));
        let expr = expr_let(pat, bound, value, span);
        // Set the source of the expression.
        Ok(expr)
    }
}

fn parse_expr_eval(pair: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(pair.as_rule(), Rule::expr_eval);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let sub = parse_expr(pairs.next().unwrap(), ctx)?;
    pairs.next().unwrap(); // Skip `Rule::semicolon`.
    let main = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    Ok(expr_eval(sub, main, Some(span)))
}

fn parse_expr_lam(expr: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    let span = Span::from_pair(&ctx.source, &expr);
    let mut pairs = expr.into_inner();
    let mut pats = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::pattern_nounion {
        let pat = parse_pattern_nounion(pairs.next().unwrap(), ctx);
        pats.push(pat);
    }
    let mut expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx)?;
    let mut pat_body_span = expr.source.clone();
    let var = var_local(PARAM_NAME);
    for pat in pats.iter().rev() {
        pat_body_span = Span::unite_opt(&pat_body_span, &pat.info.source);
        expr = expr_abs_param_src(
            vec![var.clone()],
            expr_let(
                pat.clone(),
                expr_var(FullName::local(PARAM_NAME), pat.info.source.clone()),
                expr,
                pat_body_span.clone(),
            ),
            pat_body_span.clone(),
            pat.info.source.clone(),
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

fn parse_expr_match(expr: Pair<Rule>, ctx: &mut ParseContext) -> Result<Arc<ExprNode>, Errors> {
    assert_eq!(expr.as_rule(), Rule::expr_match);
    let span = Span::from_pair(&ctx.source, &expr);
    let mut pairs = expr.into_inner();
    let cond = pairs.next().unwrap();
    let cond = parse_expr(cond, ctx)?;
    let mut cases = vec![];
    while pairs.peek().is_some() {
        let pair = pairs.next().unwrap();
        let pat = parse_pattern_case(pair, ctx); // Parse pattern.
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::match_arrow); // Skip `=>`.
        let pair = pairs.next().unwrap();
        let expr = parse_expr_with_new_do(pair, ctx)?; // Parse value expression.
        if pairs.peek().is_some() {
            // Skip `,` if exists.
            let pair = pairs.next().unwrap();
            assert_eq!(pair.as_rule(), Rule::comma_);
        }
        cases.push((pat, expr));
    }
    // Forbid empty match.
    if cases.is_empty() {
        return Err(Errors::from_msg_srcs(
            "Empty `match` is not allowed.".to_string(),
            &[&Some(span)],
        ));
    }
    Ok(expr_match(cond, cases, Some(span)))
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
        if pair.as_rule() == Rule::comma_ {
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
            tycon(make_tuple_name_abs(tuple_size as u32)),
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

    // Get type of FFI_CALL (pure, IO, or IOS).
    let call_ffi_pair = pairs.next().unwrap();
    let is_io = call_ffi_pair.as_rule() == Rule::ffi_call_c_io_symbol;
    let is_ios = call_ffi_pair.as_rule() == Rule::ffi_call_c_ios_symbol;

    let ret_ty = parse_ffi_c_fun_ty(pairs.next().unwrap(), ctx);
    let fun_name = pairs.next().unwrap().as_str().to_string();
    let param_tys = parse_ffi_param_tys(pairs.next().unwrap(), ctx);

    let mut is_var_args = false;
    if let Some(pair) = pairs.peek() {
        if pair.as_rule() == Rule::ffi_va_args {
            is_var_args = true;
            pairs.next();
        }
    }

    let mut args = pairs
        .map(|pair| parse_expr(pair, ctx))
        .collect::<Result<Vec<_>, _>>()?;

    // Validate number of arguments.
    let required_arg_num = param_tys.len() + if is_ios { 1 } else { 0 };
    let wrong_arg_num = (!is_var_args && args.len() != required_arg_num)
        || (is_var_args && args.len() < required_arg_num);
    if wrong_arg_num {
        return Err(Errors::from_msg_srcs(
            format!(
                "Wrong number of arguments in FFI_CALL{} expression.",
                if is_io {
                    "_IO"
                } else if is_ios {
                    "_IOS"
                } else {
                    ""
                }
            ),
            &[&Some(span)],
        ));
    }

    let expr = if is_io {
        // Wrap the function call with IO.
        const IOS_NAME: &str = "#ios";
        args.push(expr_var(FullName::local(IOS_NAME), None));
        let ffi_call = expr_ffi_call(
            fun_name,
            ret_ty,
            param_tys,
            is_var_args,
            args,
            true,
            Some(span.clone()),
        );
        let runner = expr_abs(vec![var_local(IOS_NAME)], ffi_call, Some(span.clone()));
        expr_make_struct(
            make_io_tycon().global_to_absolute(),
            vec![(IO_DATA_NAME.to_string(), runner)],
        )
        .set_source(Some(span))
    } else {
        expr_ffi_call(
            fun_name,
            ret_ty,
            param_tys,
            is_var_args,
            args,
            is_ios,
            Some(span),
        )
    };
    Ok(expr)
}

fn parse_ffi_c_fun_ty(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<TyCon> {
    assert_eq!(pair.as_rule(), Rule::ffi_c_fun_ty);
    let mut name = if pair.as_str() == "()" {
        make_tuple_name_abs(0)
    } else {
        let mut name = pair.as_str().to_string();
        for (c_type_name, sign, size) in ctx.config.c_type_sizes.get_c_types() {
            if c_type_name == pair.as_str() {
                name = format!("{}{}", sign, size);
            }
        }
        FullName::from_strs(&[STD_NAME], &name)
    };
    name.set_absolute();
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
    let ty = ty.set_source(Some(span.clone()));
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
        let opt_val_radix = parse_integer_literal_string(val_str);
        if opt_val_radix.is_none() {
            return Err(Errors::from_msg_srcs(
                format!(
                    "A literal string `{}` cannot be parsed as an integer.",
                    val_str
                ),
                &[&Some(span)],
            ));
        }
        let (val, radix) = opt_val_radix.unwrap();

        // Check size.
        if radix == 10 || radix == 8 {
            // Decimal or octal
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
        } else {
            // Binary or hexadecimal
            // In this case, 0b1111111_I8 should be successfully parsed to -1, so check the range as the unsigned value.
            let val_abs = if val < BigInt::parse_bytes(b"0", 10).unwrap() {
                -val.clone()
            } else {
                val.clone()
            };
            let ty_name_unsigned = if ty_name.starts_with('I') {
                ty_name.replacen('I', "U", 1)
            } else {
                ty_name.to_string()
            };
            let (ty_min_unsigned, ty_max_unsigned) = integral_ty_range(&ty_name_unsigned);
            if !(ty_min_unsigned <= val_abs && val_abs <= ty_max_unsigned) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "The value of an integer literal `{}` is out of range of `{}`.",
                        val_str, ty_name_unsigned
                    ),
                    &[&Some(span)],
                ));
            }
        }
        // Now stringify val and parse it again as i128.
        let val = val.to_str_radix(10).parse::<i128>().unwrap();
        Ok(expr_int_lit(val as u64, ty, Some(span)))
    }
}

// Parse integer literal string such as "-5", "-0xff" or "123e4", and return its value and radix.
fn parse_integer_literal_string(s: &str) -> Option<(BigInt, usize)> {
    if s.len() == 0 {
        return None;
    }
    if s.starts_with("0x") {
        return BigInt::parse_bytes(s.trim_start_matches("0x").as_bytes(), 16).map(|x| (x, 16));
    }
    if s.starts_with("-0x") {
        return BigInt::parse_bytes(s.trim_start_matches("-0x").as_bytes(), 16).map(|x| (-x, 16));
    }
    if s.starts_with("0o") {
        return BigInt::parse_bytes(s.trim_start_matches("0o").as_bytes(), 8).map(|x| (x, 8));
    }
    if s.starts_with("-0o") {
        return BigInt::parse_bytes(s.trim_start_matches("-0o").as_bytes(), 8).map(|x| (-x, 8));
    }
    if s.starts_with("0b") {
        return BigInt::parse_bytes(s.trim_start_matches("0b").as_bytes(), 2).map(|x| (x, 2));
    }
    if s.starts_with("-0b") {
        return BigInt::parse_bytes(s.trim_start_matches("-0b").as_bytes(), 2).map(|x| (-x, 2));
    }
    let split = s.split('e').collect::<Vec<_>>();
    if split.len() > 2 {
        return None;
    }
    if split.len() == 1 {
        // 'e' is not contained.
        return BigInt::parse_bytes(s.as_bytes(), 10).map(|x| (x, 10));
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
    Some((ret, 10))
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
    Ok(make_string_lit(string, Some(span)))
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
    if pairs.peek().is_some() {
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::type_arrow);
        let arrow_span = Span::from_pair(&ctx.source, &pair);
        let pair = pairs.next().unwrap();
        let dst_ty = parse_type(pair, ctx);
        type_fun_with_arrow_src(src_ty, dst_ty, Some(arrow_span))
    } else {
        src_ty
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
    let tv_name = pair.as_str();
    let span = Span::from_pair(&ctx.source, &pair);
    let tv = make_tyvar(tv_name, &kind_star());
    type_from_tyvar(tv).set_source(Some(span))
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
        if pair.as_rule() == Rule::comma_ {
            extra_comma = true;
            break;
        }
        types.push(parse_type(pair, ctx));
    }
    // Is it just a bracketed type?, e.g., (I64)
    let is_bracketed_type = types.len() == 1 && !extra_comma;
    if is_bracketed_type {
        types[0].clone()
    } else {
        // It is a genuine tuple type.
        let tuple_size = types.len();
        ctx.tuple_sizes.push(tuple_size as u32);
        let mut res = type_tycon(&tycon(make_tuple_name_abs(tuple_size as u32)))
            .set_source(Some(span.clone()));
        for ty in types {
            res = type_tyapp(res, ty).set_source(Some(span.clone()));
        }
        res
    }
    .set_source(Some(span))
}

fn parse_pattern_nounion(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_nounion);
    let span = Span::from_pair(&ctx.source, &pair);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::pattern_var => parse_pattern_var(pair, ctx),
        Rule::pattern_tuple => parse_pattern_tuple(pair, ctx),
        Rule::pattern_struct => parse_pattern_struct(pair, ctx),
        _ => unreachable!(),
    }
    .set_source(span)
}

fn parse_pattern_case(pair: Pair<Rule>, ctx: &mut ParseContext) -> Arc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_case);
    let span = Span::from_pair(&ctx.source, &pair);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::pattern_var => parse_pattern_var(pair, ctx),
        Rule::pattern_union => parse_pattern_union(pair, ctx),
        Rule::pattern_struct => parse_pattern_struct(pair, ctx),
        Rule::pattern_tuple => parse_pattern_tuple(pair, ctx),
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
        .map(|pair| parse_pattern_nounion(pair, ctx))
        .collect::<Vec<_>>();
    let tuple_size = pats.len();
    ctx.tuple_sizes.push(tuple_size as u32);
    PatternNode::make_struct(
        tycon(make_tuple_name_abs(tuple_size as u32)),
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
        let pat = parse_pattern_nounion(pairs.next().unwrap(), ctx);
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
    let pair = pairs.next().unwrap();
    assert_eq!(pair.as_rule(), Rule::type_field_name);
    let variant = FullName::new(&NameSpace::new(names), pair.as_str());
    let pat = if let Some(pair) = pairs.next() {
        parse_pattern_nounion(pair, ctx)
    } else {
        PatternNode::make_struct(tycon(make_tuple_name_abs(0 as u32)), vec![])
    };
    PatternNode::make_union(variant, pat).set_source(span)
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
        module: (module, Some(module_span.clone())),
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

fn parse_import_statements(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> Result<Vec<ImportStatement>, Errors> {
    assert_eq!(pair.as_rule(), Rule::file_only_import_statements);
    let mut pairs = pair.into_inner();
    let mod_info = parse_module_defn(pairs.next().unwrap(), ctx);
    ctx.module_name = mod_info.name;
    let mut import_stmts = vec![];
    for pair in pairs {
        if pair.as_rule() != Rule::import_statement {
            break;
        }
        import_stmts.push(parse_import_statement(pair, ctx));
    }
    Ok(import_stmts)
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
        Rule::expr_unary => "expression".to_string(),
        Rule::name => "name".to_string(),
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
        Rule::comma_ => ",".to_string(),
        Rule::export_symbol => "FFI_EXPORT".to_string(),
        Rule::global_defns => "definitions".to_string(),
        Rule::exported_c_function_name => "C function name".to_string(),
        Rule::operator_and_then => "`;;`".to_string(),
        Rule::match_arrow => "`=>`".to_string(),
        Rule::double_colon => "`::`".to_string(),
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
