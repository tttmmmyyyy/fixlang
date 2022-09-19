#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;

use std::process;

use pest::error::Error;

use super::*;

pub fn parse_source(source: &str) -> Arc<ExprInfo> {
    let file = FixParser::parse(Rule::file, source);
    let file = match file {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", message_parse_error(e));
            process::exit(1)
        }
    };
    parse_file(file)
}

fn parse_file(mut file: Pairs<Rule>) -> Arc<ExprInfo> {
    let pair = file.next().unwrap();
    match pair.as_rule() {
        Rule::expr => return parse_expr(pair),
        _ => unreachable!(),
    }
}

fn parse_expr(pair: Pair<Rule>) -> Arc<ExprInfo> {
    assert_eq!(pair.as_rule(), Rule::expr);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_app_seq(pair)
}

fn parse_expr_app_seq(pair: Pair<Rule>) -> Arc<ExprInfo> {
    assert_eq!(pair.as_rule(), Rule::expr_app_seq);
    let mut pairs = pair.into_inner();
    let mut ret = parse_expr_nlc_tyapp(pairs.next().unwrap());
    for pair in pairs {
        ret = app(ret, parse_expr_nlc_tyapp(pair));
    }
    ret
}

fn parse_expr_nlc_tyapp(pair: Pair<Rule>) -> Arc<ExprInfo> {
    assert_eq!(pair.as_rule(), Rule::expr_nlc_tyapp);
    let mut pairs = pair.into_inner();
    let mut expr = parse_expr_nlc(pairs.next().unwrap());
    match pairs.next() {
        Some(pair) => {
            let types = parse_tyapp_bracket(pair);
            for ty in types {
                expr = app_ty(expr, ty);
            }
        }
        _ => {}
    };
    expr
}

fn parse_expr_nlc(pair: Pair<Rule>) -> Arc<ExprInfo> {
    assert_eq!(pair.as_rule(), Rule::expr_nlc);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_lit => parse_expr_lit(pair),
        Rule::expr_var => parse_expr_var(pair),
        Rule::let_expr => parse_let_expr(pair),
        Rule::if_expr => parse_if_expr(pair),
        Rule::lam_expr => parse_lam_expr(pair),
        Rule::forall_expr => parse_forall_expr(pair),
        Rule::bracket_expr => parse_bracket_expr(pair),
        _ => unreachable!(),
    }
}

fn parse_tyapp_bracket(pair: Pair<Rule>) -> Vec<Arc<Type>> {
    let pairs = pair.into_inner();
    let mut ret: Vec<Arc<Type>> = vec![];
    for pair in pairs {
        ret.push(parse_type(pair));
    }
    ret
}

fn parse_expr_lit(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_int_lit => parse_expr_int_lit(pair),
        Rule::expr_bool_lit => parse_expr_bool_lit(pair),
        _ => unreachable!(),
    }
}

fn parse_expr_var(expr: Pair<Rule>) -> Arc<ExprInfo> {
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

fn parse_expr_int_lit(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let val = expr.as_str().parse::<i64>().unwrap();
    int(val)
}

fn parse_expr_bool_lit(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let val = expr.as_str().parse::<bool>().unwrap();
    bool(val)
}

fn parse_type(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_app => parse_type_app(pair),
        Rule::type_fun => parse_type_fun(pair),
        Rule::type_except_app_fun => parse_type_except_app_fun(pair),
        _ => unreachable!(),
    }
}

fn parse_type_bracket(type_expr: Pair<Rule>) -> Arc<Type> {
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    parse_type(pair)
}

fn parse_type_except_app_fun(type_expr: Pair<Rule>) -> Arc<Type> {
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
    let mut ret = parse_type_except_app_fun(head);
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
    let src_ty = parse_type_except_fun(pairs.next().unwrap());
    let dst_ty = parse_type(pairs.next().unwrap());
    type_fun(src_ty, dst_ty)
}

fn parse_type_except_fun(pair: Pair<Rule>) -> Arc<Type> {
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_app => parse_type_app(pair),
        Rule::type_except_app_fun => parse_type_except_app_fun(pair),
        _ => unreachable!(),
    }
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
    make_bultin_tycon(type_expr.as_str())
}

fn rule_to_string(r: Rule) -> String {
    match r {
        Rule::EOI => todo!(),
        Rule::sep => todo!(),
        Rule::expr_int_lit => todo!(),
        Rule::expr_bool_lit => todo!(),
        Rule::expr_lit => todo!(),
        Rule::keywords => todo!(),
        Rule::let_in => todo!(),
        Rule::var_char => todo!(),
        Rule::expr_var => todo!(),
        Rule::expr_var_typed => todo!(),
        Rule::let_expr => todo!(),
        Rule::if_expr => todo!(),
        Rule::lam_expr => todo!(),
        Rule::forall_expr => todo!(),
        Rule::bracket_expr => todo!(),
        Rule::expr => todo!(),
        Rule::type_expr => todo!(),
        Rule::type_bracket => todo!(),
        Rule::type_except_app_fun => todo!(),
        Rule::type_except_fun => todo!(),
        Rule::type_var => todo!(),
        Rule::type_lit => todo!(),
        Rule::type_app => todo!(),
        Rule::type_tycon_app => todo!(),
        Rule::type_fun => todo!(),
        Rule::type_forall => todo!(),
        Rule::tycon => todo!(),
        Rule::block_comment => todo!(),
        Rule::block_commented_character => todo!(),
        Rule::line_comment => todo!(),
        Rule::line_commented_character => todo!(),
        Rule::file => todo!(),
        Rule::expr_nlc => todo!(),
        Rule::tyapp_bracket => todo!(),
        Rule::expr_nlc_tyapp => todo!(),
        Rule::expr_app_seq => todo!(),
    }
}

fn message_parse_error(e: Error<Rule>) -> String {
    String::from("TODO")
}
