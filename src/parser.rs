#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;
use super::*;

pub fn parse_source(source: &str) -> Arc<ExprInfo> {
    let file = FixParser::parse(Rule::file, source).unwrap();
    parse_file(file)
}

fn parse_file(mut file: Pairs<Rule>) -> Arc<ExprInfo> {
    let pair = file.next().unwrap();
    match pair.as_rule() {
        Rule::expr => return parse_expr(pair),
        _ => unreachable!(),
    }
}

fn parse_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::app_expr => parse_app_expr(pair),
        Rule::not_app_expr => parse_not_app_expr(pair),
        _ => unreachable!(),
    }
}

fn parse_app_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let mut subexprs = expr.into_inner();
    let mut ret = parse_not_app_expr(subexprs.next().unwrap());
    for pair in subexprs {
        ret = app(ret, parse_not_app_expr(pair));
    }
    ret
}

fn parse_not_app_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::lit_expr => parse_lit_expr(pair),
        Rule::var_expr => parse_var_expr(pair),
        Rule::let_expr => parse_let_expr(pair),
        Rule::lam_expr => parse_lam_expr(pair),
        Rule::if_expr => parse_if_expr(pair),
        Rule::bracket_expr => parse_bracket_expr(pair),
        _ => unreachable!(),
    }
}

fn parse_lit_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::int_lit_expr => parse_int_expr(pair),
        Rule::bool_lit_expr => parse_bool_lit_expr(pair),
        _ => unreachable!(),
    }
}

fn parse_var_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    var(expr.as_str())
}

fn parse_var_var(var: Pair<Rule>) -> Arc<Var> {
    var_var(var.as_str())
}

fn parse_let_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let mut pairs = expr.into_inner();
    let var = pairs.next().unwrap();
    let bound = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    let_in(parse_var_var(var), parse_expr(bound), parse_expr(val))
}

fn parse_lam_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let mut pairs = expr.into_inner();
    let var = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    lam(parse_var_var(var), parse_expr(val))
}

fn parse_if_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let mut pairs = expr.into_inner();
    let cond = pairs.next().unwrap();
    let then_val = pairs.next().unwrap();
    let else_val = pairs.next().unwrap();
    conditional(parse_expr(cond), parse_expr(then_val), parse_expr(else_val))
}

fn parse_bracket_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let inner = expr.into_inner().next().unwrap();
    parse_expr(inner)
}

fn parse_int_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let val = expr.as_str().parse::<i64>().unwrap();
    int(val)
}

fn parse_bool_lit_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let val = expr.as_str().parse::<bool>().unwrap();
    bool(val)
}
