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
        Rule::tyapp_expr => parse_tyapp_expr(pair),
        Rule::expr_except_app_tyapp => parse_expr_except_app_tyapp(pair),
        _ => unreachable!(),
    }
}

fn parse_app_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let mut subexprs = expr.into_inner();
    let mut ret = parse_expr_except_app(subexprs.next().unwrap());
    for pair in subexprs {
        ret = app(ret, parse_expr_except_app(pair));
    }
    ret
}

fn parse_expr_except_app(pair: Pair<Rule>) -> Arc<ExprInfo> {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::tyapp_expr => parse_tyapp_expr(pair),
        Rule::expr_except_app_tyapp => parse_expr_except_app_tyapp(pair),
        _ => unreachable!(),
    }
}

fn parse_tyapp_expr(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let mut pairs = expr.into_inner();
    let mut ret = parse_expr_except_app_tyapp(pairs.next().unwrap());
    for pair in pairs {
        ret = app_ty(ret, parse_type(pair));
    }
    ret
}

fn parse_expr_except_app_tyapp(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::lit_expr => parse_lit_expr(pair),
        Rule::var_expr => parse_var_expr(pair),
        Rule::let_expr => parse_let_expr(pair),
        Rule::lam_expr => parse_lam_expr(pair),
        Rule::forall_expr => parse_forall_expr(pair),
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
    var_var(var.as_str(), None)
}

fn parse_var_var_with_type(var_with_type: Pair<Rule>) -> Arc<Var> {
    let mut pairs = var_with_type.into_inner();
    let var = pairs.next().unwrap();
    let ty = pairs.next().unwrap();
    var_var(var.as_str(), Some(parse_type(ty)))
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
    let var_with_type = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    lam(parse_var_var_with_type(var_with_type), parse_expr(val))
}

fn parse_forall_expr(pair: Pair<Rule>) -> Arc<ExprInfo> {
    let mut pairs = pair.into_inner();
    let mut vars: Vec<Arc<TyVar>> = Default::default();
    let mut expr = loop {
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::type_var => {
                vars.push(tyvar_var(pair.as_str()));
            }
            Rule::expr => {
                break parse_expr(pair);
            }
            _ => {
                unreachable!()
            }
        }
    };
    for var in vars.iter().rev() {
        expr = forall(var.clone(), expr);
    }
    expr
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

fn parse_type(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_app => parse_type_app(pair),
        Rule::type_fun => parse_type_fun(pair),
        Rule::type_expr_not_app_or_fun => parse_type_expr_not_app_or_fun(pair),
        _ => unreachable!(),
    }
}

fn parse_type_bracket(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    parse_type(pair)
}

fn parse_type_expr_not_app_or_fun(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_var => parse_type_var(pair),
        Rule::type_lit => parse_type_lit(pair),
        Rule::type_tycon_app => parse_type_tycon_app(pair),
        Rule::type_forall => parse_type_forall(pair),
        Rule::type_bracket => parse_type_bracket(pair),
        _ => unreachable!(),
    }
}

fn parse_type_var(type_expr: Pair<Rule>) -> Arc<Type> {
    tyvar_ty(type_expr.as_str())
}

fn parse_type_lit(type_expr: Pair<Rule>) -> Arc<Type> {
    make_bultin_type(type_expr.as_str())
}

fn parse_type_app(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let head = pairs.next().unwrap();
    let mut ret = parse_type_expr_not_app_or_fun(head);
    for pair in pairs {
        ret = type_app(ret, parse_type(pair))
    }
    ret
}

fn parse_type_tycon_app(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let tycon = pairs.next().unwrap();
    let tycon = parse_tycon(tycon);
    let mut args: Vec<Arc<Type>> = Default::default();
    for pair in pairs {
        args.push(parse_type(pair));
    }
    tycon_app(tycon, args)
}

fn parse_type_fun(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let src_ty = parse_type(pairs.next().unwrap());
    let dst_ty = parse_type(pairs.next().unwrap());
    type_fun(src_ty, dst_ty)
}

fn parse_type_forall(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let mut vars: Vec<Arc<TyVar>> = Default::default();
    let mut type_expr = loop {
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::type_var => {
                vars.push(tyvar_var(pair.as_str()));
            }
            Rule::type_expr => {
                break parse_type(pair);
            }
            _ => {
                unreachable!()
            }
        }
    };
    for var in vars.iter().rev() {
        type_expr = type_forall(var.clone(), type_expr);
    }
    type_expr
}

fn parse_tycon(type_expr: Pair<Rule>) -> Arc<TyCon> {
    tycon(
        u32::MAX, /* implies N/A */
        type_expr.as_str(),
        u32::MAX, /* implies N/A */
    )
}
