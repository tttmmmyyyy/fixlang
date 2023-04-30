#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;

use std::{cmp::min, mem::swap};

use pest::error::Error;

use super::*;

// lifetime-free version of pest::Span
#[derive(Clone)]
pub struct Span {
    pub input: Rc<String>,
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[allow(dead_code)]
    pub fn empty(src: &Rc<String>) -> Self {
        Self {
            input: src.clone(),
            start: usize::max_value(),
            end: 0,
        }
    }

    pub fn from_pair(src: &Rc<String>, pair: &Pair<Rule>) -> Self {
        let span = pair.as_span();
        Self {
            input: src.clone(),
            start: span.start(),
            end: span.end(),
        }
    }

    pub fn unite(&self, other: &Self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn unite_opt(lhs: &Option<Span>, rhs: &Option<Span>) -> Option<Span> {
        if lhs.is_none() {
            return None;
        }
        if rhs.is_none() {
            return None;
        }
        Some(lhs.clone().unwrap().unite(rhs.as_ref().unwrap()))
    }

    // Show source codes around this span.
    pub fn to_string(&self) -> String {
        let span = pest::Span::new(self.input.as_str(), self.start, self.end).unwrap();

        let mut linenum_str_size = 0;
        for line_span in span.lines_span() {
            let linenum = line_span.start_pos().line_col().0;
            linenum_str_size = linenum_str_size.max(linenum.to_string().len());
        }

        let mut ret: String = String::default();
        ret += &format!(
            "at {}:{}-{}:{}\n",
            span.start_pos().line_col().0,
            span.start_pos().line_col().1,
            span.end_pos().line_col().0,
            span.end_pos().line_col().1,
        );
        ret += &(" ".repeat(linenum_str_size) + " | " + "\n");
        for line_span in span.lines_span() {
            let linenum_str = line_span.start_pos().line_col().0.to_string();
            ret +=
                &(linenum_str.clone() + &" ".repeat(linenum_str_size - linenum_str.len()) + " | ");
            ret += String::from(line_span.as_str()).trim_end();
            ret += "\n";
            ret += &(" ".repeat(linenum_str_size) + " | ");
            let start_pos = span.start_pos().max(line_span.start_pos());
            let end_pos = span.end_pos().min(line_span.end_pos());
            let start_col = start_pos.line_col().1;
            let span_len = end_pos.pos() - start_pos.pos();
            ret += &(" ".repeat(start_col - 1) + &"^".repeat(span_len));
            ret += "\n";
        }
        ret
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
            expr = expr_abs(vec![var], expr, None);
            let bind_function = expr_var(
                FullName::from_strs(&[STD_NAME, MONAD_NAME], MONAD_BIND_NAME),
                Some(operator_src),
            );
            expr = expr_app(bind_function, vec![expr], None);
            expr = expr_app(expr, vec![monad], None)
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

pub fn parse_source(source: &str) -> FixModule {
    let source = Rc::new(String::from(source));
    let file = FixParser::parse(Rule::file, &source);
    let file = match file {
        Ok(res) => res,
        Err(e) => error_exit(&message_parse_error(e, &source)),
    };
    parse_file(file, &source)
}

fn parse_file(mut file: Pairs<Rule>, src: &Rc<String>) -> FixModule {
    let pair = file.next().unwrap();
    match pair.as_rule() {
        Rule::module => return parse_module(pair, src),
        _ => unreachable!(),
    }
}

fn parse_module(pair: Pair<Rule>, src: &Rc<String>) -> FixModule {
    assert_eq!(pair.as_rule(), Rule::module);
    let mut pairs = pair.into_inner();
    let module_name = parse_module_defn(pairs.next().unwrap(), src);
    let namespace = NameSpace::new(vec![module_name.clone()]);
    let mut fix_mod = FixModule::new(module_name.clone());

    let mut type_defns: Vec<TypeDefn> = Vec::new();
    let mut global_name_type_signs: Vec<(FullName, Rc<Scheme>)> = vec![];
    let mut global_value_defns: Vec<(FullName, Rc<ExprNode>)> = vec![];
    let mut trait_infos: Vec<TraitInfo> = vec![];
    let mut trait_impls: Vec<TraitInstance> = vec![];
    let mut import_statements: Vec<ImportStatement> = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::global_defns => parse_global_defns(
                pair,
                src,
                &namespace,
                &mut global_name_type_signs,
                &mut global_value_defns,
                &mut type_defns,
                &mut trait_infos,
            ),
            Rule::trait_impl => {
                trait_impls.push(parse_trait_impl(pair, src, &module_name));
            }
            Rule::import_statement => {
                import_statements.push(parse_import_statement(pair, src));
            }
            _ => unreachable!(),
        }
    }

    fix_mod.add_global_values(global_value_defns, global_name_type_signs);
    fix_mod.add_type_defns(type_defns);
    fix_mod.add_traits(trait_infos, trait_impls);
    fix_mod.add_import_statements(import_statements);

    fix_mod
}

fn parse_global_defns(
    pair: Pair<Rule>,
    src: &Rc<String>,
    namespace: &NameSpace,
    global_name_type_signs: &mut Vec<(FullName, Rc<Scheme>)>,
    global_value_defns: &mut Vec<(FullName, Rc<ExprNode>)>,
    type_defns: &mut Vec<TypeDefn>,
    trait_infos: &mut Vec<TraitInfo>,
) {
    assert_eq!(pair.as_rule(), Rule::global_defns);
    let pairs = pair.into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::global_defns_in_namespace => {
                parse_global_defns_in_namespace(
                    pair,
                    src,
                    namespace,
                    global_name_type_signs,
                    global_value_defns,
                    type_defns,
                    trait_infos,
                );
            }
            Rule::type_defn => {
                type_defns.push(parse_type_defn(pair, src, &namespace));
            }
            Rule::global_name_type_sign => {
                global_name_type_signs.push(parse_global_name_type_sign(pair, src, &namespace));
            }
            Rule::global_name_defn => {
                global_value_defns.push(parse_global_name_defn(pair, src, &namespace));
            }
            Rule::trait_defn => {
                trait_infos.push(parse_trait_defn(pair, src, &namespace));
            }
            _ => unreachable!(),
        }
    }
}

fn parse_global_defns_in_namespace(
    pair: Pair<Rule>,
    src: &Rc<String>,
    namespace: &NameSpace,
    global_name_type_signs: &mut Vec<(FullName, Rc<Scheme>)>,
    global_value_defns: &mut Vec<(FullName, Rc<ExprNode>)>,
    type_defns: &mut Vec<TypeDefn>,
    trait_infos: &mut Vec<TraitInfo>,
) {
    assert_eq!(pair.as_rule(), Rule::global_defns_in_namespace);
    let mut pairs = pair.into_inner();
    let namespace = namespace.append(parse_namespace(pairs.next().unwrap(), src));
    for pair in pairs {
        parse_global_defns(
            pair,
            src,
            &namespace,
            global_name_type_signs,
            global_value_defns,
            type_defns,
            trait_infos,
        );
    }
}

fn parse_trait_defn(pair: Pair<Rule>, src: &Rc<String>, namespace: &NameSpace) -> TraitInfo {
    assert_eq!(pair.as_rule(), Rule::trait_defn);
    let mut pairs = pair.into_inner();
    let kinds = if pairs.peek().unwrap().as_rule() == Rule::predicates {
        let pair = pairs.next().unwrap();
        let (preds, kinds) = parse_predicates(pair, src);
        if !preds.is_empty() {
            error_exit("in trait definition, specification of the kinds are only allowed.");
        }
        kinds
    } else {
        vec![]
    };
    let tyvar = pairs.next().unwrap().as_str().to_string();
    let trait_name = pairs.next().unwrap().as_str().to_string();
    let methods: HashMap<Name, QualType> = pairs
        .map(|pair| parse_trait_member_defn(pair, src))
        .collect();
    TraitInfo {
        id: TraitId::from_fullname(FullName::new(namespace, &trait_name)),
        type_var: tyvar_from_name(&tyvar, &kind_star()),
        methods,
        kind_predicates: kinds,
    }
}

fn parse_trait_member_defn(pair: Pair<Rule>, src: &Rc<String>) -> (Name, QualType) {
    assert_eq!(pair.as_rule(), Rule::trait_member_defn);
    let mut pairs = pair.into_inner();
    let method_name = pairs.next().unwrap().as_str().to_string();
    let qual_type = parse_type_qualified(pairs.next().unwrap(), src);
    (method_name, qual_type)
}

fn parse_trait_impl(pair: Pair<Rule>, src: &Rc<String>, module_name: &Name) -> TraitInstance {
    assert_eq!(pair.as_rule(), Rule::trait_impl);
    let mut pairs = pair.into_inner();
    let qual_pred = parse_predicate_qualified(pairs.next().unwrap(), src);
    let methods: HashMap<Name, Rc<ExprNode>> = pairs
        .map(|pair| parse_trait_member_impl(pair, src))
        .collect();
    TraitInstance {
        qual_pred,
        methods,
        define_module: module_name.clone(),
    }
}

fn parse_trait_member_impl(pair: Pair<Rule>, src: &Rc<String>) -> (Name, Rc<ExprNode>) {
    assert_eq!(pair.as_rule(), Rule::trait_member_impl);
    let mut pairs = pair.into_inner();
    let method_name = pairs.next().unwrap().as_str().to_string();
    let expr = parse_expr_with_new_do(pairs.next().unwrap(), src);
    (method_name, expr)
}

fn parse_predicate_qualified(pair: Pair<Rule>, src: &Rc<String>) -> QualPredicate {
    assert_eq!(pair.as_rule(), Rule::predicate_qualified);
    let mut pairs = pair.into_inner();
    let (predicates, kinds) = if pairs.peek().unwrap().as_rule() == Rule::predicates {
        parse_predicates(pairs.next().unwrap(), src)
    } else {
        (vec![], vec![])
    };
    let predicate = parse_predicate(pairs.next().unwrap(), src);
    let qp = QualPredicate {
        context: predicates,
        kind_preds: kinds,
        predicate,
    };
    qp
}

fn parse_global_name_type_sign(
    pair: Pair<Rule>,
    src: &Rc<String>,
    namespace: &NameSpace,
) -> (FullName, Rc<Scheme>) {
    assert_eq!(pair.as_rule(), Rule::global_name_type_sign);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let qual_type = parse_type_qualified(pairs.next().unwrap(), src);
    let preds = qual_type.preds.clone();
    let ty = qual_type.ty.clone();
    (
        FullName::new(namespace, &name),
        Scheme::generalize(ty.free_vars(), preds, ty),
    )
}

fn parse_global_name_defn(
    pair: Pair<Rule>,
    src: &Rc<String>,
    namespace: &NameSpace,
) -> (FullName, Rc<ExprNode>) {
    assert_eq!(pair.as_rule(), Rule::global_name_defn);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let expr = parse_expr_with_new_do(pairs.next().unwrap(), src);
    (FullName::new(namespace, &name), expr)
}

fn parse_type_qualified(pair: Pair<Rule>, src: &Rc<String>) -> QualType {
    assert_eq!(pair.as_rule(), Rule::type_qualified);
    let mut pairs = pair.into_inner();
    let (preds, kinds) = if pairs.peek().unwrap().as_rule() == Rule::predicates {
        parse_predicates(pairs.next().unwrap(), src)
    } else {
        (vec![], vec![])
    };
    let ty = parse_type(pairs.next().unwrap(), src);
    let qt = QualType {
        preds,
        ty,
        kind_preds: kinds,
    };
    qt
}

fn parse_predicates(pair: Pair<Rule>, src: &Rc<String>) -> (Vec<Predicate>, Vec<KindPredicate>) {
    assert_eq!(pair.as_rule(), Rule::predicates);
    let pairs = pair.into_inner();
    let mut ps: Vec<Predicate> = Default::default();
    let mut ks: Vec<KindPredicate> = Default::default();
    for pair in pairs {
        if pair.as_rule() == Rule::predicate {
            ps.push(parse_predicate(pair, src));
        } else if pair.as_rule() == Rule::predicate_kind {
            ks.push(parse_predicate_kind(pair, src));
        } else {
            unreachable!()
        }
    }
    (ps, ks)
}

fn parse_predicate_kind(pair: Pair<Rule>, src: &Rc<String>) -> KindPredicate {
    assert_eq!(pair.as_rule(), Rule::predicate_kind);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();
    let kind = parse_kind(pairs.next().unwrap(), src);
    KindPredicate { name, kind }
}

fn parse_predicate(pair: Pair<Rule>, src: &Rc<String>) -> Predicate {
    assert_eq!(pair.as_rule(), Rule::predicate);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let ty = parse_type(pairs.next().unwrap(), src);
    let trait_id = parse_trait(pairs.next().unwrap(), src);
    let mut pred = Predicate::make(trait_id, ty);
    pred.set_source(span);
    pred
}

fn parse_trait(pair: Pair<Rule>, _src: &Rc<String>) -> TraitId {
    assert_eq!(pair.as_rule(), Rule::trait_name);
    let mut pairs = pair.into_inner();
    let mut fullname = FullName::from_strs(&[], "");
    while pairs.peek().unwrap().as_rule() == Rule::namespace_item {
        fullname
            .namespace
            .names
            .push(pairs.next().unwrap().as_str().to_string());
    }
    let pair = pairs.next().unwrap();
    assert_eq!(pair.as_rule(), Rule::trait_local_name);
    fullname.name = pair.as_str().to_string();
    TraitId { name: fullname }
}

fn parse_kind(pair: Pair<Rule>, src: &Rc<String>) -> Rc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind);
    let mut pairs = pair.into_inner();
    let mut res: Rc<Kind> = parse_kind_nlr(pairs.next().unwrap(), src);
    for pair in pairs {
        res = kind_arrow(res, parse_kind_nlr(pair, src));
    }
    res
}

fn parse_kind_nlr(pair: Pair<Rule>, src: &Rc<String>) -> Rc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind_nlr);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    if pair.as_rule() == Rule::kind_star {
        parse_kind_star(pair, src)
    } else if pair.as_rule() == Rule::kind_braced {
        parse_kind_braced(pair, src)
    } else {
        unreachable!()
    }
}

fn parse_kind_star(pair: Pair<Rule>, _src: &Rc<String>) -> Rc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind_star);
    kind_star()
}

fn parse_kind_braced(pair: Pair<Rule>, src: &Rc<String>) -> Rc<Kind> {
    assert_eq!(pair.as_rule(), Rule::kind_braced);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    parse_kind(pair, src)
}

fn parse_module_defn(pair: Pair<Rule>, _src: &Rc<String>) -> String {
    pair.into_inner().next().unwrap().as_str().to_string()
}

fn parse_type_defn(pair: Pair<Rule>, src: &Rc<String>, namespace: &NameSpace) -> TypeDefn {
    assert_eq!(pair.as_rule(), Rule::type_defn);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let mut tyvars: Vec<Name> = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::type_var {
        tyvars.push(pairs.next().unwrap().as_str().to_string());
    }
    let pair = pairs.next().unwrap();
    let type_value = if pair.as_rule() == Rule::struct_defn {
        parse_struct_defn(pair, src)
    } else if pair.as_rule() == Rule::union_defn {
        parse_union_defn(pair, src)
    } else {
        unreachable!();
    };
    TypeDefn {
        name: FullName::new(namespace, name),
        value: type_value,
        tyvars,
    }
}

fn parse_struct_defn(pair: Pair<Rule>, src: &Rc<String>) -> TypeDeclValue {
    assert_eq!(pair.as_rule(), Rule::struct_defn);
    let mut pairs = pair.into_inner();
    let mut fields: Vec<Field> = Vec::new();
    let mut is_unbox = false; // Default value
    if pairs.peek().unwrap().as_rule() == Rule::box_or_unbox {
        is_unbox = parse_box_unbox(pairs.next().unwrap(), src);
    }
    for pair in pairs {
        fields.push(parse_type_field(pair, src));
    }
    TypeDeclValue::Struct(Struct { fields, is_unbox })
}

fn parse_union_defn(pair: Pair<Rule>, src: &Rc<String>) -> TypeDeclValue {
    assert_eq!(pair.as_rule(), Rule::union_defn);
    let mut pairs = pair.into_inner();
    let mut fields: Vec<Field> = Vec::new();
    let mut is_unbox = true; // Default value
    if pairs.peek().unwrap().as_rule() == Rule::box_or_unbox {
        is_unbox = parse_box_unbox(pairs.next().unwrap(), src);
    }
    for pair in pairs {
        fields.push(parse_type_field(pair, src));
    }
    TypeDeclValue::Union(Union { fields, is_unbox })
}

// Return true if unbox.
fn parse_box_unbox(pair: Pair<Rule>, _src: &Rc<String>) -> bool {
    assert_eq!(pair.as_rule(), Rule::box_or_unbox);
    if pair.as_str() == "box" {
        return false;
    } else if pair.as_str() == "unbox" {
        return true;
    }
    unreachable!();
}

fn parse_type_field(pair: Pair<Rule>, src: &Rc<String>) -> Field {
    assert_eq!(pair.as_rule(), Rule::type_field);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let ty = parse_type(pairs.next().unwrap(), src);
    Field {
        name: name.to_string(),
        ty,
    }
}

fn parse_expr(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_type_annotation(pair, msc, src)
}

fn parse_expr_with_new_do(pair: Pair<Rule>, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr);
    let mut msc = DoContext::default();
    let expr = parse_expr(pair, &mut msc, src);
    msc.expand_binds(expr)
}

fn parse_expr_type_annotation(
    pair: Pair<Rule>,
    msc: &mut DoContext,
    src: &Rc<String>,
) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_type_annotation);
    let span = Span::from_pair(src, &pair);
    let mut pairs = pair.into_inner();
    let mut expr = parse_expr_rtl_app(pairs.next().unwrap(), msc, src);
    match pairs.next() {
        None => {}
        Some(ty) => {
            expr = expr_tyanno(expr, parse_type(ty, src), Some(span));
        }
    }
    expr
}

// Parse combinator sequence, e.g., `f x y` or `x & f & g`
fn parse_combinator_sequence(
    pair: Pair<Rule>,
    src: &Rc<String>,
    inner_parser: fn(Pair<Rule>, &mut DoContext, &Rc<String>) -> Rc<ExprNode>,
    msc: &mut DoContext,
) -> Vec<Rc<ExprNode>> {
    pair.into_inner()
        .map(|pair| inner_parser(pair, msc, src))
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
    src: &Rc<String>,
    ops: HashMap<&str, BinaryOpInfo>,
    operator_rule: Rule,
    inner_parser: fn(Pair<Rule>, &mut DoContext, &Rc<String>) -> Rc<ExprNode>,
    msc: &mut DoContext,
) -> Rc<ExprNode> {
    let mut pairs = pair.into_inner();
    let mut expr = inner_parser(pairs.next().unwrap(), msc, src);
    let mut next_operation = BinaryOpInfo::default();
    let mut next_op_span: Option<Span> = None;
    for pair in pairs {
        if pair.as_rule() == operator_rule {
            next_operation = ops[pair.as_str()].clone();
            next_op_span = Some(Span::from_pair(src, &pair));
        } else {
            let mut lhs = expr;
            let mut rhs = inner_parser(pair, msc, src);
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
fn parse_expr_cmp(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_cmp);
    parse_binary_operator_sequence(
        pair,
        src,
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
        msc,
    )
}

// Operator && (right-associative)
fn parse_expr_and(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_and);
    let exprs = pair
        .into_inner()
        .map(|p| parse_expr_cmp(p, msc, src))
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
fn parse_expr_or(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_or);
    let exprs = pair
        .into_inner()
        .map(|p| parse_expr_and(p, msc, src))
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
fn parse_expr_plus(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_plus);
    parse_binary_operator_sequence(
        pair,
        src,
        HashMap::from([
            ("+", BinaryOpInfo::new(ADD_TRAIT_NAME, ADD_TRAIT_ADD_NAME)),
            (
                "-",
                BinaryOpInfo::new(SUBTRACT_TRAIT_NAME, SUBTRACT_TRAIT_SUBTRACT_NAME),
            ),
        ]),
        Rule::operator_plus,
        parse_expr_mul,
        msc,
    )
}

// Operator *,/,% (left associative)
fn parse_expr_mul(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_mul);
    parse_binary_operator_sequence(
        pair,
        src,
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
        msc,
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
fn parse_expr_unary(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    let span = Span::from_pair(&src, &pair);
    let pairs = pair.into_inner();
    let mut ops: Vec<UnaryOpInfo> = vec![];
    let mut spans: Vec<Span> = vec![];
    for pair in pairs {
        match pair.as_rule() {
            Rule::operator_unary => {
                spans.push(Span::from_pair(&src, &pair));
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
                let mut expr = parse_expr_composition(pair, msc, src);
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
fn parse_expr_rtl_app(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_rtl_app);
    let exprs = parse_combinator_sequence(pair, src, parse_expr_or, msc);
    let mut exprs_iter = exprs.iter().rev();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), vec![ret], span);
    }
    ret
}

// Parse function composition operator >> and <<.
fn parse_expr_composition(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
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
    let mut expr = parse_expr_bind(pairs.next().unwrap(), msc, src);
    while pairs.peek().is_some() {
        let op = pairs.next().unwrap();
        assert_eq!(op.as_rule(), Rule::operator_composition);
        let op_span = Span::from_pair(src, &op);
        let compose = expr_var(
            FullName::from_strs(&[STD_NAME], COMPOSE_FUNCTION_NAME),
            Some(op_span),
        );
        let rhs = parse_expr_bind(pairs.next().unwrap(), msc, src);
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
fn parse_expr_bind(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_bind);
    let mut stars = vec![];
    let mut pairs = pair.into_inner();
    while pairs.peek().unwrap().as_rule() == Rule::operator_bind {
        let star_pair = pairs.next().unwrap();
        stars.push(Span::from_pair(src, &star_pair));
    }
    let mut expr = parse_expr_ltr_app(pairs.next().unwrap(), msc, src);
    while !stars.is_empty() {
        expr = msc.push_monad(expr, stars.pop().unwrap());
    }
    expr
}

// Parse left to right application sequence, e.g., `x.f.g`. (left-associative)
fn parse_expr_ltr_app(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_ltr_app);
    let exprs = parse_combinator_sequence(pair, src, parse_expr_app, msc);
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
fn parse_expr_app(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_app);
    let mut pairs = pair.into_inner();
    let head = parse_expr_nlr(pairs.next().unwrap(), msc, src);
    let mut args = vec![];
    if pairs.peek().is_some() {
        // If parentheses for arguments are given,
        args = parse_arg_list(pairs.next().unwrap(), msc, src);
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

fn parse_arg_list(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Vec<Rc<ExprNode>> {
    assert_eq!(pair.as_rule(), Rule::arg_list);
    parse_combinator_sequence(pair, src, parse_expr, msc)
}

fn parse_expr_nlr(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_nlr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_lit => parse_expr_lit(pair, msc, src),
        Rule::expr_var => parse_expr_var(pair, src),
        Rule::expr_let => parse_expr_let(pair, msc, src),
        Rule::expr_if => parse_expr_if(pair, msc, src),
        Rule::expr_do => parse_expr_do(pair, msc, src),
        Rule::expr_lam => parse_expr_lam(pair, msc, src),
        Rule::expr_tuple => parse_expr_tuple(pair, msc, src),
        Rule::expr_make_struct => parse_expr_make_struct(pair, msc, src),
        Rule::expr_call_c => parse_expr_call_c(pair, msc, src),
        _ => unreachable!(),
    }
}

fn parse_expr_var(pair: Pair<Rule>, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_var);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let namespace = if pairs.peek().unwrap().as_rule() == Rule::namespace {
        parse_namespace(pairs.next().unwrap(), src)
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

fn parse_namespace(pair: Pair<Rule>, _src: &Rc<String>) -> NameSpace {
    assert_eq!(pair.as_rule(), Rule::namespace);
    let pairs = pair.into_inner();
    let mut ret: Vec<String> = Vec::new();
    for pair in pairs {
        ret.push(pair.as_str().to_string());
    }
    NameSpace::new(ret)
}

fn parse_expr_lit(expr: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_number_lit => parse_expr_number_lit(pair, src),
        Rule::expr_bool_lit => parse_expr_bool_lit(pair, src),
        Rule::expr_string_lit => parse_expr_string_lit(pair, src),
        Rule::expr_array_lit => parse_expr_array_lit(pair, msc, src),
        Rule::expr_nullptr_lit => parse_expr_nullptr_lit(pair, src),
        Rule::expr_u8_lit => parse_expr_u8_lit(pair, src),
        _ => unreachable!(),
    }
}

fn parse_expr_let(expr: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let pat = parse_pattern(pairs.next().unwrap(), src);
    if pat.pattern.validate_duplicated_vars() {
        error_exit_with_src(
            &format!("Each name defined in a pattern must appear exactly at once. "),
            &Some(span),
        );
    }
    let _eq_of_let = pairs.next().unwrap();
    let bound = parse_expr(pairs.next().unwrap(), msc, src);
    let _in_of_let = pairs.next().unwrap();
    let val = parse_expr_with_new_do(pairs.next().unwrap(), src);
    expr_let(pat, bound, val, Some(span))
}

fn parse_expr_lam(expr: Pair<Rule>, _msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let mut pats = vec![];
    while pairs.peek().unwrap().as_rule() == Rule::pattern {
        pats.push(parse_pattern(pairs.next().unwrap(), src));
    }
    let mut expr = parse_expr_with_new_do(pairs.next().unwrap(), src);
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

fn parse_expr_if(expr: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(expr.as_rule(), Rule::expr_if);
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let cond = pairs.next().unwrap();
    let then_val = pairs.next().unwrap();
    let else_val = pairs.next().unwrap();
    expr_if(
        parse_expr(cond, msc, src),
        parse_expr_with_new_do(then_val, src),
        parse_expr_with_new_do(else_val, src),
        Some(span),
    )
}

fn parse_expr_do(pair: Pair<Rule>, _msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert!(pair.as_rule() == Rule::expr_do);
    let pair = pair.into_inner().next().unwrap();
    let mut msc = DoContext::default();
    let expr = parse_expr(pair, &mut msc, src);
    let expr = msc.expand_binds(expr);
    expr
}

fn parse_expr_tuple(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_tuple);
    let span = Span::from_pair(&src, &pair);
    let exprs = pair
        .into_inner()
        .map(|p| parse_expr(p, msc, src).set_source(Some(span.clone())))
        .collect::<Vec<_>>();
    if exprs.len() == 1 {
        exprs[0].clone()
    } else {
        expr_make_struct(
            tycon(make_tuple_name(exprs.len() as u32)),
            exprs
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, expr)| (i.to_string(), expr))
                .collect(),
        )
    }
}

fn parse_expr_make_struct(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_make_struct);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let tycon = parse_tycon(pairs.next().unwrap());
    let mut fields = vec![];
    while pairs.peek().is_some() {
        let field_name = pairs.next().unwrap().as_str().to_string();
        let field_expr = parse_expr(pairs.next().unwrap(), msc, src);
        fields.push((field_name, field_expr));
    }
    expr_make_struct(tycon, fields).set_source(Some(span))
}

fn parse_expr_call_c(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_call_c);
    let span = Span::from_pair(src, &pair);
    let mut pairs = pair.into_inner();
    let ret_ty = parse_ffi_c_fun_ty(pairs.next().unwrap());
    let fun_name = pairs.next().unwrap().as_str().to_string();
    let param_tys = parse_ffi_param_tys(pairs.next().unwrap());
    let is_var_args = if pairs.peek().unwrap().as_rule() == Rule::ffi_var_args {
        pairs.next();
        true
    } else {
        false
    };
    let args = pairs.map(|pair| parse_expr(pair, msc, src)).collect();
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

fn parse_expr_number_lit(pair: Pair<Rule>, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_number_lit);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    assert_eq!(pair.as_rule(), Rule::number_lit_body);
    let val_str = pair.as_str();
    let is_float = val_str.contains(".");
    let ty = match pairs.next() {
        Some(pair) => {
            assert_eq!(pair.as_rule(), Rule::number_lit_type);
            if pair.as_str() == "U8" {
                make_u8_ty()
            } else if pair.as_str() == "I32" {
                make_i32_ty()
            } else if pair.as_str() == "U32" {
                make_u32_ty()
            } else if pair.as_str() == "I64" {
                make_i64_ty()
            } else if pair.as_str() == "U64" {
                make_u64_ty()
            } else if pair.as_str() == "F32" {
                make_f32_ty()
            } else if pair.as_str() == "F64" {
                make_f64_ty()
            } else {
                unreachable!()
            }
        }
        None => {
            if is_float {
                make_f64_ty()
            } else {
                make_i64_ty()
            }
        }
    };
    if is_float {
        let val = val_str.parse::<f64>();
        if val.is_err() {
            error_exit_with_src(
                "a literal string `{}` cannot be parsed as an floating number.",
                &Some(span),
            )
        }
        let val = val.unwrap();
        expr_float_lit(val, ty, Some(span))
    } else {
        let val = val_str.parse::<i128>();
        if val.is_err() {
            error_exit_with_src(
                "a literal string `{}` cannot be parsed as an integer.",
                &Some(span),
            )
        }
        let val = val.unwrap();
        expr_int_lit(val as u64, ty, Some(span))
    }
}

fn parse_expr_nullptr_lit(pair: Pair<Rule>, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_nullptr_lit);
    let span = Span::from_pair(&src, &pair);
    expr_nullptr_lit(Some(span))
}

fn parse_expr_bool_lit(pair: Pair<Rule>, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_bool_lit);
    let val = pair.as_str().parse::<bool>().unwrap();
    let span = Span::from_pair(&src, &pair);
    expr_bool_lit(val, Some(span))
}

fn parse_expr_array_lit(pair: Pair<Rule>, msc: &mut DoContext, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_array_lit);
    let span = Span::from_pair(&src, &pair);
    let elems = pair
        .into_inner()
        .map(|pair| parse_expr(pair, msc, src))
        .collect::<Vec<_>>();
    expr_array_lit(elems, Some(span))
}

fn parse_expr_string_lit(pair: Pair<Rule>, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_string_lit);
    let span = Span::from_pair(&src, &pair);
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
                        None => error_exit(&format!("invalid unicode character: u{:X}", code)),
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

fn parse_expr_u8_lit(pair: Pair<Rule>, src: &Rc<String>) -> Rc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_u8_lit);
    let span = Span::from_pair(&src, &pair);
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
                    if c == '\"' {
                        byte = 34;
                    } else if c == '\\' {
                        byte = 92;
                    } else if c == 'n' {
                        byte = 10;
                    } else if c == 'r' {
                        byte = 13;
                    } else if c == 't' {
                        byte = 9;
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

fn parse_type(pair: Pair<Rule>, src: &Rc<String>) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_expr);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_fun => parse_type_fun(pair, src),
        _ => unreachable!(),
    }
    .set_source(Some(span))
}

fn parse_type_fun(pair: Pair<Rule>, src: &Rc<String>) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_fun);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let src_ty = parse_type_tyapp(pairs.next().unwrap(), src);
    match pairs.next() {
        Some(pair) => {
            let dst_ty = parse_type(pair, src);
            type_fun(src_ty, dst_ty)
        }
        None => src_ty,
    }
    .set_source(Some(span))
}

fn parse_type_tyapp(pair: Pair<Rule>, src: &Rc<String>) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_tyapp);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    let mut ret = parse_type_nlr(pair, src);
    for pair in pairs {
        ret = type_tyapp(ret, parse_type_nlr(pair, src));
    }
    ret.set_source(Some(span))
}

fn parse_type_nlr(pair: Pair<Rule>, src: &Rc<String>) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_nlr);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_tycon => parse_type_tycon(pair, src),
        Rule::type_var => parse_type_var(pair, src),
        Rule::type_tuple => parse_type_tuple(pair, src),
        _ => unreachable!(),
    }
    .set_source(Some(span))
}

fn parse_type_var(pair: Pair<Rule>, src: &Rc<String>) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_var);
    let span = Span::from_pair(&src, &pair);
    type_tyvar(pair.as_str(), &kind_star()).set_source(Some(span))
}

fn parse_type_tycon(pair: Pair<Rule>, src: &Rc<String>) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_tycon);
    let span = Span::from_pair(&src, &pair);
    type_tycon(&parse_tycon(pair)).set_source(Some(span))
}

fn parse_tycon(pair: Pair<Rule>) -> Rc<TyCon> {
    assert_eq!(pair.as_rule(), Rule::type_tycon);
    tycon(FullName::from_strs(&[], pair.as_str()))
}

fn parse_type_tuple(pair: Pair<Rule>, src: &Rc<String>) -> Rc<TypeNode> {
    assert_eq!(pair.as_rule(), Rule::type_tuple);
    let span = Span::from_pair(&src, &pair);
    let types = pair
        .into_inner()
        .map(|p| parse_type(p, src))
        .collect::<Vec<_>>();
    if types.len() == 1 {
        types[0].clone()
    } else {
        let mut res = type_tycon(&tycon(make_tuple_name(types.len() as u32)));
        for ty in types {
            res = type_tyapp(res, ty);
        }
        res
    }
    .set_source(Some(span))
}

fn parse_pattern(pair: Pair<Rule>, src: &Rc<String>) -> Rc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern);
    let span = Span::from_pair(src, &pair);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::pattern_var => parse_pattern_var(pair, src),
        Rule::pattern_tuple => parse_pattern_tuple(pair, src),
        Rule::pattern_struct => parse_pattern_struct(pair, src),
        Rule::pattern_union => parse_pattern_union(pair, src),
        _ => unreachable!(),
    }
    .set_source(span)
}

fn parse_pattern_var(pair: Pair<Rule>, src: &Rc<String>) -> Rc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_var);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let var_name = pairs.next().unwrap().as_str();
    let ty = pairs.next().map(|ty| parse_type(ty, src));
    PatternNode::make_var(var_local(var_name), ty).set_source(span)
}

fn parse_pattern_tuple(pair: Pair<Rule>, src: &Rc<String>) -> Rc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_tuple);
    let span = Span::from_pair(&src, &pair);
    let pairs = pair.into_inner();
    let pats = pairs
        .map(|pair| parse_pattern(pair, src))
        .collect::<Vec<_>>();
    PatternNode::make_struct(
        tycon(make_tuple_name(pats.len() as u32)),
        pats.iter()
            .enumerate()
            .map(|(i, pat)| (i.to_string(), pat.clone()))
            .collect(),
    )
    .set_source(span)
}

fn parse_pattern_struct(pair: Pair<Rule>, src: &Rc<String>) -> Rc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_struct);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.clone().into_inner();
    let tycon = parse_tycon(pairs.next().unwrap());
    let mut field_to_pats = Vec::default();
    let mut field_names: HashSet<Name> = Default::default();
    while pairs.peek().is_some() {
        let field_name = pairs.next().unwrap().as_str().to_string();

        // Validate that field_name doesn't appear upto here.
        if field_names.contains(&field_name) {
            error_exit(&format!(
                "in the struct pattern `{}`, field `{}` appears multiple times.",
                pair.as_str(),
                field_name
            ));
        } else {
            field_names.insert(field_name.clone());
        }

        let pat = parse_pattern(pairs.next().unwrap(), src);
        field_to_pats.push((field_name, pat));
    }
    PatternNode::make_struct(tycon, field_to_pats).set_source(span)
}

fn parse_pattern_union(pair: Pair<Rule>, src: &Rc<String>) -> Rc<PatternNode> {
    assert_eq!(pair.as_rule(), Rule::pattern_union);
    let span = Span::from_pair(&src, &pair);
    let mut pairs = pair.into_inner();
    let tycon = parse_tycon(pairs.next().unwrap());
    let field_name = pairs.next().unwrap().as_str().to_string();
    let pat = parse_pattern(pairs.next().unwrap(), src);
    PatternNode::make_union(tycon, field_name, pat).set_source(span)
}

fn parse_import_statement(pair: Pair<Rule>, src: &Rc<String>) -> ImportStatement {
    assert_eq!(pair.as_rule(), Rule::import_statement);
    let span = Span::from_pair(&src, &pair);
    let pair = pair.into_inner().next().unwrap();
    let path = match pair.as_rule() {
        Rule::rel_import_path => parse_rel_import_path(pair, src),
        Rule::abs_import_path => parse_abs_import_path(pair, src),
        _ => unreachable!(),
    };
    ImportStatement {
        path,
        source: Some(span),
    }
}

fn parse_rel_import_path(pair: Pair<Rule>, src: &Rc<String>) -> ImportPath {
    assert_eq!(pair.as_rule(), Rule::rel_import_path);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::current_directory_path => parse_current_directory_path(pair, src),
        Rule::parent_directory_path => parse_parent_directory_path(pair, src),
        _ => unreachable!(),
    }
}

fn parse_abs_import_path(pair: Pair<Rule>, src: &Rc<String>) -> ImportPath {
    assert_eq!(pair.as_rule(), Rule::abs_import_path);
    let pair = pair.into_inner().next().unwrap();
    ImportPath::Absolute(parse_import_path(pair, src))
}

fn parse_current_directory_path(pair: Pair<Rule>, src: &Rc<String>) -> ImportPath {
    assert_eq!(pair.as_rule(), Rule::current_directory_path);
    let pair = pair.into_inner().next().unwrap();
    ImportPath::Relative(0, parse_import_path(pair, src))
}

fn parse_parent_directory_path(pair: Pair<Rule>, src: &Rc<String>) -> ImportPath {
    assert_eq!(pair.as_rule(), Rule::parent_directory_path);
    let mut pairs = pair.into_inner();
    let mut count: u32 = 0;
    while pairs.peek().unwrap().as_rule() == Rule::dot_dot_slash {
        pairs.next();
        count += 1;
    }
    ImportPath::Relative(count, parse_import_path(pairs.next().unwrap(), src))
}

fn parse_import_path(pair: Pair<Rule>, _src: &Rc<String>) -> Vec<Name> {
    assert_eq!(pair.as_rule(), Rule::import_path);
    pair.into_inner().map(|p| p.as_str().to_string()).collect()
}

fn rule_to_string(r: &Rule) -> String {
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
        Rule::operator_mul => "*".to_string(),
        Rule::operator_plus => "+".to_string(),
        Rule::operator_and => "&&".to_string(),
        Rule::operator_or => "||".to_string(),
        Rule::type_nlr => "type".to_string(),
        _ => format!("{:?}", r),
    }
}

fn message_parse_error(e: Error<Rule>, src: &Rc<String>) -> String {
    let mut msg: String = Default::default();

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
            if suggestion.is_none()
                && positives
                    .iter()
                    .find(|rule| **rule == Rule::arg_list)
                    .is_some()
            {
                suggestion = Some(
                    "Expression ended unexpectedly. Maybe forgot semicolon after definition?"
                        .to_string(),
                )
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
            end: min(s + 1, src.len()),
        },
        pest::error::InputLocation::Span((s, e)) => Span {
            input: src.clone(),
            start: s,
            end: e,
        },
    };
    msg += &span.to_string();
    msg
}
