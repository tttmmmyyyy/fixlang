#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;

use num_bigint::BigInt;
use std::{cmp::min, mem::swap};

use pest::error::Error;

use super::*;

struct ParseContext {
    // The list of sizes of tuples used in this module.
    tuple_sizes: Vec<u32>,
    // Context for parsing *-operator.
    do_context: DoContext,
    // The source code.
    source: SourceFile,
    // The module name.
    module_name: NameSpace,
    // Current namespace.
    namespace: NameSpace,
}

impl ParseContext {
    fn from_source(source: SourceFile) -> Self {
        Self {
            tuple_sizes: vec![],
            do_context: DoContext::default(),
            source,
            module_name: NameSpace::new(vec![]),
            namespace: NameSpace::new(vec![]),
        }
    }

    fn append_namespace(&mut self, namespace: NameSpace) -> NameSpace {
        let res = self.namespace;
        self.namespace.append(namespace);
        res
    }

    fn set_namespace(&mut self, namespace: NameSpace) {
        self.namespace = namespace;
    }
}

#[derive(Default)]
struct DoContext {
    counter: u32,
    monads: Vec<BindOperatorInfo>, // (Monadic action, the result of action).
}

struct BindOperatorInfo {
    operator_src: Span,
    operand: Rc<ExprNode>,
    result_var: Rc<Var>,
}

impl DoContext {
    // Pushes monadic value, and returns expression that represents the result of monadic action.
    fn push_monad(&mut self, monad: Rc<ExprNode>, operator_src: Span) -> Rc<ExprNode> {
        let src = monad.source.clone();
        let var_name = FullName::local(&format!("%monadic_arg{}", self.counter));
        let var_var = var_var(var_name.clone());
        let var_expr = expr_var(var_name, src);
        self.counter += 1;
        self.monads.push(BindOperatorInfo {
            operator_src,
            operand: monad,
            result_var: var_var,
        });
        var_expr
    }

    fn expand_binds(&mut self, mut expr: Rc<ExprNode>) -> Rc<ExprNode> {
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
            expr = expr_app(expr, vec![monad], src)
                .set_app_order(AppSourceCodeOrderType::ArgumentIsFormer);
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

pub fn parse_source_temporary_file(source: &str, file_name: &str, hash: &str) -> Program {
    if !check_temporary_source(file_name, hash) {
        save_temporary_source(source, file_name, hash);
    }
    parse_source(
        source,
        temporary_source_path(file_name, hash).to_str().unwrap(),
    )
}

pub fn parse_source(source: &str, file_name: &str) -> Program {
    let source_file = SourceFile {
        string: Some(Rc::new(source.to_string())),
        file_path: file_name.to_string(),
    };
    let file = FixParser::parse(Rule::file, source);
    let file = match file {
        Ok(res) => res,
        Err(e) => error_exit(&message_parse_error(e, &source_file)),
    };
    parse_file(file, source_file)
}

fn parse_file(mut file: Pairs<Rule>, src: SourceFile) -> Program {
    let pair = file.next().unwrap();
    match pair.as_rule() {
        Rule::module => return parse_module(pair, src),
        _ => unreachable!(),
    }
}

fn parse_module(pair: Pair<Rule>, src: SourceFile) -> Program {
    assert_eq!(pair.as_rule(), Rule::module);
    let mut ctx: ParseContext = ParseContext::from_source(src);

    let mut pairs = pair.into_inner();
    ctx.module_name = parse_module_defn(pairs.next().unwrap());
    let mut fix_mod = Program::single_module(ctx.module_name.clone());

    let mut type_defns: Vec<TypeDefn> = Vec::new();
    let mut global_value_decls: Vec<GlobalValueDecl> = vec![];
    let mut global_value_defns: Vec<GlobalValueDefn> = vec![];
    let mut trait_infos: Vec<TraitInfo> = vec![];
    let mut trait_aliases: Vec<TraitAlias> = vec![];
    let mut trait_impls: Vec<TraitInstance> = vec![];
    let mut import_statements: Vec<ImportStatement> = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::global_defns => parse_global_defns(
                pair,
                &mut ctx,
                &mut global_value_decls,
                &mut global_value_defns,
                &mut type_defns,
                &mut trait_infos,
                &mut trait_aliases,
            ),
            Rule::trait_impl => {
                trait_impls.push(parse_trait_impl(pair, &mut ctx));
            }
            Rule::import_statement => {
                import_statements.push(parse_import_statement(pair, &mut ctx));
            }
            _ => unreachable!(),
        }
    }

    fix_mod.add_global_values(global_value_defns, global_value_decls);
    fix_mod.add_type_defns(type_defns);
    fix_mod.add_traits(trait_infos, trait_impls, trait_aliases);
    fix_mod.add_import_statements(import_statements);
    fix_mod.used_tuple_sizes.append(&mut ctx.tuple_sizes);

    fix_mod
}

fn parse_global_defns(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
    global_value_decls: &mut Vec<GlobalValueDecl>,
    global_value_defns: &mut Vec<GlobalValueDefn>,
    type_defns: &mut Vec<TypeDefn>,
    trait_infos: &mut Vec<TraitInfo>,
    trait_aliases: &mut Vec<TraitAlias>,
) {
    assert_eq!(pair.as_rule(), Rule::global_defns);
    let pairs = pair.into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::global_defns_in_namespace => {
                parse_global_defns_in_namespace(
                    pair,
                    ctx,
                    global_value_decls,
                    global_value_defns,
                    type_defns,
                    trait_infos,
                    trait_aliases,
                );
            }
            Rule::type_defn => {
                type_defns.push(parse_type_defn(pair, ctx));
            }
            Rule::global_name_type_sign => {
                global_value_decls.push(parse_global_value_decl(pair, ctx));
            }
            Rule::global_name_defn => {
                global_value_defns.push(parse_global_name_defn(pair, ctx));
            }
            Rule::trait_defn => {
                trait_infos.push(parse_trait_defn(pair, ctx));
            }
            Rule::trait_alias_defn => {
                trait_aliases.push(parse_trait_alias(pair, ctx));
            }
            _ => unreachable!(),
        }
    }
}

fn parse_global_defns_in_namespace(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
    global_value_decls: &mut Vec<GlobalValueDecl>,
    global_value_defns: &mut Vec<GlobalValueDefn>,
    type_defns: &mut Vec<TypeDefn>,
    trait_infos: &mut Vec<TraitInfo>,
    trait_aliases: &mut Vec<TraitAlias>,
) {
    assert_eq!(pair.as_rule(), Rule::global_defns_in_namespace);
    let mut pairs = pair.into_inner();
    let namespace = parse_namespace(pairs.next().unwrap());
    let bak_namespace = ctx.append_namespace(namespace);
    for pair in pairs {
        parse_global_defns(
            pair,
            ctx,
            global_value_decls,
            global_value_defns,
            type_defns,
            trait_infos,
            trait_aliases,
        );
    }
    ctx.set_namespace(bak_namespace);
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
        values.push(parse_trait_fullname(pair, ctx));
    }
    TraitAlias {
        id,
        value: values,
        source: Some(span),
        kind: kind_star(), // Will be set to a correct value in TraitEnv::set_kinds.
    }
}

fn parse_trait_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> TraitInfo {
    assert_eq!(pair.as_rule(), Rule::trait_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let kinds = if pairs.peek().unwrap().as_rule() == Rule::predicates {
        let pair = pairs.next().unwrap();
        let (preds, kinds) = parse_predicates(pair, ctx);
        if !preds.is_empty() {
            error_exit_with_src(
                "The current Fix does not support super-trait; only kinds of the type parameter can be specified as the assumption for trait definition.",
                &preds.first().unwrap().info.source
            );
        }
        kinds
    } else {
        vec![]
    };
    let tyvar = pairs.next().unwrap().as_str().to_string();
    assert_eq!(pairs.peek().unwrap().as_rule(), Rule::trait_name);
    let trait_name = pairs.next().unwrap().as_str().to_string();
    let methods: HashMap<Name, QualType> = pairs
        .map(|pair| parse_trait_member_defn(pair, ctx))
        .collect();
    TraitInfo {
        id: TraitId::from_fullname(FullName::new(&ctx.namespace, &trait_name)),
        type_var: tyvar_from_name(&tyvar, &kind_star()),
        methods,
        kind_predicates: kinds,
        source: Some(span),
    }
}

fn parse_trait_member_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> (Name, QualType) {
    assert_eq!(pair.as_rule(), Rule::trait_member_defn);
    let mut pairs = pair.into_inner();
    let method_name = pairs.next().unwrap().as_str().to_string();
    let qual_type = parse_type_qualified(pairs.next().unwrap(), ctx);
    (method_name, qual_type)
}

fn parse_trait_impl(pair: Pair<Rule>, ctx: &mut ParseContext) -> TraitInstance {
    assert_eq!(pair.as_rule(), Rule::trait_impl);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let qual_pred = parse_predicate_qualified(pairs.next().unwrap(), ctx);
    let methods: HashMap<Name, Rc<ExprNode>> = pairs
        .map(|pair| parse_trait_member_impl(pair, ctx))
        .collect();
    TraitInstance {
        qual_pred,
        methods,
        define_module: ctx.module_name.clone(),
        source: Some(span),
    }
}

fn parse_trait_member_impl(pair: Pair<Rule>, ctx: &mut ParseContext) -> (Name, Rc<ExprNode>) {
    assert_eq!(pair.as_rule(), Rule::trait_member_impl);
    let mut pairs = pair.into_inner();
    let method_name = pairs.next().unwrap().as_str().to_string();
    let expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx);
    (method_name, expr)
}

fn parse_predicate_qualified(pair: Pair<Rule>, ctx: &mut ParseContext) -> QualPredicate {
    assert_eq!(pair.as_rule(), Rule::predicate_qualified);
    let mut pairs = pair.into_inner();
    let (predicates, kinds) = if pairs.peek().unwrap().as_rule() == Rule::predicates {
        parse_predicates(pairs.next().unwrap(), ctx)
    } else {
        (vec![], vec![])
    };
    let predicate = parse_predicate(pairs.next().unwrap(), ctx);
    let qp = QualPredicate {
        context: predicates,
        kind_preds: kinds,
        predicate,
    };
    qp
}

fn parse_global_value_decl(pair: Pair<Rule>, ctx: &mut ParseContext) -> GlobalValueDecl {
    assert_eq!(pair.as_rule(), Rule::global_name_type_sign);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let qual_type = parse_type_qualified(pairs.next().unwrap(), ctx);
    let preds = qual_type.preds.clone();
    let ty = qual_type.ty.clone();
    GlobalValueDecl {
        name: FullName::new(&ctx.namespace, &name),
        ty: Scheme::generalize(ty.free_vars(), preds, ty),
        src: Some(span),
    }
}

fn parse_global_name_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> GlobalValueDefn {
    assert_eq!(pair.as_rule(), Rule::global_name_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx);
    GlobalValueDefn {
        name: FullName::new(&ctx.namespace, &name),
        expr: expr,
        src: Some(span),
    }
}

fn parse_type_qualified(pair: Pair<Rule>, ctx: &mut ParseContext) -> QualType {
    assert_eq!(pair.as_rule(), Rule::type_qualified);
    let mut pairs = pair.into_inner();
    let (preds, kinds) = if pairs.peek().unwrap().as_rule() == Rule::predicates {
        parse_predicates(pairs.next().unwrap(), ctx)
    } else {
        (vec![], vec![])
    };
    for pred in &preds {
        match &pred.ty.ty {
            Type::TyVar(_) => {}
            _ => {
                error_exit_with_src("Currently, trait bound has to be of the form `tv : SomeTrait` for some type variable `tv`.", &pred.info.source);
            }
        }
    }
    let ty = parse_type(pairs.next().unwrap(), ctx);
    let qt = QualType {
        preds,
        ty,
        kind_preds: kinds,
    };
    qt
}

fn parse_predicates(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
) -> (Vec<Predicate>, Vec<KindPredicate>) {
    assert_eq!(pair.as_rule(), Rule::predicates);
    let pairs = pair.into_inner();
    let mut ps: Vec<Predicate> = Default::default();
    let mut ks: Vec<KindPredicate> = Default::default();
    for pair in pairs {
        if pair.as_rule() == Rule::predicate {
            ps.push(parse_predicate(pair, ctx));
        } else if pair.as_rule() == Rule::predicate_kind {
            ks.push(parse_predicate_kind(pair, ctx));
        } else {
            unreachable!()
        }
    }
    (ps, ks)
}

fn parse_predicate_kind(pair: Pair<Rule>, ctx: &mut ParseContext) -> KindPredicate {
    assert_eq!(pair.as_rule(), Rule::predicate_kind);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let kind = parse_kind(pairs.next().unwrap(), ctx);
    KindPredicate {
        name,
        kind,
        source: Some(span),
    }
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

fn parse_kind(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind);
    let mut pairs = pair.into_inner();
    let mut res: Rc<Kind> = parse_kind_nlr(pairs.next().unwrap(), ctx);
    for pair in pairs {
        res = kind_arrow(res, parse_kind_nlr(pair, ctx));
    }
    res
}

fn parse_kind_nlr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<Kind> {
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

fn parse_kind_star(pair: Pair<Rule>, _ctx: &mut ParseContext) -> Rc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind_star);
    kind_star()
}

fn parse_kind_braced(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind_braced);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    parse_kind(pair, ctx)
}

fn parse_module_defn(pair: Pair<Rule>) -> NameSpace {
    parse_namespace(pair.into_inner().next().unwrap())
}

fn parse_type_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> TypeDefn {
    assert_eq!(pair.as_rule(), Rule::type_defn);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    assert_eq!(pairs.peek().unwrap().as_rule(), Rule::type_name);
    let name = pairs.next().unwrap().as_str();
    let mut tyvars: Vec<Name> = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::type_var {
        tyvars.push(pairs.next().unwrap().as_str().to_string());
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
    TypeDefn {
        name: FullName::new(&ctx.namespace, name),
        value: type_value,
        tyvars,
        source: Some(span),
    }
}

fn parse_struct_defn(pair: Pair<Rule>, ctx: &mut ParseContext) -> TypeDeclValue {
    assert_eq!(pair.as_rule(), Rule::struct_defn);
    let mut pairs = pair.into_inner();
    let mut fields: Vec<Field> = Vec::new();
    let mut is_unbox = true; // Default value
    if pairs.peek().unwrap().as_rule() == Rule::box_or_unbox {
        is_unbox = parse_box_unbox(pairs.next().unwrap(), ctx);
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
    }
}

fn parse_expr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_type_annotation(pair, ctx)
}

fn parse_expr_with_new_do(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr);

    // Here use new DoContext.
    let old_doctx = std::mem::replace(&mut ctx.do_context, DoContext::default());
    let expr = parse_expr(pair, ctx);
    let expr = ctx.do_context.expand_binds(expr);

    // Restore old DoContext.
    ctx.do_context = old_doctx;

    expr
}

fn parse_expr_type_annotation(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_type_annotation);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let mut expr = parse_expr_rtl_app(pairs.next().unwrap(), ctx);
    match pairs.next() {
        None => {}
        Some(ty) => {
            expr = expr_tyanno(expr, parse_type(ty, ctx), Some(span));
        }
    }
    expr
}

// Parse combinator sequence, e.g., `f x y` or `x & f & g`
fn parse_combinator_sequence(
    pair: Pair<Rule>,
    ctx: &mut ParseContext,
    inner_parser: fn(Pair<Rule>, &mut ParseContext) -> Rc<ExprNode>,
) -> Vec<Rc<ExprNode>> {
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
    inner_parser: fn(Pair<Rule>, &mut ParseContext) -> Rc<ExprNode>,
) -> Rc<ExprNode> {
    let mut pairs = pair.into_inner();
    let mut expr = inner_parser(pairs.next().unwrap(), ctx);
    let mut next_operation = BinaryOpInfo::default();
    let mut next_op_span: Option<Span> = None;
    for pair in pairs {
        if pair.as_rule() == operator_rule {
            next_operation = ops[pair.as_str()].clone();
            next_op_span = Some(Span::from_pair(&ctx.source, &pair));
        } else {
            let mut lhs = expr;
            let mut rhs = inner_parser(pair, ctx);
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
    expr
}

// comparison operators (left-associative)
fn parse_expr_cmp(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
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
fn parse_expr_and(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_and);
    let exprs = pair
        .into_inner()
        .map(|p| parse_expr_cmp(p, ctx))
        .collect::<Vec<_>>();

    fn and_boolean_exprs(ps: &[Rc<ExprNode>]) -> Rc<ExprNode> {
        if ps.len() == 1 {
            ps[0].clone()
        } else {
            let sub = and_boolean_exprs(&ps[1..]);
            expr_if(ps[0].clone(), sub, expr_bool_lit(false, None), None)
        }
    }

    and_boolean_exprs(&exprs)
}

// Operator || (right-associative)
fn parse_expr_or(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_or);
    let exprs = pair
        .into_inner()
        .map(|p| parse_expr_and(p, ctx))
        .collect::<Vec<_>>();

    fn or_boolean_exprs(ps: &[Rc<ExprNode>]) -> Rc<ExprNode> {
        if ps.len() == 1 {
            ps[0].clone()
        } else {
            let sub = or_boolean_exprs(&ps[1..]);
            expr_if(ps[0].clone(), expr_bool_lit(true, None), sub, None)
        }
    }

    or_boolean_exprs(&exprs)
}

// Operator +/- (left associative)
fn parse_expr_plus(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
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
fn parse_expr_mul(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
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
fn parse_expr_unary(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    let span = Span::from_pair(&ctx.source, &pair);
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
                let mut expr = parse_expr_composition(pair, ctx);
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
                let expr = expr.set_source(Some(span));
                return expr;
            }
        }
    }
    unreachable!()
}

// Parse right to left application sequence, e.g., `g $ f $ x`. (right-associative)
fn parse_expr_rtl_app(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_rtl_app);
    let exprs = parse_combinator_sequence(pair, ctx, parse_expr_or);
    let mut exprs_iter = exprs.iter().rev();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), vec![ret], span);
    }
    ret
}

// Parse function composition operator >> and <<.
fn parse_expr_composition(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    fn unite_src_from_expr(lhs: &Rc<ExprNode>, rhs: &Rc<ExprNode>) -> Option<Span> {
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
    let mut expr = parse_expr_bind(pairs.next().unwrap(), ctx);
    while pairs.peek().is_some() {
        let op = pairs.next().unwrap();
        assert_eq!(op.as_rule(), Rule::operator_composition);
        let op_span = Span::from_pair(&ctx.source, &op);
        let compose = expr_var(
            FullName::from_strs(&[STD_NAME], COMPOSE_FUNCTION_NAME),
            Some(op_span),
        );
        let rhs = parse_expr_bind(pairs.next().unwrap(), ctx);
        match op.as_str() {
            ">>" => {
                let span = unite_src_from_expr(&compose, &expr);
                expr = expr_app(compose, vec![expr], span)
                    .set_app_order(AppSourceCodeOrderType::ArgumentIsFormer);
                let span = unite_src_from_expr(&expr, &rhs);
                expr = expr_app(expr, vec![rhs], span)
                    .set_app_order(AppSourceCodeOrderType::FunctionIsFormer);
            }
            "<<" => {
                let span = unite_src_from_expr(&compose, &rhs);
                let right_expr = expr_app(compose, vec![rhs.clone()], span)
                    .set_app_order(AppSourceCodeOrderType::ArgumentIsFormer);
                let span = unite_src_from_expr(&right_expr, &rhs);
                expr = expr_app(right_expr, vec![expr], span)
                    .set_app_order(AppSourceCodeOrderType::FunctionIsFormer);
            }
            _ => {
                unreachable!()
            }
        }
    }
    expr
}

// Parse monadic bind syntax `*x`.
fn parse_expr_bind(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_bind);
    let mut stars = vec![];
    let mut pairs = pair.into_inner();
    while pairs.peek().unwrap().as_rule() == Rule::operator_bind {
        let star_pair = pairs.next().unwrap();
        stars.push(Span::from_pair(&ctx.source, &star_pair));
    }
    let mut expr = parse_expr_ltr_app(pairs.next().unwrap(), ctx);
    while !stars.is_empty() {
        expr = ctx.do_context.push_monad(expr, stars.pop().unwrap());
    }
    expr
}

// Parse left to right application sequence, e.g., `x.f.g`. (left-associative)
fn parse_expr_ltr_app(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_ltr_app);
    let exprs = parse_combinator_sequence(pair, ctx, parse_expr_app);
    let mut exprs_iter = exprs.iter();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), vec![ret], span)
            .set_app_order(AppSourceCodeOrderType::ArgumentIsFormer);
    }
    ret
}

// Parse application sequence, e.g., `f(x, y)`. (left-associative)
fn parse_expr_app(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_app);
    let mut pairs = pair.into_inner();
    let head = parse_expr_nlr(pairs.next().unwrap(), ctx);
    let mut args = vec![];
    if pairs.peek().is_some() {
        // If parentheses for arguments are given,
        args = parse_arg_list(pairs.next().unwrap(), ctx);
        if args.len() == 0 {
            // `f()` is interpreted as application to unit: `f $ ()`.
            args.push(expr_make_struct(tycon(make_tuple_name(0)), vec![]))
        }
    }
    let mut ret = head;
    for expr in args {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(ret, vec![expr.clone()], span);
    }
    ret
}

fn parse_arg_list(pair: Pair<Rule>, ctx: &mut ParseContext) -> Vec<Rc<ExprNode>> {
    assert_eq!(pair.as_rule(), Rule::arg_list);
    parse_combinator_sequence(pair, ctx, parse_expr)
}

fn parse_expr_nlr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_nlr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_lit => parse_expr_lit(pair, ctx),
        Rule::expr_var => parse_expr_var(pair, ctx),
        Rule::expr_let => parse_expr_let(pair, ctx),
        Rule::expr_eval => parse_expr_eval(pair, ctx),
        Rule::expr_if => parse_expr_if(pair, ctx),
        Rule::expr_do => parse_expr_do(pair, ctx),
        Rule::expr_lam => parse_expr_lam(pair, ctx),
        Rule::expr_tuple => parse_expr_tuple(pair, ctx),
        Rule::expr_make_struct => parse_expr_make_struct(pair, ctx),
        Rule::expr_call_c => parse_expr_call_c(pair, ctx),
        _ => unreachable!(),
    }
}

fn parse_expr_var(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_var);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let namespace = if pairs.peek().unwrap().as_rule() == Rule::namespace {
        parse_namespace(pairs.next().unwrap())
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

fn parse_namespace(pair: Pair<Rule>) -> NameSpace {
    assert_eq!(pair.as_rule(), Rule::namespace);
    let pairs = pair.into_inner();
    let mut ret: Vec<String> = Vec::new();
    for pair in pairs {
        ret.push(pair.as_str().to_string());
    }
    NameSpace::new(ret)
}

fn parse_expr_lit(expr: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_number_lit => parse_expr_number_lit(pair, ctx),
        Rule::expr_bool_lit => parse_expr_bool_lit(pair, ctx),
        Rule::expr_string_lit => parse_expr_string_lit(pair, ctx),
        Rule::expr_array_lit => parse_expr_array_lit(pair, ctx),
        Rule::expr_nullptr_lit => parse_expr_nullptr_lit(pair, ctx),
        Rule::expr_u8_lit => parse_expr_u8_lit(pair, ctx),
        _ => unreachable!(),
    }
}

fn parse_expr_let(expr: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    let span = Span::from_pair(&ctx.source, &expr);
    let mut pairs = expr.into_inner();
    let pat = parse_pattern(pairs.next().unwrap(), ctx);
    let _eq_of_let = pairs.next().unwrap();
    let bound = parse_expr(pairs.next().unwrap(), ctx);
    let _in_of_let = pairs.next().unwrap();
    let val = parse_expr_with_new_do(pairs.next().unwrap(), ctx);
    expr_let(pat, bound, val, Some(span))
}

fn parse_expr_eval(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_eval);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let bound = parse_expr(pairs.next().unwrap(), ctx);
    let val = parse_expr_with_new_do(pairs.next().unwrap(), ctx);
    let pat = PatternNode::make_var(var_local(EVAL_VAR_NAME), None);
    expr_let(pat, bound, val, Some(span))
}

fn parse_expr_lam(expr: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    let span = Span::from_pair(&ctx.source, &expr);
    let mut pairs = expr.into_inner();
    let mut pats = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::pattern {
        let pat = parse_pattern(pairs.next().unwrap(), ctx);
        pats.push(pat);
    }
    let mut expr = parse_expr_with_new_do(pairs.next().unwrap(), ctx);
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
    expr.set_source(Some(span))
}

fn parse_expr_if(expr: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(expr.as_rule(), Rule::expr_if);
    let span = Span::from_pair(&ctx.source, &expr);
    let mut pairs = expr.into_inner();
    let cond = pairs.next().unwrap();
    let then_val = pairs.next().unwrap();
    let else_val = pairs.next().unwrap();
    expr_if(
        parse_expr(cond, ctx),
        parse_expr_with_new_do(then_val, ctx),
        parse_expr_with_new_do(else_val, ctx),
        Some(span),
    )
}

fn parse_expr_do(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert!(pair.as_rule() == Rule::expr_do);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_with_new_do(pair, ctx)
}

fn parse_expr_tuple(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_tuple);
    let span = Span::from_pair(&ctx.source, &pair);
    let exprs = pair
        .into_inner()
        .map(|p| parse_expr(p, ctx).set_source(Some(span.clone())))
        .collect::<Vec<_>>();
    if exprs.len() == 1 {
        exprs[0].clone()
    } else {
        let tuple_size = exprs.len();
        ctx.tuple_sizes.push(tuple_size as u32);
        expr_make_struct(
            tycon(make_tuple_name(tuple_size as u32)),
            exprs
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, expr)| (i.to_string(), expr))
                .collect(),
        )
    }
}

fn parse_expr_make_struct(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_make_struct);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let tycon = parse_tycon(pairs.next().unwrap());
    let mut fields = vec![];
    while pairs.peek().is_some() {
        let field_name = pairs.next().unwrap().as_str().to_string();
        let field_expr = parse_expr(pairs.next().unwrap(), ctx);
        fields.push((field_name, field_expr));
    }
    expr_make_struct(tycon, fields).set_source(Some(span))
}

fn parse_expr_call_c(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_call_c);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let ret_ty = parse_ffi_c_fun_ty(pairs.next().unwrap());
    let fun_name = pairs.next().unwrap().as_str().to_string();
    let param_tys = parse_ffi_param_tys(pairs.next().unwrap());
    let is_var_args =
        if pairs.peek().is_some() && pairs.peek().unwrap().as_rule() == Rule::ffi_var_args {
            pairs.next();
            true
        } else {
            false
        };
    let args: Vec<_> = pairs.map(|pair| parse_expr(pair, ctx)).collect();

    // Validate number of arguments.
    if args.len() < param_tys.len() || (!is_var_args && args.len() > param_tys.len()) {
        error_exit_with_src(
            "Wrong number of arguments in CALL_C expression.",
            &Some(span),
        );
    }

    expr_call_c(fun_name, ret_ty, param_tys, is_var_args, args, Some(span))
}

fn parse_ffi_c_fun_ty(pair: Pair<Rule>) -> Rc<TyCon> {
    assert_eq!(pair.as_rule(), Rule::ffi_c_fun_ty);
    let name = if pair.as_str() == "()" {
        make_tuple_name(0)
    } else {
        FullName::from_strs(&[STD_NAME], pair.as_str())
    };
    tycon(name)
}

fn parse_ffi_param_tys(pair: Pair<Rule>) -> Vec<Rc<TyCon>> {
    assert_eq!(pair.as_rule(), Rule::ffi_param_tys);
    pair.into_inner()
        .map(|pair| parse_ffi_c_fun_ty(pair))
        .collect()
}

fn parse_expr_number_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
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
                error_exit_with_src(
                    "Mismatch between literal format and specified type. Note that floating point literals must contain a decimal point.",
                    &Some(span),
                );
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
            error_exit_with_src(
                &format!(
                    "A literal string `{}` cannot be parsed as a floating number.",
                    val_str
                ),
                &Some(span),
            )
        }
        let val = val.unwrap();
        expr_float_lit(val, ty, Some(span))
    } else {
        // Integral literal
        let val = parse_integral_string_lit(val_str);
        if val.is_none() {
            error_exit_with_src(
                &format!(
                    "A literal string `{}` cannot be parsed as an integer.",
                    val_str
                ),
                &Some(span),
            )
        }
        let val = val.unwrap();

        // Check size.
        let (ty_min, ty_max) = integral_ty_range(ty_name);
        if !(ty_min <= val && val <= ty_max) {
            error_exit_with_src(
                &format!(
                    "The value of an integer literal `{}` is out of range of `{}`.",
                    val_str, ty_name
                ),
                &Some(span),
            )
        }

        // Now stringify val and parse it again as i128.
        let val = val.to_str_radix(10).parse::<i128>().unwrap();
        expr_int_lit(val as u64, ty, Some(span))
    }
}

fn parse_integral_string_lit(s: &str) -> Option<BigInt> {
    if s.len() == 0 {
        return None;
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

fn parse_expr_nullptr_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_nullptr_lit);
    let span = Span::from_pair(&ctx.source, &pair);
    expr_nullptr_lit(Some(span))
}

fn parse_expr_bool_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_bool_lit);
    let val = pair.as_str().parse::<bool>().unwrap();
    let span = Span::from_pair(&ctx.source, &pair);
    expr_bool_lit(val, Some(span))
}

fn parse_expr_array_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_array_lit);
    let span = Span::from_pair(&ctx.source, &pair);
    let elems = pair
        .into_inner()
        .map(|pair| parse_expr(pair, ctx))
        .collect::<Vec<_>>();
    expr_array_lit(elems, Some(span))
}

fn parse_expr_string_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
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
                        None => error_exit_with_src(
                            &format!("Invalid unicode character: u{:X}", code),
                            &Some(span),
                        ),
                        Some(c) => c,
                    };
                    out_string.push(c);
                }
            }
        }
    }
    let string = String::from_iter(out_string.iter());
    make_string_from_rust_string(string, Some(span))
}

fn parse_expr_u8_lit(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<ExprNode> {
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

fn parse_type(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<TypeNode> {
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

fn parse_type_fun(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<TypeNode> {
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

fn parse_type_tyapp(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<TypeNode> {
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

fn parse_type_nlr(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<TypeNode> {
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

fn parse_type_var(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_var);
    let span = Span::from_pair(&ctx.source, &pair);
    type_tyvar(pair.as_str(), &kind_star()).set_source(Some(span))
}

fn parse_type_tycon(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_tycon);
    let span = Span::from_pair(&ctx.source, &pair);
    type_tycon(&parse_tycon(pair)).set_source(Some(span))
}

fn parse_tycon(pair: Pair<Rule>) -> Rc<TyCon> {
    assert_eq!(pair.as_rule(), Rule::type_tycon);
    tycon(parse_capital_fullname(pair.into_inner().next().unwrap()))
}

fn parse_type_tuple(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_tuple);
    let span = Span::from_pair(&ctx.source, &pair);
    let types = pair
        .into_inner()
        .map(|p| parse_type(p, ctx))
        .collect::<Vec<_>>();
    if types.len() == 1 {
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

fn parse_pattern(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<PatternNode> {
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

fn parse_pattern_var(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_var);
    let span = Span::from_pair(&ctx.source, &pair);
    let mut pairs = pair.into_inner();
    let var_name = pairs.next().unwrap().as_str();
    let ty = pairs.next().map(|ty| parse_type(ty, ctx));
    PatternNode::make_var(var_local(var_name), ty).set_source(span)
}

fn parse_pattern_tuple(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<PatternNode> {
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

fn parse_pattern_struct(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<PatternNode> {
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

fn parse_pattern_union(pair: Pair<Rule>, ctx: &mut ParseContext) -> Rc<PatternNode> {
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
    let target_module = parse_namespace(pair.into_inner().next().unwrap());
    ImportStatement {
        importer: ctx.module_name.clone(),
        importee: target_module,
        source: Some(span),
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
        Rule::expr_number_lit => "integer or floating number".to_string(),
        Rule::expr_bool_lit => "boolean".to_string(),
        Rule::expr_nlr => "expression".to_string(),
        Rule::var => "variable".to_string(),
        Rule::in_of_let => "`in` or `;`".to_string(),
        Rule::eq_of_let => "`=`".to_string(),
        Rule::type_expr => "type".to_string(),
        Rule::arg_list => "arguments".to_string(),
        Rule::operator_mul => "`*`".to_string(),
        Rule::operator_plus => "`+`".to_string(),
        Rule::operator_and => "`&&`".to_string(),
        Rule::operator_or => "`||`".to_string(),
        Rule::type_nlr => "type".to_string(),
        Rule::operator_composition => join_by_or(&["<<", ">>"]),
        Rule::operator_cmp => join_by_or(&["==", "!=", "<=", ">=", "<", ">"]),
        Rule::trait_impl => "`impl`".to_string(),
        Rule::import_statement => "`import`".to_string(),
        _ => format!("{:?}", r),
    }
}

fn message_parse_error(e: Error<Rule>, src: &SourceFile) -> String {
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
    msg += "\n";

    // Show line and column number.
    let span = match e.location {
        pest::error::InputLocation::Pos(s) => Span {
            input: src.clone(),
            start: s,
            end: min(s + 1, src.string().len()),
        },
        pest::error::InputLocation::Span((s, e)) => Span {
            input: src.clone(),
            start: s,
            end: e,
        },
    };
    msg += "\n";
    msg += &span.to_string();
    msg
}
