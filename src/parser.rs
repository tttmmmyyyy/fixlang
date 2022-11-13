#[derive(Parser)]
#[grammar = "grammer.pest"]
struct FixParser;

use pest::error::Error;

use super::*;

// lifetime-free version of pest::Span
#[derive(Clone)]
pub struct Span {
    input: Arc<String>,
    start: usize,
    end: usize,
}

impl Span {
    pub fn from_pair(src: &Arc<String>, pair: &Pair<Rule>) -> Self {
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

fn unite_span(lhs: &Option<Span>, rhs: &Option<Span>) -> Option<Span> {
    match lhs {
        None => rhs.clone(),
        Some(s) => rhs.clone().map(|t| s.unite(&t)),
    }
}

pub fn parse_source(source: &str) -> FixModule {
    let source = Arc::new(String::from(source));
    let file = FixParser::parse(Rule::file, &source);
    let file = match file {
        Ok(res) => res,
        Err(e) => error_exit(&message_parse_error(e)),
    };
    parse_file(file, &source)
}

fn parse_file(mut file: Pairs<Rule>, src: &Arc<String>) -> FixModule {
    let pair = file.next().unwrap();
    match pair.as_rule() {
        Rule::module => return parse_module(pair, src),
        _ => unreachable!(),
    }
}

fn parse_module(pair: Pair<Rule>, src: &Arc<String>) -> FixModule {
    assert_eq!(pair.as_rule(), Rule::module);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let mut type_decls: Vec<TypeDecl> = Vec::new();
    let mut expr: Option<Arc<ExprNode>> = None;
    for pair in pairs {
        match pair.as_rule() {
            Rule::type_decl => {
                type_decls.push(parse_type_decl(pair, src));
            }
            Rule::expr => {
                expr = Some(parse_expr(pair, src));
            }
            _ => unreachable!(),
        }
    }
    FixModule {
        name: name.to_string(),
        type_decls,
        expr: expr.unwrap(),
    }
}

fn parse_type_decl(pair: Pair<Rule>, src: &Arc<String>) -> TypeDecl {
    assert_eq!(pair.as_rule(), Rule::type_decl);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let mut fields: Vec<StructField> = Vec::new();
    for pair in pairs {
        fields.push(parse_type_field(pair, src));
    }
    TypeDecl {
        name: name.to_string(),
        value: TypeDeclValue::Struct(fields),
    }
}

fn parse_type_field(pair: Pair<Rule>, src: &Arc<String>) -> StructField {
    assert_eq!(pair.as_rule(), Rule::type_field);
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str();
    let ty = parse_type(pairs.next().unwrap());
    StructField {
        name: name.to_string(),
        ty,
    }
}

fn parse_expr(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr);
    let pair = pair.into_inner().next().unwrap();
    parse_expr_rtl_app(pair, src)
}

// Parse combinator sequence, e.g., `f x y` or `x & f & g`
fn parse_combinator_sequence(
    pair: Pair<Rule>,
    src: &Arc<String>,
    inner_parser: fn(Pair<Rule>, &Arc<String>) -> Arc<ExprNode>,
) -> Vec<Arc<ExprNode>> {
    pair.into_inner()
        .map(|pair| inner_parser(pair, src))
        .collect()
}

// Parse right to left application sequence, e.g., `g $ f $ x`. (right-associative)
fn parse_expr_rtl_app(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_rtl_app);
    let exprs = parse_combinator_sequence(pair, src, parse_expr_ltr_app);
    let mut exprs_iter = exprs.iter().rev();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), ret, span);
    }
    ret
}

// Parse left to right application sequence, e.g., `x & f & g`. (left-associative)
fn parse_expr_ltr_app(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_ltr_app);
    let exprs = parse_combinator_sequence(pair, src, parse_expr_app);
    let mut exprs_iter = exprs.iter();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(expr.clone(), ret, span);
    }
    ret
}

// Parse application sequence, e.g., `f x y`. (left-associative)
fn parse_expr_app(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_app);
    let exprs = parse_combinator_sequence(pair, src, parse_expr_nlr);
    let mut exprs_iter = exprs.iter();
    let mut ret = exprs_iter.next().unwrap().clone();
    for expr in exprs_iter {
        let span = unite_span(&expr.source, &ret.source);
        ret = expr_app(ret, expr.clone(), span);
    }
    ret
}

fn parse_expr_nlr(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::expr_nlr);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_lit => parse_expr_lit(pair, src),
        Rule::var => parse_var_as_expr(pair, src),
        Rule::expr_let => parse_expr_let(pair, src),
        Rule::expr_if => parse_expr_if(pair, src),
        Rule::expr_lam => parse_expr_lam(pair, src),
        Rule::expr_braced => parse_bracket_expr(pair, src),
        _ => unreachable!(),
    }
}

// fn parse_tyapp_bracket(pair: Pair<Rule>) -> Vec<Arc<TypeNode>> {
//     let pairs = pair.into_inner();
//     let mut ret: Vec<Arc<TypeNode>> = vec![];
//     for pair in pairs {
//         ret.push(parse_type(pair));
//     }
//     ret
// }

fn parse_expr_lit(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let pair = expr.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::expr_int_lit => parse_expr_int_lit(pair, src),
        Rule::expr_bool_lit => parse_expr_bool_lit(pair, src),
        _ => unreachable!(),
    }
}

fn parse_var_as_expr(pair: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    assert_eq!(pair.as_rule(), Rule::var);
    expr_var(pair.as_str(), Some(Span::from_pair(&src, &pair)))
}

fn parse_var_as_var(pair: Pair<Rule>, src: &Arc<String>) -> Arc<Var> {
    assert_eq!(pair.as_rule(), Rule::var);
    var_local(pair.as_str(), None, Some(Span::from_pair(&src, &pair)))
}

fn parse_expr_let(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let var = pairs.next().unwrap();
    let _eq_of_let = pairs.next().unwrap();
    let bound = pairs.next().unwrap();
    let _in_of_let = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    expr_let(
        parse_var_as_var(var, src),
        parse_expr(bound, src),
        parse_expr(val, src),
        Some(span),
    )
}

fn parse_expr_lam(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let var = pairs.next().unwrap();
    let _arrow_of_lam = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    expr_abs(parse_var_as_var(var, src), parse_expr(val, src), Some(span))
}

fn parse_expr_if(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let mut pairs = expr.into_inner();
    let cond = pairs.next().unwrap();
    let then_val = pairs.next().unwrap();
    let else_val = pairs.next().unwrap();
    expr_if(
        parse_expr(cond, src),
        parse_expr(then_val, src),
        parse_expr(else_val, src),
        Some(span),
    )
}

fn parse_bracket_expr(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let inner = expr.into_inner().next().unwrap();
    parse_expr(inner, src).set_source(Some(span))
}

fn parse_expr_int_lit(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let span = Span::from_pair(&src, &expr);
    let val = expr.as_str().parse::<i64>().unwrap();
    int(val, Some(span))
}

fn parse_expr_bool_lit(expr: Pair<Rule>, src: &Arc<String>) -> Arc<ExprNode> {
    let val = expr.as_str().parse::<bool>().unwrap();
    let span = Span::from_pair(&src, &expr);
    bool(val, Some(span))
}

fn parse_type(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_expr);
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_fun => parse_type_fun(pair),
        _ => unreachable!(),
    }
}

fn parse_type_fun(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_fun);
    let mut pairs = type_expr.into_inner();
    let src_ty = parse_type_tyapp(pairs.next().unwrap());
    let dst_ty = parse_type(pairs.next().unwrap());
    type_fun(src_ty, dst_ty)
}

fn parse_type_tyapp(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_tyapp);
    let mut pairs = type_expr.into_inner();
    let mut pair = pairs.next().unwrap();
    let mut ret = parse_type_nlr(pair);
    for pair in pairs {
        ret = type_tyapp(ret, parse_type(pair));
    }
    ret
}

fn parse_type_nlr(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_nlr);
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::type_tycon => parse_type_tycon(pair),
        Rule::type_braced => parse_type_braced(pair),
        _ => unreachable!(),
    }
}

fn parse_type_tycon(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_tycon);
    type_tycon(&tycon(type_expr.as_str()))
}

fn parse_type_braced(type_expr: Pair<Rule>) -> Arc<TypeNode> {
    assert_eq!(type_expr.as_rule(), Rule::type_braced);
    let mut pairs = type_expr.into_inner();
    let pair = pairs.next().unwrap();
    parse_type(pair)
}

fn rule_to_string(r: &Rule) -> String {
    match r {
        Rule::EOI => "end-of-input".to_string(),
        Rule::expr_int_lit => "integer".to_string(),
        Rule::expr_bool_lit => "boolean".to_string(),
        Rule::expr_nlr => "expression".to_string(),
        Rule::var => "variable".to_string(),
        Rule::in_of_let => "`in` or `;`".to_string(),
        Rule::eq_of_let => "`=`".to_string(),
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
