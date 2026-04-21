use std::error::Error;
use crate::ast::{BinOp, Expr, LogicalOp, UnaryOp};
use pest::{Parser};
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct FeoxParser;


// pub struct ParserEnv {
//     pub vars: HashMap<String, usize>,
//     pub counter: usize,
// }
// 
// impl ParserEnv {
//     pub fn new() -> ParserEnv {
//         ParserEnv {
//             vars: HashMap::new(),
//             counter: 0,
//         }
//     }
//     pub fn idx(&mut self, name: &str) -> usize {
//         if let Some(idx) = self.vars.get(name) {
//             *idx
//         } else {
//             let cur = self.counter;
//             self.counter += 1;
//             self.vars.insert(name.to_string(), cur);
//             cur
//         }
//     }
// }

pub fn parse(source: &str) -> Result<Vec<Expr>, Box<dyn Error>> {
    let pairs = FeoxParser::parse(Rule::program, source)?;
    Ok(parse_program(pairs))
}

fn parse_program(pairs: Pairs<Rule>) -> Vec<Expr> {
    pairs
        .filter(|p| p.as_rule() != Rule::EOI)
        .map(|p| parse_expr(p))
        .collect()
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::assign => parse_assign(pair),
        Rule::declare => parse_declare(pair),
        Rule::lambda => parse_lambda(pair),
        Rule::if_ => parse_if(pair),
        Rule::while_ => parse_while(pair),
        Rule::for_ => parse_for(pair),
        Rule::mod_ => parse_mod(pair),
        Rule::return_ => parse_return(pair),
        Rule::break_ => Expr::Break,
        Rule::continue_ => Expr::Continue,

        Rule::logical_and
        | Rule::logical_or => parse_logical_chain(pair),

        Rule::or
        | Rule::and
        | Rule::xor
        | Rule::cmp
        | Rule::range
        | Rule::add
        | Rule::mul
        | Rule::pow => parse_binary_chain(pair),

        Rule::unary => parse_unary(pair),
        Rule::postfix => parse_postfix(pair),
        Rule::primary => parse_primary(pair),

        _ => unreachable!("{:?}", pair.as_rule()),
    }
}

fn parse_postfix(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let mut expr = parse_primary(inner.next().unwrap());

    for p in inner {
        match p.as_rule() {
            Rule::call => {
                let args = p.into_inner().map(|p| parse_expr(p)).collect();

                expr = Expr::Call {
                    func: Box::new(expr),
                    args,
                };
            }

            Rule::index => {
                let idx = parse_expr(p.into_inner().next().unwrap());

                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(idx),
                };
            }

            Rule::method_call => {
                let mut mc = p.into_inner();
                let method = mc.next().unwrap().as_str().to_string();
                let mut args: Vec<Expr> = mc.map(|p| parse_expr(p)).collect();
                args.insert(0, expr);

                expr = Expr::Call {
                    func: Box::new(Expr::Ident(method)),
                    args,
                };
            }

            _ => unreachable!(),
        }
    }

    expr
}

fn parse_declare(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expr(inner.next().unwrap());

    Expr::Declare {
        name,
        value: Box::new(value),
    }
}

fn parse_mod(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let modulus = parse_expr(inner.next().unwrap());
    let block = parse_block(inner.next().unwrap());

    Expr::Mod {
        modulus: Box::new(modulus),
        body: Box::new(block),
    }
}

fn parse_return(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let value = inner.next().map(|p| parse_expr(p));

    Expr::Return(value.map(Box::new))
}
fn parse_assign(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut indices = Vec::new();
    let mut value = Expr::Nil;
    for p in inner {
        match p.as_rule() {
            Rule::index => indices.push(parse_expr(p.into_inner().next().unwrap())),
            _ => value = parse_expr(p),
        }
    }

    Expr::Assign {
        name,
        indices,
        value: Box::new(value),
    }
}


fn parse_logical_chain(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let mut expr = parse_expr(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let rhs = parse_expr(inner.next().unwrap());
            expr = Expr::LogicalOp {
                op: match op.as_str() {
                    "&&" => LogicalOp::And,
                    "||" => LogicalOp::Or,
                    _ => unreachable!("{}", op.as_str()),
                },
                left: Box::new(expr),
                right: Box::new(rhs),

        }
    }

    expr
}

fn parse_binary_chain(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let mut expr = parse_expr(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let rhs = parse_expr(inner.next().unwrap());
        if op.as_str() == ".." || op.as_str() == "..=" {
            expr = Expr::Range {
                start: Box::new(expr),
                end: Box::new(rhs),
                inclusive: op.as_str() == "..=",
            }
        } else {
            expr = Expr::BinOp {
                op: match op.as_str() {
                    "+" => BinOp::Add,
                    "-" => BinOp::Sub,
                    "*" => BinOp::Mul,
                    "/" => BinOp::Div,
                    "%" => BinOp::Rem,
                    "&" => BinOp::And,
                    "^" => BinOp::Xor,
                    "|" => BinOp::Or,
                    "**" => BinOp::Pow,
                    "==" => BinOp::Eq,
                    "!=" => BinOp::Neq,
                    "<=" => BinOp::Le,
                    "<" => BinOp::Lt,
                    ">=" => BinOp::Ge,
                    ">" => BinOp::Gt,
                    _ => unreachable!("{}", op.as_str()),
                },
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
    }

    expr
}

fn parse_unary(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.clone().into_inner();

    if pair.as_str().starts_with('-') || pair.as_str().starts_with('!') {
        let op = pair.as_str().chars().next().unwrap();
        let expr = parse_unary(inner.next().unwrap());

        Expr::UnaryOp {
            op: match op {
                '-' => UnaryOp::Neg,
                '!' => UnaryOp::Not,
                _ => unreachable!(),
            },
            expr: Box::new(expr),
        }
    } else {
        parse_postfix(inner.next().unwrap())
    }
}

fn parse_primary(pair: Pair<Rule>) -> Expr {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::number => {
            let n = inner.as_str().parse::<i64>().unwrap();
            Expr::Number(n)
        }

        Rule::string => {
            let s = inner.as_str();
            let st = s[1..s.len() - 1].to_string();
            let mut chs = st.chars();
            let mut res = String::new();

            while let Some(c) = chs.next() {
                if c == '\\' {
                    match chs.next() {
                        Some('n') => res.push('\n'),
                        Some('r') => res.push('\r'),
                        Some('t') => res.push('\t'),
                        Some('"') => res.push('"'),
                        Some('\\') => res.push('\\'),
                        _ => unreachable!(),
                    }
                } else {
                    res.push(c);
                }
            }
            Expr::String(res)
        }

        Rule::char => {
            let chars = inner.as_str().chars().collect::<Vec<_>>();
            let c = chars[1];
            Expr::Char(
                if c == '\\' {
                    match chars[2] {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\'' => '\'',
                        '\\' => '\\',
                        _ => unreachable!(),
                    }
                }
                else {
                    c
                }
            )
        }

        Rule::bool => Expr::Bool(inner.as_str() == "true"),

        Rule::nil => Expr::Nil,

        Rule::ident => Expr::Ident(
            inner.as_str().to_string(),
        ),

        Rule::expr => parse_expr(inner),

        Rule::array => parse_array(inner),
        Rule::block => parse_block(inner),
        Rule::bignum => {
            let s = inner.as_str();
            let (base, exp) = s.split_once('e').unwrap();
            let n = 10_i64.pow(exp.parse::<i64>().unwrap().try_into().unwrap())
                * base.parse::<i64>().unwrap();
            Expr::Number(n)
        }
        Rule::len =>
            Expr::Len(Box::new(parse_expr(inner.into_inner().next().unwrap()))),
        Rule::input => Expr::Input,
        Rule::print =>
            Expr::Print(Box::new(parse_expr(inner.into_inner().next().unwrap()))),
        Rule::push => {
            let mut inner = inner.into_inner();
            Expr::Push(Box::new(parse_expr(inner.next().unwrap())), Box::new(parse_expr(inner.next().unwrap())))

        }


        _ => parse_expr(inner),
    }
}

fn parse_array(pair: Pair<Rule>) -> Expr {
    let elems = pair
        .into_inner()
        .map(|p| parse_expr(p))
        .collect();

    Expr::Array(elems)
}

fn parse_block(pair: Pair<Rule>) -> Expr {
    let stmts = pair
        .into_inner()
        .map(|p| parse_expr(p))
        .collect();

    Expr::Block(stmts)
}

fn parse_if(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let cond = parse_expr(inner.next().unwrap());
    let then = parse_block(inner.next().unwrap());
    let else_ = inner.next().map(|p| parse_block(p));

    Expr::If {
        cond: Box::new(cond),
        then: Box::new(then),
        else_: else_.map(Box::new),
    }
}

fn parse_lambda(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let args_pair = inner.next().unwrap();
    let args = args_pair
        .into_inner()
        .map(|p| p.as_str().to_string())
        .collect();

    let body = parse_expr(inner.next().unwrap());

    Expr::Lambda {
        args,
        body: Box::new(body),
    }
}

fn parse_while(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let cond = parse_expr(inner.next().unwrap());
    let body = parse_block(inner.next().unwrap());

    Expr::While {
        cond: Box::new(cond),
        body: Box::new(body),
    }
}

fn parse_for(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();

    let var = inner.next().unwrap().as_str().to_string();
    let iter = parse_expr(inner.next().unwrap());
    let body = parse_block(inner.next().unwrap());

    Expr::For {
        var,
        iter: Box::new(iter),
        body: Box::new(body),
    }
}
