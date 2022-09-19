#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;

use pest::error::Error;

use super::*;

pub fn parse_source(source: &str) -> Arc<ExprInfo> {
    let file = FixParser::parse(Rule::file, source);
    let file = match file {
        Ok(res) => res,
        Err(e) => error_exit(&message_parse_error(e)),
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
        Rule::var => parse_var_as_expr(pair),
        Rule::expr_let => parse_expr_let(pair),
        Rule::expr_if => parse_expr_if(pair),
        Rule::expr_lam => parse_expr_lam(pair),
        Rule::expr_forall => parse_forall_expr(pair),
        Rule::expr_braced => parse_bracket_expr(pair),
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

fn parse_var_as_expr(pair: Pair<Rule>) -> Arc<ExprInfo> {
    assert_eq!(pair.as_rule(), Rule::var);
    var(pair.as_str())
}

fn parse_var_as_var(pair: Pair<Rule>) -> Arc<Var> {
    assert_eq!(pair.as_rule(), Rule::var);
    var_var(pair.as_str(), None)
}

fn parse_var_typed_as_var(pair: Pair<Rule>) -> Arc<Var> {
    assert_eq!(pair.as_rule(), Rule::var_typed);
    let mut pairs = pair.into_inner();
    let var = pairs.next().unwrap();
    let ty = pairs.next().unwrap();
    var_var(var.as_str(), Some(parse_type(ty)))
}

fn parse_expr_let(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let mut pairs = expr.into_inner();
    let var = pairs.next().unwrap();
    let _eq_of_let = pairs.next().unwrap();
    let bound = pairs.next().unwrap();
    let _in_of_let = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    let_in(parse_var_as_var(var), parse_expr(bound), parse_expr(val))
}

fn parse_expr_lam(expr: Pair<Rule>) -> Arc<ExprInfo> {
    let mut pairs = expr.into_inner();
    let var_with_type = pairs.next().unwrap();
    let _arrow_of_lam = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    lam(parse_var_typed_as_var(var_with_type), parse_expr(val))
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

fn parse_expr_if(expr: Pair<Rule>) -> Arc<ExprInfo> {
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

fn parse_type_braced(type_expr: Pair<Rule>) -> Arc<Type> {
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
        Rule::type_braced => parse_type_braced(pair),
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

fn rule_to_string(r: &Rule) -> String {
    match r {
        Rule::EOI => "end-of-input".to_string(),
        Rule::expr_int_lit => "integer".to_string(),
        Rule::expr_bool_lit => "boolean".to_string(),
        Rule::expr_nlc => "expression".to_string(),
        Rule::var => "variable".to_string(),
        Rule::in_of_let => "`in` or `;`".to_string(),
        Rule::tyapp_bracket => "`<{types}>`".to_string(),
        Rule::eq_of_let => "`=`".to_string(),
        Rule::var_typed => "variable with type annotation".to_string(),
        Rule::type_expr => "type".to_string(),
        Rule::arrow_of_lam => "`->`".to_string(),
        _ => format!("{:?}", r),
    }
}

fn message_parse_error(e: Error<Rule>) -> String {
    let mut msg: String = Default::default();

    // Show error content.
    msg += "parse error: expected ";
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
            }
            if negatives.len() > 0 {
                msg += "neither ";
                let words: Vec<String> = negatives.iter().map(rule_to_string).collect();
                msg += &concat_words(words, "nor");
            }
        }
        pest::error::ErrorVariant::CustomError { message: _ } => unreachable!(),
    };
    msg += "\n";

    // Show line and column number.
    // TODO: Show filename here.
    let (line, col) = match e.line_col {
        pest::error::LineColLocation::Pos(s) => s,
        pest::error::LineColLocation::Span(s, _) => s,
    };
    msg += &format!("at {}:{}", line, col);
    msg += "\n";

    // Show source code.
    let linenum_str = line.to_string();
    let linnum_chars = linenum_str.len();
    msg += &(" ".repeat(linnum_chars) + " | " + "\n");
    msg += &(linenum_str.clone() + " | ");
    msg += e.line();
    msg += "\n";
    msg += &(" ".repeat(linnum_chars) + " | ");
    msg += &(" ".repeat(col - 1) + "^");
    msg += "\n";
    msg
}
